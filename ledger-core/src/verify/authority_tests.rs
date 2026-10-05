//! Authority records through the gate: schema rules, L006, L007, A003, A005.

use crate::authority::fixture::{self, accepted, away, genesis, grant, role};
use crate::authority::{Basis, Capability};
use crate::finding::VerifyClass;
use crate::graph::GraphClass;
use crate::testkit;
use crate::verify::{verify, Options, Report};

fn run(store: &crate::store::Store) -> Report {
    verify(store, &Options::offline(testkit::date("2026-10-15")))
}

fn steward() -> Vec<crate::authority::Role> {
    vec![role("steward", Capability::ALL)]
}

fn messages(report: &Report) -> String {
    format!("{:?} {:?}", report.findings, report.graph)
}

#[test]
fn a_genesis_grant_accepted_by_its_holder_is_conformant() {
    let g = genesis("1", "steward");
    let store = fixture::store(steward(), fixture::changeset(vec![g.clone()], vec![accepted("2", &g)]));
    let report = run(&store);
    assert!(report.is_conformant(), "{}", messages(&report));
}

#[test]
fn a_grant_of_an_undeclared_role_is_a_schema_fault() {
    let g = grant("1", "ghost", "a@x", "*", 0);
    let report = run(&fixture::store(steward(), fixture::changeset(vec![g], Vec::new())));
    assert!(report.findings.iter().any(|f| f.class == VerifyClass::Schema && f.message.contains("not declared")));
}

#[test]
fn a_misshapen_genesis_and_a_limited_primary_are_schema_faults() {
    let mut g = genesis("1", "steward");
    g.scope = "ns:x".parse().expect("scope");
    let g = fixture::reseal(g);
    let limited = fixture::limited(grant("2", "steward", "a@x", "*", 0), &[crate::authority::Limit::NoGrants]);
    let report = run(&fixture::store(steward(), fixture::changeset(vec![g, limited], Vec::new())));
    let text = messages(&report);
    assert!(text.contains("self-granted, scope *"), "{text}");
    assert!(text.contains("primary grant must not carry limits"), "{text}");
}

#[test]
fn only_the_holder_accepts_and_only_the_grants_own_hash() {
    let g = genesis("1", "steward");
    let mut by_other = accepted("2", &g);
    by_other.actor = testkit::identity("someone-else@x");
    let mut wrong_hash = accepted("3", &g);
    wrong_hash.signs = testkit::zero_hash();
    let report = run(&fixture::store(steward(), fixture::changeset(vec![g], vec![by_other, wrong_hash])));
    let text = messages(&report);
    assert!(text.contains("only the holder accepts"), "{text}");
    assert!(text.contains("not the grant's hash"), "{text}");
}

#[test]
fn an_unavailability_must_stand_in_the_basis_it_claims() {
    let g = genesis("1", "steward");
    let mut cs = fixture::changeset(vec![g.clone()], vec![accepted("2", &g)]);
    let mut wrong = away("3", &g, "2026-10-10T00:00:00Z", None);
    wrong.by = testkit::identity("stranger@x");
    wrong.basis = Basis::SelfDeclared;
    cs.unavailabilities.push(wrong);
    let report = run(&fixture::store(steward(), cs));
    assert!(messages(&report).contains("claims basis `self`"), "{}", messages(&report));
}

#[test]
fn l006_refuses_a_model_holder() {
    let g = grant("1", "steward", "claude@anthropic.com", "*", 0);
    let report = run(&fixture::store(steward(), fixture::changeset(vec![g], Vec::new())));
    assert!(report.findings.iter().any(|f| f.class == VerifyClass::L006 && f.message.contains("grant holder")));
}

#[test]
fn l007_catches_a_grant_edited_after_it_was_sealed() {
    let mut g = genesis("1", "steward");
    g.holder = testkit::identity("usurper@x");
    g.granted_by = g.holder.clone();
    let report = run(&fixture::store(steward(), fixture::changeset(vec![g], Vec::new())));
    assert!(report.findings.iter().any(|f| f.class == VerifyClass::L007), "{}", messages(&report));
}

#[test]
fn a003_two_live_grants_at_one_place_in_the_order() {
    let a = grant("1", "steward", "a@x", "ns:hafeok.ledger", 0);
    let b = grant("2", "steward", "b@x", "ns:hafeok.ledger", 0);
    let cs = fixture::changeset(vec![a.clone(), b.clone()], vec![accepted("3", &a), accepted("4", &b)]);
    let report = run(&fixture::store(steward(), cs));
    assert!(report.graph.iter().any(|g| g.class == GraphClass::A003), "{}", messages(&report));
}

#[test]
fn a005_two_live_genesis_grants() {
    let a = genesis("1", "steward");
    let mut b = genesis("2", "steward");
    b.external_ref = Some("another mandate".into());
    let b = fixture::reseal(b);
    let report = run(&fixture::store(steward(), fixture::changeset(vec![a, b], Vec::new())));
    assert!(report.graph.iter().any(|g| g.class == GraphClass::A005), "{}", messages(&report));
}

#[test]
fn every_authority_node_is_emitted_with_its_shape_triples() {
    let g = genesis("1", "steward");
    let mut cs = fixture::changeset(vec![g.clone()], vec![accepted("2", &g)]);
    cs.unavailabilities.push(away("3", &g, "2026-10-10T00:00:00Z", Some("2026-10-12T00:00:00Z")));
    let ttl = crate::graph::turtle::emit(&fixture::store(steward(), cs));
    for needle in [
        "<urn:ledger-role:steward>", "ledger:may \"accept-decision\"", "a ledger:Grant",
        "ledger:genesis \"true\"", "ledger:rank \"0\"^^xsd:integer", "a ledger:GrantAcceptance",
        "ledger:signsHash", "a ledger:Unavailability", "ledger:basis \"self\"",
    ] {
        assert!(ttl.contains(needle), "missing {needle}:\n{ttl}");
    }
}

#[test]
fn l006_refuses_a_model_revoker_in_either_revocation_shape() {
    for format in [1, 6] {
        let sealed = testkit::sealed(testkit::version());
        let acceptance = testkit::acceptance(&sealed);
        let mut cs = testkit::changeset(vec![sealed], vec![acceptance]);
        cs.format = format;
        let mut r = testkit::legacy_revocation("2026-10-02T09:00:00Z", "a model unsaying a person");
        if format == 6 {
            r.id = Some("rev:01K2C4YQJ3F8M0PT5W7NZ9RDY2".parse().expect("rev id"));
            r.revokes = r.acceptance.take().map(crate::authority::Revocable::Acceptance);
            r.actor = Some(testkit::identity("claude@anthropic.com"));
            r.by = None;
            r.hash = Some(crate::authority::payload::revocation_hash(&r));
        } else {
            r.by = Some(testkit::identity("noreply@anthropic.com"));
        }
        cs.revocations.push(r);
        let report = run(&testkit::store(cs));
        assert!(
            report.findings.iter().any(|f| f.class == VerifyClass::L006 && f.message.contains("revoker")),
            "format {format}: {}",
            messages(&report)
        );
    }
}
