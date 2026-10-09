//! `identity add | rotate | revoke` — filing key bindings, regenerating the trust root.
//!
//! Each verb appends one key-binding entry, signed where the namespace's
//! policy requires it, and then rewrites `.decisions/allowed_signers` from
//! the log, so the derived file always matches what `verify` re-derives.
//! Who may file which binding is D7's rule, [`crate::authority::filing::may_file`]
//! — the one `verify` re-judges every filed binding with: the genesis
//! holder's self-bound first binding in the namespace (signed by the key
//! it binds), a principal's first key filed and signed by the genesis
//! holder (`identity add --for <principal>`), and every further `add`,
//! `rotate` and `revoke` the principal's own, signed by a key of theirs
//! live in the namespace — or a `revoke` by the genesis holder. Keys are
//! per namespace (ruling 47): nothing bound elsewhere counts here. A
//! namespace whose policy requires `-sk` keys refuses a software key.

use crate::authority::filing::may_file;
use crate::authority::{Authority, BindingAct, KeyBinding};
use crate::id::KeyBindingId;
use crate::identity::Identity;

use super::sign_ops::ToSign;
use super::{Applied, Author, AuthorError};

/// A public key, as OpenSSH writes it: type then base64.
pub struct KeyArgs {
    pub namespace: String,
    pub key_type: String,
    pub key: String,
    /// Whose key it is, when the genesis holder files a principal's first
    /// key (D7); absent means the actor's own.
    pub principal: Option<Identity>,
}

impl Author {
    /// Bind a key in a namespace: the actor's own, or (as the genesis
    /// holder) a principal's first.
    pub fn identity_add(&mut self, args: KeyArgs) -> Result<Applied, AuthorError> {
        self.bind(BindingAct::Add, None, Some(args))
    }

    /// Close one of the actor's open bindings and open a new key in one act.
    pub fn identity_rotate(&mut self, closes: &KeyBindingId, args: KeyArgs) -> Result<Applied, AuthorError> {
        self.bind(BindingAct::Rotate, Some(closes.clone()), Some(args))
    }

    /// Close a binding's window — the principal's act, or the genesis
    /// holder's (a compromised key's owner may not be the one to notice).
    pub fn identity_revoke(&mut self, closes: &KeyBindingId) -> Result<Applied, AuthorError> {
        self.bind(BindingAct::Revoke, Some(closes.clone()), None)
    }

    fn bind(
        &mut self,
        act: BindingAct,
        closes: Option<KeyBindingId>,
        key: Option<KeyArgs>,
    ) -> Result<Applied, AuthorError> {
        let store = self.load();
        let auth = Authority::build(&store);
        let target = closes.as_ref().map(|id| {
            auth.bindings.iter().copied().find(|b| b.id == *id && b.act.opens()).ok_or_else(|| AuthorError::Usage(format!("{id} opens no key window")))
        }).transpose()?;
        let (namespace, principal) = match target {
            Some(b) => (b.namespace.clone(), b.principal.clone()),
            None => {
                let k = key.as_ref().ok_or_else(|| AuthorError::Usage("`add` names a key".into()))?;
                (k.namespace.clone(), k.principal.clone().unwrap_or_else(|| self.who.clone()))
            }
        };
        // A `rotate` is signed by the key it closes (LP-4.12, ruling 53).
        let closed_key = target.filter(|_| act == BindingAct::Rotate).and_then(|b| b.key_type.clone().zip(b.key.clone()));
        // The namespace's own authority decides who its genesis holder is
        // (ruling 47); its bindings still range over the store (issue 6).
        let auth = Authority::of(&store, &namespace);
        let policy = auth.policy(&namespace).cloned().ok_or_else(|| {
            AuthorError::Usage(format!("namespace `{namespace}` has no policy — `ledger init --namespace {namespace}` first"))
        })?;
        if let Some(k) = key.as_ref().filter(|k| policy.require_sk && !k.key_type.starts_with("sk-")) {
            return Err(AuthorError::Unauthorized(format!(
                "`{namespace}`'s policy requires a hardware-backed (`sk-`) key; `{}` is a software key",
                k.key_type
            )));
        }
        let genesis = auth.genesis().filter(|g| g.holder == self.who);
        // D7: the genesis holder's first key in the namespace is self-bound
        // (ruling 47: once per namespace, whatever they hold elsewhere).
        let first_in_ns = auth.bindings.iter().all(|b| b.principal != principal);
        let self_bound = act == BindingAct::Add && principal == self.who && first_in_ns && genesis.is_some();
        let mut binding = KeyBinding {
            id: self.mint.mint_id("key").map_err(AuthorError::Io)?,
            act,
            under: (principal != self.who).then(|| auth.genesis().map(|g| g.id.clone())).flatten(),
            principal,
            namespace,
            key_type: key.as_ref().map(|k| k.key_type.clone()),
            key: key.as_ref().map(|k| k.key.clone()),
            closes,
            self_bound,
            mandate: if self_bound { genesis.and_then(|g| g.external_ref.clone()) } else { None },
            by: self.who.clone(),
            at: self.now,
            hash: crate::hash::VersionHash::zero(),
        };
        binding.hash = crate::authority::payload::binding_hash(&binding);
        may_file(&auth, &binding).map_err(|rule| AuthorError::Unauthorized(format!("{rule} (D7)")))?;
        let ulid = binding.id.ulid().to_string();
        let own = binding.key_type.clone().zip(binding.key.clone());
        let what = ToSign {
            namespace: &binding.namespace,
            ulid: &ulid,
            bytes: crate::authority::payload::binding_bytes(&binding),
            own_key: own.as_ref().filter(|_| self_bound).or(closed_key.as_ref()).map(|(t, k)| (t.as_str(), k.as_str())),
        };
        self.sign_under(&store, Some(&policy), what)?;
        let line = format!("identity {act}: {} in `{}` — {}", binding.principal, binding.namespace, binding.id);
        let mut candidate = self.shell(None)?;
        candidate.key_bindings.push(binding);
        self.refusal_check(&store, &candidate, |_| false)?;
        let path = self.append_signed(&candidate)?;
        let synced = crate::authority::signers::write(&self.load()).map_err(AuthorError::Io)?;
        let mut lines = vec![line];
        lines.extend(synced.lines());
        Ok(Applied { path, lines })
    }
}
