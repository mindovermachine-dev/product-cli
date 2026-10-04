//! `A006` — every governed act, re-judged as of the act (D5 (a), D9 (d)).
//!
//! `verify` is the only gate that sees a file nobody made with the verb, so
//! the role check the verbs run is run again here, over history: every
//! acceptance and every revocation, against the authority records as they
//! stood at the act's position (D6, [`crate::authority::Authority::as_of`]),
//! through the same function the verbs call
//! ([`crate::authority::authorize_named`]). It checks the grant the act
//! names and never searches for another; a governed act that names none
//! fails.
//!
//! **Position rule** (D5 (c)). An act before its namespace's first policy is
//! not role-checked; every other act is. A grant's revocation is checked
//! once the store has a genesis.

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
            let pos = landing.position(&path, a.at);
            let auth = Authority::as_of(store, landing, pos);
            let ns = a.decision.namespace();
            let Some(policy) = auth.policy(ns) else { continue };
            let set = sets.get(&a.decision.to_string()).map(String::as_str).unwrap_or_default();
            let target = Target::Decision { namespace: ns, set };
            let id = a.id.to_string();
            let act = ActRef { subject: &id, actor: &a.actor, under: a.under.as_ref(), act: Act::Accept, at: a.at };
            out.extend(judge(&auth, &act, target, Some(&policy.accept_role)));
        }
        for r in logged.file.revocations.iter().filter(|r| r.is_entity()) {
            let (Some(target), Some(actor)) = (r.target(), r.actor()) else { continue };
            let id = r.subject();
            let act = |kind| ActRef { subject: &id, actor, under: r.under.as_ref(), act: kind, at: r.at };
            let pos = landing.position(&path, r.at);
            let auth = Authority::as_of(store, landing, pos);
            let verdict = match &target {
                Revocable::Acceptance(acc) => {
                    let Some(revoked) = store.log.iter().flat_map(|l| l.file.acceptances.iter()).find(|a| a.id == *acc) else { continue };
                    let ns = revoked.decision.namespace();
                    let Some(policy) = auth.policy(ns) else { continue };
                    let set = sets.get(&revoked.decision.to_string()).map(String::as_str).unwrap_or_default();
                    let t = Target::Decision { namespace: ns, set };
                    judge(&auth, &act(Act::RevokeAcceptance), t, Some(&policy.accept_role))
                }
                Revocable::Grant(g) => {
                    let Some(grant) = auth.grants.get(&g.to_string()).copied() else { continue };
                    if auth.genesis().is_none() {
                        continue;
                    }
                    let t = Target::Scope(&grant.scope);
                    judge(&auth, &act(Act::RevokeGrant), t, None)
                }
            };
            out.extend(verdict);
        }
    }
    out
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
