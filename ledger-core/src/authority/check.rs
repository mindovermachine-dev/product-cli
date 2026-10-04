//! The role check — may this actor do this act over this scope, now?
//!
//! One function, [`authorize`], which `accept`, `revoke` and the grant
//! verbs all call, and one, [`authorize_named`], which `verify` calls to
//! re-check the grant an act names (`A006`, D5/D9 (d)). The actor needs a
//! grant that is, in order:
//!
//! 1. held by the actor, of a role that `may` the act's capability (and,
//!    when the caller names one, of that role — a namespace policy maps
//!    `accept-decision` to one role);
//! 2. over a scope covering the act's target;
//! 3. unrevoked and unsuperseded;
//! 4. accepted by its holder;
//! 5. available at the act's time (no unavailability covers it);
//! 6. if a fallback, not limited from this act, and **active**: no live,
//!    available grant of the same role at a lower rank covers the act's
//!    target (D9 (e), ruled 2026-10-02). A fallback acts only when those
//!    before it cannot. The comparison is per target through [`covers`],
//!    not by identical scope strings: a `fallback-1` over a set waits on a
//!    primary over `*` in its role. Another role never outranks, and equal
//!    rank acts concurrently.
//!
//! When no grant passes, the refusal names the furthest any grant got, so
//! "you hold the role but have not accepted it" reads differently from
//! "you hold nothing here". When several pass, [`super::choice`] picks the
//! one the act is made under (D9 (b), (c)).

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
    /// Several grants qualify in these roles; `--as <role>` must choose.
    Ambiguous(Vec<String>),
    /// `--as` named a role in which no grant qualifies.
    NotInRole(String),
    /// The chosen grant's role claims more than another qualifying role.
    Broader { grant: String, role: String, narrower: String },
    /// The grant an act names is not filed, or is not the actor's.
    NotTheirs(String),
    /// `grant new` below the genesis gives only the grantor's own role;
    /// these are the grants held that could grant, none in that role.
    OwnRoleOnly(Vec<String>),
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
            Self::Ambiguous(roles) => write!(
                f,
                "holds grants that qualify in {} — name the role with `--as <role>`",
                roles.join(", ")
            ),
            Self::NotInRole(r) => write!(f, "holds no grant of `{r}` that may do this here"),
            Self::Broader { grant, role, narrower } => write!(
                f,
                "would act under {grant} (`{role}`), but `{narrower}` also qualifies and claims a subset of it — act as `{narrower}`"
            ),
            Self::NotTheirs(g) => write!(f, "names {g}, which is not a filed grant of theirs"),
            Self::OwnRoleOnly(held) => {
                write!(f, "acts under {} and may grant only that role — the genesis grants others", held.join(", "))
            }
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

impl Authorized {
    fn of(grant: &Grant) -> Self {
        Self { grant: grant.id.to_string(), role: grant.role.clone(), genesis: grant.genesis }
    }

    /// The confirmation line every governed verb prints (D9 (b)).
    pub fn line(&self) -> String {
        format!("under {} (`{}`)", self.grant, self.role)
    }
}

/// May `actor` do `act` over `target` at `at`, and under which grant?
/// `role`, when given, is the one role whose grants count (the policy's
/// `accept_role`); `as_role` is the actor's `--as` (D9 (b)).
pub fn authorize(
    auth: &Authority<'_>,
    actor: &Identity,
    act: Act,
    target: Target<'_>,
    at: DateTime<Utc>,
    role: Option<&str>,
    as_role: Option<&str>,
) -> Result<Authorized, Denial> {
    let found = candidates(auth, actor, act, target, at, role)?;
    super::choice::choose(auth, &found, as_role)
}

/// Every grant of the actor's that passes the check: the candidates an act
/// may be made under. Empty is the furthest denial.
pub fn candidates<'a>(
    auth: &Authority<'a>,
    actor: &Identity,
    act: Act,
    target: Target<'_>,
    at: DateTime<Utc>,
    role: Option<&str>,
) -> Result<Vec<&'a Grant>, Denial> {
    let mut best = Denial::NoGrant;
    let mut found = Vec::new();
    for grant in auth.grants.values().copied().filter(|g| g.holder == *actor) {
        let capable = auth.roles.get(&grant.role).is_some_and(|r| r.may(act.capability()));
        if !capable || role.is_some_and(|r| r != grant.role) {
            continue;
        }
        match judge(auth, grant, act, target, at) {
            Ok(()) => found.push(grant),
            Err(denial) => best = best.max(denial),
        }
    }
    if found.is_empty() {
        return Err(best);
    }
    Ok(found)
}

/// The check on the grant an act names (D9 (d)): held by the actor, of a
/// role that may do it (in `role`, when given), and passing every step as
/// of `at` in `auth` — which the caller builds as of the act's position
/// (D6). Never searches for another grant.
pub fn authorize_named(
    auth: &Authority<'_>,
    actor: &Identity,
    named: &str,
    act: Act,
    target: Target<'_>,
    at: DateTime<Utc>,
    role: Option<&str>,
) -> Result<Authorized, Denial> {
    let grant = auth
        .grants
        .get(named)
        .copied()
        .filter(|g| g.holder == *actor)
        .ok_or_else(|| Denial::NotTheirs(named.to_string()))?;
    let capable = auth.roles.get(&grant.role).is_some_and(|r| r.may(act.capability()));
    if !capable || role.is_some_and(|r| r != grant.role) {
        return Err(Denial::NoGrant);
    }
    judge(auth, grant, act, target, at).map(|()| Authorized::of(grant))
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
    match outranking(auth, grant, target, at) {
        Some(before) => Err(Denial::Outranked(id, before)),
        None => Ok(()),
    }
}

/// A grant of the same role at a lower rank, covering the target, that can
/// act now.
fn outranking(auth: &Authority<'_>, grant: &Grant, target: Target<'_>, at: DateTime<Utc>) -> Option<String> {
    auth.grants
        .values()
        .filter(|g| g.role == grant.role && covers(&g.scope, target))
        .filter(|g| g.order.rank() < grant.order.rank())
        .find(|g| auth.is_live(g) && auth.is_available(g, at))
        .map(|g| g.id.to_string())
}

#[path = "check_tests.rs"]
#[cfg(test)]
mod tests;
