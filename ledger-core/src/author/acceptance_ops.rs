//! `ledger accept` / `ledger revoke` — signing, or unsaying a signature.
//!
//! Acceptance signs the latest version's *hash*, never the id, and the
//! actor is whoever git config says is here (OD-3: no `--as`, one source of
//! truth). A model or CI identity is refused by the same `L006` rule the
//! gate runs; an expired-on-arrival acceptance by the same `L003`. This
//! module never creates an acceptance for anyone but the configured
//! identity, and revocation appends — history keeps the mistake and the
//! correction both. A revocation is its own `rev:` entity (spec v1.7,
//! ruling 3): the acceptance it names is never edited, so the acceptance's
//! content — and any signature over it — stays fixed.

use chrono::NaiveDate;

use crate::acceptance::{Acceptance, AcceptanceScope};
use crate::authority::{Act, Authority, Authorized, Revocable, Target};
use crate::id::{AcceptanceId, DecisionId};
use crate::store::Store;
use crate::verify::view::View;

use super::{Applied, Author, AuthorError};

/// What an acceptance covers.
pub struct AcceptArgs {
    pub decision: DecisionId,
    pub expires_at: Option<NaiveDate>,
}

/// What a revocation unsays, and why.
pub struct RevokeArgs {
    pub acceptance: AcceptanceId,
    pub reason: String,
}

/// The one hash an acceptance may sign: the tip of the parent DAG. A
/// forked chain has no signable tip — signing either side would bless one
/// writer's content over the other's without an arbitration on record.
fn signable_tip(
    view: &View,
    decision: &DecisionId,
) -> Result<crate::hash::VersionHash, AuthorError> {
    let subject = decision.to_string();
    if view.is_forked(&subject) {
        return Err(AuthorError::Conflict(format!(
            "the version chain of {decision} is forked — there is no one latest version to sign; `ledger merge --resolve` arbitrates first"
        )));
    }
    view.latest
        .get(&subject)
        .and_then(|i| view.versions.get(*i))
        .map(|v| v.raw.hash.clone())
        .ok_or_else(|| AuthorError::Usage(format!("{decision} has no filed version to accept")))
}

impl Author {
    /// The role check on a decision (spec v1.7): in a namespace under
    /// policy, the actor needs a live, accepted, available grant of the
    /// policy's accept role covering the decision. A namespace without a
    /// policy is a pre-v2 store, and nothing is role-checked there.
    pub(crate) fn decision_authority(
        &self,
        store: &Store,
        view: &View,
        decision: &DecisionId,
        act: Act,
    ) -> Result<Option<Authorized>, AuthorError> {
        let auth = Authority::build(store);
        let Some(policy) = auth.policy(decision.namespace()) else { return Ok(None) };
        let set = view
            .latest
            .get(&decision.to_string())
            .and_then(|i| view.versions.get(*i))
            .map(|v| v.raw.set.clone())
            .unwrap_or_default();
        let target = Target::Decision { namespace: decision.namespace(), set: &set };
        self.authorized(store, act, target, Some(&policy.accept_role)).map(Some)
    }

    /// Sign the latest version of a decision as the configured identity.
    pub fn accept(&mut self, args: AcceptArgs) -> Result<Applied, AuthorError> {
        let store = self.load();
        let view = View::build(&store);
        let hash = signable_tip(&view, &args.decision)?;
        self.refuse_duplicate(&view, &args.decision, &hash)?;
        let held = self.decision_authority(&store, &view, &args.decision, Act::Accept)?;
        let acceptance = Acceptance {
            id: self.mint.mint_id("acc").map_err(AuthorError::Io)?,
            decision: args.decision.clone(),
            version: hash.clone(),
            actor: self.who.clone(),
            at: self.now,
            scope: AcceptanceScope::Version,
            expires_at: args.expires_at,
            under: held.as_ref().map(super::authority_ops::under_of).transpose()?,
            signature: String::new(),
        };
        let policy = Authority::build(&store).policy(args.decision.namespace()).cloned();
        let what = super::sign_ops::ToSign {
            namespace: args.decision.namespace(),
            ulid: acceptance.id.ulid(),
            bytes: crate::authority::payload::acceptance_bytes(&acceptance),
            own_key: None,
        };
        self.sign_under(&store, policy.as_ref(), what)?;
        let signed = self.pending_sidecars.iter().map(|s| format!("signed — sig/{}", s.file)).collect::<Vec<_>>();
        let mut candidate = self.shell(None)?;
        candidate.acceptances.push(acceptance);
        self.refusal_check(&store, &candidate, |_| false)?;
        let path = self.append_signed(&candidate)?;
        let mut lines = vec![format!(
            "{} accepted {} of {} — the signature names this exact state",
            self.who,
            hash.short(),
            args.decision
        )];
        if let Some(by) = held {
            lines.push(format!("under {} (`{}`)", by.grant, by.role));
        }
        lines.extend(signed);
        Ok(Applied { path, lines })
    }

    /// A second live signature of the same hash by the same actor adds
    /// nothing; refusing it keeps the log an act-record, not an echo.
    fn refuse_duplicate(
        &self,
        view: &View,
        decision: &DecisionId,
        hash: &crate::hash::VersionHash,
    ) -> Result<(), AuthorError> {
        let already = view.acceptances.iter().map(|a| a.acceptance).any(|a| {
            !view.is_revoked(a)
                && a.decision == *decision
                && a.version == *hash
                && a.actor == self.who
                && !a.is_expired(self.today())
        });
        if already {
            return Err(AuthorError::Conflict(format!(
                "{decision} already carries your live acceptance of {} — accepting twice adds nothing",
                hash.short()
            )));
        }
        Ok(())
    }

    /// Unsay a prior acceptance, with the reason on record (#66): a
    /// format-6 `rev:` entity naming the acceptance, which is never edited.
    /// In a namespace under policy the revoker needs the accept role, as an
    /// acceptor would; `L006` refuses a model revoker everywhere.
    pub fn revoke(&mut self, args: RevokeArgs) -> Result<Applied, AuthorError> {
        let store = self.load();
        let view = View::build(&store);
        if view.revoked.contains(&args.acceptance.to_string()) {
            return Err(AuthorError::Conflict(format!(
                "{} is already revoked — one reversal is enough",
                args.acceptance
            )));
        }
        let revoked = view.acceptances.iter().map(|a| a.acceptance).find(|a| a.id == args.acceptance);
        let held = match revoked {
            Some(acceptance) => self.decision_authority(&store, &view, &acceptance.decision, Act::RevokeAcceptance)?,
            None => None,
        };
        let under = held.as_ref().map(super::authority_ops::under_of).transpose()?;
        let revocation = self.revocation(Revocable::Acceptance(args.acceptance.clone()), args.reason.clone(), under)?;
        let id = revocation.id.as_ref().map(ToString::to_string).unwrap_or_default();
        if let Some(acceptance) = revoked {
            let ns = acceptance.decision.namespace();
            let policy = Authority::build(&store).policy(ns).cloned();
            let ulid = revocation.id.as_ref().map(|i| i.ulid().to_string()).unwrap_or_default();
            let what = super::sign_ops::ToSign {
                namespace: ns,
                ulid: &ulid,
                bytes: crate::authority::payload::revocation_bytes(&revocation),
                own_key: None,
            };
            self.sign_under(&store, policy.as_ref(), what)?;
        }
        let mut candidate = self.shell(None)?;
        candidate.revocations.push(revocation);
        self.refusal_check(&store, &candidate, |_| false)?;
        let path = self.append_signed(&candidate)?;
        Ok(Applied {
            path,
            lines: vec![format!("revoked {} by {id} — {}", args.acceptance, args.reason)],
        })
    }
}
