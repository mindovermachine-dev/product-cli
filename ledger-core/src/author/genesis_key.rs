//! The genesis holder's key at `init --namespace` (#96).
//!
//! **At bootstrap** the key `git config user.signingkey` names is bound in
//! the same change-set as the genesis grant and the first policy: the
//! genesis holder's self-bound first binding (§3.10.5), carrying the
//! mandate, dated with the policy and signed by the key it binds. That
//! closes the window in which the first self-bound binding to land for the
//! address — anyone's — would be the one trusted. The same key signs the
//! first policy. The same holds in a later namespace while the genesis
//! holder has no key anywhere in the store.
//!
//! **No usable key** (ruled 2026-10-05): `init --namespace` refuses, naming
//! what is missing, unless `--without-key` says to proceed unbound — and
//! then `verify` keeps saying the window is open until a key is bound.
//!
//! **Once the genesis holder has a key**, `init` in a later namespace binds
//! that live key there in the same act — their own `add`, signed by a key of
//! theirs already trusted, dated with the policy — and signs the first
//! policy with it, the signature `verify` then requires. If every key of
//! theirs is closed, `init` refuses as it does with no usable key, unless
//! `--without-key` — then it warns.

use crate::authority::payload::{binding_bytes, binding_hash, policy_bytes};
use crate::authority::{Authority, BindingAct, Grant, KeyBinding, Policy};
use crate::changeset::ChangeSet;
use crate::signing::ssh;
use crate::store::Store;

use super::sign_ops::ToSign;
use super::{Author, AuthorError};

impl Author {
    /// The genesis holder's key at `init`: bind it while they have none in
    /// the store, else sign the first policy with their live key. Returns
    /// the lines to print.
    pub(super) fn genesis_key(
        &mut self,
        store: &Store,
        candidate: &mut ChangeSet,
        genesis: &Grant,
        policy: &Policy,
        without_key: bool,
    ) -> Result<Vec<String>, AuthorError> {
        if Authority::build(store).bindings.iter().any(|b| b.principal == self.who) {
            return self.carry_genesis_key(store, candidate, policy, without_key);
        }
        match configured(&self.root) {
            Ok((key, key_type, blob)) => self.bind_genesis_key(store, candidate, genesis, policy, (&key, key_type, blob)),
            Err(missing) if without_key => Ok(vec![format!(
                "warning: no key bound ({missing}; --without-key) — until `ledger identity add --namespace {}`, the first self-bound binding to land for {} is the one trusted (D7)",
                policy.namespace, self.who
            )]),
            Err(missing) => Err(AuthorError::Usage(format!(
                "refused — {missing}. `init --namespace` binds the genesis holder's key in the same act, so the first self-bound binding for {} is theirs (#96): set `git config user.signingkey` to your SSH key, or pass `--without-key` to initialise unbound",
                self.who
            ))),
        }
    }

    /// Bind the configured key as the genesis holder's first binding, and
    /// sign the first policy with it.
    fn bind_genesis_key(
        &mut self,
        store: &Store,
        candidate: &mut ChangeSet,
        genesis: &Grant,
        policy: &Policy,
        (key, key_type, blob): (&std::path::Path, String, String),
    ) -> Result<Vec<String>, AuthorError> {
        let mut binding = KeyBinding {
            id: self.mint.mint_id("key").map_err(AuthorError::Io)?,
            act: BindingAct::Add,
            principal: self.who.clone(),
            namespace: policy.namespace.clone(),
            key_type: Some(key_type.clone()),
            key: Some(blob.clone()),
            closes: None,
            self_bound: true,
            mandate: genesis.external_ref.clone(),
            by: self.who.clone(),
            under: None,
            at: policy.at,
            hash: crate::hash::VersionHash::zero(),
        };
        binding.hash = binding_hash(&binding);
        let own = Some((key_type.as_str(), blob.as_str()));
        let ulid = binding.id.ulid().to_string();
        let what = ToSign { namespace: &binding.namespace, ulid: &ulid, bytes: binding_bytes(&binding), own_key: own, any_namespace: false };
        self.sign_under(store, Some(policy), what)?;
        let ulid = policy.id.ulid().to_string();
        let what = ToSign { namespace: &policy.namespace, ulid: &ulid, bytes: policy_bytes(policy), own_key: own, any_namespace: false };
        self.sign_under(store, Some(policy), what)?;
        let line = format!("bound {} as {}'s first key ({key_type}), self-bound; it signs the policy — {}", key.display(), self.who, binding.id);
        candidate.key_bindings.push(binding);
        Ok(vec![line])
    }

    /// In a later namespace, while the genesis holder holds a live key in
    /// the store: bind it here in the same act — their own `add`, signed by
    /// a key of theirs already trusted, dated with the policy — and sign the
    /// first policy with it. They can then sign in this namespace with no
    /// separate `identity add`.
    fn carry_genesis_key(&mut self, store: &Store, candidate: &mut ChangeSet, policy: &Policy, without_key: bool) -> Result<Vec<String>, AuthorError> {
        let auth = Authority::build(store);
        // A close ends the key in every namespace (ruled 2026-10-06).
        let live = |b: &&KeyBinding| b.act.opens() && b.principal == self.who && !crate::authority::key_close::is_closed(&auth.bindings, b);
        if !auth.bindings.iter().any(&live) {
            return self.no_live_key(&auth, policy, without_key);
        }
        let mut lines = Vec::new();
        if !auth.bindings.iter().any(|b| live(b) && b.namespace == policy.namespace) {
            let (key, key_type, blob) = configured(&self.root).map_err(|missing| {
                AuthorError::Usage(format!("refused — {missing}; `init --namespace` binds {}'s live key in the new namespace and signs its policy with it", self.who))
            })?;
            let mut binding = KeyBinding {
                id: self.mint.mint_id("key").map_err(AuthorError::Io)?,
                act: BindingAct::Add,
                principal: self.who.clone(),
                namespace: policy.namespace.clone(),
                key_type: Some(key_type.clone()),
                key: Some(blob),
                closes: None,
                self_bound: false,
                mandate: None,
                by: self.who.clone(),
                under: None,
                at: policy.at,
                hash: crate::hash::VersionHash::zero(),
            };
            binding.hash = binding_hash(&binding);
            let ulid = binding.id.ulid().to_string();
            let what = ToSign { namespace: &binding.namespace, ulid: &ulid, bytes: binding_bytes(&binding), own_key: None, any_namespace: true };
            self.sign_under(store, Some(policy), what)?;
            lines.push(format!("bound {} ({key_type}) for {} in `{}`, signed by their key trusted elsewhere — {}", key.display(), self.who, policy.namespace, binding.id));
            candidate.key_bindings.push(binding);
        }
        let ulid = policy.id.ulid().to_string();
        let what = ToSign { namespace: &policy.namespace, ulid: &ulid, bytes: policy_bytes(policy), own_key: None, any_namespace: true };
        self.sign_under(store, Some(policy), what)?;
        lines.push(format!("signed {} with {}'s live key", policy.id, self.who));
        Ok(lines)
    }
}

impl Author {
    /// The genesis holder has bindings, every one closed: nothing of theirs
    /// can vouch for a binding here or sign the policy. Refused, naming the
    /// closed keys, unless `--without-key` says to proceed unbound (ruled
    /// 2026-10-05).
    fn no_live_key(&self, auth: &Authority<'_>, policy: &Policy, without_key: bool) -> Result<Vec<String>, AuthorError> {
        let ids: Vec<String> = auth.bindings.iter().filter(|b| b.act.opens() && b.principal == self.who).map(|b| b.id.to_string()).collect();
        let missing = format!("no live key: every key bound to {} is closed ({})", self.who, ids.join(", "));
        if without_key {
            return Ok(vec![format!(
                "warning: {missing}; --without-key — `{}` is initialised with no key bound for {} and its first policy unsigned",
                policy.namespace, self.who
            )]);
        }
        Err(AuthorError::Usage(format!(
            "refused — {missing}. `init --namespace` binds the genesis holder's live key in the new namespace and signs its policy with it: bind a live key first, or pass `--without-key` to initialise unbound"
        )))
    }
}

/// The configured key and its public half, or what is missing.
fn configured(root: &std::path::Path) -> Result<(std::path::PathBuf, String, String), String> {
    let key = ssh::configured_key(root).ok_or_else(|| "no key: `git config user.signingkey` is unset".to_string())?;
    match ssh::public_half(&key) {
        Ok((key_type, blob, _)) => Ok((key, key_type, blob)),
        Err(why) => Err(format!("no usable key: `git config user.signingkey` names {}, which is unusable ({why})", key.display())),
    }
}
