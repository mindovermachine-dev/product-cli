//! The `ssh` scheme — `ssh-keygen -Y sign | verify | check-novalidate`.
//!
//! Signing: `ssh-keygen -Y sign -f <key> -n ledger-accept@<ns>` over the
//! signed bytes on stdin. `<ns>` is the ledger namespace of the store that
//! holds the entity — never a flag, never derived from the principal.
//! Verifying: `ssh-keygen -Y verify -f <allowed_signers> -I <principal> -n
//! ledger-accept@<ns> -s <sig> -Overify-time=<at>`, where the
//! allowed-signers text is the derived trust root ([`crate::authority::signers`]).
//! `check-novalidate` names the key that made a signature (by fingerprint),
//! so the gate can tell which binding's window it falls in.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

/// The signature namespace for a ledger namespace.
pub fn sig_namespace(namespace: &str) -> String {
    format!("ledger-accept@{namespace}")
}

/// `ssh-keygen`'s time form: `YYYYMMDDhhmmssZ`.
pub fn keygen_time(at: &DateTime<Utc>) -> String {
    at.format("%Y%m%d%H%M%SZ").to_string()
}

/// Whether `ssh-keygen` is on the PATH.
pub fn available() -> bool {
    Command::new("ssh-keygen").arg("-?").output().is_ok()
}

/// A scratch directory removed on drop.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Result<Self, String> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let nanos = Utc::now().timestamp_nanos_opt().unwrap_or_default();
        let dir = std::env::temp_dir().join(format!("ledger-sig-{}-{n}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        Ok(Self(dir))
    }

    fn file(&self, name: &str, bytes: &[u8]) -> Result<PathBuf, String> {
        let path = self.0.join(name);
        std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
        Ok(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(mut cmd: Command, stdin: &[u8]) -> Result<(bool, String, Vec<u8>), String> {
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not run ssh-keygen ({e}) — install OpenSSH 8.2 or later"))?;
    if let Some(mut input) = child.stdin.take() {
        input.write_all(stdin).map_err(|e| e.to_string())?;
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    Ok((out.status.success(), text, out.stdout))
}

/// Sign `message` with the key at `key` (a private key, or a public key
/// whose private half the agent holds) for `namespace`.
pub fn sign(key: &Path, namespace: &str, message: &[u8]) -> Result<Vec<u8>, String> {
    let mut cmd = Command::new("ssh-keygen");
    cmd.args(["-q", "-Y", "sign", "-f"]).arg(key).args(["-n", &sig_namespace(namespace)]);
    let (ok, text, stdout) = run(cmd, message)?;
    if !ok || stdout.is_empty() {
        return Err(format!("ssh-keygen -Y sign failed: {}", text.trim()));
    }
    Ok(stdout)
}

/// Verify `signature` over `message` as `principal`'s, against the
/// allowed-signers text, at `at`.
pub fn verify(
    allowed_signers: &str,
    principal: &str,
    namespace: &str,
    signature: &[u8],
    message: &[u8],
    at: &DateTime<Utc>,
) -> Result<(), String> {
    let scratch = Scratch::new()?;
    let signers = scratch.file("allowed_signers", allowed_signers.as_bytes())?;
    let sig = scratch.file("message.sig", signature)?;
    let mut cmd = Command::new("ssh-keygen");
    cmd.args(["-Y", "verify", "-f"])
        .arg(&signers)
        .args(["-I", principal, "-n", &sig_namespace(namespace), "-s"])
        .arg(&sig)
        .arg(format!("-Overify-time={}", keygen_time(at)));
    let (ok, text, _) = run(cmd, message)?;
    if ok {
        Ok(())
    } else {
        Err(text.lines().find(|l| !l.trim().is_empty()).unwrap_or("signature does not verify").trim().to_string())
    }
}

/// The fingerprint (`SHA256:…`) of the key that made `signature`, when it
/// is a valid signature over `message` in `namespace` at all.
pub fn signer_fingerprint(namespace: &str, signature: &[u8], message: &[u8]) -> Result<String, String> {
    let scratch = Scratch::new()?;
    let sig = scratch.file("message.sig", signature)?;
    let mut cmd = Command::new("ssh-keygen");
    cmd.args(["-Y", "check-novalidate", "-n", &sig_namespace(namespace), "-s"]).arg(&sig);
    let (ok, text, _) = run(cmd, message)?;
    if !ok {
        return Err(format!("not a valid `{}` signature over these bytes", sig_namespace(namespace)));
    }
    text.split_whitespace()
        .find(|w| w.starts_with("SHA256:"))
        .map(str::to_string)
        .ok_or_else(|| format!("ssh-keygen named no key: {}", text.trim()))
}

/// The OpenSSH fingerprint of a public key's base64 blob: `SHA256:` and the
/// unpadded base64 of the blob's SHA-256.
pub fn fingerprint(key_base64: &str) -> Option<String> {
    let blob = super::dsse::base64_decode(key_base64)?;
    let digest = Sha256::digest(&blob);
    Some(format!("SHA256:{}", super::dsse::base64_encode(&digest).trim_end_matches('=')))
}

/// What `git config user.signingkey` names, resolved against `root`: the
/// key `ledger` signs with, as git's own SSH signing does.
pub fn configured_key(root: &Path) -> Option<PathBuf> {
    let out = Command::new("git").arg("-C").arg(root).args(["config", "user.signingkey"]).output().ok()?;
    let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !out.status.success() || raw.is_empty() {
        return None;
    }
    let expanded = match raw.strip_prefix("~/") {
        Some(rest) => std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default().join(rest),
        None => PathBuf::from(&raw),
    };
    Some(if expanded.is_absolute() { expanded } else { root.join(expanded) })
}

/// The public half of a configured key: `(type, base64)`, and whether the
/// private half is on disk (`false`: the agent holds it).
pub fn public_half(key: &Path) -> Result<(String, String, bool), String> {
    let text = std::fs::read_to_string(key).map_err(|e| format!("{}: {e}", key.display()))?;
    let on_disk = text.contains("PRIVATE KEY");
    let public = if on_disk {
        let pub_path = PathBuf::from(format!("{}.pub", key.display()));
        match std::fs::read_to_string(&pub_path) {
            Ok(p) => p,
            Err(_) => derive_public(key)?,
        }
    } else {
        text
    };
    let mut parts = public.split_whitespace();
    match (parts.next(), parts.next()) {
        (Some(t), Some(k)) => Ok((t.to_string(), k.to_string(), on_disk)),
        _ => Err(format!("{} names no OpenSSH public key", key.display())),
    }
}

fn derive_public(key: &Path) -> Result<String, String> {
    let out = Command::new("ssh-keygen").arg("-y").arg("-f").arg(key).output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("ssh-keygen -y could not read {}", key.display()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}
