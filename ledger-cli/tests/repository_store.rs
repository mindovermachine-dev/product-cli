//! This repository's own store, re-laid out under `.decisions/ns/<namespace>/`
//! in one commit (PRD §3.11, issue 10; AC-82).
//!
//! The history before that commit holds the flat layout of revision v1.8,
//! so this repository is verified by the legacy capability (LP-3.35): with
//! it, `ledger verify --export` is conformant with blame on, which is every
//! acceptance's `L009` author and every entity's immutability verdict as
//! before the re-layout; without it, the repository is refused with exit 2,
//! naming the first flat commit, and never reported conformant. CI checks
//! out the full history, which both runs need.

use std::path::{Path, PathBuf};
use std::process::Output;

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn ledger(args: &[&str]) -> Output {
    let mut cmd = assert_cmd::Command::cargo_bin("ledger").expect("binary");
    cmd.arg("--root").arg(workspace()).args(args);
    cmd.output().expect("run")
}

fn text(out: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))
}

/// Whether the clone carries the flat commits at all: a shallow clone may
/// not, and then there is no flat history to read or to refuse.
fn first_flat_commit() -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(workspace())
        .args(["log", "--first-parent", "--reverse", "--format=%H", "--", ".decisions/log", ".decisions/roles", ".decisions/sig"])
        .output()
        .expect("git");
    String::from_utf8_lossy(&out.stdout).lines().next().map(str::to_string)
}

#[test]
fn the_store_is_laid_out_one_directory_per_namespace() {
    let store = workspace().join(".decisions");
    for flat in ["sets", "roles", "log", "sig", "allowed_signers"] {
        assert!(!store.join(flat).exists(), "{flat} is a flat path (LP-3.34)");
    }
    let namespaces = ledger_core::layout::namespaces(&workspace());
    assert_eq!(namespaces, ["hafeok.ddd", "hafeok.ledger"]);
    let logs = |ns: &str| std::fs::read_dir(ledger_core::layout::log_dir(&workspace(), ns)).expect("log").count();
    assert_eq!((logs("hafeok.ddd"), logs("hafeok.ledger")), (164, 23), "PRD §3.11");
}

#[test]
fn with_the_legacy_capability_this_repository_verifies_conformant_with_blame_on() {
    let out = ledger(&["verify", "--export"]);
    let report = text(&out);
    assert_eq!(out.status.code(), Some(0), "{report}");
    assert!(report.contains("conformant"), "{report}");
    assert!(report.contains("export: every committed export matches the log byte for byte"), "{report}");
    // Blame was on: every acceptance's introducing commit was found along
    // the file's lineage, never the re-layout commit (AC-82).
    assert!(!report.contains("[L009]") && !report.contains("[L007]"), "{report}");
}

#[test]
fn without_the_capability_this_repository_is_refused_naming_its_first_flat_commit() {
    let Some(first) = first_flat_commit() else {
        eprintln!("skipped: this clone does not carry the flat history");
        return;
    };
    let out = ledger(&["verify", "--export", "--no-legacy-layout"]);
    let report = text(&out);
    assert_eq!(out.status.code(), Some(2), "the gate could not run (ruling 82): {report}");
    assert!(report.contains(&first), "names the first flat commit {first}: {report}");
    assert!(!report.contains("conformant"), "never reported conformant: {report}");
}
