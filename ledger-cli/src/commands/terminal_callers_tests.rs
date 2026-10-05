//! The signing entry points: called from the CLI command modules alone, and
//! reached only by invocations the terminal gate holds.
//!
//! [`super::gate`] runs in `commands::run` before dispatch, so a call that
//! lives only in a command module and is reached only by a
//! [`Mode::Terminal`] invocation cannot run without a terminal. These tests
//! pin both halves: the call sites, by a scan of every non-test source file
//! in the workspace; and the invocations reaching each, by `mode`.
//! **Honest limit:** this is a property of this workspace's source. Any
//! program that links `ledger-core` can call the same methods; the control
//! that requires a person is the key, not the terminal.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use clap::Parser;

use super::*;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

const DEC: &str = "dec:fixture.ledger/01K5M000000000000000000001";
const GRANT: &str = "grant:01K5M000000000000000000002";
const BINDING: &str = "key:01K5M000000000000000000003";
const SIGN: &str = "ledger-cli/src/commands/sign.rs";
const AUTHORITY: &str = "ledger-cli/src/commands/authority.rs";

/// Every `Author` method that signs or writes an authority record: the one
/// module allowed to call it, and every invocation that can make it write.
/// `accept_group` and `accept_batch` write only with `--confirm`; without
/// it they are the selection dry run, which writes nothing.
const ENTRY_POINTS: &[(&str, &str, &[&[&str]])] = &[
    ("accept", SIGN, &[&["accept", DEC]]),
    ("accept_group", SIGN, &[&["accept", "--set", "s", "--confirm", "sha256:00"], &["accept", "--group", "mechanical", "--confirm", "sha256:00"]]),
    ("accept_batch", SIGN, &[&["accept", "--batch", "b.yml", "--repository", "r", "--branch", "b", "--confirm", "sha256:00"]]),
    ("revoke", SIGN, &[&["revoke", "acc:01K5M000000000000000000005", "--reason", "r"]]),
    ("init_namespace", AUTHORITY, &[&["init", "--namespace", "n", "--external-ref", "m"]]),
    ("declare_role", AUTHORITY, &[&["role", "declare", "r", "--may", "accept-decision"]]),
    ("grant", AUTHORITY, &[&["grant", "new", "acceptor", "--to", "a@x", "--scope", "*"]]),
    ("accept_grant", AUTHORITY, &[&["grant", "accept", GRANT]]),
    ("revoke_grant", AUTHORITY, &[&["grant", "revoke", GRANT, "--reason", "r"]]),
    ("unavailable", AUTHORITY, &[&["unavailable", GRANT]]),
    ("available", AUTHORITY, &[&["available", "unav:01K5M000000000000000000004"]]),
    ("identity_add", AUTHORITY, &[&["identity", "add", "--namespace", "n", "--key-file", "k.pub"]]),
    ("identity_rotate", AUTHORITY, &[&["identity", "rotate", BINDING, "--key-file", "k.pub"]]),
    ("identity_revoke", AUTHORITY, &[&["identity", "revoke", BINDING]]),
    ("policy_set", AUTHORITY, &[&["policy", "set", "--namespace", "n"]]),
];

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Every Rust file under a `src/` directory, tests left out: `*_tests.rs`,
/// `tests.rs`, and anything after a file's first `#[cfg(test)]`.
fn sources(dir: &Path, rel: &str, out: &mut Vec<(String, String)>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        let rel = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
        if path.is_dir() {
            if !matches!(name.as_str(), "target" | ".git" | "tests" | "node_modules" | "bin" | "obj") {
                sources(&path, &rel, out);
            }
        } else if name.ends_with(".rs") && rel.contains("/src/") && !name.ends_with("_tests.rs") && name != "tests.rs" {
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            let code = text.split("#[cfg(test)]").next().unwrap_or_default();
            out.push((rel, code.to_string()));
        }
    }
}

/// The (method, file) pairs where an entry point is called. Only files that
/// can reach `ledger-core` are read: ones naming `ledger_core`, and
/// `ledger-core`'s own (where `self.<method>(` would be an internal caller).
fn call_sites() -> BTreeSet<(String, String)> {
    let mut files = Vec::new();
    sources(&workspace(), "", &mut files);
    let mut found = BTreeSet::new();
    for (rel, code) in files.iter().filter(|(rel, code)| rel.starts_with("ledger-core/") || code.contains("ledger_core")) {
        for line in code.lines().filter(|l| !l.trim_start().starts_with("//")) {
            for (method, _, _) in ENTRY_POINTS {
                if line.contains(&format!(".{method}(")) {
                    found.insert(((*method).to_string(), rel.clone()));
                }
            }
        }
    }
    found
}

#[test]
fn every_signing_entry_point_is_called_only_from_its_cli_command_module() {
    let expected: BTreeSet<(String, String)> = ENTRY_POINTS.iter().map(|(m, f, _)| ((*m).to_string(), (*f).to_string())).collect();
    assert_eq!(call_sites(), expected, "a signing entry point gained or lost a call site — every caller must be a command module the terminal gate holds");
}

#[test]
fn every_invocation_that_reaches_a_signing_entry_point_needs_a_terminal() {
    for (method, _, invocations) in ENTRY_POINTS {
        for args in *invocations {
            let argv = std::iter::once("ledger").chain(args.iter().copied());
            let command = Cli::try_parse_from(argv).unwrap_or_else(|e| panic!("{args:?}: {e}")).command;
            assert!(matches!(mode(&command), Mode::Terminal(_)), "`ledger {}` reaches `{method}` and must need a terminal", args.join(" "));
        }
    }
}

#[test]
fn the_gate_runs_before_dispatch() {
    let run = std::fs::read_to_string(workspace().join("ledger-cli/src/commands/mod.rs")).unwrap_or_default();
    assert!(
        run.contains("terminal::gate(&command).and_then(|()| dispatch(command, root))"),
        "commands::run must gate before it dispatches"
    );
}
