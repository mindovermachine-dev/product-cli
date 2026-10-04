//! Which grant an act is made under (D9 (b), (c), ruled 2026-10-02).
//!
//! The candidates are the actor's grants that pass the role check for this
//! act and target ([`super::check::candidates`]). With one candidate it is
//! used, and the verb shows it in its confirmation. With several, `--as
//! <role>` is required; within the named role the narrowest covering scope
//! wins, then the lowest rank. The chosen grant is then refused when
//! another candidate's role carries strictly fewer capabilities — always
//! the role with the fewest claims (D9 (c)). Fewest-claims is the verb's
//! rule only: acting under a broader role confers no power the actor
//! lacks, so `verify` does not re-judge it.

use std::collections::BTreeSet;

use super::check::{Authorized, Denial};
use super::grant::{Grant, GrantScope};
use super::view::Authority;

/// How wide a scope reaches: narrower sorts first.
fn breadth(scope: &GrantScope) -> u8 {
    match scope {
        GrantScope::Set(_) | GrantScope::Pattern(_) => 0,
        GrantScope::Namespace(_) => 1,
        GrantScope::All => 2,
    }
}

/// Pick the grant among `found` (all passing), honouring `as_role`.
pub fn choose(
    auth: &Authority<'_>,
    found: &[&Grant],
    as_role: Option<&str>,
) -> Result<Authorized, Denial> {
    let pool: Vec<&Grant> = match as_role {
        Some(role) => found.iter().copied().filter(|g| g.role == role).collect(),
        None if found.len() > 1 => {
            let roles: BTreeSet<String> = found.iter().map(|g| g.role.clone()).collect();
            return Err(Denial::Ambiguous(roles.into_iter().collect()));
        }
        None => found.to_vec(),
    };
    let chosen = pool
        .iter()
        .copied()
        .min_by(|a, b| {
            (breadth(&a.scope), a.order.rank(), a.id.to_string())
                .cmp(&(breadth(&b.scope), b.order.rank(), b.id.to_string()))
        })
        .ok_or_else(|| Denial::NotInRole(as_role.unwrap_or_default().to_string()))?;
    let claims = |role: &str| auth.roles.get(role).map_or(usize::MAX, |r| r.may.len());
    let mine = claims(&chosen.role);
    if let Some(narrower) = found.iter().find(|g| claims(&g.role) < mine) {
        return Err(Denial::Broader {
            grant: chosen.id.to_string(),
            role: chosen.role.clone(),
            narrower: narrower.role.clone(),
        });
    }
    Ok(Authorized { grant: chosen.id.to_string(), role: chosen.role.clone(), genesis: chosen.genesis })
}

#[path = "choice_tests.rs"]
#[cfg(test)]
mod tests;
