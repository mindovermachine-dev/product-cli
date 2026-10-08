//! Cross-entry schema rules for the authority records — what each record names must exist.
//!
//! The duplicate-id rule also covers the two ids outside the authority
//! records that a revocation or a reference resolves by: an acceptance's
//! id, and a decision's identity object (ruling 49).
//!
//! The authority twin of "a revocation naming an acceptance nobody filed":
//! every reference resolves, every acceptance of a grant is the holder's
//! and signs the grant's own hash, every unavailability is declared on the
//! basis it claims, every availability is the holder's, nothing is revoked
//! or closed twice, and a namespace's policy chain has one root and one
//! tip. All `SCHEMA` findings: a record that names nothing, or names the
//! wrong thing, does not describe an act.

use std::collections::{BTreeMap, BTreeSet};

use crate::finding::Finding;
use crate::store::Store;

use super::availability::{Basis, Unavailability};
use super::grant::{Grant, GrantScope, Order};
use super::revocation::Revocable;
use super::role::Capability;
use super::view::Authority;

/// Every cross-entry authority fault in the store.
pub fn faults(store: &Store, auth: &Authority<'_>) -> Vec<Finding> {
    let mut out = duplicate_ids(store);
    out.extend(grant_refs(auth));
    out.extend(unavailability_refs(auth));
    out.extend(revocation_refs(store, auth));
    out.extend(binding_refs(auth));
    out.extend(policy_refs(auth));
    out
}

fn fault(subject: &str, message: impl Into<String>) -> Finding {
    Finding::schema(subject, message)
}

/// Every id filed more than once: an authority record's, an acceptance's
/// (LP-8.8), or a decision identity object's, which appears in the one
/// change-set that introduces the decision (LP-5.11). An id that is not
/// unique cannot name what a revocation or a reference reaches (ruling 49).
fn duplicate_ids(store: &Store) -> Vec<Finding> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut introduced: BTreeMap<String, usize> = BTreeMap::new();
    for cs in store.log.iter().map(|l| &l.file) {
        for d in &cs.decisions {
            *introduced.entry(d.id.to_string()).or_default() += 1;
        }
        let ids = cs.acceptances.iter().map(|a| a.id.to_string())
            .chain(cs.grants.iter().map(|g| g.id.to_string()))
            .chain(cs.grant_acceptances.iter().map(|g| g.id.to_string()))
            .chain(cs.unavailabilities.iter().map(|u| u.id.to_string()))
            .chain(cs.availabilities.iter().map(|a| a.id.to_string()))
            .chain(cs.revocations.iter().filter_map(|r| r.id.as_ref().map(ToString::to_string)))
            .chain(cs.key_bindings.iter().map(|b| b.id.to_string()))
            .chain(cs.policies.iter().map(|p| p.id.to_string()));
        for id in ids {
            *seen.entry(id).or_default() += 1;
        }
    }
    let ids = seen.into_iter().filter(|(_, n)| *n > 1).map(|(id, _)| fault(&id, "this id is filed twice"));
    let decisions = introduced.into_iter().filter(|(_, n)| *n > 1).map(|(id, _)| {
        fault(&id, "this decision identity object is filed twice — a decision is introduced in one change-set only")
    });
    ids.chain(decisions).collect()
}

fn grant_refs(auth: &Authority<'_>) -> Vec<Finding> {
    let mut out = Vec::new();
    for g in auth.grants.values() {
        let id = g.id.to_string();
        if !auth.roles.contains_key(&g.role) {
            out.push(fault(&id, format!("names role `{}`, which is not declared under roles/", g.role)));
        }
        if let Some(old) = &g.supersedes {
            if *old == g.id || !auth.grants.contains_key(&old.to_string()) {
                out.push(fault(&id, format!("supersedes {old}, which is not another filed grant")));
            }
        }
    }
    for ga in &auth.grant_acceptances {
        let id = ga.id.to_string();
        match auth.grants.get(&ga.grant.to_string()) {
            None => out.push(fault(&id, format!("accepts {}, which is not a filed grant", ga.grant))),
            Some(g) if ga.actor != g.holder => {
                out.push(fault(&id, format!("is filed by {} but {} holds the grant — only the holder accepts", ga.actor, g.holder)))
            }
            Some(g) if ga.signs != g.hash => {
                out.push(fault(&id, format!("signs {}, which is not the grant's hash", ga.signs.short())))
            }
            Some(_) => {}
        }
    }
    out
}

fn unavailability_refs(auth: &Authority<'_>) -> Vec<Finding> {
    let mut out = Vec::new();
    for u in auth.unavailabilities.values() {
        let id = u.id.to_string();
        match auth.grants.get(&u.grant.to_string()) {
            None => out.push(fault(&id, format!("names {}, which is not a filed grant", u.grant))),
            Some(g) if !basis_holds(auth, u, g) => {
                out.push(fault(&id, format!("claims basis `{}`, which {} does not have", u.basis, u.by)))
            }
            Some(_) => {}
        }
    }
    let mut ended: BTreeSet<String> = BTreeSet::new();
    for av in &auth.availabilities {
        let id = av.id.to_string();
        let Some(u) = auth.unavailabilities.get(&av.ends.to_string()) else {
            out.push(fault(&id, format!("ends {}, which is not a filed unavailability", av.ends)));
            continue;
        };
        let holder = auth.grants.get(&u.grant.to_string()).map(|g| &g.holder);
        if holder != Some(&av.by) {
            out.push(fault(&id, "only the holder of the unavailable grant may end the interval"));
        }
        if av.available_at <= u.from {
            out.push(fault(&id, "`available_at` must fall after the interval's `from`"));
        }
        if !ended.insert(av.ends.to_string()) {
            out.push(fault(&id, format!("{} is already ended — an interval ends once", av.ends)));
        }
    }
    out
}

/// Whether the declarer stands in the relation the basis claims.
fn basis_holds(auth: &Authority<'_>, u: &Unavailability, g: &Grant) -> bool {
    match u.basis {
        Basis::SelfDeclared => u.by == g.holder,
        Basis::Grantor => u.by == g.granted_by,
        Basis::FallbackOfGenesis => {
            g.genesis
                && auth.grants.values().any(|fb| {
                    fb.holder == u.by
                        && fb.role == g.role
                        && fb.scope == GrantScope::All
                        && fb.order == Order::fallback(1)
                })
        }
    }
}

fn revocation_refs(store: &Store, auth: &Authority<'_>) -> Vec<Finding> {
    let filed_acceptances: BTreeSet<String> = store
        .log
        .iter()
        .flat_map(|l| l.file.acceptances.iter().map(|a| a.id.to_string()))
        .collect();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out = Vec::new();
    for r in &auth.revocations {
        let Some(target) = r.target() else { continue };
        let subject = r.subject();
        if let Revocable::Grant(g) = &target {
            if !auth.grants.contains_key(&g.to_string()) {
                out.push(fault(&subject, format!("revokes {g}, which is not a filed grant")));
            }
        }
        let known = match &target {
            Revocable::Grant(g) => auth.grants.contains_key(&g.to_string()),
            Revocable::Acceptance(a) => filed_acceptances.contains(&a.to_string()),
        };
        if known && !seen.insert(target.to_string()) {
            out.push(fault(&subject, format!("{target} is already revoked — a record is revoked once")));
        }
    }
    out
}

fn binding_refs(auth: &Authority<'_>) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut closed: BTreeSet<String> = BTreeSet::new();
    let mandates: BTreeSet<&str> =
        auth.grants.values().filter(|g| g.genesis).filter_map(|g| g.external_ref.as_deref()).collect();
    for b in &auth.bindings {
        let id = b.id.to_string();
        if auth.policies_of(&b.namespace).is_empty() {
            out.push(fault(&id, format!("binds a key in `{}`, a namespace with no policy — `ledger init --namespace` first", b.namespace)));
        }
        if b.self_bound && !b.mandate.as_deref().is_some_and(|m| mandates.contains(m)) {
            out.push(fault(&id, "a self-bound binding's mandate is the genesis grant's external ref"));
        }
        let Some(target) = &b.closes else { continue };
        let opened = auth.bindings.iter().find(|o| o.id == *target && o.act.opens());
        match opened {
            None => out.push(fault(&id, format!("closes {target}, which opens no key window"))),
            Some(o) if o.principal != b.principal || o.namespace != b.namespace => {
                out.push(fault(&id, format!("closes {target}, another principal's or namespace's key")))
            }
            Some(_) if !closed.insert(target.to_string()) => {
                out.push(fault(&id, format!("{target} is already closed — a window closes once")))
            }
            Some(_) => {}
        }
    }
    out
}

fn policy_refs(auth: &Authority<'_>) -> Vec<Finding> {
    let mut out = Vec::new();
    let namespaces: BTreeSet<&str> = auth.policies.iter().map(|p| p.namespace.as_str()).collect();
    for ns in namespaces {
        let chain = auth.policies_of(ns);
        let hashes: BTreeSet<String> = chain.iter().map(|p| p.hash.to_string()).collect();
        let roots = chain.iter().filter(|p| p.replaces.is_none()).count();
        if roots > 1 {
            out.push(fault(ns, "the namespace is initialised twice — later policies replace the one in force"));
        }
        if auth.policy_tips(ns) > 1 {
            out.push(fault(ns, "the policy chain forks — two policies replace one, and none is in force"));
        }
        for p in chain {
            let id = p.id.to_string();
            if p.replaces.as_ref().is_some_and(|h| !hashes.contains(&h.to_string())) {
                out.push(fault(&id, "replaces a hash that is no policy of this namespace"));
            }
            let capable = auth.roles.get(&p.accept_role).is_some_and(|r| r.may(Capability::AcceptDecision));
            if !capable {
                out.push(fault(&id, format!("maps accept-decision to `{}`, which is no declared role that may accept", p.accept_role)));
            }
        }
    }
    out
}
