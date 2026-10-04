//! The `dsse` scheme — verification only (#65, D4).
//!
//! DSSE signing is the hosted service's; the open CLI verifies its
//! envelopes so a hosted ledger stays verifiable from the repository
//! alone. A sidecar `.decisions/sig/<ulid>.dsse.sig` holds one envelope:
//!
//! ```json
//! {"payloadType": "application/vnd.ledger.signed-bytes.v1",
//!  "payload": "<base64 of the entity's signed bytes>",
//!  "signatures": [{"keyid": "key:<ulid>", "sig": "<base64>"}]}
//! ```
//!
//! The payload must be exactly the entity's signed bytes. A signature is
//! ed25519 over DSSE's pre-authentication encoding (`DSSEv1 <len> <type>
//! <len> <payload>`), by a key the signer has bound in the namespace
//! (`ssh-ed25519` key bindings — the key material the policy's scheme is
//! checked against; windows are judged by the caller, as for `ssh`).

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::Deserialize;

/// The one payload type a ledger envelope carries.
pub const PAYLOAD_TYPE: &str = "application/vnd.ledger.signed-bytes.v1";

#[derive(Deserialize)]
struct Envelope {
    #[serde(rename = "payloadType")]
    payload_type: String,
    payload: String,
    signatures: Vec<EnvelopeSig>,
}

#[derive(Deserialize)]
struct EnvelopeSig {
    #[serde(default)]
    #[allow(dead_code)]
    keyid: String,
    sig: String,
}

/// DSSE's pre-authentication encoding.
pub fn pae(payload_type: &str, payload: &[u8]) -> Vec<u8> {
    let mut out = format!("DSSEv1 {} {} {} ", payload_type.len(), payload_type, payload.len()).into_bytes();
    out.extend_from_slice(payload);
    out
}

/// Verify an envelope over `message` by any of `keys` (OpenSSH base64
/// `ssh-ed25519` blobs). `Ok` names the key that verified.
pub fn verify(envelope: &[u8], message: &[u8], keys: &[String]) -> Result<String, String> {
    let env: Envelope = serde_json::from_slice(envelope).map_err(|e| format!("not a DSSE envelope: {e}"))?;
    if env.payload_type != PAYLOAD_TYPE {
        return Err(format!("payloadType is `{}`, not `{PAYLOAD_TYPE}`", env.payload_type));
    }
    let payload = base64_decode(&env.payload).ok_or("payload is not base64")?;
    if payload != message {
        return Err("the envelope's payload is not this entity's signed bytes".to_string());
    }
    let signed = pae(&env.payload_type, &payload);
    for key in keys {
        let Some(vk) = ed25519_key(key) else { continue };
        for s in &env.signatures {
            let Some(bytes) = base64_decode(&s.sig) else { continue };
            let Ok(sig) = Signature::from_slice(&bytes) else { continue };
            if vk.verify(&signed, &sig).is_ok() {
                return Ok(key.clone());
            }
        }
    }
    Err("no signature in the envelope verifies against the signer's bound ed25519 keys".to_string())
}

/// The 32-byte ed25519 key inside an OpenSSH `ssh-ed25519` blob.
pub fn ed25519_key(blob_base64: &str) -> Option<VerifyingKey> {
    let blob = base64_decode(blob_base64)?;
    let (kind, rest) = ssh_string(&blob)?;
    if kind != b"ssh-ed25519" {
        return None;
    }
    let (key, _) = ssh_string(rest)?;
    VerifyingKey::from_bytes(key.try_into().ok()?).ok()
}

/// The OpenSSH blob for an ed25519 key, base64 — the form a key binding holds.
pub fn ed25519_blob(key: &VerifyingKey) -> String {
    let mut blob = Vec::new();
    for part in [b"ssh-ed25519".as_slice(), key.as_bytes().as_slice()] {
        blob.extend_from_slice(&u32::try_from(part.len()).unwrap_or_default().to_be_bytes());
        blob.extend_from_slice(part);
    }
    base64_encode(&blob)
}

fn ssh_string(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    let len = u32::from_be_bytes(bytes.get(..4)?.try_into().ok()?) as usize;
    let body = bytes.get(4..4 + len)?;
    Some((body, bytes.get(4 + len..)?))
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard base64, padded.
pub fn base64_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk.iter().enumerate().fold(0u32, |acc, (i, b)| acc | (u32::from(*b) << (16 - 8 * i)));
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(char::from(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// Standard base64, padding optional; `None` on any other character.
pub fn base64_decode(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    let mut acc = 0u32;
    let mut bits = 0;
    for c in text.trim().bytes().filter(|b| *b != b'=') {
        let v = ALPHABET.iter().position(|a| *a == c)? as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(u8::try_from((acc >> bits) & 0xFF).ok()?);
        }
    }
    Some(out)
}

#[path = "dsse_tests.rs"]
#[cfg(test)]
mod tests;
