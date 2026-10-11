//! Signatures — sidecars, schemes, and what each signable entity signs (#70).
//!
//! A signature never lives in a log file (#65, D2): it is a sidecar at
//! `.decisions/ns/<ns>/sig/<ulid>.<scheme>.sig` (LP-3.34), one per scheme,
//! under the namespace of the entity whose ULID names it, and it signs that entity's closed payload — the
//! exact bytes its content hash digests ([`crate::hash::signed_bytes`]).
//! The signable entities are the acceptance, the revocation of an
//! acceptance, the key binding and the namespace policy. Grants and grant
//! acceptances are not signed yet (D5 (b), #82).
//!
//! - [`ssh`] signs and verifies with `ssh-keygen -Y`, against the derived
//!   `allowed_signers` and the act's own `at` (`-Overify-time`).
//! - [`dsse`] verifies only: signing is the hosted service's.
//! - `none` is no sidecar at all, valid only where policy lists it.

pub mod check;
pub mod dsse;
mod review;
pub mod ssh;
pub mod subject;

use std::path::{Path, PathBuf};

use crate::authority::Scheme;
use crate::finding::Finding;

/// The directory under a namespace's directory that holds sidecars.
pub const SIG_DIR: &str = crate::layout::SIG_DIR;

/// One signature file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sidecar {
    /// The namespace whose directory holds it (LP-3.34).
    pub namespace: String,
    /// The ULID of the entity it signs (the id's part after the scheme).
    pub ulid: String,
    pub scheme: Scheme,
    /// The file name under `sig/`.
    pub file: String,
    pub bytes: Vec<u8>,
}

impl Sidecar {
    /// The file name a sidecar for `ulid` under `scheme` takes.
    pub fn file_name(ulid: &str, scheme: Scheme) -> String {
        format!("{ulid}.{scheme}.sig")
    }

    /// Parse a sidecar's name: `<ulid>.<ssh|dsse>.sig`.
    pub fn parse_name(name: &str) -> Result<(String, Scheme), String> {
        let stem = name.strip_suffix(".sig").ok_or_else(|| format!("`{name}` is not a `.sig` file"))?;
        let (ulid, scheme) = stem.rsplit_once('.').ok_or_else(|| format!("`{name}` is not `<ulid>.<scheme>.sig`"))?;
        let scheme: Scheme = scheme.parse()?;
        if scheme == Scheme::None {
            return Err(format!("`{name}`: `none` is no signature, so it has no sidecar"));
        }
        crate::id::validate_ulid(ulid).map_err(|e| format!("`{name}`: {e}"))?;
        Ok((ulid.to_string(), scheme))
    }
}

/// Load every sidecar under `dir` (a namespace's `sig/`), with a `SCHEMA`
/// finding for each file that is not a well-named sidecar.
pub fn load(dir: &Path, namespace: &str) -> (Vec<Sidecar>, Vec<Finding>) {
    let mut out = Vec::new();
    let mut faults = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return (out, faults) };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.is_file()).collect();
    paths.sort();
    for path in paths {
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        match (Sidecar::parse_name(&name), std::fs::read(&path)) {
            (Ok((ulid, scheme)), Ok(bytes)) => out.push(Sidecar { namespace: namespace.to_string(), ulid, scheme, file: name, bytes }),
            (Err(e), _) => faults.push(Finding::schema(&format!("sig/{name}"), e)),
            (_, Err(e)) => faults.push(Finding::schema(&format!("sig/{name}"), e.to_string())),
        }
    }
    (out, faults)
}

/// Write a sidecar under its namespace's `sig/` directory.
pub fn write(root: &Path, sidecar: &Sidecar) -> Result<PathBuf, String> {
    let dir = crate::layout::sig_dir(root, &sidecar.namespace);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(&sidecar.file);
    if path.exists() {
        return Err(format!("{} already exists — a signature is written once", path.display()));
    }
    std::fs::write(&path, &sidecar.bytes).map_err(|e| e.to_string())?;
    Ok(path)
}
