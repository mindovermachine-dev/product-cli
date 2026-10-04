//! The hashed payloads of the authority records and the acceptance — one
//! law, five prefixes.
//!
//! Every payload is a closed field list canonicalised by the format's one
//! canonical-JSON law ([`crate::canon::put`] / [`crate::canon::put_set`],
//! `ledger-format-v1.md` §4.2) and digested by [`crate::hash::domain_hash`]
//! under its own domain-separation prefix. Each builder destructures its
//! record with **no `..` rest pattern**, so a field added to a record is a
//! compile error here until someone decides whether it is hashed — the
//! guard `canon::canonical_json` already carries for versions.
//!
//! What is outside each payload: the stored `hash` itself (circular), and
//! the record's own `at` except on a revocation, an acceptance, a key
//! binding, and (D8, format 7) a policy. `under` (D9 (a), format 7) is in
//! every payload that names it, hashed when present and omitted when
//! absent, so no digest filed before it moves.
//!
//! Each `*_bytes` function returns the exact byte sequence its `*_hash`
//! digests — [`crate::hash::signed_bytes`] over the canonical JSON — which
//! is what a signature signs.

use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::{Map, Value};

use crate::canon::{put, put_set};
use crate::hash::{domain_hash, VersionHash};

use crate::acceptance::Acceptance;
use crate::hash::signed_bytes;

use super::binding::KeyBinding;
use super::grant::Grant;
use super::policy::Policy;
use super::revocation::Revocation;

/// The grant payload's prefix.
pub const GRANT_FORM: &str = "ledger.authority-grant.v1";
/// The revocation payload's prefix.
pub const REVOCATION_FORM: &str = "ledger.revocation.v1";
/// The key-binding payload's prefix.
pub const BINDING_FORM: &str = "ledger.identity-binding.v1";
/// The namespace-policy payload's prefix.
pub const POLICY_FORM: &str = "ledger.namespace-policy.v1";
/// The acceptance payload's prefix (spec v1.8).
pub const ACCEPTANCE_FORM: &str = "ledger.acceptance.v1";

/// The lexical form every hashed instant takes: RFC 3339, whole seconds,
/// `Z`. The export carries `at` in exactly this form.
pub fn stamp(at: &DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn flag(b: bool) -> Option<String> {
    b.then(|| "true".to_string())
}

fn seal(prefix: &str, map: Map<String, Value>) -> VersionHash {
    let digest = domain_hash(prefix, Value::Object(map).to_string().as_bytes());
    // A digest this module just rendered always parses; the fallback keeps
    // the function total without an unwrap.
    digest.parse().unwrap_or_else(|_| VersionHash::zero())
}

/// The canonical JSON a grant hashes over.
pub fn grant_json(g: &Grant) -> String {
    Value::Object(grant_map(g)).to_string()
}

fn bytes(prefix: &str, map: Map<String, Value>) -> Vec<u8> {
    signed_bytes(prefix, Value::Object(map).to_string().as_bytes())
}

fn grant_map(g: &Grant) -> Map<String, Value> {
    let Grant {
        id, role, scope, holder, granted_by, order, limits, genesis, external_ref, supersedes,
        under, at: _, hash: _,
    } = g;
    let mut m = Map::new();
    put(&mut m, "id", Some(id.to_string()));
    put(&mut m, "role", Some(role.clone()));
    put(&mut m, "scope", Some(scope.to_string()));
    put(&mut m, "holder", Some(holder.to_string()));
    put(&mut m, "granted_by", Some(granted_by.to_string()));
    put(&mut m, "order", Some(order.to_string()));
    put_set(&mut m, "limits", limits.iter().map(ToString::to_string));
    put(&mut m, "genesis", flag(*genesis));
    put(&mut m, "external_ref", external_ref.clone());
    put(&mut m, "supersedes", supersedes.as_ref().map(ToString::to_string));
    put(&mut m, "under", under.as_ref().map(ToString::to_string));
    m
}

/// A grant's digest.
pub fn grant_hash(g: &Grant) -> VersionHash {
    seal(GRANT_FORM, grant_map(g))
}

/// A revocation's closed payload `{revokes, actor, at, reason}` plus
/// `under`. Computable for the legacy shape too, which is how a
/// pre-format-6 revocation gets a content identity in the graph without a
/// stored hash.
fn revocation_map(r: &Revocation) -> Map<String, Value> {
    let Revocation { id: _, revokes: _, acceptance: _, at, actor: _, by: _, reason, under, hash: _ } = r;
    let mut m = Map::new();
    put(&mut m, "revokes", r.target().map(|t| t.to_string()));
    put(&mut m, "actor", r.actor().map(ToString::to_string));
    put(&mut m, "at", Some(stamp(at)));
    put(&mut m, "reason", Some(reason.clone()));
    put(&mut m, "under", under.as_ref().map(ToString::to_string));
    m
}

pub fn revocation_hash(r: &Revocation) -> VersionHash {
    seal(REVOCATION_FORM, revocation_map(r))
}

pub fn revocation_bytes(r: &Revocation) -> Vec<u8> {
    bytes(REVOCATION_FORM, revocation_map(r))
}

fn binding_map(b: &KeyBinding) -> Map<String, Value> {
    let KeyBinding {
        id, act, principal, namespace, key_type, key, closes, self_bound, mandate, by, under, at,
        hash: _,
    } = b;
    let mut m = Map::new();
    put(&mut m, "id", Some(id.to_string()));
    put(&mut m, "act", Some(act.to_string()));
    put(&mut m, "principal", Some(principal.to_string()));
    put(&mut m, "namespace", Some(namespace.clone()));
    put(&mut m, "key_type", key_type.clone());
    put(&mut m, "key", key.clone());
    put(&mut m, "closes", closes.as_ref().map(ToString::to_string));
    put(&mut m, "self_bound", flag(*self_bound));
    put(&mut m, "mandate", mandate.clone());
    put(&mut m, "by", Some(by.to_string()));
    put(&mut m, "under", under.as_ref().map(ToString::to_string));
    // A binding's window opens (or closes) at its own time, so the time is
    // content here: re-dating a binding must move its hash.
    put(&mut m, "at", Some(stamp(at)));
    m
}

/// A key binding's digest.
pub fn binding_hash(b: &KeyBinding) -> VersionHash {
    seal(BINDING_FORM, binding_map(b))
}

pub fn binding_bytes(b: &KeyBinding) -> Vec<u8> {
    bytes(BINDING_FORM, binding_map(b))
}

fn policy_map(p: &Policy) -> Map<String, Value> {
    let Policy {
        id, namespace, schemes, require_sk, accept_role, reaccept_within_days, replaces, by,
        under, at, hash: _, at_hashed,
    } = p;
    let mut m = Map::new();
    put(&mut m, "id", Some(id.to_string()));
    put(&mut m, "namespace", Some(namespace.clone()));
    put_set(&mut m, "schemes", schemes.iter().map(ToString::to_string));
    put(&mut m, "require_sk", flag(*require_sk));
    put(&mut m, "accept_role", Some(accept_role.clone()));
    put(&mut m, "reaccept_within_days", reaccept_within_days.map(|d| d.to_string()));
    put(&mut m, "replaces", replaces.as_ref().map(ToString::to_string));
    put(&mut m, "by", Some(by.to_string()));
    put(&mut m, "under", under.as_ref().map(ToString::to_string));
    // D8: the change's time is its `-Overify-time`, so from format 7 it is
    // content. A format-6 policy keeps the payload it was hashed under.
    put(&mut m, "at", at_hashed.then(|| stamp(at)));
    m
}

/// A policy's digest.
pub fn policy_hash(p: &Policy) -> VersionHash {
    seal(POLICY_FORM, policy_map(p))
}

pub fn policy_bytes(p: &Policy) -> Vec<u8> {
    bytes(POLICY_FORM, policy_map(p))
}

/// The acceptance's closed payload (ruling 4 as amended by D9):
/// `{decision, version, actor, at, scope, expires_at, under}`. `scope` is
/// its wire form (`version` or `class:<ref>`), `expires_at` a calendar
/// date. Acceptances carried no digest before spec v1.8, so none moves;
/// the digest is computed, never stored.
fn acceptance_map(a: &Acceptance) -> Map<String, Value> {
    let Acceptance { id: _, decision, version, actor, at, scope, expires_at, under, signature: _ } = a;
    let mut m = Map::new();
    put(&mut m, "decision", Some(decision.to_string()));
    put(&mut m, "version", Some(version.to_string()));
    put(&mut m, "actor", Some(actor.to_string()));
    put(&mut m, "at", Some(stamp(at)));
    put(&mut m, "scope", Some(scope.to_string()));
    put(&mut m, "expires_at", expires_at.map(|d| d.format("%Y-%m-%d").to_string()));
    put(&mut m, "under", under.as_ref().map(ToString::to_string));
    m
}

/// An acceptance's content hash.
pub fn acceptance_hash(a: &Acceptance) -> VersionHash {
    seal(ACCEPTANCE_FORM, acceptance_map(a))
}

pub fn acceptance_bytes(a: &Acceptance) -> Vec<u8> {
    bytes(ACCEPTANCE_FORM, acceptance_map(a))
}

#[path = "payload_tests.rs"]
#[cfg(test)]
mod tests;
