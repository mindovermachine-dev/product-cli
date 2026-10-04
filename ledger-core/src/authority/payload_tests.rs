//! The payloads' digests: pinned against the code before spec v1.8, so no
//! format-6 digest moves, and each format-7 field moves only its own.

use crate::authority::fixture::{self, genesis, grant};
use crate::authority::{BindingAct, KeyBinding, Policy, Revocable, Revocation, Scheme};
use crate::hash::VersionHash;
use crate::testkit;

use super::*;

// Computed by `main` at 88b3de1 (before `under` and the policy's `at`)
// over exactly these records.
const PIN_GRANT: &str = "sha256:fe78d33b3b80a9e3d4d93a271881736f34240586becd8448d1613cea260c316e";
const PIN_GENESIS: &str = "sha256:72b02e659b9bb331bd4cdc8e509b547e57bf84093058e2f2832076a821b3cc46";
const PIN_POLICY: &str = "sha256:15acaafd3955bfdc55dcbdfdfc8f644189b64f19556be86b136e18171e6e6fff";
const PIN_BINDING: &str = "sha256:79c8cedb7a884f74805e829fecd9874223e6f2ad3009f5d5194effa3ff1d436a";
const PIN_REVOCATION: &str = "sha256:60bf0b25a02a61cd0dac4accdb2e942224d4a3cee9e3e039a566e4b13f5f3666";

fn policy() -> Policy {
    Policy {
        id: format!("pol:{}", fixture::ulid("3")).parse().expect("id"),
        namespace: "hafeok.ledger".into(),
        schemes: vec![Scheme::Ssh],
        require_sk: true,
        accept_role: "acceptor".into(),
        reaccept_within_days: Some(30),
        replaces: None,
        by: testkit::identity(fixture::GENESIS_HOLDER),
        under: None,
        at: testkit::stamp("2026-10-01T09:00:00Z"),
        hash: VersionHash::zero(),
        at_hashed: false,
    }
}

fn binding() -> KeyBinding {
    KeyBinding {
        id: format!("key:{}", fixture::ulid("4")).parse().expect("id"),
        act: BindingAct::Add,
        principal: testkit::identity(fixture::GENESIS_HOLDER),
        namespace: "hafeok.ledger".into(),
        key_type: Some("ssh-ed25519".into()),
        key: Some("AAAAC3NzaC1lZDI1NTE5AAAAIabc".into()),
        closes: None,
        self_bound: true,
        mandate: Some("contract 2026/117".into()),
        by: testkit::identity(fixture::GENESIS_HOLDER),
        under: None,
        at: testkit::stamp("2026-10-01T09:10:00Z"),
        hash: VersionHash::zero(),
    }
}

fn revocation() -> Revocation {
    Revocation {
        id: Some(format!("rev:{}", fixture::ulid("5")).parse().expect("id")),
        revokes: Some(Revocable::Grant(grant("1", "architect", "architect@customer.example", "ns:hafeok.ledger", 1).id)),
        acceptance: None,
        at: testkit::stamp("2026-10-02T00:00:00Z"),
        actor: Some(testkit::identity(fixture::GENESIS_HOLDER)),
        by: None,
        reason: "moved".into(),
        under: None,
        hash: None,
    }
}

fn under() -> crate::id::GrantId {
    format!("grant:{}", fixture::ulid("9")).parse().expect("grant id")
}

#[test]
fn no_format_6_digest_moves() {
    assert_eq!(grant_hash(&grant("1", "architect", "architect@customer.example", "ns:hafeok.ledger", 1)).to_string(), PIN_GRANT);
    assert_eq!(grant_hash(&genesis("2", "steward")).to_string(), PIN_GENESIS);
    assert_eq!(policy_hash(&policy()).to_string(), PIN_POLICY);
    assert_eq!(binding_hash(&binding()).to_string(), PIN_BINDING);
    assert_eq!(revocation_hash(&revocation()).to_string(), PIN_REVOCATION);
}

#[test]
fn under_moves_the_digest_of_every_payload_that_names_it() {
    let mut g = grant("1", "architect", "architect@customer.example", "ns:hafeok.ledger", 1);
    g.under = Some(under());
    assert_ne!(grant_hash(&g).to_string(), PIN_GRANT);
    let mut p = policy();
    p.under = Some(under());
    assert_ne!(policy_hash(&p).to_string(), PIN_POLICY);
    let mut b = binding();
    b.under = Some(under());
    assert_ne!(binding_hash(&b).to_string(), PIN_BINDING);
    let mut r = revocation();
    r.under = Some(under());
    assert_ne!(revocation_hash(&r).to_string(), PIN_REVOCATION);
}

#[test]
fn a_format_7_policy_hashes_its_at_and_a_format_6_one_does_not() {
    let mut p = policy();
    p.at = testkit::stamp("2026-10-03T00:00:00Z");
    assert_eq!(policy_hash(&p).to_string(), PIN_POLICY, "format 6: `at` is outside the payload");
    p.at_hashed = true;
    let first = policy_hash(&p);
    assert_ne!(first.to_string(), PIN_POLICY);
    p.at = testkit::stamp("2026-10-04T00:00:00Z");
    assert_ne!(policy_hash(&p), first, "format 7: re-dating a policy moves its hash");
    let text = String::from_utf8(policy_bytes(&p)).expect("utf-8");
    assert!(text.contains("\"at\":\"2026-10-04T00:00:00Z\""), "{text}");
}

#[test]
fn the_acceptance_payload_is_the_closed_list_and_omits_absent_fields() {
    let v = testkit::sealed(testkit::version());
    let mut a = testkit::acceptance(&v);
    a.expires_at = None;
    let text = String::from_utf8(acceptance_bytes(&a)).expect("utf-8");
    let (prefix, json) = text.split_once('\n').expect("prefix line");
    assert_eq!(prefix, ACCEPTANCE_FORM);
    let value: serde_json::Value = serde_json::from_str(json).expect("json");
    let keys: Vec<&String> = value.as_object().expect("object").keys().collect();
    assert_eq!(keys, ["actor", "at", "decision", "scope", "version"], "absent expires_at and under are omitted");
    let plain = acceptance_hash(&a);
    a.under = Some(under());
    a.expires_at = Some(testkit::date("2027-01-01"));
    let text = String::from_utf8(acceptance_bytes(&a)).expect("utf-8");
    assert!(text.contains("\"expires_at\":\"2027-01-01\"") && text.contains("\"under\":\"grant:"), "{text}");
    assert_ne!(acceptance_hash(&a), plain);
    assert_eq!(acceptance_hash(&a).to_string(), crate::hash::domain_hash(ACCEPTANCE_FORM, json_of(&a).as_bytes()));
}

fn json_of(a: &crate::acceptance::Acceptance) -> String {
    let text = String::from_utf8(acceptance_bytes(a)).expect("utf-8");
    text.split_once('\n').map(|(_, j)| j.to_string()).unwrap_or_default()
}
