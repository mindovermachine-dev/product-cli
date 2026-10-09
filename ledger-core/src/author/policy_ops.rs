//! `policy set` — filing the next version of a namespace's policy.
//!
//! A policy change is a log entry naming the hash of the policy it
//! replaces, filed by the live, available genesis holder. The ruling (#65)
//! is that a change is *signed under the policy in force before it*;
//! signing is Session B's, so until then the entry carries the replaced
//! hash and the check that it was signed under that policy is deferred
//! (recorded in `ledger/sessions/2026-10-session-a.md`).

use crate::authority::payload::policy_hash;
use crate::authority::{Authority, Policy, Scheme};

use super::{Applied, Author, AuthorError};

/// What a policy change states; `None` keeps the policy in force's value.
pub struct PolicyArgs {
    pub namespace: String,
    pub schemes: Option<Vec<Scheme>>,
    pub require_sk: Option<bool>,
    pub accept_role: Option<String>,
    /// `Some(None)` clears the deadline.
    pub reaccept_within_days: Option<Option<u32>>,
}

impl Author {
    /// A policy change is the act of the namespace's own live, available
    /// genesis holder (LP-6.28, ruling 47), and it never maps acceptance to
    /// the genesis role (D9 (f)).
    fn refuse_policy_change(&self, auth: &Authority<'_>, args: &PolicyArgs) -> Result<(), AuthorError> {
        let genesis_now = auth.genesis().is_some_and(|g| g.holder == self.who && auth.is_available(g, self.now));
        if !genesis_now {
            return Err(AuthorError::Unauthorized(
                "a policy change is the live, available genesis holder's act".to_string(),
            ));
        }
        if let Some(g) = auth.genesis().filter(|g| args.accept_role.as_deref() == Some(g.role.as_str())) {
            return Err(AuthorError::Usage(format!(
                "the accept role must differ from the genesis role `{}` (D9 (f))",
                g.role
            )));
        }
        Ok(())
    }

    /// File the namespace's next policy.
    pub fn policy_set(&mut self, args: PolicyArgs) -> Result<Applied, AuthorError> {
        let store = self.load();
        let auth = Authority::of(&store, &args.namespace);
        let current = auth.policy(&args.namespace).cloned().ok_or_else(|| {
            AuthorError::Usage(format!("namespace `{}` has no policy — `ledger init --namespace` first", args.namespace))
        })?;
        self.refuse_policy_change(&auth, &args)?;
        let mut next = Policy {
            id: self.mint.mint_id("pol").map_err(AuthorError::Io)?,
            namespace: current.namespace.clone(),
            schemes: args.schemes.unwrap_or_else(|| current.schemes.clone()),
            require_sk: args.require_sk.unwrap_or(current.require_sk),
            accept_role: args.accept_role.unwrap_or_else(|| current.accept_role.clone()),
            reaccept_within_days: args.reaccept_within_days.unwrap_or(current.reaccept_within_days),
            replaces: Some(current.hash.clone()),
            by: self.who.clone(),
            under: auth.genesis().map(|g| g.id.clone()),
            at: self.now,
            hash: crate::hash::VersionHash::zero(),
        };
        next.hash = policy_hash(&next);
        let line = format!("{} replaces {} for `{}`", next.id, current.id, next.namespace);
        // Signed under the policy in force before it (D1), by the genesis
        // holder, with its own `at` the verify-time (D8).
        let ulid = next.id.ulid().to_string();
        let what = super::sign_ops::ToSign {
            namespace: &next.namespace,
            ulid: &ulid,
            bytes: crate::authority::payload::policy_bytes(&next),
            own_key: None,
            any_namespace: false,
        };
        self.sign_under(&store, Some(&current), what)?;
        let mut candidate = self.shell(None)?;
        candidate.policies.push(next);
        self.refusal_check(&store, &candidate, |_| false)?;
        let path = self.append_signed(&candidate)?;
        Ok(Applied { path, lines: vec![line] })
    }
}
