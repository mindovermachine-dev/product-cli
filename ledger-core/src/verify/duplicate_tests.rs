//! Ruling 49: an id, or a decision identity object, filed twice is `SCHEMA`.
//!
//! The cases of the verification report's section 3 (3a, 3b), in memory.
//! The governed case, 3c, needs signatures and a repository and lives in
//! `ledger-cli/tests/legacy_revocation.rs`.

use crate::changeset::{ChangeSet, DecisionRecord};
use crate::finding::VerifyClass;
use crate::store::LoggedChangeSet;
use crate::testkit;

use super::{verify, Options, Report};

const SECOND: &str = "01K2C4YQJ3F8M0PT5W7NZ9RDY5";

/// The `pass` shape: one sealed, accepted version.
fn first() -> ChangeSet {
    let sealed = testkit::sealed(testkit::version());
    let acceptance = testkit::acceptance(&sealed);
    testkit::changeset(vec![sealed], vec![acceptance])
}

fn second() -> ChangeSet {
    ChangeSet::empty(
        1,
        format!("cs:{SECOND}").parse().expect("change-set id"),
        testkit::stamp("2026-08-11T09:14:22Z"),
        testkit::identity("someone-else@example"),
        None,
    )
}

fn run(extra: ChangeSet) -> Report {
    let mut store = testkit::store(first());
    store.log.push(LoggedChangeSet {
        path: std::path::PathBuf::from(format!("/fixture/.decisions/log/{SECOND}.yml")),
        file: extra,
    });
    verify(&store, &Options::offline(testkit::date("2026-08-12")))
}

fn schema_on(report: &Report, subject: &str, words: &str) -> bool {
    report.findings.iter().any(|f| f.class == VerifyClass::Schema && f.subject == subject && f.message.contains(words))
}

fn second_acceptance() -> crate::acceptance::Acceptance {
    let mut acc = testkit::acceptance(&testkit::sealed(testkit::version()));
    acc.actor = testkit::identity("second-human@example");
    acc.at = testkit::stamp("2026-08-11T09:20:00Z");
    acc
}

#[test]
fn a_decision_identity_object_filed_twice_is_a_schema_fault() {
    let mut cs = second();
    cs.decisions.push(DecisionRecord {
        id: testkit::decision_id(),
        created_at: testkit::stamp("2026-08-11T09:14:22Z"),
        created_by: testkit::identity("someone-else@example"),
    });
    let report = run(cs);
    assert!(schema_on(&report, &testkit::decision_id().to_string(), "filed twice"), "{:?}", report.findings);
}

#[test]
fn an_acceptance_id_filed_twice_is_a_schema_fault() {
    let mut cs = second();
    cs.acceptances.push(second_acceptance());
    let report = run(cs);
    assert!(schema_on(&report, &testkit::acceptance_id().to_string(), "filed twice"), "{:?}", report.findings);
}

/// 3b: one holder's revocation of the shared id used to revoke the other's
/// acceptance, and the store was conformant. The duplicate is now refused.
#[test]
fn a_revocation_of_a_shared_acceptance_id_does_not_leave_the_store_conformant() {
    let mut cs = second();
    cs.acceptances.push(second_acceptance());
    let mut rev = testkit::legacy_revocation("2026-08-12T09:00:00Z", "withdrawn");
    rev.by = Some(testkit::identity("second-human@example"));
    cs.revocations.push(rev);
    let report = run(cs);
    assert!(!report.is_conformant());
    assert!(schema_on(&report, &testkit::acceptance_id().to_string(), "filed twice"), "{:?}", report.findings);
}

#[test]
fn one_identity_object_per_decision_stays_conformant() {
    let report = run(second());
    assert!(report.is_conformant(), "{:?}", report.findings);
}
