//! `unavailable` / `available` — filing a holder's absence, or ending it early.
//!
//! A holder files their own unavailability freely (basis `self`). Filing
//! someone else's needs `declare-unavailability` over the grant's scope,
//! and a basis the declarer stands in: the grantor, or — against the
//! genesis grant only — the first fallback of the genesis role. Ending an
//! interval early is the holder's act alone, whoever declared it, and is a
//! record of its own: the interval is never edited.

use chrono::{DateTime, Utc};

use crate::authority::{Act, Authority, Availability, Basis, Grant, GrantScope, Order, Target, Unavailability};
use crate::id::{GrantId, UnavailabilityId};

use super::{Applied, Author, AuthorError};

/// What an unavailability states.
pub struct UnavailableArgs {
    pub grant: GrantId,
    pub from: DateTime<Utc>,
    pub until: Option<DateTime<Utc>>,
    pub reason: Option<String>,
}

impl Author {
    /// File an unavailability interval on a grant.
    pub fn unavailable(&mut self, args: UnavailableArgs) -> Result<Applied, AuthorError> {
        let store = self.load();
        let auth = Authority::build(&store);
        let grant = auth.grants.get(&args.grant.to_string()).copied().cloned().ok_or_else(|| {
            AuthorError::Usage(format!("{} is not a filed grant", args.grant))
        })?;
        let basis = self.basis(&auth, &grant)?;
        if basis != Basis::SelfDeclared {
            self.authorized(&store, Act::DeclareUnavailability, Target::Scope(&grant.scope), None)?;
        }
        let interval = Unavailability {
            id: self.mint.mint_id("unav").map_err(AuthorError::Io)?,
            grant: grant.id.clone(),
            from: args.from,
            until: args.until,
            basis,
            reason: args.reason,
            by: self.who.clone(),
            at: self.now,
        };
        let line = format!("{} unavailable from {} (basis `{basis}`) — {}", grant.holder, args.from, interval.id);
        let mut candidate = self.shell(None)?;
        candidate.unavailabilities.push(interval);
        self.refusal_check(&store, &candidate, |_| false)?;
        let path = self.append(&candidate)?;
        Ok(Applied { path, lines: vec![line] })
    }

    /// The basis this actor stands in for this grant, or a refusal.
    fn basis(&self, auth: &Authority<'_>, grant: &Grant) -> Result<Basis, AuthorError> {
        if grant.holder == self.who {
            return Ok(Basis::SelfDeclared);
        }
        if grant.granted_by == self.who && !grant.genesis {
            return Ok(Basis::Grantor);
        }
        let first_fallback = auth.grants.values().any(|fb| {
            fb.holder == self.who
                && fb.role == grant.role
                && fb.scope == GrantScope::All
                && fb.order == Order::fallback(1)
                && auth.is_live(fb)
        });
        if grant.genesis && first_fallback {
            return Ok(Basis::FallbackOfGenesis);
        }
        Err(AuthorError::Unauthorized(format!(
            "{} is neither the holder nor the grantor of {}, nor the genesis role's first fallback",
            self.who, grant.id
        )))
    }

    /// End an unavailability early — the holder's act.
    pub fn available(&mut self, id: &UnavailabilityId, at: Option<DateTime<Utc>>) -> Result<Applied, AuthorError> {
        let store = self.load();
        let auth = Authority::build(&store);
        let interval = auth.unavailabilities.get(&id.to_string()).copied().cloned().ok_or_else(|| {
            AuthorError::Usage(format!("{id} is not a filed unavailability"))
        })?;
        let holder = auth.grants.get(&interval.grant.to_string()).map(|g| g.holder.clone());
        if holder.as_ref() != Some(&self.who) {
            return Err(AuthorError::Unauthorized(format!("only the holder of {} ends {id}", interval.grant)));
        }
        if auth.availabilities.iter().any(|a| a.ends == *id) {
            return Err(AuthorError::Conflict(format!("{id} is already ended")));
        }
        let ending = Availability {
            id: self.mint.mint_id("avail").map_err(AuthorError::Io)?,
            ends: id.clone(),
            available_at: at.unwrap_or(self.now),
            by: self.who.clone(),
            at: self.now,
        };
        let line = format!("{} available again from {} — ends {id}", self.who, ending.available_at);
        let mut candidate = self.shell(None)?;
        candidate.availabilities.push(ending);
        self.refusal_check(&store, &candidate, |_| false)?;
        let path = self.append(&candidate)?;
        Ok(Applied { path, lines: vec![line] })
    }
}
