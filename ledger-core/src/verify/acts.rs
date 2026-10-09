//! `A006` — every governed act, re-judged as of the act (D5 (a), D9 (d)).
//!
//! `verify` is the only gate that sees a file nobody made with the verb, so
//! the role check the verbs run is run again here, over history: every
//! acceptance and every revocation, against the authority records as they
//! stood at the act's position (D6, [`crate::authority::Authority::as_of`])
//! in the act's own namespace (ruling 47), through the same function the verbs call
//! ([`crate::authority::authorize_named`]). It checks the grant the act
//! names and never searches for another; a governed act that names none
//! fails. A policy, first or change, is the genesis holder's act: the
//! grant it names must be the genesis grant as of the policy.
//!
//! **Position rule** (D5 (c)). An act before its namespace's first policy is
//! not role-checked; every other act is. A grant's revocation is checked
//! once the store has a genesis. An old-style revocation (formats 1–5, no
//! id, no `under`) is valid only before the first policy; one that is not
//! before it fails.

use crate::authority::{authorize_named, Act, Authority, Revocable, Target};
use crate::graph::{GraphClass, GraphFinding};
use crate::landing::{relative, Landing};
use crate::store::Store;

fn finding(subject: &str, message: String) -> GraphFinding {
    GraphFinding { class: GraphClass::A006, subject: subject.to_string(), message }
}

/// Every `A006` finding in the store.
pub fn unauthorised(store: &Store, landing: &Landing) -> Vec<GraphFinding> {
    let sets = decision_sets(store);
    let mut out = Vec::new();
    for logged in &store.log {
        let path = relative(&store.root, &logged.path);
        for a in &logged.file.acceptances {
            let pos = landing.position(&path, &crate::landed::key("acceptances", &a.id.to_string()), a.at);
            let ns = a.decision.namespace();
            let auth = Authority::as_of(store, landing, pos, ns);
            let Some(policy) = auth.policy(ns) else { continue };
            let set = sets.get(&a.decision.to_string()).map(String::as_str).unwrap_or_default();
            let target = Target::Decision { namespace: ns, set };
            let id = a.id.to_string();
            let act = ActRef { subject: &id, actor: &a.actor, under: a.under.as_ref(), act: Act::Accept, at: a.at };
            out.extend(judge(&auth, &act, target, Some(&policy.accept_role)));
        }
        for r in &logged.file.revocations {
            out.extend(revocation_verdict(store, landing, &sets, (&path, &logged.namespace), r));
        }
        for p in &logged.file.policies {
            out.extend(policy_verdict(store, landing, &path, p));
        }
    }
    out
}

/// One policy, first or change, judged as of its position (D6): the genesis
/// holder's act, so the grant it names (`under`) must be the genesis grant
/// as of the policy, held by its `by`, live and available at its `at`. A
/// signature says only who filed it, not that they were the one who may.
fn policy_verdict(store: &Store, landing: &Landing, path: &str, p: &crate::authority::Policy) -> Option<GraphFinding> {
    let id = p.id.to_string();
    let pos = landing.position(path, &crate::landed::key("policies", &id), p.at);
    let auth = Authority::as_of(store, landing, pos, &p.namespace);
    let Some(genesis) = auth.genesis().map(|g| g.id.clone()) else {
        return Some(finding(&id, format!(
            "{} sets `{}`'s policy with no live genesis grant of `{}` as of the policy — a policy is the genesis holder's act, and each namespace has its own genesis (LP-6.31, ruling 47)",
            p.by, p.namespace, p.namespace
        )));
    };
    if let Some(under) = p.under.as_ref().filter(|u| **u != genesis) {
        return Some(finding(&id, format!(
            "{} sets `{}`'s policy under {under}, which is not the genesis grant ({genesis}) as of the policy — a policy is the genesis holder's act",
            p.by, p.namespace
        )));
    }
    let scope = crate::authority::GrantScope::Namespace(p.namespace.clone());
    let act = ActRef { subject: &id, actor: &p.by, under: p.under.as_ref(), act: Act::SetPolicy, at: p.at };
    judge(&auth, &act, Target::Scope(&scope), None)
}

/// One revocation, either shape, judged as of its position. An old-style
/// revocation (`acceptance`, `by`) names no grant and can carry no
/// signature, so it is valid only before its namespace's first policy; one
/// that is not before it (D6) fails like a `rev:` revocation naming none.
/// `(path, dir)` is the revocation's file and the namespace whose directory
/// holds it: a grant's revocation is judged in that namespace's authority.
fn revocation_verdict(
    store: &Store,
    landing: &Landing,
    sets: &std::collections::BTreeMap<String, String>,
    (path, dir): (&str, &str),
    r: &crate::authority::Revocation,
) -> Option<GraphFinding> {
    let (Some(target), Some(actor)) = (r.target(), r.actor()) else { return None };
    let id = r.subject();
    let act = |kind| ActRef { subject: &id, actor, under: r.under.as_ref(), act: kind, at: r.at };
    let pos = landing.position(path, &crate::landed::revocation_key(r), r.at);
    match &target {
        Revocable::Acceptance(acc) => {
            let revoked = store.log.iter().flat_map(|l| l.file.acceptances.iter()).find(|a| a.id == *acc)?;
            let ns = revoked.decision.namespace();
            let auth = Authority::as_of(store, landing, pos, ns);
            let policy = auth.policy(ns)?;
            if !r.is_entity() {
                return Some(finding(&id, format!(
                    "an old-style revocation (acceptance, by) by {actor} is not before `{ns}`'s first policy (D6) — the old shape is valid only as a pre-policy act; a governed revocation is a `rev:` entity naming the grant it is made under"
                )));
            }
            let set = sets.get(&revoked.decision.to_string()).map(String::as_str).unwrap_or_default();
            let t = Target::Decision { namespace: ns, set };
            judge(&auth, &act(Act::RevokeAcceptance), t, Some(&policy.accept_role))
        }
        Revocable::Grant(g) => {
            let auth = Authority::as_of(store, landing, pos, dir);
            let grant = auth.grants.get(&g.to_string()).copied()?;
            auth.genesis()?;
            judge(&auth, &act(Act::RevokeGrant), Target::Scope(&grant.scope), None)
        }
    }
}

/// One act, as `A006` reads it.
struct ActRef<'a> {
    subject: &'a str,
    actor: &'a crate::identity::Identity,
    under: Option<&'a crate::id::GrantId>,
    act: Act,
    at: chrono::DateTime<chrono::Utc>,
}

fn judge(auth: &Authority<'_>, a: &ActRef<'_>, target: Target<'_>, role: Option<&str>) -> Option<GraphFinding> {
    let Some(under) = a.under else {
        return Some(finding(a.subject, format!("a governed act names no grant (`under`) — {} acts under one", a.actor)));
    };
    authorize_named(auth, a.actor, &under.to_string(), a.act, target, a.at, role)
        .err()
        .map(|d| finding(a.subject, format!("{} {d}, as of the act ({})", a.actor, a.act.as_str())))
}

/// Each decision's set, from its latest-filed version.
fn decision_sets(store: &Store) -> std::collections::BTreeMap<String, String> {
    store
        .log
        .iter()
        .flat_map(|l| l.file.versions.iter())
        .map(|v| (v.decision.to_string(), v.set.clone()))
        .collect()
}
