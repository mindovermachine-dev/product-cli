//! `accept` and `revoke` refuse a non-interactive caller and write nothing.
//!
//! Signing is a person's act (PRD §5, §10): run without a terminal on stdin
//! — piped, scripted, from an agent harness — the signing verbs exit
//! non-zero before the store is opened. There is no override to test: no
//! flag and no environment variable turns the check off. The same
//! invocations under a pseudo-terminal succeed, so the refusal is the
//! terminal check and nothing else.

mod common;

use std::collections::BTreeMap;

use common::Repo;

/// Every file under `.decisions/`, with its bytes: "writes nothing" is
/// checked against the whole store, not one directory.
fn snapshot(repo: &Repo) -> BTreeMap<String, Vec<u8>> {
    fn walk(dir: &std::path::Path, base: &std::path::Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, base, out);
            } else {
                let rel = path.strip_prefix(base).unwrap_or(&path).display().to_string();
                out.insert(rel, std::fs::read(&path).unwrap_or_default());
            }
        }
    }
    let mut out = BTreeMap::new();
    let store = repo.path().join(".decisions");
    walk(&store, &store, &mut out);
    out
}

fn refused_without_a_terminal(repo: &Repo, args: &[&str]) {
    let before = snapshot(repo);
    let out = repo.piped(args);
    let text = common::both(&out);
    assert_ne!(out.status.code(), Some(0), "ledger {args:?} must refuse piped stdin:\n{text}");
    assert!(text.contains("runs only at a terminal"), "{text}");
    assert!(text.contains("Nothing was written"), "{text}");
    assert_eq!(snapshot(repo), before, "ledger {args:?} wrote under a refusal");
}

fn filed() -> (Repo, String) {
    let repo = Repo::human();
    repo.declare();
    let id = repo.add("Monetary amounts use decimal, never double.", &[]);
    (repo, id)
}

#[test]
fn accept_naming_a_decision_refuses_piped_stdin() {
    let (repo, id) = filed();
    refused_without_a_terminal(&repo, &["accept", &id]);
    // The same invocation at a terminal signs: the refusal was the check.
    let signed = repo.ok_tty(&["accept", &id]);
    assert!(signed.contains("accepted"), "{signed}");
}

#[test]
fn a_selection_dry_run_stays_scriptable_but_its_confirm_refuses() {
    let (repo, _) = filed();
    let read = repo.ok(&["accept", "--set", "ledger-design"]);
    let manifest = read
        .split_whitespace()
        .find(|w| w.starts_with("sha256:"))
        .expect("the dry run prints its manifest")
        .to_string();
    refused_without_a_terminal(&repo, &["accept", "--set", "ledger-design", "--confirm", &manifest]);
    repo.ok_tty(&["accept", "--set", "ledger-design", "--confirm", &manifest]);
}

#[test]
fn revoke_refuses_piped_stdin() {
    let (repo, id) = filed();
    repo.ok_tty(&["accept", &id]);
    let acc = snapshot(&repo)
        .values()
        .flat_map(|bytes| String::from_utf8_lossy(bytes).into_owned().lines().map(str::to_string).collect::<Vec<_>>())
        .find_map(|line| line.trim().strip_prefix("- id: acc:").map(|u| format!("acc:{u}")))
        .expect("the acceptance just filed");
    refused_without_a_terminal(&repo, &["revoke", &acc, "--reason", "filed in error"]);
}

#[test]
fn piped_stdin_is_refused_even_with_a_closed_or_empty_stdin() {
    let (repo, id) = filed();
    let before = snapshot(&repo);
    let mut cmd = assert_cmd::Command::cargo_bin("ledger").expect("binary");
    let out = cmd
        .arg("--root")
        .arg(repo.path())
        .args(["accept", &id])
        .write_stdin("y\nyes\n")
        .output()
        .expect("run");
    assert_ne!(out.status.code(), Some(0), "typing `yes` into a pipe is not a terminal");
    assert_eq!(snapshot(&repo), before);
}
