//! The genesis holder's key at `init --namespace` (#96; per namespace, ruling 47).
//!
//! The key `git config user.signingkey` names is bound in the same
//! change-set as the namespace's genesis grant and first policy: the
//! genesis holder's self-bound first binding in the namespace (protocol
//! §4.10), carrying the namespace's mandate, dated with the policy and
//! signed by the key it binds. That closes the window in which the first
//! self-bound binding to land for the address in this namespace — anyone's
//! — would be the one trusted. The same key signs the first policy. Every
//! namespace is opened this way, whatever the holder has bound elsewhere: a
//! key is trusted in the namespace it is bound in and nowhere else, so the
//! same key may be self-bound in several namespaces, and a key closed in
//! one namespace is a different key here (a repository notice names the
//! split, never a finding).
//!
//! **No usable key** (ruled 2026-10-05): `init --namespace` refuses, naming
//! what is missing, unless `--without-key` says to proceed unbound — and
//! then `verify` keeps saying the window is open until a key is bound.

use crate::authority::payload::{binding_bytes, binding_hash, policy_bytes};
use crate::authority::{BindingAct, Grant, KeyBinding, Policy};
use crate::changeset::ChangeSet;
use crate::signing::ssh;
use crate::store::Store;

use super::sign_ops::ToSign;
use super::{Author, AuthorError};

impl Author {
    /// The genesis holder's key at `init`: the namespace's self-bound first
    /// binding, which signs the first policy. Returns the lines to print.
    pub(super) fn genesis_key(
        &mut self,
        store: &Store,
        candidate: &mut ChangeSet,
        genesis: &Grant,
        policy: &Policy,
        without_key: bool,
    ) -> Result<Vec<String>, AuthorError> {
        match configured(&self.root) {
            Ok((key, key_type, blob)) => self.bind_genesis_key(store, candidate, genesis, policy, (&key, key_type, blob)),
            Err(missing) if without_key => Ok(vec![format!(
                "warning: no key bound ({missing}; --without-key) — until `ledger identity add --namespace {}`, the first self-bound binding to land for {} there is the one trusted (D7)",
                policy.namespace, self.who
            )]),
            Err(missing) => Err(AuthorError::Usage(format!(
                "refused — {missing}. `init --namespace` binds the genesis holder's key in the same act, so the first self-bound binding for {} in the namespace is theirs (#96): set `git config user.signingkey` to your SSH key, or pass `--without-key` to initialise unbound",
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
        let what = ToSign { namespace: &binding.namespace, ulid: &ulid, bytes: binding_bytes(&binding), own_key: own };
        self.sign_under(store, Some(policy), what)?;
        let ulid = policy.id.ulid().to_string();
        let what = ToSign { namespace: &policy.namespace, ulid: &ulid, bytes: policy_bytes(policy), own_key: own };
        self.sign_under(store, Some(policy), what)?;
        let line = format!("bound {} as {}'s first key in `{}` ({key_type}), self-bound; it signs the policy — {}", key.display(), self.who, policy.namespace, binding.id);
        candidate.key_bindings.push(binding);
        Ok(vec![line])
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
