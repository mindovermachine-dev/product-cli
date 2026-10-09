//! The crossing stage over in-memory stores: a `supersedes` into another namespace, judged on the live claim.

use super::findings;
use crate::id::DecisionId;
use crate::testkit;
use crate::verify::view::View;
use crate::version::VersionRaw;

const SUCCESSOR: &str = "dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDY0";
const ELSEWHERE: &str = "dec:other.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDY1";

fn id(s: &str) -> DecisionId {
    s.parse().expect("id")
}

/// The successor's first version, superseding `old`.
fn claim(old: &str) -> VersionRaw {
    let mut v = testkit::version();
    v.decision = id(SUCCESSOR);
    v.supersedes = Some(id(old));
    testkit::sealed(v)
}

#[test]
fn a_latest_version_superseding_another_namespace_is_a_schema_fault_naming_both() {
    let store = testkit::store(testkit::changeset(vec![claim(ELSEWHERE)], Vec::new()));
    let found = findings(&View::build(&store));
    assert_eq!(found.len(), 1, "{found:?}");
    let text = found[0].to_string();
    assert!(text.starts_with(&format!("[SCHEMA] {SUCCESSOR}:")), "{text}");
    assert!(text.contains(ELSEWHERE) && text.contains("`other.ledger`") && text.contains("ruling 43"), "{text}");
}

#[test]
fn a_next_version_that_drops_the_edge_repairs_it_and_history_is_not_judged() {
    let first = claim(ELSEWHERE);
    let mut withdrawn = first.clone();
    withdrawn.parent = Some(first.hash.clone());
    withdrawn.supersedes = None;
    let withdrawn = testkit::sealed(withdrawn);
    let store = testkit::store(testkit::changeset(vec![first, withdrawn], Vec::new()));
    let view = View::build(&store);
    assert!(view.versions.iter().any(|v| v.raw.supersedes.is_some()), "the claim stays in history");
    assert!(findings(&view).is_empty(), "{:?}", findings(&view));
}

#[test]
fn a_supersession_within_the_namespace_is_not_a_crossing() {
    let old = testkit::sealed(testkit::version());
    let store = testkit::store(testkit::changeset(vec![old, claim(&testkit::decision_id().to_string())], Vec::new()));
    assert!(findings(&View::build(&store)).is_empty());
}
