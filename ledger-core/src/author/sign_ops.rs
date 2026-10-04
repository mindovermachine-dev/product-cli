//! Signing at write time (#70): one helper every signing verb calls.
//!
//! The policy in force for the namespace decides: `ssh` listed — the verb
//! signs with the key `git config user.signingkey` names, exactly as git's
//! own SSH signing finds it; `dsse` listed — refused, DSSE signing is the
//! hosted service's; only `none` — nothing to sign. Before signing, the
//! verb refuses:
//!
//! - a configured key that is not the signer's live key in the namespace
//!   (or, for a self-bound binding, the key being bound);
//! - a software key where the policy requires hardware-backed (`-sk`) keys;
//! - an **agent-reported unconfirmed key**: a software key whose private
//!   half only an agent holds (the configured file is a public key). Nothing
//!   then confirms each signature — a touch for `-sk`, a passphrase or the
//!   person at the terminal for a key on disk — so an agent could sign
//!   silently. `ssh-agent` does not report a key's `-c` constraint, so the
//!   CLI cannot tell a confirmed agent key from an unconfirmed one and
//!   refuses both.
//!
//! The signature becomes a pending sidecar: gated with the change-set
//! (`refusal_check` sees it) and written beside it by [`Author::append_signed`].

use crate::authority::{Authority, Policy, Scheme};
use crate::signing::{ssh, Sidecar};
use crate::store::Store;

use super::{Author, AuthorError};

/// What one signature is over.
pub(crate) struct ToSign<'a> {
    pub namespace: &'a str,
    pub ulid: &'a str,
    pub bytes: Vec<u8>,
    /// The key a self-bound binding binds — it signs itself.
    pub own_key: Option<(&'a str, &'a str)>,
}

impl Author {
    /// Sign `what` as this author if `policy` requires it.
    pub(crate) fn sign_under(&mut self, store: &Store, policy: Option<&Policy>, what: ToSign<'_>) -> Result<(), AuthorError> {
        let Some(policy) = policy else { return Ok(()) };
        if policy.schemes.contains(&Scheme::Dsse) {
            return Err(AuthorError::Unauthorized(format!(
                "`{}`'s policy requires a `dsse` signature; DSSE envelopes are made by the hosted ledger service — this CLI verifies them and never signs",
                what.namespace
            )));
        }
        if !policy.schemes.contains(&Scheme::Ssh) {
            return Ok(());
        }
        let key = ssh::configured_key(&self.root).ok_or_else(|| {
            AuthorError::Unauthorized(format!(
                "`{}`'s policy requires an `ssh` signature — set `git config user.signingkey` to your key bound there",
                what.namespace
            ))
        })?;
        self.refuse_key(store, policy, &what, &key)?;
        let bytes = ssh::sign(&key, what.namespace, &what.bytes).map_err(AuthorError::Io)?;
        self.pending_sidecars.push(Sidecar {
            ulid: what.ulid.to_string(),
            scheme: Scheme::Ssh,
            file: Sidecar::file_name(what.ulid, Scheme::Ssh),
            bytes,
        });
        Ok(())
    }

    /// The three key refusals: not `-sk` under an `-sk` policy, an
    /// agent-held software key, a key that is not the signer's live one.
    fn refuse_key(&self, store: &Store, policy: &Policy, what: &ToSign<'_>, key: &std::path::Path) -> Result<(), AuthorError> {
        let (key_type, blob, on_disk) = ssh::public_half(key).map_err(AuthorError::Usage)?;
        let hardware = key_type.starts_with("sk-");
        if policy.require_sk && !hardware {
            return Err(AuthorError::Unauthorized(format!(
                "`{}`'s policy requires a hardware-backed (`sk-`) key; {} is a software key ({key_type})",
                what.namespace,
                key.display()
            )));
        }
        if !on_disk && !hardware {
            return Err(AuthorError::Unauthorized(format!(
                "{} is a software key an agent holds: nothing confirms each signature, so an agent could sign unseen — use the private key file, or a hardware-backed (`sk-`) key",
                key.display()
            )));
        }
        let bound = match what.own_key {
            Some((t, k)) => t == key_type && k == blob,
            None => self.live_key(store, what.namespace, &key_type, &blob),
        };
        if !bound {
            return Err(AuthorError::Unauthorized(format!(
                "{} is not {}'s live key in `{}` — the signature would not verify",
                key.display(),
                self.who,
                what.namespace
            )));
        }
        Ok(())
    }

    /// Whether `(key_type, blob)` is an open window of this author's in `ns`.
    fn live_key(&self, store: &Store, ns: &str, key_type: &str, blob: &str) -> bool {
        let auth = Authority::build(store);
        auth.bindings.iter().any(|b| {
            b.act.opens()
                && b.namespace == ns
                && b.principal == self.who
                && b.key_type.as_deref() == Some(key_type)
                && b.key.as_deref() == Some(blob)
                && !auth.bindings.iter().any(|c| c.closes.as_ref() == Some(&b.id))
        })
    }

    /// Append the change-set and write its pending signatures beside it —
    /// signatures first, so a log entry never lands without the sidecar it
    /// was gated with.
    pub(crate) fn append_signed(&mut self, candidate: &crate::changeset::ChangeSet) -> Result<std::path::PathBuf, AuthorError> {
        let dir = self.root.join(crate::STORE_DIR);
        let mut written = Vec::new();
        for sidecar in std::mem::take(&mut self.pending_sidecars) {
            match crate::signing::write(&dir, &sidecar) {
                Ok(path) => written.push(path),
                Err(e) => {
                    written.iter().for_each(|p| drop(std::fs::remove_file(p)));
                    return Err(AuthorError::Io(e));
                }
            }
        }
        self.append(candidate).inspect_err(|_| written.iter().for_each(|p| drop(std::fs::remove_file(p))))
    }
}
