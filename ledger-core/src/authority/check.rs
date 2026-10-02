//! The role check — may this actor do this act over this scope, now?
//!
//! One function, [`authorize`], which `accept`, `revoke` and the grant
//! verbs all call. The actor needs a grant that is, in order:
//!
//! 1. held by the actor, of a role that `may` the act's capability (and,
//!    when the caller names one, of that role — a namespace policy maps
//!    `accept-decision` to one role);
//! 2. over a scope covering the act's target;
//! 3. unrevoked and unsuperseded;
//! 4. accepted by its holder;
//! 5. available at the act's time (no unavailability covers it);
//! 6. if a fallback, not limited from this act, and **active**: every
//!    standing, accepted, available grant of the same role and scope at a
//!    lower rank is absent. A fallback acts only when those before it
//!    cannot.
//!
//! When no grant passes, the refusal names the furthest any grant got, so
//! "you hold the role but have not accepted it" reads differently from
//! "you hold nothing here".

use std::fmt;

use chrono::{DateTime, Utc};

use crate::identity::Identity;

use super::grant::{Grant, GrantScope};
use super::role::Act;
use super::view::Authority;

/// What an act is over.
#[derive(Debug, Clone, Copy)]
pub enum Target<'t> {
    /// A decision, by its namespace and set.
    Decision { namespace: &'t str, set: &'t str },
    /// A grant scope — granting or revoking over it.
    Scope(&'t GrantScope),
}

/// Whether a grant's scope covers a target. `*` covers everything; a
/// namespace scope covers its own decisions and its own namespace scope; a
/// set scope covers the set's decisions and its own set scope; a pattern
/// scope covers only itself. Namespaces match exactly — `ns:a` does not
/// cover `a.b` (a hierarchy reading is a ruling not yet made).
pub fn covers(scope: &GrantScope, target: Target<'_>) -> bool {
    match (scope, target) {
        (GrantScope::All, _) => true,
        (GrantScope::Namespace(n), Target::Decision { namespace, .. }) => n == namespace,
        (GrantScope::Set(s), Target::Decision { set, .. }) => s == set,
        (GrantScope::Pattern(_), Target::Decision { .. }) => false,
        (own, Target::Scope(wanted)) => own == wanted,
    }
}

/// Why the check refused, ordered by how far the best grant got.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Denial {
    /// No grant held by the actor carries the capability (in the role).
    NoGrant,
    /// The actor's grants carry the capability, but none covers the target.
    WrongScope(String),
    /// The covering grant is revoked or superseded.
    NotStanding(String),
    /// The covering grant was never accepted by its holder.
    NotAccepted(String),
    /// The covering grant's holder is unavailable at the act's time.
    Unavailable(String),
    /// The covering grant is a fallback limited from this act.
    Limited(String, &'static str),
    /// The covering grant is a fallback, and a grant before it can act.
    Outranked(String, String),
}

impl fmt::Display for Denial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoGrant => f.write_str("holds no grant of a role that may do this"),
            Self::WrongScope(g) => write!(f, "holds {g}, whose scope does not cover this"),
            Self::NotStanding(g) => write!(f, "holds {g}, which is revoked or superseded"),
            Self::NotAccepted(g) => write!(f, "holds {g} but has not accepted it (`ledger grant accept {g}`)"),
            Self::Unavailable(g) => write!(f, "holds {g} but is filed unavailable now"),
            Self::Limited(g, l) => write!(f, "holds fallback {g}, which carries the limit `{l}`"),
            Self::Outranked(g, by) => write!(f, "holds fallback {g}, but {by} ranks before it and can act"),
        }
    }
}

/// The grant an act was authorised under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authorized {
    pub grant: String,
    pub role: String,
    pub genesis: bool,
}

/// May `actor` do `act` over `target` at `at`? `role`, when given, is the
/// one role whose grants count (the policy's `accept_role`).
pub fn authorize(
    auth: &Authority<'_>,
    actor: &Identity,
    act: Act,
    target: Target<'_>,
    at: DateTime<Utc>,
    role: Option<&str>,
) -> Result<Authorized, Denial> {
    let mut best = Denial::NoGrant;
    for grant in auth.grants.values().filter(|g| g.holder == *actor) {
        let capable = auth.roles.get(&grant.role).is_some_and(|r| r.may(act.capability()));
        if !capable || role.is_some_and(|r| r != grant.role) {
            continue;
        }
        match judge(auth, grant, act, target, at) {
            Ok(()) => {
                return Ok(Authorized {
                    grant: grant.id.to_string(),
                    role: grant.role.clone(),
                    genesis: grant.genesis,
                })
            }
            Err(denial) => best = best.max(denial),
        }
    }
    Err(best)
}

/// Steps 2–6 for one capable grant.
fn judge(
    auth: &Authority<'_>,
    grant: &Grant,
    act: Act,
    target: Target<'_>,
    at: DateTime<Utc>,
) -> Result<(), Denial> {
    let id = grant.id.to_string();
    if !covers(&grant.scope, target) {
        return Err(Denial::WrongScope(id));
    }
    if !auth.is_standing(grant) {
        return Err(Denial::NotStanding(id));
    }
    if !auth.is_accepted(grant) {
        return Err(Denial::NotAccepted(id));
    }
    if !auth.is_available(grant, at) {
        return Err(Denial::Unavailable(id));
    }
    if grant.order.is_primary() {
        return Ok(());
    }
    if let Some(limit) = act.withheld_by().filter(|l| grant.limits.contains(l)) {
        return Err(Denial::Limited(id, limit.as_str()));
    }
    match outranking(auth, grant, at) {
        Some(before) => Err(Denial::Outranked(id, before)),
        None => Ok(()),
    }
}

/// A grant of the same role and scope at a lower rank that can act now.
fn outranking(auth: &Authority<'_>, grant: &Grant, at: DateTime<Utc>) -> Option<String> {
    auth.grants
        .values()
        .filter(|g| g.role == grant.role && g.scope == grant.scope)
        .filter(|g| g.order.rank() < grant.order.rank())
        .find(|g| auth.is_live(g) && auth.is_available(g, at))
        .map(|g| g.id.to_string())
}

#[path = "check_tests.rs"]
#[cfg(test)]
mod tests;
