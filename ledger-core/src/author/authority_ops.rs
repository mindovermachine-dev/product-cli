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
//! **Genesis.** Every `init --namespace` opens a trust root of its own
//! (LP-6.31, ruling 47): the root role carrying the four authority
//! capabilities ([`Capability::ROOT`]) and none of the decision ones,
//! declared under the namespace's `roles/`; the genesis grant (self-granted,
//! `*` — the whole of this namespace — primary, naming the out-of-band
//! mandate); the holder's acceptance of it; and the namespace's first
//! policy, whose `accept_role` is a different role (D9 (f)). Nobody holds
//! the accept role until it is granted — to the genesis holder too, if that
//! person is to accept. A namespace has one trust root (`A005`); a store
//! has as many as it has namespaces, and nothing links them. Authority
//! records are filed in change-sets of their own, apart from decisions
//! (PRD §3.12).

use crate::authority::payload::grant_hash;
use crate::authority::{
    authorize, Act, Authority, Authorized, Capability, Grant, GrantAcceptance, GrantScope, Limit,
    Order, Policy, Role, Scheme, Target,
};
use crate::id::GrantId;
use crate::store::Store;

use super::{Applied, Author, AuthorError};

/// What `init --namespace` states.
pub struct InitNamespaceArgs {
    pub namespace: String,
    /// The mandate outside the tool the namespace's genesis rests on —
    /// required every time: each namespace is opened on its own (ruling 47).
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
    /// The namespace whose `roles/` holds the file: explicit, else the one
    /// namespace the store has a directory for.
    pub namespace: Option<String>,
    pub id: String,
    pub title: Option<String>,
    pub may: Vec<Capability>,
    pub notes: Option<String>,
}

/// What `grant new` states.
pub struct GrantArgs {
    /// The namespace whose directory holds the grant and whose authority
    /// the grantor acts in: explicit, else a `ns:` scope's, else the one
    /// namespace the store has a directory for.
    pub namespace: Option<String>,
    pub role: String,
    pub holder: crate::identity::Identity,
    pub scope: GrantScope,
    pub order: Order,
    pub limits: Vec<Limit>,
    pub supersedes: Option<GrantId>,
}

impl Author {
    /// The role check in `namespace`'s authority, as a verb's refusal
    /// (ruling 47: a grant acts in the namespace whose directory holds it).
    pub(crate) fn authorized(
        &self,
        store: &Store,
        namespace: &str,
        act: Act,
        target: Target<'_>,
        role: Option<&str>,
    ) -> Result<Authorized, AuthorError> {
        let as_role = self.as_role.as_deref();
        authorize(&Authority::of(store, namespace), &self.who, act, target, self.now, role, as_role).map_err(|d| {
            AuthorError::Unauthorized(format!("{} may not {} in `{namespace}`: {d}", self.who, act.as_str()))
        })
    }

    /// Put a namespace under policy, opening its own trust root (LP-6.31).
    pub fn init_namespace(&mut self, args: InitNamespaceArgs) -> Result<Applied, AuthorError> {
        crate::id::validate_namespace(&args.namespace).map_err(AuthorError::Usage)?;
        let mut store = self.load();
        let auth = Authority::of(&store, &args.namespace);
        if auth.policy(&args.namespace).is_some() {
            return Err(AuthorError::Conflict(format!(
                "namespace `{}` is already under policy — `ledger policy set` changes it",
                args.namespace
            )));
        }
        if let Some(g) = auth.genesis() {
            return Err(AuthorError::Conflict(format!(
                "namespace `{}` already has a live genesis grant ({}) — a namespace has one trust root (A005)",
                args.namespace, g.id
            )));
        }
        let mut candidate = self.shell(Some(format!("init namespace {}", args.namespace)))?;
        let (mut new_roles, root_role, mut lines) = self.bootstrap(&store, &args, &mut candidate)?;
        let accept_role = args.accept_role.clone().unwrap_or_else(|| DEFAULT_ACCEPT_ROLE.to_string());
        if accept_role == root_role {
            return Err(AuthorError::Usage(format!(
                "the accept role must differ from the genesis role `{root_role}` — the genesis role acts on \
                 the authority structure, not on decisions (D9 (f))"
            )));
        }
        if store.role_in(&args.namespace, &accept_role).is_none() {
            new_roles.push(self.new_role(&args.namespace, &accept_role, "Accepts decisions", &[Capability::AcceptDecision]));
            lines.push(format!("declared role `{accept_role}` in `{}` — may accept-decision; held by nobody until granted", args.namespace));
        }
        let root = candidate.grants.first().cloned().ok_or_else(|| AuthorError::Io("no genesis grant".into()))?;
        let policy = self.first_policy(&args.namespace, &accept_role, Some(root.id.clone()))?;
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
            lines.extend(crate::authority::signers::write(&self.load()).map_err(AuthorError::Io)?.lines());
        }
        Ok(Applied { path, lines })
    }

    fn new_role(&self, namespace: &str, id: &str, title: &str, may: &[Capability]) -> Role {
        Role {
            format: crate::format::AUTHORITY_FORMAT,
            id: id.to_string(),
            namespace: namespace.to_string(),
            title: Some(title.into()),
            owner: self.who.clone(),
            may: may.to_vec(),
            created_at: self.today(),
            notes: None,
        }
    }

    /// The namespace's genesis records: the root role (if new in it), grant,
    /// its acceptance. An existing role of the root id must carry every root
    /// capability and no decision capability (D9 (f), extended by ruling 50).
    fn bootstrap(
        &mut self,
        store: &Store,
        args: &InitNamespaceArgs,
        candidate: &mut crate::changeset::ChangeSet,
    ) -> Result<(Vec<Role>, String, Vec<String>), AuthorError> {
        let mandate = args.external_ref.clone().filter(|r| !r.trim().is_empty()).ok_or_else(|| {
            AuthorError::Usage(format!(
                "a namespace is opened on its own mandate (ruling 47) — `--external-ref` names the one `{}`'s genesis rests on",
                args.namespace
            ))
        })?;
        let new_roles = match store.role_in(&args.namespace, &args.role) {
            None => vec![self.new_role(&args.namespace, &args.role, "Genesis steward", Capability::ROOT)],
            Some(existing) => match existing.genesis_refusal() {
                Some(why) => return Err(AuthorError::Conflict(format!("role `{}` cannot be the genesis role: {why}", args.role))),
                None => Vec::new(),
            },
        };
        let grant = self.seal_grant(&GrantArgs {
            namespace: Some(args.namespace.clone()),
            role: args.role.clone(),
            holder: self.who.clone(),
            scope: GrantScope::All,
            order: Order::PRIMARY,
            limits: Vec::new(),
            supersedes: None,
        }, Some(mandate), None)?;
        let line = format!("genesis {} — {} holds `{}` over * (the whole of `{}`)", grant.id, self.who, args.role, args.namespace);
        candidate.grant_acceptances.push(self.grant_acceptance(&grant)?);
        candidate.grants.push(grant);
        Ok((new_roles, args.role.clone(), vec![line]))
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

    pub(super) fn grant_acceptance(&mut self, grant: &Grant) -> Result<GrantAcceptance, AuthorError> {
        Ok(GrantAcceptance {
            id: self.mint.mint_id("gacc").map_err(AuthorError::Io)?,
            grant: grant.id.clone(),
            signs: grant.hash.clone(),
            actor: self.who.clone(),
            at: self.now,
        })
    }

    /// Write a role file under its namespace's `roles/` (LP-3.34).
    fn write_role(&self, role: &Role) -> Result<(), AuthorError> {
        let path = crate::layout::roles_dir(&self.root, &role.namespace).join(role.file_name());
        if path.exists() {
            return Err(AuthorError::Conflict(format!("role `{}` is already declared", role.id)));
        }
        let text = serde_yaml::to_string(role).map_err(|e| AuthorError::Io(e.to_string()))?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| AuthorError::Io(e.to_string()))?;
        }
        product_core::fileops::write_file_atomic(&path, &text).map_err(|e| AuthorError::Io(e.to_string()))
    }

    /// Declare a role in a namespace: `grant-role` over `*` there, withheld
    /// by `no-role-edits`.
    pub fn declare_role(&mut self, args: RoleArgs) -> Result<Applied, AuthorError> {
        let store = self.load();
        let namespace = super::home::resolve_namespace_dir(&self.root, args.namespace.as_deref())?;
        self.authorized(&store, &namespace, Act::DeclareRole, Target::Scope(&GrantScope::All), None)?;
        crate::set::DecisionSet::validate_id(&args.id).map_err(AuthorError::Usage)?;
        if args.may.is_empty() {
            return Err(AuthorError::Usage("a role names at least one capability (`--may`)".into()));
        }
        let role = Role {
            format: crate::format::AUTHORITY_FORMAT,
            id: args.id,
            namespace,
            title: args.title,
            owner: self.who.clone(),
            may: args.may,
            created_at: self.today(),
            notes: args.notes,
        };
        self.write_role(&role)?;
        let path = crate::layout::roles_dir(&self.root, &role.namespace).join(role.file_name());
        let caps: Vec<&str> = role.may.iter().map(|c| c.as_str()).collect();
        Ok(Applied { path, lines: vec![format!("declared role `{}` in `{}` — may {}", role.id, role.namespace, caps.join(", "))] })
    }

}
