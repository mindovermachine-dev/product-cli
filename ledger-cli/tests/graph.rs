//! The L2 acceptance: the index is provably disposable, coverage
//! distinguishes all seven states over the fixture built to exhibit them,
//! and the graph stage reports through `verify` with unchanged exit
//! semantics.

use std::path::Path;
use std::process::Output;

use assert_cmd::Command;

mod common;

/// The date at which the coverage fixture exhibits every state.
const TODAY: &str = "2027-01-01";

fn ledger(root: &Path, args: &[&str]) -> Output {
    let mut cmd = Command::cargo_bin("ledger").expect("binary");
    cmd.arg("--root").arg(root).args(args);
    cmd.output().expect("run")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// A scratch copy of the coverage fixture, so tests can write an index.
fn scratch_coverage() -> tempfile::TempDir {
    common::stage_fixture("coverage")
}

#[test]
fn the_rebuild_is_byte_identical_after_deleting_the_index() {
    let dir = scratch_coverage();
    let out = ledger(dir.path(), &["reindex"]);
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    let index = dir.path().join(".decisions/index/ledger.ttl");
    let first = std::fs::read(&index).expect("index bytes");
    assert!(!first.is_empty());

    std::fs::remove_dir_all(dir.path().join(".decisions/index")).expect("delete index");
    let out = ledger(dir.path(), &["reindex"]);
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    let rebuilt = std::fs::read(&index).expect("rebuilt bytes");
    assert_eq!(first, rebuilt, "state leaked into the cache: the rebuild differs");
}

#[test]
fn coverage_distinguishes_all_seven_states_on_the_fixture() {
    let staged = common::stage_fixture("coverage");
    let root = staged.path().to_path_buf();
    let out = ledger(&root, &["coverage", "--today", TODAY]);
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    let text = stdout(&out);
    for state in [
        "undecided 1",
        "awaiting-acceptance 2",
        "decided 1",
        "escaped-priced 1",
        "escape-review-due 1",
        "expired 1",
        "superseded 2",
    ] {
        assert!(text.contains(state), "missing `{state}` in:\n{text}");
    }
    assert!(text.contains("by namespace:"), "{text}");
    assert!(text.contains("fixture.coverage:"), "{text}");
    assert!(text.contains(" -> "), "the chain renders: {text}");
    assert!(text.contains("the honest limit:"), "§8's disclosure is not optional: {text}");
}

#[test]
fn coverage_json_carries_the_states_and_the_chain_machine_readably() {
    let staged = common::stage_fixture("coverage");
    let root = staged.path().to_path_buf();
    let out = ledger(&root, &["coverage", "--today", TODAY, "--json"]);
    let parsed: serde_json::Value = serde_json::from_str(&stdout(&out)).expect("json");
    assert_eq!(parsed["decisions"], 9);
    assert_eq!(parsed["by_set"]["coverage-fixture"]["superseded"], 2);
    assert_eq!(parsed["chains"][0].as_array().map(Vec::len), Some(3));
    assert!(parsed["honest_limit"].as_str().is_some_and(|s| s.contains("enumerated")));
}

#[test]
fn a_superseded_versions_acceptances_are_historical_in_blame_and_status() {
    // d7 (the chain root) is superseded; the fixture's `decided` decision
    // d3 stays live. Status renders the supersession terminally.
    let staged = common::stage_fixture("coverage");
    let root = staged.path().to_path_buf();
    let out = ledger(&root, &["status", "--today", TODAY]);
    let text = stdout(&out);
    assert!(text.contains("superseded by dec:fixture.coverage/"), "{text}");
}

#[test]
fn the_graph_stage_reports_through_verify_with_exit_one() {
    // Break the fixture copy: point a supersession at a decision nobody
    // filed. The file gate stays quiet; the graph stage fails the run.
    let dir = scratch_coverage();
    let log_dir = ledger_core::layout::log_dir(dir.path(), "fixture.coverage");
    let target = std::fs::read_dir(&log_dir)
        .expect("read")
        .flatten()
        .map(|e| e.path())
        .find(|p| {
            std::fs::read_to_string(p).expect("read").contains("supersedes:")
        })
        .expect("a supersession change-set");
    let text = std::fs::read_to_string(&target).expect("read");
    let broken = regex_replace_supersedes(&text);
    std::fs::write(&target, broken).expect("write");
    // Re-seal the hash so L007 stays quiet and the graph finding isolates.
    reseal(&target);

    let out = ledger(dir.path(), &["verify", "--today", TODAY, "--no-blame", "--gate", "readiness"]);
    assert_eq!(out.status.code(), Some(1), "{}", String::from_utf8_lossy(&out.stderr));
    let err = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(err.contains("graph stage — "), "{err}");
    assert!(err.contains("[G001]"), "{err}");
}

// ---------------------------------------------------------------------------
// Latest derives from the parent DAG, not from ULID order (L3).
// ---------------------------------------------------------------------------

#[test]
fn latest_follows_the_parent_dag_when_ulid_order_contradicts_it() {
    // The two-clocks fixture files the *root* version under the later ULID:
    // a writer whose clock runs ahead. ULID order would call the root
    // latest and the signed revision historical; the parent DAG knows the
    // revision is the tip, so the store is conformant and decided.
    let staged = common::stage_fixture("two-clocks");
    let root = staged.path().to_path_buf();
    let out = ledger(&root, &["verify", "--today", "2026-08-10", "--no-blame"]);
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(
        stdout(&out).contains("conformant"),
        "{}",
        stdout(&out)
    );

    let status = ledger(&root, &["status", "--today", "2026-08-10"]);
    let text = stdout(&status);
    assert!(text.contains("[c754a1e1e9f6] decided"), "the DAG tip is latest: {text}");
    assert!(!text.contains("awaiting"), "the acceptance signs the tip, not history: {text}");
}

#[test]
fn g004_a_forked_chain_fails_verify_and_names_both_tips() {
    // The forked fixture is what a plain git merge of two branches' logs
    // produces: one parent, two divergent revisions. No file is malformed;
    // the graph stage names the fork and the store is non-conformant until
    // a human arbitrates.
    let staged = common::stage_fixture("forked");
    let root = staged.path().to_path_buf();
    let out = ledger(&root, &["verify", "--today", "2026-08-10", "--no-blame"]);
    assert_eq!(out.status.code(), Some(1), "{}", String::from_utf8_lossy(&out.stderr));
    let err = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(err.contains("[G004]"), "{err}");
    assert!(err.contains("merge --resolve"), "{err}");
    for cls in ["L001", "L007", "SCHEMA", "G001", "G002", "G003"] {
        assert!(!err.contains(&format!("[{cls}]")), "the fork isolates: {err}");
    }
    // Ruling 54: a forked decision has no latest version, so `G004` is its
    // one finding and it is not "allocated, awaiting acceptance".
    assert!(err.contains("non-conformant — 1 finding(s)"), "{err}");
    assert!(!err.contains("awaiting acceptance"), "{err}");

    // `status` lists it apart from the seven states, with both tips and no
    // stand-in hash.
    let status = ledger(&root, &["status", "--today", "2026-08-10"]);
    let text = stdout(&status);
    assert!(text.contains("1 forked — no latest version until `ledger merge --resolve` arbitrates:"), "{text}");
    assert!(text.contains("tips 523a5b4b0e93, a289f499364f"), "{text}");
    assert!(!text.contains("[a289f499364f]"), "no tip stands in as the latest: {text}");

    // `show` gives each tip a screen, marked as one of its tips; `coverage`
    // lists the decision apart from the seven states.
    let show = stdout(&ledger(&root, &["show", "dec:fixture.forked/01K2C4YQJ3F8M0PT5W7NZ9RDXV", "--today", "2026-08-10"]));
    assert!(show.starts_with("2 decision(s)"), "{show}");
    assert_eq!(show.matches("this is one of its tips, and none is the latest").count(), 2, "{show}");
    assert!(show.contains("523a5b4b0e93") && show.contains("a289f499364f"), "{show}");
    let coverage = stdout(&ledger(&root, &["coverage", "--today", "2026-08-10"]));
    assert!(coverage.contains("forked, no state until `ledger merge --resolve`: dec:fixture.forked/01K2C4YQJ3F8M0PT5W7NZ9RDXV"), "{coverage}");
}

/// Point the file's `supersedes:` at a ULID nobody filed.
fn regex_replace_supersedes(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        if let Some(prefix) = line.split("supersedes:").next().filter(|_| line.contains("supersedes:")) {
            out.push_str(&format!(
                "{prefix}supersedes: dec:fixture.coverage/01K2C4YQJ3F8M0PT5W7NZ9RDY9\n"
            ));
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Recompute and rewrite the stored hashes in one log file.
fn reseal(path: &Path) {
    let original = std::fs::read_to_string(path).expect("read");
    let parsed: ledger_core::changeset::ChangeSet =
        serde_yaml::from_str(&original).expect("parse");
    let mut text = original.clone();
    for version in &parsed.versions {
        let computed = ledger_core::hash::version_hash(version).to_string();
        text = text.replace(&version.hash.to_string(), &computed);
    }
    std::fs::write(path, text).expect("write");
}
