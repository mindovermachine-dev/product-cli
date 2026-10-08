//! Ruling 57: `L006` judges an escape's acceptor on every version, not only the latest.

use crate::allocation::AllocationKind;
use crate::finding::VerifyClass;
use crate::testkit;

use super::{verify, Options};

/// Ruling 57, the verification report's case 8a: an escape priced by a model
/// identity, then a child version that re-allocates the decision. `L006` is
/// not latest-only, so the escape in history still fails it.
#[test]
fn l006_judges_an_escape_acceptor_on_a_version_that_is_no_longer_latest() {
    let mut escape = testkit::version();
    escape.allocation = Some(AllocationKind::Escaped);
    escape.discharge.clear();
    escape.exposure = Some("downstream may break".into());
    escape.accepted_by = Some(testkit::identity("claude@example.com"));
    escape.review_by = Some(testkit::date("2027-01-01"));
    let escape = testkit::sealed(escape);
    let mut child = testkit::version();
    child.parent = Some(escape.hash.clone());
    let child = testkit::sealed(child);
    let mut cs = testkit::changeset(vec![escape], Vec::new());
    cs.versions.push(child);
    let report = verify(&testkit::store(cs), &Options::offline(testkit::date("2026-08-10")));
    let classes: Vec<VerifyClass> = report.findings.iter().map(|f| f.class).collect();
    assert_eq!(classes, vec![VerifyClass::L006], "{:?}", report.findings);
}
