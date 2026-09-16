use super::*;

use crate::blobs::DiskBlobs;
use crate::judgement::Judge;
use std::collections::BTreeMap;

fn store() -> (tempfile::TempDir, EvalStore<DiskBlobs>) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = EvalStore::new(DiskBlobs::new(dir.path()));
    (dir, store)
}

fn run() -> RunRecord {
    RunRecord::new("01ABC", "spec-flow", "act/settle-a-basket", "item-totals")
}

fn judgement(model: &str, task: &str) -> Judgement {
    let shown: BTreeMap<String, String> =
        [("task".to_string(), task.to_string())].into_iter().collect();
    Judgement::new("01ABC", Judge::new(model, None), shown, vec![])
}

#[test]
fn a_run_is_written_and_read_back() {
    let (_dir, store) = store();
    store.write_run(&run()).expect("writes");
    assert_eq!(store.read_run("01ABC").expect("reads").id, "01ABC");
}

#[test]
fn an_absent_run_is_not_found() {
    let (_dir, store) = store();
    assert!(store.read_run("nope").is_err());
}

/// One run keeps every opinion of it.
#[test]
fn two_judges_of_one_run_are_both_kept() {
    let (_dir, store) = store();
    for model in ["glm-5.2", "mistral-medium-3.5-128b"] {
        store.write_judgement(&judgement(model, "act/a")).expect("writes");
    }
    assert_eq!(store.read_judgements("01ABC").len(), 2);
}

/// Re-asking the same judge the same question replaces, never accretes.
#[test]
fn the_same_judge_over_the_same_context_files_once() {
    let (_dir, store) = store();
    store.write_judgement(&judgement("glm-5.2", "act/a")).expect("writes");
    store.write_judgement(&judgement("glm-5.2", "act/a")).expect("writes again");
    assert_eq!(store.read_judgements("01ABC").len(), 1);
}

/// A new occasion differs in its context, and lands beside the first.
#[test]
fn the_same_judge_over_a_new_context_files_again() {
    let (_dir, store) = store();
    store.write_judgement(&judgement("glm-5.2", "act/a")).expect("writes");
    store.write_judgement(&judgement("glm-5.2", "act/b")).expect("writes");
    assert_eq!(store.read_judgements("01ABC").len(), 2);
}

/// Measurement must not be filed among the things that judged it.
#[test]
fn runs_and_judgements_are_kept_apart() {
    assert!(run_key("01ABC").starts_with(RUNS));
    assert!(judgements_prefix("01ABC").starts_with(JUDGEMENTS));
    assert_ne!(RUNS, JUDGEMENTS);
}

/// One corrupt entry must not hide the rest of the store.
#[test]
fn an_unreadable_entry_is_skipped_rather_than_raised() {
    let (_dir, store) = store();
    store.write_run(&run()).expect("writes");
    store.blobs().put("runs/junk.json", "{ not json ]").expect("junk");

    assert_eq!(store.read_runs().len(), 1);
}

#[test]
fn a_model_name_becomes_part_of_a_key() {
    let key = judgement_key(&judgement("mistral-medium-3.5-128b", "act/a"));
    assert!(key.starts_with("judgements/01ABC/mistral-medium-3-5-128b."), "{key}");
    assert!(key.ends_with(".json"), "{key}");
}

/// The keys are the same whatever holds them, which is what makes a swap a swap.
#[test]
fn the_layout_does_not_depend_on_the_backend() {
    assert_eq!(run_key("01ABC"), "runs/01ABC.json");
    assert_eq!(judgements_prefix("01ABC"), "judgements/01ABC");
}
