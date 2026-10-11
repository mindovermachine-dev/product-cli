//! The flat layout of revision v1.8 in history (LP-3.35; rulings 82, 97).
//!
//! A store made before v1.9 keeps its flat paths in history, and the
//! re-layout is one commit that moves every file under
//! `.decisions/ns/<namespace>/`. A verifier with the legacy capability
//! follows each file across both path patterns for `L007`, `L009` and the
//! base overlay, and reads no flat semantics; one without it refuses the
//! repository with exit 2, naming the first flat commit, and never reports
//! it conformant (AC-82).

mod common;

use std::path::Path;

use common::{hand, Repo};

const CS: &str = "01K2C4YQJ3F8M0PT5W7NZ9RDXW";
const NS: &str = "fixture.ledger";
/// The `pass` fixture's acceptor.
const ACCEPTOR: &str = "fixture-human@example";
/// Whoever makes the re-layout commit: not the acceptor, so `L009` would
/// name them if the pickaxe stopped at the move.
const RELAYER: &str = "relayout@example";

/// A repository whose first commit holds the `pass` fixture in the flat
/// layout of v1.8, committed by its acceptor. Returns the repo and that
/// commit's id.
fn flat_history() -> (Repo, String) {
    let repo = Repo::with_identity(ACCEPTOR);
    let from = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pass/.decisions");
    for sub in ["sets", "log"] {
        let to = repo.path().join(".decisions").join(sub);
        std::fs::create_dir_all(&to).expect("mkdir");
        for entry in std::fs::read_dir(from.join(sub)).expect("fixture").flatten() {
            std::fs::copy(entry.path(), to.join(entry.file_name())).expect("copy");
        }
    }
    hand::commit(&repo, "the store, in the flat layout of v1.8");
    let first = String::from_utf8_lossy(&common::git(repo.path(), &["rev-parse", "HEAD"]).stdout).trim().to_string();
    (repo, first)
}

/// Re-lay the store out in one commit by `RELAYER`: `sets/` and `log/` move
/// under `ns/<NS>/`, byte for byte, as `git mv` moves them.
fn relayout(repo: &Repo) {
    std::fs::create_dir_all(repo.ns_dir(NS)).expect("mkdir");
    for sub in ["sets", "log"] {
        repo.git(&["mv", &format!(".decisions/{sub}"), &format!(".decisions/ns/{NS}/{sub}")]);
    }
    repo.act_as(RELAYER);
    repo.git(&["commit", "-q", "-m", "re-lay out under ns/<namespace>/ (spec v1.9)"]);
}

/// `verify` with blame on and the legacy capability, as the reference runs it.
fn verify(repo: &Repo, extra: &[&str]) -> (i32, String) {
    let mut args = vec!["verify", "--today", "2026-08-10"];
    args.extend_from_slice(extra);
    let out = repo.ledger(&args);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

#[test]
fn re_laid_out_in_one_commit_the_store_verifies_with_its_original_authors_and_no_l007() {
    let (repo, _) = flat_history();
    relayout(&repo);
    let (code, text) = verify(&repo, &[]);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("conformant"), "{text}");
    // Blame was on and the acceptor is not the re-layer: `L009` found the
    // original introduction through the flat path, not the move.
    assert!(!text.contains("[L009]") && !text.contains("skipped"), "{text}");
    // The flat files the move deleted are no removal: every entity is
    // present, unchanged, under its namespace's directory.
    assert!(!text.contains("[L007]"), "{text}");
}

#[test]
fn an_entity_dropped_by_the_re_layout_is_l007_by_entity() {
    let (repo, _) = flat_history();
    std::fs::create_dir_all(repo.ns_dir(NS)).expect("mkdir");
    for sub in ["sets", "log"] {
        repo.git(&["mv", &format!(".decisions/{sub}"), &format!(".decisions/ns/{NS}/{sub}")]);
    }
    // The acceptance is left behind in the move.
    let moved = repo.log_dir(NS).join(format!("{CS}.yml"));
    let text = std::fs::read_to_string(&moved).expect("read");
    let (keep, _) = text.split_once("acceptances:").expect("acceptances");
    std::fs::write(&moved, keep).expect("write");
    repo.act_as(RELAYER);
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "re-lay out, dropping the acceptance"]);
    let (code, text) = verify(&repo, &[]);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[L007] acc:01K2C4YQJ3F8M0PT5W7NZ9RDXX") && text.contains("is gone"), "{text}");
}

#[test]
fn a_flat_file_removed_rather_than_moved_is_l007_as_before() {
    let (repo, _) = flat_history();
    std::fs::create_dir_all(repo.ns_dir(NS)).expect("mkdir");
    repo.git(&["mv", ".decisions/sets", &format!(".decisions/ns/{NS}/sets")]);
    repo.git(&["rm", "-q", "-r", ".decisions/log"]);
    repo.act_as(RELAYER);
    repo.git(&["commit", "-q", "-m", "the log removed, not moved"]);
    let (code, text) = verify(&repo, &[]);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[L007]") && text.contains("is gone"), "{text}");
}

#[test]
fn a_format_declaration_changed_across_the_re_layout_is_l007() {
    let (repo, _) = flat_history();
    std::fs::create_dir_all(repo.ns_dir(NS)).expect("mkdir");
    for sub in ["sets", "log"] {
        repo.git(&["mv", &format!(".decisions/{sub}"), &format!(".decisions/ns/{NS}/{sub}")]);
    }
    let moved = repo.log_dir(NS).join(format!("{CS}.yml"));
    let text = std::fs::read_to_string(&moved).expect("read");
    std::fs::write(&moved, text.replacen("format: 1\n", "format: 5\n", 1)).expect("write");
    repo.act_as(RELAYER);
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "re-lay out, raising the declaration past what the content needs"]);
    let (code, text) = verify(&repo, &[]);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[L007]") && text.contains("its format declaration was 1 at"), "{text}");
}

#[test]
fn without_the_capability_a_flat_history_is_refused_with_exit_2_naming_the_first_flat_commit() {
    let (repo, first) = flat_history();
    relayout(&repo);
    let out = repo.ledger(&["verify", "--no-legacy-layout", "--today", "2026-08-10"]);
    let text = common::both(&out);
    assert_eq!(out.status.code(), Some(2), "the gate could not run: {text}");
    assert!(text.contains(&first), "names the first flat commit: {text}");
    assert!(text.contains("ruling 82") && text.contains("flat layout"), "{text}");
    assert!(!text.contains("conformant"), "never reported conformant: {text}");
    // Any flat commit on the base refuses too.
    let out = repo.ledger(&["verify", "--no-legacy-layout", "--base", "HEAD~1", "--today", "2026-08-10"]);
    assert_eq!(out.status.code(), Some(2), "{}", common::both(&out));
}

#[test]
fn a_history_written_at_v1_9_needs_no_capability() {
    let repo = Repo::human();
    repo.declare();
    repo.add("Money is decimal.", &[]);
    hand::commit(&repo, "a v1.9 store");
    let (code, text) = verify(&repo, &["--no-legacy-layout"]);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("conformant"), "{text}");
}

/// A second flat change-set, filed on the flat base after the re-layout
/// branched off: another decision of the fixture's set, accepted.
fn second_flat_change_set(repo: &Repo) {
    let text = std::fs::read_to_string(repo.path().join(format!(".decisions/log/{CS}.yml"))).expect("read");
    let mut cs: ledger_core::changeset::ChangeSet = serde_yaml::from_str(&text).expect("parse");
    cs.id = "cs:01K2C4YQJ3F8M0PT5W7NZ9RDY0".parse().expect("id");
    let decision: ledger_core::id::DecisionId = "dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDY1".parse().expect("id");
    cs.decisions[0].id = decision.clone();
    cs.versions[0].decision = decision.clone();
    cs.versions[0].statement = "Time is UTC.".into();
    cs.versions[0].hash = ledger_core::hash::version_hash(&cs.versions[0]);
    cs.acceptances[0].id = "acc:01K2C4YQJ3F8M0PT5W7NZ9RDY2".parse().expect("id");
    cs.acceptances[0].decision = decision;
    cs.acceptances[0].version = cs.versions[0].hash.clone();
    std::fs::write(repo.path().join(format!(".decisions/log/{}", cs.file_name())), serde_yaml::to_string(&cs).expect("yaml")).expect("write");
}

#[test]
fn the_base_overlay_reads_a_flat_base_for_the_re_layouts_own_pull_request() {
    let (repo, _) = flat_history();
    repo.git(&["checkout", "-q", "-b", "relayout"]);
    relayout(&repo);
    // Meanwhile the flat base moves on: a second change-set lands on main.
    repo.git(&["checkout", "-q", "main"]);
    repo.act_as(ACCEPTOR);
    second_flat_change_set(&repo);
    hand::commit(&repo, "a second decision, still flat");
    repo.git(&["checkout", "-q", "relayout"]);
    let alone = repo.ledger(&["verify", "--json", "--today", "2026-08-10"]);
    let alone: serde_json::Value = serde_json::from_slice(&alone.stdout).expect("json");
    assert_eq!(alone["decisions"], 1, "the branch's own store");
    let merged = repo.ledger(&["verify", "--json", "--base", "main", "--today", "2026-08-10"]);
    let text = common::both(&merged);
    assert_eq!(merged.status.code(), Some(0), "{text}");
    let merged: serde_json::Value = serde_json::from_slice(&merged.stdout).expect("json");
    assert_eq!(merged["decisions"], 2, "the base's flat change-set is overlaid: {merged}");
    assert!(merged["findings"].as_array().is_some_and(Vec::is_empty), "{merged}");
}
