//! The gate's `dsse` path, end to end over an in-memory store: an envelope
//! by a bound key verifies; a tampered one, or one by an unbound key, is
//! `L011`.

use ed25519_dalek::{Signer, SigningKey};

use crate::authority::fixture::{self, accepted, genesis, role};
use crate::authority::payload::{acceptance_bytes, binding_bytes, binding_hash, policy_hash};
use crate::authority::{BindingAct, Capability, KeyBinding, Policy, Scheme};
use crate::finding::VerifyClass;
use crate::hash::VersionHash;
use crate::landing::Landing;
use crate::signing::dsse::{base64_encode, ed25519_blob, pae, PAYLOAD_TYPE};
use crate::signing::Sidecar;
use crate::testkit;

use super::check;

const NS: &str = "fixture.ledger";

fn envelope(key: &SigningKey, bytes: &[u8]) -> Vec<u8> {
    let sig = key.sign(&pae(PAYLOAD_TYPE, bytes));
    serde_json::json!({
        "payloadType": PAYLOAD_TYPE,
        "payload": base64_encode(bytes),
        "signatures": [{"keyid": "", "sig": base64_encode(&sig.to_bytes())}],
    })
    .to_string()
    .into_bytes()
}

fn sidecar(ulid: &str, bytes: Vec<u8>) -> Sidecar {
    Sidecar { ulid: ulid.to_string(), scheme: Scheme::Dsse, file: Sidecar::file_name(ulid, Scheme::Dsse), bytes }
}

fn policy(g: &crate::authority::Grant) -> Policy {
    let mut policy = Policy {
        id: format!("pol:{}", fixture::ulid("3")).parse().expect("id"),
        namespace: NS.into(),
        schemes: vec![Scheme::Dsse],
        require_sk: false,
        accept_role: "acceptor".into(),
        reaccept_within_days: None,
        replaces: None,
        by: testkit::identity(fixture::GENESIS_HOLDER),
        under: Some(g.id.clone()),
        at: testkit::stamp("2026-10-01T09:00:00Z"),
        hash: VersionHash::zero(),
    };
    policy.hash = policy_hash(&policy);
    policy
}

fn self_bound(g: &crate::authority::Grant, bound: &SigningKey) -> KeyBinding {
    let holder = testkit::identity(fixture::GENESIS_HOLDER);
    let mut binding = KeyBinding {
        id: format!("key:{}", fixture::ulid("4")).parse().expect("id"),
        act: BindingAct::Add,
        principal: holder.clone(),
        namespace: NS.into(),
        key_type: Some("ssh-ed25519".into()),
        key: Some(ed25519_blob(&bound.verifying_key())),
        closes: None,
        self_bound: true,
        mandate: g.external_ref.clone(),
        by: holder,
        under: None,
        at: testkit::stamp("2026-10-01T09:10:00Z"),
        hash: VersionHash::zero(),
    };
    binding.hash = binding_hash(&binding);
    binding
}

fn store(signer: &SigningKey, tamper: bool) -> crate::store::Store {
    let g = genesis("1", "steward");
    let bound = SigningKey::from_bytes(&[7u8; 32]);
    let binding = self_bound(&g, &bound);
    let mut version = testkit::version();
    version.decision = format!("dec:{NS}/01K2C4YQJ3F8M0PT5W7NZ9RDXA").parse().expect("dec");
    let version = testkit::sealed(version);
    let mut acc = testkit::acceptance(&version);
    acc.actor = testkit::identity(fixture::GENESIS_HOLDER);
    acc.at = testkit::stamp("2026-10-02T09:00:00Z");
    let mut acc_bytes = acceptance_bytes(&acc);
    if tamper {
        acc_bytes.push(b' ');
    }
    let mut cs = fixture::changeset(vec![g.clone()], vec![accepted("2", &g)]);
    cs.policies = vec![policy(&g)];
    cs.key_bindings = vec![binding.clone()];
    cs.versions = vec![version];
    cs.acceptances = vec![acc.clone()];
    let mut s = fixture::store(vec![role("steward", Capability::ROOT), role("acceptor", &[Capability::AcceptDecision])], cs);
    s.sidecars = vec![
        sidecar(binding.id.ulid(), envelope(&bound, &binding_bytes(&binding))),
        sidecar(acc.id.ulid(), envelope(signer, &acc_bytes)),
    ];
    s
}

#[test]
fn a_dsse_envelope_by_the_bound_key_verifies() {
    let key = SigningKey::from_bytes(&[7u8; 32]);
    let s = store(&key, false);
    let out = check(&s, &Landing::unknown(), testkit::date("2026-10-15"));
    assert!(out.findings.is_empty(), "{:?}", out.findings);
    assert_eq!(out.trusted.len(), 1, "the self-bound binding is trusted");
}

#[test]
fn a_tampered_envelope_or_an_unbound_key_is_l011() {
    let key = SigningKey::from_bytes(&[7u8; 32]);
    let s = store(&key, true);
    let tampered = check(&s, &Landing::unknown(), testkit::date("2026-10-15"));
    assert!(tampered.findings.iter().any(|f| f.class == VerifyClass::L011), "{:?}", tampered.findings);
    let stranger = SigningKey::from_bytes(&[9u8; 32]);
    let s = store(&stranger, false);
    let unbound = check(&s, &Landing::unknown(), testkit::date("2026-10-15"));
    assert!(unbound.findings.iter().any(|f| f.class == VerifyClass::L011), "{:?}", unbound.findings);
}
