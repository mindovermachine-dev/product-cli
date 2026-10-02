//! `identity add | rotate | revoke` — filing key bindings, regenerating the trust root.
//!
//! Each verb appends one key-binding entry and then rewrites
//! `.decisions/allowed_signers` from the log, so the derived file always
//! matches what `verify` re-derives. A principal binds only their own key.
//! The first binding in a namespace is the genesis holder's, self-bound
//! under the genesis grant's mandate; every later one needs a live grant
//! whose scope reaches the namespace. A namespace whose policy requires
//! `-sk` keys refuses a software key. Signing each binding under the
//! policy in force is Session B's (`L011`).

use crate::authority::{Authority, BindingAct, GrantScope, KeyBinding};
use crate::id::KeyBindingId;

use super::{Applied, Author, AuthorError};

/// A public key, as OpenSSH writes it: type then base64.
pub struct KeyArgs {
    pub namespace: String,
    pub key_type: String,
    pub key: String,
}

impl Author {
    /// Bind a key of the actor's own in a namespace.
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
        let (namespace, principal) = match &closes {
            Some(id) => self.closable(&auth, id, act)?,
            None => (key.as_ref().map(|k| k.namespace.clone()).unwrap_or_default(), self.who.clone()),
        };
        let policy = auth.policy(&namespace).ok_or_else(|| {
            AuthorError::Usage(format!("namespace `{namespace}` has no policy — `ledger init --namespace {namespace}` first"))
        })?;
        if let Some(k) = &key {
            if policy.require_sk && !k.key_type.starts_with("sk-") {
                return Err(AuthorError::Unauthorized(format!(
                    "`{namespace}`'s policy requires a hardware-backed (`sk-`) key; `{}` is a software key",
                    k.key_type
                )));
            }
        }
        let (self_bound, mandate) = self.bootstrap_or_grant(&auth, &namespace, act)?;
        let mut binding = KeyBinding {
            id: self.mint.mint_id("key").map_err(AuthorError::Io)?,
            act,
            principal,
            namespace,
            key_type: key.as_ref().map(|k| k.key_type.clone()),
            key: key.map(|k| k.key),
            closes,
            self_bound,
            mandate,
            by: self.who.clone(),
            at: self.now,
            hash: crate::hash::VersionHash::zero(),
        };
        binding.hash = crate::authority::payload::binding_hash(&binding);
        let line = format!("identity {act}: {} in `{}` — {}", binding.principal, binding.namespace, binding.id);
        let mut candidate = self.shell(None)?;
        candidate.key_bindings.push(binding);
        self.refusal_check(&store, &candidate, |_| false)?;
        let path = self.append(&candidate)?;
        crate::authority::signers::write(&self.load()).map_err(AuthorError::Io)?;
        Ok(Applied { path, lines: vec![line, format!("regenerated {}", crate::authority::signers::FILE)] })
    }

    /// The open binding `id` names, if the actor may close it.
    fn closable(
        &self,
        auth: &Authority<'_>,
        id: &KeyBindingId,
        act: BindingAct,
    ) -> Result<(String, crate::identity::Identity), AuthorError> {
        let open = auth.bindings.iter().find(|b| b.id == *id && b.act.opens()).ok_or_else(|| {
            AuthorError::Usage(format!("{id} opens no key window"))
        })?;
        if auth.bindings.iter().any(|b| b.closes.as_ref() == Some(id)) {
            return Err(AuthorError::Conflict(format!("{id}'s window is already closed")));
        }
        let genesis_holder = auth.genesis().is_some_and(|g| g.holder == self.who);
        let allowed = open.principal == self.who || (act == BindingAct::Revoke && genesis_holder);
        if !allowed {
            return Err(AuthorError::Unauthorized(format!("{id} binds {}'s key", open.principal)));
        }
        Ok((open.namespace.clone(), open.principal.clone()))
    }

    /// Whether this is the namespace's self-bound genesis binding, else
    /// whether the actor holds a live grant reaching the namespace.
    fn bootstrap_or_grant(
        &self,
        auth: &Authority<'_>,
        namespace: &str,
        act: BindingAct,
    ) -> Result<(bool, Option<String>), AuthorError> {
        let first = auth.bindings.iter().all(|b| b.namespace != namespace);
        if let Some(g) = auth.genesis().filter(|g| g.holder == self.who) {
            return Ok(if first && act == BindingAct::Add { (true, g.external_ref.clone()) } else { (false, None) });
        }
        let reaches = auth.grants.values().any(|g| {
            let scoped = match &g.scope {
                GrantScope::All | GrantScope::Set(_) => true,
                GrantScope::Namespace(n) => n == namespace,
                GrantScope::Pattern(_) => false,
            };
            g.holder == self.who && scoped && auth.is_live(g)
        });
        if first || !reaches {
            return Err(AuthorError::Unauthorized(format!(
                "{} holds no live grant reaching `{namespace}`{}",
                self.who,
                if first { ", and the first binding there is the genesis holder's" } else { "" }
            )));
        }
        Ok((false, None))
    }
}
