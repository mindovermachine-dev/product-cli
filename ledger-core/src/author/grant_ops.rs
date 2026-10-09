//! `grant new|accept|revoke` — the grant verbs, each in one namespace's authority.
//!
//! A grant belongs to the namespace whose directory holds it, and the
//! grantor acts in that namespace's authority (LP-6.31, ruling 47): `grant
//! new` is homed explicitly, by a `ns:` scope, or by the store's one
//! namespace; `accept` and `revoke` read the namespace the grant is filed
//! in. Each verb builds the record it intends to file, asks the role check
//! whether the actor may, and runs the same delta-gate every verb runs.

use crate::authority::payload::revocation_hash;
use crate::authority::{Act, Authority, Authorized, GrantScope, Revocable, Revocation, Target};
use crate::id::GrantId;
use crate::store::Store;

use super::authority_ops::GrantArgs;
use super::{Applied, Author, AuthorError};

impl Author {
    /// Grant a role: `grant-role` over the new grant's scope, in the
    /// namespace whose directory will hold it. Below the genesis, a grantor
    /// gives only the role it acts under.
    pub fn grant(&mut self, args: GrantArgs) -> Result<Applied, AuthorError> {
        let store = self.load();
        let namespace = self.grant_home(&args)?;
        if store.role_in(&namespace, &args.role).is_none() {
            return Err(AuthorError::Usage(format!(
                "role `{}` is not declared in `{namespace}` — `ledger role declare --namespace {namespace}` it",
                args.role
            )));
        }
        let auth = Authority::of(&store, &namespace);
        let as_role = self.as_role.as_deref();
        let by = crate::authority::choice::grantor(&auth, &self.who, &args.scope, &args.role, self.now, as_role)
            .map_err(|d| AuthorError::Unauthorized(format!("{} may not {}: {d}", self.who, Act::Grant.as_str())))?;
        let grant = self.seal_grant(&args, None, Some(under_of(&by)?))?;
        let line = format!("{} granted `{}` over {} to {} ({})", self.who, grant.role, grant.scope, grant.holder, grant.order);
        let note = format!("grant {}", grant.id);
        let id = grant.id.clone();
        let mut candidate = self.shell(Some(note))?;
        candidate.grants.push(grant);
        self.refusal_check(&store, &candidate, |_| false)?;
        let path = self.append(&candidate)?;
        Ok(Applied { path, lines: vec![line, format!("{id} is live once its holder runs `ledger grant accept {id}`"), by.line()] })
    }

    /// The namespace a new grant is filed in: explicit, a `ns:` scope's,
    /// else the store's one namespace directory.
    fn grant_home(&self, args: &GrantArgs) -> Result<String, AuthorError> {
        let scoped = match &args.scope {
            GrantScope::Namespace(ns) => Some(ns.as_str()),
            _ => None,
        };
        if let (Some(explicit), Some(scoped)) = (args.namespace.as_deref(), scoped) {
            if explicit != scoped {
                return Err(AuthorError::Usage(format!(
                    "the scope `ns:{scoped}` names another namespace than `--namespace {explicit}` — a scope is read inside its own namespace (ruling 47)"
                )));
            }
        }
        super::home::resolve_namespace_dir(&self.root, args.namespace.as_deref().or(scoped))
    }

    /// The namespace whose directory holds the grant `id`.
    fn filed_in(store: &Store, id: &GrantId) -> Result<String, AuthorError> {
        super::home::grant_namespace(store, &id.to_string()).ok_or_else(|| AuthorError::Usage(format!("{id} is not a filed grant")))
    }

    /// The holder accepts a grant: it signs the grant's hash.
    pub fn accept_grant(&mut self, id: &GrantId) -> Result<Applied, AuthorError> {
        let store = self.load();
        let auth = Authority::of(&store, &Self::filed_in(&store, id)?);
        let grant = auth.grants.get(&id.to_string()).copied().cloned().ok_or_else(|| {
            AuthorError::Usage(format!("{id} is not a filed grant"))
        })?;
        if grant.holder != self.who {
            return Err(AuthorError::Unauthorized(format!("{id} is held by {}; only the holder accepts it", grant.holder)));
        }
        if auth.is_accepted(&grant) {
            return Err(AuthorError::Conflict(format!("{id} is already accepted — accepting twice adds nothing")));
        }
        let mut candidate = self.shell(None)?;
        candidate.grant_acceptances.push(self.grant_acceptance(&grant)?);
        self.refusal_check(&store, &candidate, |_| false)?;
        let path = self.append(&candidate)?;
        Ok(Applied { path, lines: vec![format!("{} accepted {id} — `{}` over {} is live", self.who, grant.role, grant.scope)] })
    }

    /// Revoke a grant: `revoke-grant` over its scope, withheld by
    /// `no-grant-revocations`. Files a `rev:` entity; the grant is untouched.
    pub fn revoke_grant(&mut self, id: &GrantId, reason: String) -> Result<Applied, AuthorError> {
        let store = self.load();
        let namespace = Self::filed_in(&store, id)?;
        let auth = Authority::of(&store, &namespace);
        let grant = auth.grants.get(&id.to_string()).copied().cloned().ok_or_else(|| {
            AuthorError::Usage(format!("{id} is not a filed grant"))
        })?;
        if auth.is_revoked(&Revocable::Grant(id.clone())) {
            return Err(AuthorError::Conflict(format!("{id} is already revoked — one revocation is enough")));
        }
        let by = self.authorized(&store, &namespace, Act::RevokeGrant, Target::Scope(&grant.scope), None)?;
        let revocation = self.revocation(Revocable::Grant(id.clone()), reason, Some(under_of(&by)?))?;
        let line = format!("revoked {id} — {}", revocation.reason);
        let mut candidate = self.shell(None)?;
        candidate.revocations.push(revocation);
        self.refusal_check(&store, &candidate, |_| false)?;
        let path = self.append(&candidate)?;
        Ok(Applied { path, lines: vec![line, by.line()] })
    }

    /// A sealed `rev:` entity for `target` (spec v1.7).
    pub(crate) fn revocation(
        &mut self,
        target: Revocable,
        reason: String,
        under: Option<GrantId>,
    ) -> Result<Revocation, AuthorError> {
        if reason.trim().is_empty() {
            return Err(AuthorError::Usage("a revocation carries its reason".to_string()));
        }
        let mut r = Revocation {
            id: Some(self.mint.mint_id("rev").map_err(AuthorError::Io)?),
            revokes: Some(target),
            acceptance: None,
            at: self.now,
            actor: Some(self.who.clone()),
            by: None,
            reason,
            under,
            hash: None,
        };
        r.hash = Some(revocation_hash(&r));
        Ok(r)
    }
}


/// The grant id an authorised act records as its `under`.
pub(crate) fn under_of(by: &Authorized) -> Result<GrantId, AuthorError> {
    by.grant.parse().map_err(AuthorError::Io)
}
