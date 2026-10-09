//! The genesis role never accepts (D9 (f), ruling 50) — held at verification, not at the writer only.
//!
//! The genesis (root) role acts on the authority structure; accepting is a
//! separate role, granted separately (LP-6.17). The writers refuse both
//! breaches, but a hand-written store could carry them, and then the
//! genesis grant alone accepted. Both are `SCHEMA` here:
//!
//! - a role that a genesis grant names, carrying a decision capability;
//! - a policy whose `accept_role` is **the genesis role at the policy's
//!   position**: the role of the genesis grant live as of that policy (D6,
//!   [`Authority::as_of`]), which is the grant `A006` judges the policy's
//!   own author against.

use std::collections::BTreeSet;

use crate::authority::Authority;
use crate::finding::Finding;
use crate::landing::{relative, Landing};
use crate::store::Store;

/// Every genesis-role fault in the store.
pub(crate) fn findings(store: &Store, landing: &Landing) -> Vec<Finding> {
    let mut out = decision_capable_roots(store);
    for logged in &store.log {
        let path = relative(&store.root, &logged.path);
        for p in &logged.file.policies {
            let id = p.id.to_string();
            let pos = landing.position(&path, &crate::landed::key("policies", &id), p.at);
            let Some(genesis) = Authority::as_of(store, landing, pos, &p.namespace).genesis() else { continue };
            if genesis.role == p.accept_role {
                out.push(Finding::schema(&id, format!(
                    "maps accept-decision to `{}`, the genesis role as of the policy ({}) — the accept role is never the genesis role (LP-6.17, D9 (f))",
                    p.accept_role, genesis.id
                )));
            }
        }
    }
    out
}

/// Each role a genesis grant names, in the grant's own namespace, that
/// carries a decision capability.
fn decision_capable_roots(store: &Store) -> Vec<Finding> {
    let roots: BTreeSet<(&str, &str)> = store
        .log
        .iter()
        .flat_map(|l| l.file.grants.iter().map(move |g| (l.namespace.as_str(), g)))
        .filter(|(_, g)| g.genesis)
        .map(|(ns, g)| (ns, g.role.as_str()))
        .collect();
    roots
        .into_iter()
        .filter_map(|(ns, id)| store.role_in(ns, id))
        .filter_map(|role| {
            let decision: Vec<&str> = role.decision_capabilities().into_iter().map(|c| c.as_str()).collect();
            (!decision.is_empty()).then(|| {
                Finding::schema(&role.id, format!(
                    "is the genesis role and carries {} — the root role carries none of the decision capabilities (LP-6.17, D9 (f))",
                    decision.join(", ")
                ))
            })
        })
        .collect()
}
