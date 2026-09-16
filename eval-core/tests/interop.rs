//! Reading what the other runtime wrote.
//!
//! The premise of a format document with two implementations is that a store
//! written by one is readable by the other. This asserts it against a record
//! the .NET half actually produced, rather than one this crate round-tripped
//! through itself — which would only prove it agrees with itself.
//!
//! The fixture was captured from a live run and had its reply trimmed; nothing
//! else about it was touched.

use std::path::PathBuf;

use eval_core::behaviour;
use eval_core::run::RunRecord;

fn written_by_dotnet() -> RunRecord {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/dotnet-written-run.json");
    let text = std::fs::read_to_string(&path).expect("the fixture is readable");
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn a_record_written_by_the_other_runtime_parses() {
    let run = written_by_dotnet();
    assert_eq!(run.form, eval_core::run::RUN_FORM);
    assert_eq!(run.tool, "spec-flow");
    assert_eq!(run.model.as_deref(), Some("qwen3-coder-30b-a3b-instruct"));
}

/// `deny_unknown_fields` means a field one side adds is caught, not ignored.
///
/// That is the point of asserting against a foreign record: a silently dropped
/// field would leave both suites green and the store quietly lossy.
#[test]
fn the_foreign_record_carries_no_field_this_side_does_not_know() {
    // Parsing at all is the assertion; `deny_unknown_fields` does the work.
    let _ = written_by_dotnet();
}

#[test]
fn the_coordinates_it_declared_are_readable_here() {
    let run = written_by_dotnet();
    let address = run.address.as_ref().expect("the .NET half declared an address");
    let arrangement = run.arrangement.as_ref().expect("and an arrangement");

    assert_eq!(address.shown.get("task").map(String::as_str), Some("act/settle-a-basket"));
    assert!(arrangement.shown.contains_key("model"));
}

/// The digest law holds across the seam, not merely within each side.
#[test]
fn a_digest_computed_there_recomputes_here() {
    let run = written_by_dotnet();
    let address = run.address.as_ref().expect("an address");
    assert!(
        address.holds(),
        "the .NET half's digest does not recompute in Rust — the canonical form has drifted"
    );
    assert!(run.arrangement.as_ref().is_some_and(|a| a.holds()));
}

/// A foreign record takes part in the readings like any other.
#[test]
fn a_foreign_record_can_be_read_against_a_local_one() {
    let theirs = written_by_dotnet();
    let mut ours = RunRecord::new("local", "another-tool", theirs.task.clone(), "subject");
    ours.address.clone_from(&theirs.address);
    ours.arrangement.clone_from(&theirs.arrangement);
    ours.proposed.clone_from(&theirs.proposed);

    let readings = behaviour::read(&[theirs, ours]);
    assert_eq!(readings.len(), 1, "both runs sit at the one address");
    assert_eq!(readings[0].runs, 2);
    assert_eq!(readings[0].agreement, Some(1.0), "they proposed the same thing");
}
