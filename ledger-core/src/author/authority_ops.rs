//! `init --namespace`, `role declare`, `grant new|accept|revoke` — the authority verbs.
//!
//! Every verb here builds the records it intends to file, asks the role
//! check ([`crate::authority::authorize`]) whether the actor may, and then
//! runs the same delta-gate every verb runs: a write that would introduce
//! a finding is refused with the finding `verify` would report. The role
//! check is a verb-time refusal; its gate-time counterpart over history
//! (an acceptance by an actor who held no grant, `A006`) is deferred until
//! the decision-class → role mapping exists.
//!
//! **Genesis.** The first `init --namespace` in a store bootstraps the
//! trust root: the root role carrying the four authority capabilities
//! ([`Capability::ROOT`]) and none of the decision ones, the genesis grant
//! (self-granted, `*`, primary, naming the out-of-band mandate), the
//! holder's acceptance of it, and the namespace's first policy, whose
//! `accept_role` is a different role (D9 (f)). Nobody holds the accept role
//! until it is granted — to the genesis holder too, if that person is to
//! accept. Later namespaces are the genesis holder's to initialise, and
//! file a policy (and the accept role, if new) only — a store has one trust
//! root (`A005`).

use crate::authority::payload::{grant_hash, revocation_hash};
use crate::authority::{
    authorize, Act, Authority, Authorized, Capability, Grant, GrantAcceptance, GrantScope, Limit,
    Order, Policy, Revocable, Revocation, Role, Scheme, Target,
};
use crate::id::GrantId;
use crate::store::Store;

use super::{Applied, Author, AuthorError};

/// What `init --namespace` states.
pub struct InitNamespaceArgs {
    pub namespace: String,
    /// The mandate outside the tool — required when the store has no
    /// genesis yet, ignored after.
    pub external_ref: Option<String>,
    /// The genesis role's id (created with [`Capability::ROOT`] if absent).
    pub role: String,
    /// The role whose grants carry `accept-decision` here; defaults to
    /// [`DEFAULT_ACCEPT_ROLE`], and never the genesis role.
    pub accept_role: Option<String>,
    /// Proceed with no key to bind (#96): without it, `init` refuses when
    /// the genesis holder has no key in the store and none is configured.
    pub without_key: bool,
}

/// The accept role `init --namespace` names when none is given. Declared
/// with `accept-decision` alone when absent — the fewest claims.
pub const DEFAULT_ACCEPT_ROLE: &str = "acceptor";

/// What `role declare` states.
pub struct RoleArgs {
    pub id: String,
    pub title: Option<String>,
    pub may: Vec<Capability>,
    pub notes: Option<String>,
}

/// What `grant new` states.
pub struct GrantArgs {
    pub role: String,
    pub holder: crate::identity::Identity,
    pub scope: GrantScope,
    pub order: Order,
    pub limits: Vec<Limit>,
    pub supersedes: Option<GrantId>,
}

impl Author {
    /// The role check, as a verb's refusal.
    pub(crate) fn authorized(
        &self,
        store: &Store,
        act: Act,
        target: Target<'_>,
        role: Option<&str>,
    ) -> Result<Authorized, AuthorError> {
        let as_role = self.as_role.as_deref();
        authorize(&Authority::build(store), &self.who, act, target, self.now, role, as_role).map_err(|d| {
            AuthorError::Unauthorized(format!("{} may not {}: {d}", self.who, act.as_str()))
        })
    }

    /// Put a namespace under policy, bootstrapping the genesis if needed.
    pub fn init_namespace(&mut self, args: InitNamespaceArgs) -> Result<Applied, AuthorError> {
        crate::id::validate_namespace(&args.namespace).map_err(AuthorError::Usage)?;
        let mut store = self.load();
        let auth = Authority::build(&store);
        if auth.policy(&args.namespace).is_some() {
            return Err(AuthorError::Conflict(format!(
                "namespace `{}` is already under policy — `ledger policy set` changes it",
                args.namespace
            )));
        }
        let genesis = auth.genesis().cloned();
        let joined = genesis.as_ref().map(|g| g.id.clone());
        let mut candidate = self.shell(Some(format!("init namespace {}", args.namespace)))?;
        let (mut new_roles, root_role, mut lines) = match genesis.clone() {
            None => self.bootstrap(&store, &args, &mut candidate)?,
            Some(g) => self.join_genesis(&auth, &g)?,
        };
        let accept_role = args.accept_role.clone().unwrap_or_else(|| DEFAULT_ACCEPT_ROLE.to_string());
        if accept_role == root_role {
            return Err(AuthorError::Usage(format!(
                "the accept role must differ from the genesis role `{root_role}` — the genesis role acts on \
                 the authority structure, not on decisions (D9 (f))"
            )));
        }
        if store.roles.iter().all(|r| r.id != accept_role) {
            new_roles.push(self.new_role(&accept_role, "Accepts decisions", &[Capability::AcceptDecision]));
            lines.push(format!("declared role `{accept_role}` — may accept-decision; held by nobody until granted"));
        }
        let root = candidate.grants.first().cloned().or(genesis).ok_or_else(|| AuthorError::Io("no genesis grant".into()))?;
        let under = joined.or_else(|| Some(root.id.clone()));
        let policy = self.first_policy(&args.namespace, &accept_role, under)?;
        lines.push(format!("namespace `{}` under policy {} (accept role `{accept_role}`)", args.namespace, policy.id));
        lines.extend(self.genesis_key(&store, &mut candidate, &root, &policy, args.without_key)?);
        candidate.policies.push(policy);
        store.roles.extend(new_roles.iter().cloned());
        self.refusal_check(&store, &candidate, |_| false)?;
        self.file_init(&candidate, &new_roles, lines)
    }

    /// Write the new roles, the change-set with its signatures, and — when
    /// a key was bound — the regenerated `allowed_signers`.
    fn file_init(&mut self, candidate: &crate::changeset::ChangeSet, new_roles: &[Role], mut lines: Vec<String>) -> Result<Applied, AuthorError> {
        for role in new_roles {
            self.write_role(role)?;
        }
        let path = self.append_signed(candidate)?;
        if !candidate.key_bindings.is_empty() {
            crate::authority::signers::write(&self.load()).map_err(AuthorError::Io)?;
            lines.push(format!("regenerated {}", crate::authority::signers::FILE));
        }
        Ok(Applied { path, lines })
    }

    fn new_role(&self, id: &str, title: &str, may: &[Capability]) -> Role {
        Role {
            format: crate::format::AUTHORITY_FORMAT,
            id: id.to_string(),
            title: Some(title.into()),
            owner: self.who.clone(),
            may: may.to_vec(),
            created_at: self.today(),
            notes: None,
        }
    }

    /// The genesis records: the root role (if new), grant, its acceptance.
    /// An existing role of the root id must carry every root capability.
    fn bootstrap(
        &mut self,
        store: &Store,
        args: &InitNamespaceArgs,
        candidate: &mut crate::changeset::ChangeSet,
    ) -> Result<(Vec<Role>, String, Vec<String>), AuthorError> {
        let mandate = args.external_ref.clone().filter(|r| !r.trim().is_empty()).ok_or_else(|| {
            AuthorError::Usage("the store has no genesis yet — `--external-ref` names the mandate it rests on".into())
        })?;
        let new_roles = match store.role(&args.role) {
            None => vec![self.new_role(&args.role, "Genesis steward", Capability::ROOT)],
            Some(existing) => {
                let missing: Vec<&str> =
                    Capability::ROOT.iter().filter(|c| !existing.may(**c)).map(|c| c.as_str()).collect();
                if !missing.is_empty() {
                    return Err(AuthorError::Conflict(format!(
                        "role `{}` cannot be the genesis role: it lacks {} — the root role carries grant-role, \
                         revoke-grant, declare-unavailability and rotate-genesis (D9 (f))",
                        args.role,
                        missing.join(", ")
                    )));
                }
                Vec::new()
            }
        };
        let grant = self.seal_grant(&GrantArgs {
            role: args.role.clone(),
            holder: self.who.clone(),
            scope: GrantScope::All,
            order: Order::PRIMARY,
            limits: Vec::new(),
            supersedes: None,
        }, Some(mandate), None)?;
        let line = format!("genesis {} — {} holds `{}` over *", grant.id, self.who, args.role);
        candidate.grant_acceptances.push(self.grant_acceptance(&grant)?);
        candidate.grants.push(grant);
        Ok((new_roles, args.role.clone(), vec![line]))
    }

    /// A later namespace: the live, available genesis holder's act.
    fn join_genesis(
        &self,
        auth: &Authority<'_>,
        genesis: &Grant,
    ) -> Result<(Vec<Role>, String, Vec<String>), AuthorError> {
        if genesis.holder != self.who || !auth.is_available(genesis, self.now) {
            return Err(AuthorError::Unauthorized(format!(
                "a namespace is put under policy by the genesis holder ({}), available now",
                genesis.holder
            )));
        }
        Ok((Vec::new(), genesis.role.clone(), Vec::new()))
    }

    /// The namespace's first policy, made under the genesis grant (D9 (a)).
    fn first_policy(
        &mut self,
        namespace: &str,
        accept_role: &str,
        under: Option<GrantId>,
    ) -> Result<Policy, AuthorError> {
        let mut policy = Policy {
            id: self.mint.mint_id("pol").map_err(AuthorError::Io)?,
            namespace: namespace.to_string(),
            schemes: vec![Scheme::Ssh],
            require_sk: false,
            accept_role: accept_role.to_string(),
            reaccept_within_days: None,
            replaces: None,
            by: self.who.clone(),
            under,
            at: self.now,
            hash: crate::hash::VersionHash::zero(),
        };
        policy.hash = crate::authority::payload::policy_hash(&policy);
        Ok(policy)
    }

    /// A sealed grant. `under` is the grantor's grant (D9 (a)); absent on
    /// the genesis grant.
    pub(crate) fn seal_grant(
        &mut self,
        args: &GrantArgs,
        mandate: Option<String>,
        under: Option<GrantId>,
    ) -> Result<Grant, AuthorError> {
        let mut grant = Grant {
            id: self.mint.mint_id("grant").map_err(AuthorError::Io)?,
            role: args.role.clone(),
            scope: args.scope.clone(),
            holder: args.holder.clone(),
            granted_by: self.who.clone(),
            order: args.order,
            limits: args.limits.clone(),
            genesis: mandate.is_some(),
            external_ref: mandate,
            supersedes: args.supersedes.clone(),
            under,
            at: self.now,
            hash: crate::hash::VersionHash::zero(),
        };
        grant.hash = grant_hash(&grant);
        Ok(grant)
    }

    fn grant_acceptance(&mut self, grant: &Grant) -> Result<GrantAcceptance, AuthorError> {
        Ok(GrantAcceptance {
            id: self.mint.mint_id("gacc").map_err(AuthorError::Io)?,
            grant: grant.id.clone(),
            signs: grant.hash.clone(),
            actor: self.who.clone(),
            at: self.now,
        })
    }

    fn write_role(&self, role: &Role) -> Result<(), AuthorError> {
        let path = self.root.join(crate::STORE_DIR).join("roles").join(role.file_name());
        if path.exists() {
            return Err(AuthorError::Conflict(format!("role `{}` is already declared", role.id)));
        }
        let text = serde_yaml::to_string(role).map_err(|e| AuthorError::Io(e.to_string()))?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| AuthorError::Io(e.to_string()))?;
        }
        product_core::fileops::write_file_atomic(&path, &text).map_err(|e| AuthorError::Io(e.to_string()))
    }

    /// Declare a role: `grant-role` over `*`, withheld by `no-role-edits`.
    pub fn declare_role(&mut self, args: RoleArgs) -> Result<Applied, AuthorError> {
        let store = self.load();
        self.authorized(&store, Act::DeclareRole, Target::Scope(&GrantScope::All), None)?;
        crate::set::DecisionSet::validate_id(&args.id).map_err(AuthorError::Usage)?;
        if args.may.is_empty() {
            return Err(AuthorError::Usage("a role names at least one capability (`--may`)".into()));
        }
        let role = Role {
            format: crate::format::AUTHORITY_FORMAT,
            id: args.id,
            title: args.title,
            owner: self.who.clone(),
            may: args.may,
            created_at: self.today(),
            notes: args.notes,
        };
        self.write_role(&role)?;
        let path = self.root.join(crate::STORE_DIR).join("roles").join(role.file_name());
        let caps: Vec<&str> = role.may.iter().map(|c| c.as_str()).collect();
        Ok(Applied { path, lines: vec![format!("declared role `{}` — may {}", role.id, caps.join(", "))] })
    }

    /// Grant a role: `grant-role` over the new grant's scope. Below the
    /// genesis, a grantor gives only the role it acts under.
    pub fn grant(&mut self, args: GrantArgs) -> Result<Applied, AuthorError> {
        let store = self.load();
        if store.role(&args.role).is_none() {
            return Err(AuthorError::Usage(format!("role `{}` is not declared — `ledger role declare` it", args.role)));
        }
        let auth = Authority::build(&store);
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

    /// The holder accepts a grant: it signs the grant's hash.
    pub fn accept_grant(&mut self, id: &GrantId) -> Result<Applied, AuthorError> {
        let store = self.load();
        let auth = Authority::build(&store);
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
        let auth = Authority::build(&store);
        let grant = auth.grants.get(&id.to_string()).copied().cloned().ok_or_else(|| {
            AuthorError::Usage(format!("{id} is not a filed grant"))
        })?;
        if auth.is_revoked(&Revocable::Grant(id.clone())) {
            return Err(AuthorError::Conflict(format!("{id} is already revoked — one revocation is enough")));
        }
        let by = self.authorized(&store, Act::RevokeGrant, Target::Scope(&grant.scope), None)?;
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
