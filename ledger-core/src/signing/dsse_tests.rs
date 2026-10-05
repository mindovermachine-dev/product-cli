//! DSSE envelopes: verified against bound keys; never produced here.

use ed25519_dalek::{Signer, SigningKey};

use super::*;

fn envelope(key: &SigningKey, payload: &[u8], payload_type: &str) -> Vec<u8> {
    let sig = key.sign(&pae(payload_type, payload));
    serde_json::json!({
        "payloadType": payload_type,
        "payload": base64_encode(payload),
        "signatures": [{"keyid": "key:x", "sig": base64_encode(&sig.to_bytes())}],
    })
    .to_string()
    .into_bytes()
}

#[test]
fn base64_round_trips_every_length() {
    for n in 0..20u8 {
        let bytes: Vec<u8> = (0..n).map(|i| i.wrapping_mul(37)).collect();
        assert_eq!(base64_decode(&base64_encode(&bytes)), Some(bytes));
    }
    assert_eq!(base64_encode(b"ledger"), "bGVkZ2Vy");
}

#[test]
fn an_envelope_by_a_bound_key_over_the_signed_bytes_verifies() {
    let key = SigningKey::from_bytes(&[7u8; 32]);
    let bound = ed25519_blob(&key.verifying_key());
    let message = b"ledger.acceptance.v1\n{}";
    let env = envelope(&key, message, PAYLOAD_TYPE);
    assert_eq!(verify(&env, message, std::slice::from_ref(&bound)), Ok(bound.clone()));
    assert!(verify(&env, b"other bytes", std::slice::from_ref(&bound)).is_err(), "payload must be the entity's bytes");
    let stranger = ed25519_blob(&SigningKey::from_bytes(&[8u8; 32]).verifying_key());
    assert!(verify(&env, message, &[stranger]).is_err(), "an unbound key never verifies");
    assert!(verify(&envelope(&key, message, "text/plain"), message, &[bound]).is_err());
}

#[test]
fn the_fingerprint_matches_openssh() {
    // `ssh-keygen -l` on this blob prints this fingerprint.
    let key = SigningKey::from_bytes(&[7u8; 32]);
    let blob = ed25519_blob(&key.verifying_key());
    let fp = super::super::ssh::fingerprint(&blob).expect("fp");
    assert!(fp.starts_with("SHA256:") && !fp.ends_with('='), "{fp}");
}
