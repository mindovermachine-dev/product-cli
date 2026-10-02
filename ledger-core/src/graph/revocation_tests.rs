//! Revocation as its own node (ruling 3, #66): the acceptance never changes.

use std::collections::BTreeSet;

use crate::authority::payload::revocation_hash;
use crate::authority::{Revocable, Revocation};
use crate::testkit;

use super::turtle::emit;

fn accepted() -> crate::changeset::ChangeSet {
    let sealed = testkit::sealed(testkit::version());
    let acceptance = testkit::acceptance(&sealed);
    testkit::changeset(vec![sealed], vec![acceptance])
}

fn entity() -> Revocation {
    let mut r = Revocation {
        id: Some("rev:01K2C4YQJ3F8M0PT5W7NZ9RDY2".parse().expect("rev id")),
        revokes: Some(Revocable::Acceptance(testkit::acceptance_id())),
        acceptance: None,
        at: testkit::stamp("2026-10-02T09:00:00Z"),
        actor: Some(testkit::identity("fixture-human@example")),
        by: None,
        reason: "filed against the wrong version".into(),
        hash: None,
    };
    r.hash = Some(revocation_hash(&r));
    r
}

/// Every Turtle statement whose subject is the acceptance node.
fn acceptance_triples(ttl: &str) -> BTreeSet<String> {
    let subject = format!("<urn:{}>", testkit::acceptance_id());
    let block = ttl.split("\n\n").find(|b| b.trim_start().starts_with(&subject)).unwrap_or_default();
    block.lines().map(|l| l.trim().trim_end_matches([';', '.']).trim().to_string()).collect()
}

#[test]
fn the_acceptance_node_triple_set_is_fixed_across_a_revocation() {
    let before = acceptance_triples(&emit(&testkit::store(accepted())));
    assert!(before.iter().any(|t| t.contains("ledger:signsVersion")), "{before:?}");
    for revocation in [entity(), testkit::legacy_revocation("2026-08-11T09:00:00Z", "wrong version")] {
        let mut cs = accepted();
        cs.format = if revocation.is_entity() { 6 } else { 1 };
        cs.revocations.push(revocation);
        let ttl = emit(&testkit::store(cs));
        assert_eq!(acceptance_triples(&ttl), before, "no triple is added to an acceptance:\n{ttl}");
        assert!(!ttl.contains("ledger:revokedAt") && !ttl.contains("ledger:revokedBy"), "{ttl}");
        assert!(ttl.contains("a ledger:Revocation"), "{ttl}");
        assert!(ttl.contains(&format!("ledger:revokes <urn:{}>", testkit::acceptance_id())), "{ttl}");
    }
}

#[test]
fn an_entity_revocation_carries_its_id_and_every_payload_field() {
    let mut cs = accepted();
    cs.format = 6;
    let r = entity();
    let hash = r.hash.clone().expect("sealed");
    cs.revocations.push(r);
    let ttl = emit(&testkit::store(cs));
    for needle in [
        "<urn:rev:01K2C4YQJ3F8M0PT5W7NZ9RDY2>".to_string(),
        "ledger:id \"rev:01K2C4YQJ3F8M0PT5W7NZ9RDY2\"".to_string(),
        format!("ledger:hash \"{hash}\""),
        "ledger:revocationReason \"filed against the wrong version\"".to_string(),
        "prov:wasAttributedTo <mailto:fixture-human@example>".to_string(),
        "prov:generatedAtTime \"2026-10-02T09:00:00Z\"^^xsd:dateTime".to_string(),
    ] {
        assert!(ttl.contains(&needle), "missing {needle}:\n{ttl}");
    }
}

#[test]
fn a_legacy_revocation_gets_a_node_minted_from_the_acceptance_it_revokes() {
    let mut cs = accepted();
    cs.revocations.push(testkit::legacy_revocation("2026-08-11T09:00:00Z", "wrong version"));
    let ttl = emit(&testkit::store(cs));
    let node = format!("<urn:rev:legacy-{}>", testkit::ACC_ULID);
    assert!(ttl.contains(&node), "{ttl}");
    assert!(ttl.contains("ledger:hash \"sha256:"), "the payload digest is computable: {ttl}");
}

#[test]
fn the_revocation_payload_is_the_closed_four() {
    let a = entity();
    let mut moved = a.clone();
    moved.reason = "another reason".into();
    assert_ne!(revocation_hash(&a), revocation_hash(&moved));
    let mut renamed = a.clone();
    renamed.id = Some("rev:01K2C4YQJ3F8M0PT5W7NZ9RDY3".parse().expect("rev id"));
    assert_eq!(revocation_hash(&a), revocation_hash(&renamed), "the id is not in the payload");
}
