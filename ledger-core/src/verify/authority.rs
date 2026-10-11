//! The file gate's reading of the authority records (spec v1.7).
//!
//! No new class: the authority records are policed by the classes that
//! already mean what is wrong with them, each extended by the `L010`
//! amendment mechanism (stricter, additive):
//!
//! - `SCHEMA` — a record that is malformed alone
//!   ([`crate::authority::structure`]) or names what does not exist or does
//!   not fit ([`crate::authority::references`]);
//! - `L006` — every identity an authority record attributes an act to, or
//!   gives authority to, is refused by the model/bot rule: a grant's holder
//!   and grantor, a grant acceptance's actor, an unavailability's and an
//!   availability's declarer, a revocation's actor, a key binding's
//!   principal and filer, a policy's author. A model is never a holder;
//! - `L007` — a stored hash (grant, revocation, key binding, policy) that
//!   does not equal the recomputed canonical hash.

use crate::authority::payload::{binding_hash, grant_hash, policy_hash, revocation_hash};
use crate::authority::{references, structure, Authority};
use crate::finding::{Finding, VerifyClass};
use crate::hash::VersionHash;
use crate::identity::Identity;
use crate::store::Store;

/// Every authority finding in the store, across the three classes.
pub fn findings(store: &Store) -> Vec<Finding> {
    let auth = Authority::build(store);
    let mut out: Vec<Finding> = store.log.iter().flat_map(|l| structure::entry_faults(&l.file, &l.namespace)).collect();
    out.extend(references::faults(store));
    out.extend(model_actors(&auth));
    out.extend(hash_mismatches(&auth));
    out
}

/// `L006` over the authority records.
fn model_actors(auth: &Authority<'_>) -> Vec<Finding> {
    let mut named: Vec<(String, &str, &Identity)> = Vec::new();
    for g in auth.grants.values() {
        named.push((g.id.to_string(), "grant holder", &g.holder));
        named.push((g.id.to_string(), "grantor", &g.granted_by));
    }
    named.extend(auth.grant_acceptances.iter().map(|ga| (ga.id.to_string(), "grant acceptor", &ga.actor)));
    named.extend(auth.unavailabilities.values().map(|u| (u.id.to_string(), "declarer", &u.by)));
    named.extend(auth.availabilities.iter().map(|a| (a.id.to_string(), "declarer", &a.by)));
    named.extend(auth.revocations.iter().filter_map(|r| r.actor().map(|a| (r.subject(), "revoker", a))));
    for b in &auth.bindings {
        named.push((b.id.to_string(), "key principal", &b.principal));
        named.push((b.id.to_string(), "binding filer", &b.by));
    }
    named.extend(auth.policies.iter().map(|p| (p.id.to_string(), "policy author", &p.by)));
    named
        .into_iter()
        .filter_map(|(subject, role, who)| {
            who.model_or_bot_reason()
                .map(|why| Finding::new(VerifyClass::L006, &subject, format!("{role} {why}")))
        })
        .collect()
}

/// `L007` over the authority records' stored hashes.
fn hash_mismatches(auth: &Authority<'_>) -> Vec<Finding> {
    let mut pairs: Vec<(String, &VersionHash, VersionHash)> = Vec::new();
    pairs.extend(auth.grants.values().map(|g| (g.id.to_string(), &g.hash, grant_hash(g))));
    pairs.extend(
        auth.revocations
            .iter()
            .filter_map(|r| r.hash.as_ref().map(|h| (r.subject(), h, revocation_hash(r)))),
    );
    pairs.extend(auth.bindings.iter().map(|b| (b.id.to_string(), &b.hash, binding_hash(b))));
    pairs.extend(auth.policies.iter().map(|p| (p.id.to_string(), &p.hash, policy_hash(p))));
    pairs
        .into_iter()
        .filter(|(_, stored, computed)| *stored != computed)
        .map(|(subject, stored, computed)| {
            Finding::new(
                VerifyClass::L007,
                &subject,
                format!("stored hash {} does not match content, which hashes to {computed}", stored.short()),
            )
        })
        .collect()
}

#[path = "authority_tests.rs"]
#[cfg(test)]
mod tests;
