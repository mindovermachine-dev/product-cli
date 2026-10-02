//! Key-rule cases: immutability along a chain, uniqueness among live decisions.

use crate::finding::VerifyClass;
use crate::id::DecisionId;
use crate::testkit;
use crate::version::VersionRaw;
use crate::verify::{verify, Options};

fn opts() -> Options {
    Options { gate: None, today: testkit::date("2026-08-10"), blame: false }
}

fn classes(versions: Vec<VersionRaw>) -> Vec<VerifyClass> {
    let report = verify(&testkit::store(testkit::changeset(versions, Vec::new())), &opts());
    assert!(report.graph.iter().all(|g| g.class.code() != "G004"), "{:?}", report.graph);
    report.findings.iter().map(|f| f.class).collect()
}

fn keyed(key: &str) -> VersionRaw {
    let mut v = testkit::version();
    v.key = Some(key.parse().expect("key"));
    v
}

fn other_decision(tail: char) -> DecisionId {
    format!("dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDX{tail}").parse().expect("id")
}

fn child(parent: &VersionRaw, key: Option<&str>) -> VersionRaw {
    let mut v = parent.clone();
    v.parent = Some(parent.hash.clone());
    v.statement = format!("{} — revised", parent.statement);
    v.key = key.map(|k| k.parse().expect("key"));
    testkit::sealed(v)
}

#[test]
fn a_key_carried_unchanged_along_the_chain_is_conformant() {
    let root = testkit::sealed(keyed("MoneyIsDecimal"));
    let next = child(&root, Some("MoneyIsDecimal"));
    assert!(!classes(vec![root, next]).contains(&VerifyClass::L013));
}

#[test]
fn l013_a_renamed_key_is_refused() {
    let root = testkit::sealed(keyed("MoneyIsDecimal"));
    let next = child(&root, Some("MoneyIsExact"));
    assert!(classes(vec![root, next]).contains(&VerifyClass::L013));
}

#[test]
fn l013_a_dropped_key_is_refused() {
    let root = testkit::sealed(keyed("MoneyIsDecimal"));
    let next = child(&root, None);
    assert!(classes(vec![root, next]).contains(&VerifyClass::L013));
}

#[test]
fn giving_a_keyless_decision_a_key_is_a_new_version_not_a_finding() {
    let root = testkit::sealed(testkit::version());
    let next = child(&root, Some("MoneyIsDecimal"));
    assert!(!classes(vec![root, next]).contains(&VerifyClass::L013));
}

#[test]
fn l013_judges_the_merged_from_edge_too() {
    // Only the closed tip carries a key; dropping it in the reconciliation
    // renames the decision through the merge edge alone.
    let root = testkit::sealed(testkit::version());
    let left = child(&root, None);
    let mut right = child(&root, Some("MoneyIsDecimal"));
    right.statement = "the other writer".into();
    let right = testkit::sealed(right);
    let mut reconciled = child(&left, None);
    reconciled.merged_from = Some(right.hash.clone());
    let reconciled = testkit::sealed(reconciled);
    let found = classes(vec![root, left, right, reconciled]);
    assert_eq!(found.iter().filter(|c| **c == VerifyClass::L013).count(), 1, "{found:?}");
}

#[test]
fn l014_two_live_decisions_sharing_a_key_both_fail() {
    let a = testkit::sealed(keyed("MoneyIsDecimal"));
    let mut b = keyed("MoneyIsDecimal");
    b.decision = other_decision('Y');
    let b = testkit::sealed(b);
    let found = classes(vec![a, b]);
    assert_eq!(found.iter().filter(|c| **c == VerifyClass::L014).count(), 2, "{found:?}");
}

#[test]
fn the_same_key_in_two_namespaces_is_two_names() {
    let a = testkit::sealed(keyed("MoneyIsDecimal"));
    let mut b = keyed("MoneyIsDecimal");
    b.decision = "dec:hafeok.other/01K2C4YQJ3F8M0PT5W7NZ9RDXY".parse().expect("id");
    let b = testkit::sealed(b);
    assert!(!classes(vec![a, b]).contains(&VerifyClass::L014));
}

#[test]
fn a_superseded_decision_frees_its_key_for_the_successor() {
    let old = testkit::sealed(keyed("MoneyIsDecimal"));
    let mut successor = keyed("MoneyIsDecimal");
    successor.decision = other_decision('Y');
    successor.supersedes = Some(old.decision.clone());
    let successor = testkit::sealed(successor);
    assert!(!classes(vec![old, successor]).contains(&VerifyClass::L014));
}

#[test]
fn only_latest_versions_hold_a_key() {
    // A decision renamed (L013) no longer holds its old key for L014.
    let a = testkit::sealed(keyed("MoneyIsDecimal"));
    let mut b_root = keyed("Other");
    b_root.decision = other_decision('Y');
    let b_root = testkit::sealed(b_root);
    let found = classes(vec![a, b_root]);
    assert!(!found.contains(&VerifyClass::L014), "{found:?}");
}
