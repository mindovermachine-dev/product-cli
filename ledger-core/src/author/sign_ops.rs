//! Signing at write time (#70): one helper every signing verb calls.
//!
//! The policy in force for the namespace decides: `ssh` listed — the verb
//! signs with the key `git config user.signingkey` names, exactly as git's
//! own SSH signing finds it; `dsse` listed — refused, DSSE signing is the
//! hosted service's; only `none` — nothing to sign. Before signing, the
//! verb refuses:
//!
//! - a configured key that is not the signer's live key in the namespace
//!   (or, for a self-bound binding, the key being bound; for a `rotate`,
//!   the key it closes) — a key bound elsewhere does not sign here (ruling 47);
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
    /// The one key that must sign, as `(key_type, key)`: the key a
    /// self-bound binding binds (it signs itself), or the key a `rotate`
    /// closes (LP-4.12, ruling 53).
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
            namespace: what.namespace.to_string(),
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
            Some((t, k)) if t != key_type || k != blob => {
                return Err(AuthorError::Unauthorized(format!(
                    "{} is not the key this act must be signed by: a self-bound binding is signed by the key it binds, and a `rotate` by the key it closes (LP-4.12) — set `git config user.signingkey` to that key",
                    key.display()
                )));
            }
            Some(_) => true,
            None => self.live_key(store, what.namespace, &key_type, &blob),
        };
        if !bound {
            let closed = self.closed_by(store, what.namespace, &blob).map(|id| format!(": it was closed in `{}` by {id}", what.namespace)).unwrap_or_default();
            return Err(AuthorError::Unauthorized(format!(
                "{} is not {}'s live key in `{}`{closed} — the signature would not verify",
                key.display(),
                self.who,
                what.namespace
            )));
        }
        Ok(())
    }

    /// Whether `(key_type, blob)` is an open window of this author's in `ns`
    /// (a key is trusted in the namespace it is bound in, ruling 47).
    fn live_key(&self, store: &Store, ns: &str, key_type: &str, blob: &str) -> bool {
        let auth = Authority::of(store, ns);
        auth.bindings.iter().any(|b| {
            b.act.opens()
                && b.principal == self.who
                && b.key_type.as_deref() == Some(key_type)
                && b.key.as_deref() == Some(blob)
                && !crate::authority::key_close::is_closed(&auth.bindings, b)
        })
    }

    /// The close in `ns` that ended this author's key `blob` there.
    fn closed_by(&self, store: &Store, ns: &str, blob: &str) -> Option<String> {
        let auth = Authority::of(store, ns);
        let mine = auth.bindings.iter().copied().find(|b| b.act.opens() && b.principal == self.who && b.key.as_deref() == Some(blob))?;
        crate::authority::key_close::closes_of(&auth.bindings, mine).into_iter().next().map(|c| c.id.to_string())
    }

    /// Append the change-set and write its pending signatures beside it —
    /// signatures first, so a log entry never lands without the sidecar it
    /// was gated with.
    pub(crate) fn append_signed(&mut self, candidate: &crate::changeset::ChangeSet) -> Result<std::path::PathBuf, AuthorError> {
        let mut written = Vec::new();
        for sidecar in std::mem::take(&mut self.pending_sidecars) {
            match crate::signing::write(&self.root, &sidecar) {
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
