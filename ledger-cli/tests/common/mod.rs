//! A temporary git repository with a ledger store, driven through the binary.
//!
//! Shared by the integration suites that need a real repository. A signing
//! invocation, and every verb that writes an authority record, refuses a
//! non-interactive caller (#71, #85), so [`invoke`] runs it under a
//! pseudo-terminal, which is how a person runs it; everything else runs
//! with plain pipes. [`Repo::piped`] forces pipes, for the refusal
//! tests themselves.
//!
//! Every git these suites run, and every invocation of the binary (whose
//! `inbox accept` commits), goes out with the caller's git identity
//! variables cleared ([`GIT_IDENTITY_VARS`], #138): a fixture commit is
//! authored by the identity the fixture configured, never the container's.

#![allow(dead_code)]

pub mod export_only;
pub mod hand;

use std::path::Path;
use std::process::Output;

use assert_cmd::Command;

pub struct Repo {
    pub dir: tempfile::TempDir,
}

impl Repo {
    /// A git repo with an initialised store and a human identity.
    pub fn human() -> Self {
        Self::with_identity("fixture-human@example")
    }

    pub fn with_identity(email: &str) -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = Self { dir };
        repo.git(&["init", "--initial-branch=main"]);
        repo.git(&["config", "user.name", "Fixture"]);
        repo.git(&["config", "user.email", email]);
        assert_eq!(repo.ledger(&["init"]).status.code(), Some(0));
        repo
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn git(&self, args: &[&str]) {
        git(self.path(), args);
    }

    /// Switch the acting identity, as a person changing git config would.
    pub fn act_as(&self, email: &str) {
        self.git(&["config", "user.email", email]);
    }

    /// Run the binary the way a person would: signing verbs at a terminal.
    pub fn ledger(&self, args: &[&str]) -> Output {
        invoke(self.path(), args)
    }

    /// Run the binary with plain pipes whatever the verb: stdin is not a
    /// terminal, as for an agent or a CI script.
    pub fn piped(&self, args: &[&str]) -> Output {
        piped(self.path(), args)
    }

    /// Run the binary under a pseudo-terminal whatever the verb.
    pub fn tty(&self, args: &[&str]) -> Output {
        tty(self.path(), args)
    }

    /// Run a verb that must succeed, returning stdout.
    pub fn ok(&self, args: &[&str]) -> String {
        expect_code(&self.ledger(args), 0, args)
    }

    /// Run a verb under a TTY that must succeed, returning its output.
    pub fn ok_tty(&self, args: &[&str]) -> String {
        expect_code(&self.tty(args), 0, args)
    }

    /// Run a verb that must be refused (exit 1), returning stdout and
    /// stderr together (a terminal merges them anyway).
    pub fn refused(&self, args: &[&str]) -> String {
        let out = self.ledger(args);
        expect_code(&out, 1, args);
        both(&out)
    }

    /// Run a verb under a TTY that must be refused (exit 1), returning output.
    pub fn refused_tty(&self, args: &[&str]) -> String {
        expect_code(&self.tty(args), 1, args)
    }

    /// The log files in the store, sorted.
    pub fn log_files(&self) -> Vec<String> {
        let mut out: Vec<String> = std::fs::read_dir(self.path().join(".decisions/log"))
            .map(|d| d.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect())
            .unwrap_or_default();
        out.sort();
        out
    }

    /// A fresh ed25519 key pair (no passphrase) under `keys/`, outside the
    /// store; returns the private key's path.
    pub fn keygen(&self, name: &str) -> String {
        let dir = self.path().join("keys");
        std::fs::create_dir_all(&dir).expect("keys dir");
        let private = dir.join(name);
        let out = std::process::Command::new("ssh-keygen")
            .args(["-q", "-t", "ed25519", "-N", "", "-C", name, "-f"])
            .arg(&private)
            .output()
            .expect("ssh-keygen (OpenSSH) is required by the signing suites");
        assert!(out.status.success(), "ssh-keygen: {}", String::from_utf8_lossy(&out.stderr));
        private.display().to_string()
    }

    /// Sign with this key from now on, as `git config user.signingkey`.
    pub fn use_key(&self, private: &str) {
        self.git(&["config", "user.signingkey", private]);
    }

    /// Make a key, sign with it, and bind it: the genesis holder's first
    /// binding in `ns` (self-bound), or a further key of the actor's own.
    pub fn bind_own_key(&self, ns: &str, name: &str) -> String {
        let private = self.keygen(name);
        self.use_key(&private);
        self.ok(&["identity", "add", "--namespace", ns, "--key-file", &format!("{private}.pub")]);
        private
    }

    /// As the genesis holder, bind `who`'s first key in `ns` (D7); returns
    /// the private key's path for `who` to sign with.
    pub fn vouch_for(&self, ns: &str, who: &str, name: &str) -> String {
        let private = self.keygen(name);
        self.ok(&["identity", "add", "--namespace", ns, "--for", who, "--key-file", &format!("{private}.pub")]);
        private
    }

    /// Declare the default fixture set at floor T1.
    pub fn declare(&self) {
        self.ok(&["declare", "--set", "ledger-design", "--tolerance-floor", "T1"]);
    }

    /// File a constraint decision and return its id.
    pub fn add(&self, statement: &str, extra: &[&str]) -> String {
        let mut args = vec![
            "add", "--set", "ledger-design", "--namespace", "fixture.ledger", "--statement",
            statement, "--store", "constraint", "--discharge", "analyzer:DEC001",
        ];
        args.extend_from_slice(extra);
        let out = self.ok(&args);
        decision_id(&out)
    }
}

/// The git identity variables a caller's environment may export. Each one
/// overrides the `user.name` / `user.email` a fixture configures for its own
/// commits, so every git the suites spawn, directly or through the binary,
/// clears them (#138).
pub const GIT_IDENTITY_VARS: [&str; 4] =
    ["GIT_AUTHOR_NAME", "GIT_AUTHOR_EMAIL", "GIT_COMMITTER_NAME", "GIT_COMMITTER_EMAIL"];

/// A `git -C dir` command that takes its identity from the repository's own
/// configuration alone.
pub fn git_command(dir: &Path) -> std::process::Command {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("-C").arg(dir);
    for var in GIT_IDENTITY_VARS {
        cmd.env_remove(var);
    }
    cmd
}

/// Run git in `dir`, failing the test on a non-zero exit; returns its output.
pub fn git(dir: &Path, args: &[&str]) -> Output {
    let out = git_command(dir).args(args).output().expect("git");
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    out
}

/// The binary at `root`, with plain pipes and the caller's git identity
/// cleared, before any verb is given.
pub fn ledger_command(root: &Path) -> Command {
    let mut cmd = Command::cargo_bin("ledger").expect("binary");
    cmd.arg("--root").arg(root);
    for var in GIT_IDENTITY_VARS {
        cmd.env_remove(var);
    }
    cmd
}

/// Whether an invocation must run at a terminal (#71, #85): it signs, or
/// unsays a signature, or writes an authority record. The binary's own
/// classification is `commands::terminal::mode`; this mirrors it so the
/// suites drive each verb the way a person runs it.
pub fn needs_terminal(args: &[&str]) -> bool {
    let sub = args.get(1).copied();
    match args.first().copied() {
        Some("revoke" | "role" | "available" | "unavailable") => true,
        Some("accept") => {
            args.contains(&"--confirm") || args.iter().skip(1).any(|a| a.starts_with("dec:"))
        }
        Some("init") => args.contains(&"--namespace"),
        Some("grant") => true,
        Some("identity") => sub != Some("sync"),
        Some("policy") => sub == Some("set"),
        Some("inbox") => sub == Some("accept") && args.contains(&"--confirm"),
        _ => false,
    }
}

/// Run the binary, under a pseudo-terminal when the invocation needs one.
pub fn invoke(root: &Path, args: &[&str]) -> Output {
    if needs_terminal(args) {
        tty(root, args)
    } else {
        piped(root, args)
    }
}

/// Run the binary with plain pipes.
pub fn piped(root: &Path, args: &[&str]) -> Output {
    ledger_command(root).args(args).output().expect("run")
}

/// Run the binary under a pseudo-terminal (`script(1)` from util-linux), so
/// stdin is a TTY exactly as it is for a person at a shell. Output arrives
/// merged on stdout with CRLF line ends, normalised here to LF; the exit
/// status is the binary's own (`-e`).
pub fn tty(root: &Path, args: &[&str]) -> Output {
    let bin = assert_cmd::cargo::cargo_bin("ledger");
    let mut line = shell_quote(&bin.to_string_lossy());
    line.push_str(" --root ");
    line.push_str(&shell_quote(&root.to_string_lossy()));
    for a in args {
        line.push(' ');
        line.push_str(&shell_quote(a));
    }
    let mut script = std::process::Command::new("script");
    script.args(["-qefc", &line, "/dev/null"]).stdin(std::process::Stdio::null());
    for var in GIT_IDENTITY_VARS {
        script.env_remove(var);
    }
    let out = script.output().expect("script(1) from util-linux is required to drive a pseudo-terminal");
    let text = String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n");
    Output { status: out.status, stdout: text.into_bytes(), stderr: out.stderr }
}

/// Stdout then stderr, as one string.
pub fn both(out: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))
}

/// The `dec:` id an `add` printed.
pub fn decision_id(stdout: &str) -> String {
    stdout
        .split_whitespace()
        .find(|w| w.starts_with("dec:"))
        .expect("a decision id in the output")
        .to_string()
}

fn expect_code(out: &Output, code: i32, args: &[&str]) -> String {
    assert_eq!(
        out.status.code(),
        Some(code),
        "ledger {args:?} should exit {code}:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
