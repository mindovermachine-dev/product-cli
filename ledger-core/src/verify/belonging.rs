//! Every entity under `ns/<ns>/` belongs to `<ns>` (LP-3.33, ruling 45).
//!
//! A file's namespace is its directory (LP-3.34), so an entity that names
//! another namespace is in the wrong file: a decision id, the decision a
//! version or an acceptance names, a binding's or a policy's `namespace`
//! field, a revocation's target, the grant a grant acceptance or an
//! interval names, the interval an availability ends, and a sidecar under a
//! namespace other than its entity's. Each is a `SCHEMA` fault, no class
//! added. A record whose target is not filed is already a fault
//! (`authority::references`) and is left alone here.
//!
//! A change-set a verb is about to append may be homed nowhere when it
//! names nothing the store holds (`author::home`); such a candidate carries
//! an empty namespace and is judged by the finding that says why, not here.

use crate::authority::Revocable;
use crate::finding::Finding;
use crate::store::{LoggedChangeSet, Store};

/// Every entity filed outside its namespace's directory.
pub fn findings(store: &Store) -> Vec<Finding> {
    let mut out = Vec::new();
    for logged in store.log.iter().filter(|l| !l.namespace.is_empty()) {
        out.extend(own_entities(logged));
        out.extend(named_records(store, logged));
    }
    out.extend(sidecars(store));
    out
}

fn fault(subject: &str, what: &str, theirs: &str, dir: &str) -> Finding {
    Finding::schema(
        subject,
        format!(
            "{what} `{theirs}`, but the file is under `{dir}`'s directory — every entity in `ns/{dir}/` belongs to `{dir}` (LP-3.33, ruling 45)"
        ),
    )
}

/// The entities that carry their namespace themselves.
fn own_entities(logged: &LoggedChangeSet) -> Vec<Finding> {
    let dir = logged.namespace.as_str();
    let cs = &logged.file;
    let mut out = Vec::new();
    for d in cs.decisions.iter().filter(|d| d.id.namespace() != dir) {
        out.push(fault(&d.id.to_string(), "is a decision of", d.id.namespace(), dir));
    }
    for v in cs.versions.iter().filter(|v| v.decision.namespace() != dir) {
        out.push(fault(&v.decision.to_string(), &format!("version {} is of a decision of", v.hash.short()), v.decision.namespace(), dir));
    }
    for a in cs.acceptances.iter().filter(|a| a.decision.namespace() != dir) {
        out.push(fault(&a.id.to_string(), "accepts a decision of", a.decision.namespace(), dir));
    }
    for b in cs.key_bindings.iter().filter(|b| b.namespace != dir) {
        out.push(fault(&b.id.to_string(), "binds a key in", &b.namespace, dir));
    }
    for p in cs.policies.iter().filter(|p| p.namespace != dir) {
        out.push(fault(&p.id.to_string(), "is the policy of", &p.namespace, dir));
    }
    out
}

/// The records that belong where what they name belongs.
fn named_records(store: &Store, logged: &LoggedChangeSet) -> Vec<Finding> {
    let dir = logged.namespace.as_str();
    let cs = &logged.file;
    let mut out = Vec::new();
    for r in &cs.revocations {
        let (subject, what, theirs) = match r.target() {
            Some(Revocable::Acceptance(acc)) => (r.subject(), "revokes an acceptance of", acceptance_namespace(store, &acc.to_string())),
            Some(Revocable::Grant(g)) => (r.subject(), "revokes a grant filed under", directory_of(store, |cs| cs.grants.iter().any(|x| x.id == g))),
            None => continue,
        };
        if let Some(theirs) = theirs.filter(|t| t != dir) {
            out.push(fault(&subject, what, &theirs, dir));
        }
    }
    for ga in &cs.grant_acceptances {
        if let Some(theirs) = directory_of(store, |cs| cs.grants.iter().any(|g| g.id == ga.grant)).filter(|t| t != dir) {
            out.push(fault(&ga.id.to_string(), "accepts a grant filed under", &theirs, dir));
        }
    }
    for u in &cs.unavailabilities {
        if let Some(theirs) = directory_of(store, |cs| cs.grants.iter().any(|g| g.id == u.grant)).filter(|t| t != dir) {
            out.push(fault(&u.id.to_string(), "names a grant filed under", &theirs, dir));
        }
    }
    for av in &cs.availabilities {
        if let Some(theirs) = directory_of(store, |cs| cs.unavailabilities.iter().any(|u| u.id == av.ends)).filter(|t| t != dir) {
            out.push(fault(&av.id.to_string(), "ends an interval filed under", &theirs, dir));
        }
    }
    out
}

/// A sidecar sits under the namespace of the entity it signs.
fn sidecars(store: &Store) -> Vec<Finding> {
    store
        .sidecars
        .iter()
        .filter(|c| !c.namespace.is_empty())
        .filter_map(|c| {
            let theirs = signable_namespace(store, &c.ulid)?;
            (theirs != c.namespace).then(|| fault(&format!("sig/{}", c.file), "signs an entity of", &theirs, &c.namespace))
        })
        .collect()
}

/// The namespace of the acceptance `id`: its decision's.
fn acceptance_namespace(store: &Store, id: &str) -> Option<String> {
    store
        .log
        .iter()
        .flat_map(|l| l.file.acceptances.iter())
        .find(|a| a.id.to_string() == id)
        .map(|a| a.decision.namespace().to_string())
}

/// The directory of the change-set some predicate picks out.
fn directory_of(store: &Store, holds: impl Fn(&crate::changeset::ChangeSet) -> bool) -> Option<String> {
    store.log.iter().find(|l| holds(&l.file)).map(|l| l.namespace.clone())
}

/// The namespace a signable entity `ulid` belongs to: an acceptance's
/// decision's, a revocation's target's, a binding's or a policy's own.
fn signable_namespace(store: &Store, ulid: &str) -> Option<String> {
    for cs in store.log.iter().map(|l| &l.file) {
        if let Some(a) = cs.acceptances.iter().find(|a| a.id.ulid() == ulid) {
            return Some(a.decision.namespace().to_string());
        }
        if let Some(r) = cs.revocations.iter().find(|r| r.id.as_ref().is_some_and(|i| i.ulid() == ulid)) {
            return match r.target()? {
                Revocable::Acceptance(acc) => acceptance_namespace(store, &acc.to_string()),
                Revocable::Grant(g) => directory_of(store, |cs| cs.grants.iter().any(|x| x.id == g)),
            };
        }
        if let Some(b) = cs.key_bindings.iter().find(|b| b.id.ulid() == ulid) {
            return Some(b.namespace.clone());
        }
        if let Some(p) = cs.policies.iter().find(|p| p.id.ulid() == ulid) {
            return Some(p.namespace.clone());
        }
    }
    None
}

#[path = "belonging_tests.rs"]
#[cfg(test)]
mod tests;
