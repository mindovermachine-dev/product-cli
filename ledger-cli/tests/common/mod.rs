//! A temporary git repository with a ledger store, driven through the binary.
//!
//! Shared by the integration suites that need a real repository. Verbs that
//! refuse a non-interactive caller (`accept`, `revoke`) run under a
//! pseudo-terminal through [`Repo::tty`], which is how a person runs them;
//! everything else runs with plain pipes.

#![allow(dead_code)]

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
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(self.path())
            .args(args)
            .output()
            .expect("git");
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    }

    /// Switch the acting identity, as a person changing git config would.
    pub fn act_as(&self, email: &str) {
        self.git(&["config", "user.email", email]);
    }

    /// Run the binary with plain pipes: stdin is not a terminal.
    pub fn ledger(&self, args: &[&str]) -> Output {
        let mut cmd = Command::cargo_bin("ledger").expect("binary");
        cmd.arg("--root").arg(self.path()).args(args);
        cmd.output().expect("run")
    }

    /// Run the binary under a pseudo-terminal (`script(1)` from
    /// util-linux), so stdin is a TTY exactly as it is for a person at a
    /// shell. Output arrives merged on stdout with CRLF line ends; they are
    /// normalised to LF. The exit status is the binary's own.
    pub fn tty(&self, args: &[&str]) -> Output {
        let bin = assert_cmd::cargo::cargo_bin("ledger");
        let mut line = shell_quote(&bin.to_string_lossy());
        line.push_str(" --root ");
        line.push_str(&shell_quote(&self.path().to_string_lossy()));
        for a in args {
            line.push(' ');
            line.push_str(&shell_quote(a));
        }
        let out = std::process::Command::new("script")
            .args(["-qefc", &line, "/dev/null"])
            .stdin(std::process::Stdio::null())
            .output()
            .expect("script(1) from util-linux is required to drive a pseudo-terminal");
        let text = String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n");
        Output { status: out.status, stdout: text.into_bytes(), stderr: out.stderr }
    }

    /// Run a verb that must succeed, returning stdout.
    pub fn ok(&self, args: &[&str]) -> String {
        expect_code(&self.ledger(args), 0, args)
    }

    /// Run a verb under a TTY that must succeed, returning its output.
    pub fn ok_tty(&self, args: &[&str]) -> String {
        expect_code(&self.tty(args), 0, args)
    }

    /// Run a verb that must be refused (exit 1), returning stderr.
    pub fn refused(&self, args: &[&str]) -> String {
        let out = self.ledger(args);
        expect_code(&out, 1, args);
        String::from_utf8_lossy(&out.stderr).into_owned()
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
