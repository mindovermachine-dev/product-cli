//! Pinning what a judge was shown, so a verdict can be tied to a state.
//!
//! Rides `ledger_core`'s canonicalisation law under its own prefix — one law
//! in this workspace, never a second scheme. §6 of the format doc states the
//! canonical form normatively, because an implementation in another runtime
//! has to reproduce it byte for byte.

use ledger_core::canon::norm;
use ledger_core::hash::domain_hash;
use std::collections::BTreeMap;

/// Domain-separation prefix for a judgment's context digest.
pub const CONTEXT_FORM: &str = "eval.judgement-context.v1";

/// The unit separator between a key and its value.
const SEPARATOR: u8 = 0x1f;

/// The canonical body: the entries, in ascending key order, joined by newline.
///
/// A key whose value normalises to nothing is omitted entirely, so "absent"
/// and "present but empty" cannot pin differently — a distinction no reader
/// could act on is not one worth hashing.
fn body(shown: &BTreeMap<String, String>) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    for (key, value) in shown {
        let Some(normalised) = norm(value) else {
            continue;
        };
        if !out.is_empty() {
            out.push(b'\n');
        }
        out.extend_from_slice(key.as_bytes());
        out.push(SEPARATOR);
        out.extend_from_slice(normalised.as_bytes());
    }
    out
}

/// Digest what a judge was shown.
pub fn pin(shown: &BTreeMap<String, String>) -> String {
    domain_hash(CONTEXT_FORM, &body(shown))
}

/// Whether a digest still covers what its record says it covered.
///
/// Storing the inputs beside the digest is what lets a reader disbelieve a
/// verdict without reconstructing the run that produced it.
pub fn holds(shown: &BTreeMap<String, String>, digest: &str) -> bool {
    pin(shown) == digest
}

#[path = "digest_tests.rs"]
#[cfg(test)]
mod tests;
