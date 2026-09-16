//! The observation store as the CLI reports it.
//!
//! Nothing here exits non-zero. A run record is measurement, a reading over
//! several is still measurement, and an escape candidate is a finding whose
//! response belongs to a person — so the verb reports and the gate stays
//! elsewhere.

use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;
use eval_core::declaration::Declaration;
use eval_core::{Backend, EvalStore, Pinned, RunRecord};

fn store(root: &Path) -> EvalStore<Box<dyn eval_core::Blobs>> {
    EvalStore::new(Backend::disk(root.join(".spec")).open().expect("a disk store"))
}

fn spec(root: &Path) -> Command {
    let mut cmd = Command::cargo_bin("spec").expect("the spec binary builds");
    cmd.arg("--root").arg(root);
    cmd.env_remove("EVAL_STORE");
    cmd
}

fn run(id: &str, address: &str, arrangement: &str, proposed: &[&str]) -> RunRecord {
    let mut record = RunRecord::new(id, "spec-flow", "act/a", "slice")
        .at(Pinned::of([("q", address)]))
        .arranged_as(Pinned::of([("model", arrangement)]));
    record.proposed = proposed.iter().map(|s| (*s).to_string()).collect();
    record.kept.clone_from(&record.proposed);
    record
}

fn output(root: &Path, args: &[&str]) -> String {
    let out = spec(root).arg("runs").args(args).output().expect("runs runs");
    assert_eq!(out.status.code(), Some(0), "the store is read, never gated");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn an_empty_store_says_so_rather_than_failing() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert!(output(dir.path(), &[]).contains("no runs observed"));
}

#[test]
fn observed_runs_are_listed() {
    let dir = tempfile::tempdir().expect("tempdir");
    store(dir.path()).write_run(&run("01", "q1", "m", &["det/a"])).expect("writes");

    let listed = output(dir.path(), &[]);
    assert!(listed.contains("act/a"), "{listed}");
    assert!(listed.contains("1 proposed"), "{listed}");
}

/// The earthquake reading, through the CLI.
#[test]
fn a_changed_worker_at_a_fixed_address_reads_as_drift() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = store(dir.path());
    for (id, arrangement, proposed) in [
        ("01", "old", "det/a"),
        ("02", "old", "det/a"),
        ("03", "new", "det/b"),
        ("04", "new", "det/b"),
    ] {
        store.write_run(&run(id, "q1", arrangement, &[proposed])).expect("writes");
    }

    let readings = output(dir.path(), &["--readings"]);
    assert!(readings.contains("4 run(s) across 2 arrangement(s)"), "{readings}");
    assert!(readings.contains("drift                1.00"), "{readings}");
    assert!(readings.contains("never says one act was wrong"), "the note must survive");
}

/// An absent measure is printed as absent, never as zero.
#[test]
fn a_single_run_reads_as_not_enough_rather_than_zero() {
    let dir = tempfile::tempdir().expect("tempdir");
    store(dir.path()).write_run(&run("01", "q1", "m", &["det/a"])).expect("writes");

    let readings = output(dir.path(), &["--readings"]);
    assert!(readings.contains("not enough runs to say"), "{readings}");
}

#[test]
fn runs_without_an_address_are_counted_rather_than_compared() {
    let dir = tempfile::tempdir().expect("tempdir");
    let bare = RunRecord::new("01", "spec-flow", "act/a", "slice");
    store(dir.path()).write_run(&bare).expect("writes");

    assert!(output(dir.path(), &["--readings"]).contains("declared no address"));
}

#[test]
fn a_run_that_declared_nothing_is_an_escape_candidate() {
    let dir = tempfile::tempdir().expect("tempdir");
    store(dir.path()).write_run(&run("01", "q1", "m", &[])).expect("writes");

    let escapes = output(dir.path(), &["--escapes"]);
    assert!(escapes.contains("undeclared"), "{escapes}");
    assert!(escapes.contains("Findings, not a gate"), "the note must survive");
}

/// The finding a live model produced: ground named as needed, never consulted.
#[test]
fn ground_declared_and_never_read_is_reported() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut record = run("01", "q1", "m", &[]);
    record.declared = Some(
        Declaration::new("settle it", vec!["act/settle-a-basket".into()])
            .bounded_by("the act's text", "drafted for review"),
    );
    store(dir.path()).write_run(&record).expect("writes");

    let escapes = output(dir.path(), &["--escapes"]);
    assert!(escapes.contains("declared-but-unread"), "{escapes}");
    assert!(escapes.contains("act/settle-a-basket"), "{escapes}");
}

#[test]
fn a_declaration_matched_by_its_reads_finds_nothing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut record = run("01", "q1", "m", &[]);
    record.declared = Some(
        Declaration::new("settle it", vec!["spec_check".into()])
            .bounded_by("the act's text", "drafted for review"),
    );
    record.ground_read = vec!["spec_check".into()];
    store(dir.path()).write_run(&record).expect("writes");

    assert!(output(dir.path(), &["--escapes"]).contains("no escape candidates"));
}

#[test]
fn the_json_body_carries_the_note_a_reader_needs() {
    let dir = tempfile::tempdir().expect("tempdir");
    store(dir.path()).write_run(&run("01", "q1", "m", &[])).expect("writes");

    let body: serde_json::Value =
        serde_json::from_str(&output(dir.path(), &["--escapes", "--json"])).expect("json");
    assert!(body["note"].as_str().is_some_and(|n| n.contains("not a gate")));
}
