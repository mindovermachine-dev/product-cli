//! Ruling 50: the genesis role never accepts, held at verification.
//!
//! The verification report's section 2 and section 13.2, in memory: a
//! policy whose accept role is the genesis role, and a genesis role that
//! carries a decision capability. Each is `SCHEMA`; the control, the root
//! role beside a separate accept role, is conformant.

use crate::authority::fixture::{self, accepted, genesis, role};
use crate::authority::payload::policy_hash;
use crate::authority::{Capability, Grant, Policy, Scheme};
use crate::finding::VerifyClass;
use crate::hash::VersionHash;
use crate::testkit;

use super::{verify, Options, Report};

fn policy(g: &Grant, accept_role: &str) -> Policy {
    let mut p = Policy {
        id: format!("pol:{}", fixture::ulid("3")).parse().expect("id"),
        namespace: "fixture.ledger".into(),
        schemes: vec![Scheme::None],
        require_sk: false,
        accept_role: accept_role.into(),
        reaccept_within_days: None,
        replaces: None,
        by: testkit::identity(fixture::GENESIS_HOLDER),
        under: Some(g.id.clone()),
        at: testkit::stamp("2026-10-01T10:00:00Z"),
        hash: VersionHash::zero(),
    };
    p.hash = policy_hash(&p);
    p
}

fn run(steward: &[Capability], accept_role: &str) -> Report {
    let g = genesis("1", "steward");
    let mut cs = fixture::changeset(vec![g.clone()], vec![accepted("2", &g)]);
    cs.policies.push(policy(&g, accept_role));
    let roles = vec![role("steward", steward), role("acceptor", &[Capability::AcceptDecision])];
    verify(&fixture::store(roles, cs), &Options::offline(testkit::date("2026-10-15")))
}

fn schema(report: &Report, subject_starts: &str, words: &str) -> bool {
    report.findings.iter().any(|f| f.class == VerifyClass::Schema && f.subject.starts_with(subject_starts) && f.message.contains(words))
}

#[test]
fn the_root_role_beside_a_separate_accept_role_is_conformant() {
    let report = run(Capability::ROOT, "acceptor");
    assert!(report.is_conformant(), "{:?} {:?}", report.findings, report.graph);
}

/// Section 2: the genesis role widened to `accept-decision` and named as
/// the policy's accept role. Both halves are refused.
#[test]
fn a_policy_whose_accept_role_is_the_genesis_role_is_a_schema_fault() {
    let mut widened = Capability::ROOT.to_vec();
    widened.push(Capability::AcceptDecision);
    let report = run(&widened, "steward");
    assert!(schema(&report, "pol:", "the genesis role as of the policy"), "{:?}", report.findings);
    assert!(schema(&report, "steward", "is the genesis role and carries accept-decision"), "{:?}", report.findings);
}

/// Section 13.2: a genesis role carrying a decision capability, even when
/// the policy names another accept role, is refused on its own.
#[test]
fn a_genesis_role_with_any_decision_capability_is_a_schema_fault() {
    for extra in Capability::DECISION {
        let mut widened = Capability::ROOT.to_vec();
        widened.push(*extra);
        let report = run(&widened, "acceptor");
        assert!(schema(&report, "steward", &format!("carries {extra}")), "{extra}: {:?}", report.findings);
    }
}
