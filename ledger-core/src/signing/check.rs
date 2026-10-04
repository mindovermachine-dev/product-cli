//! The signature gate: `L011`, `L012`, the trusted bindings (spec v1.8).
//!
//! **Requirement.** The policy in force for the entity's namespace at the
//! entity's position (D6) says which schemes are required; a policy change
//! is judged under the policy it replaces. An entity before its namespace's
//! first policy is not checked (D5 (c)). With `none` listed, an entity may
//! carry no sidecar; otherwise every listed scheme needs a sidecar that
//! verifies. A sidecar that is present always has to verify.
//!
//! **Trust.** Key bindings are judged first, in landing order: a binding
//! counts only when D7 allows its filer ([`crate::authority::filing`]) and,
//! where policy requires it, its signature verifies against the bindings
//! already trusted (a self-bound binding against the key it binds). The
//! trusted bindings are what `allowed_signers` is derived from, so an
//! unsigned binding for an existing holder never reaches the trust root.
//!
//! **Closed keys** (ruling 12, D6). An entity signed by a key whose window
//! a later binding closed is judged by order: dated after the close fails
//! `-Overify-time`, so `L011`; landed after the close, whatever its date,
//! is `L011`; dated *and* landed before it, an acceptance is a review item
//! ("needs re-acceptance") until a later valid acceptance of the same
//! version by the same actor affirms it, and `L012` once the policy's
//! re-acceptance deadline has passed.

use std::collections::BTreeMap;

use chrono::{Duration, NaiveDate};
use serde::Serialize;

use crate::authority::filing::may_file;
use crate::authority::signers::derive_from;
use crate::authority::{Authority, KeyBinding, Policy, Scheme};
use crate::finding::{Finding, VerifyClass};
use crate::landing::{Landing, Position};
use crate::store::Store;

use super::subject::{signable_ulids, subjects, Kind, Subject};
use super::{dsse, ssh, Sidecar};

/// An acceptance signed by a key closed after it, awaiting affirmation.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Reaccept {
    pub acceptance: String,
    pub actor: String,
    pub key: String,
    pub closed_by: String,
    /// After this date the acceptance stops being citable (`L012`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<NaiveDate>,
}

/// What the signature gate found.
#[derive(Debug, Default)]
pub struct Outcome<'a> {
    pub findings: Vec<Finding>,
    pub reaccept: Vec<Reaccept>,
    pub trusted: Vec<&'a KeyBinding>,
}

/// How one signature fared.
enum Verdict<'a> {
    Valid,
    /// Valid, by a key whose window this binding later closed.
    Closed(&'a KeyBinding),
    Invalid(String),
}

/// The bindings the gate trusts, in landing order — for `allowed_signers`.
pub fn trusted_bindings<'a>(store: &'a Store, landing: &Landing) -> Vec<&'a KeyBinding> {
    let mut out = Outcome::default();
    let all = ordered(subjects(store, landing));
    trust_bindings(store, landing, &all, &mut out);
    out.trusted
}

/// Run the gate over every signable entity.
pub fn check<'a>(store: &'a Store, landing: &Landing, today: NaiveDate) -> Outcome<'a> {
    let mut out = Outcome::default();
    let known = signable_ulids(store);
    for orphan in store.sidecars.iter().filter(|s| !known.contains(&s.ulid)) {
        out.findings.push(Finding::schema(&format!("sig/{}", orphan.file), "signs no signable entity in the log"));
    }
    let all = ordered(subjects(store, landing));
    trust_bindings(store, landing, &all, &mut out);
    let positions: BTreeMap<String, Position> = all.iter().map(|s| (s.id.clone(), s.position)).collect();
    let mut verdicts: BTreeMap<String, bool> = BTreeMap::new();
    let mut closed: Vec<(&Subject<'_>, &KeyBinding)> = Vec::new();
    for s in all.iter().filter(|s| s.kind != Kind::Binding) {
        let Some(policy) = governing(store, landing, s) else { continue };
        match judge(store, s, &policy, &out.trusted, &positions) {
            Ok(None) => {
                verdicts.insert(s.id.clone(), true);
            }
            Ok(Some(close)) => {
                verdicts.insert(s.id.clone(), false);
                closed.push((s, close));
            }
            Err(message) => {
                verdicts.insert(s.id.clone(), false);
                out.findings.push(Finding::new(VerifyClass::L011, &s.id, message));
            }
        }
    }
    review_closed(store, &closed, &verdicts, today, &mut out);
    out
}

fn ordered(mut all: Vec<Subject<'_>>) -> Vec<Subject<'_>> {
    all.sort_by(|a, b| (a.position.index, a.at, &a.id).cmp(&(b.position.index, b.at, &b.id)));
    all
}

/// The policy a subject is judged under: a policy change under the one it
/// replaces; anything else under the policy in force at its position.
fn governing(store: &Store, landing: &Landing, s: &Subject<'_>) -> Option<Policy> {
    let auth = Authority::as_of(store, landing, s.position);
    match s.policy.and_then(|p| p.replaces.as_ref()) {
        Some(prior) => Authority::build(store).policies.iter().find(|p| p.hash == *prior).map(|p| (*p).clone()),
        None => auth.policy(&s.namespace).cloned(),
    }
}

/// Pass one: the bindings, in order, each against the trust built so far.
fn trust_bindings<'a>(store: &'a Store, landing: &Landing, all: &[Subject<'a>], out: &mut Outcome<'a>) {
    let positions: BTreeMap<String, Position> = all.iter().map(|s| (s.id.clone(), s.position)).collect();
    for s in all.iter().filter(|s| s.kind == Kind::Binding) {
        let Some(b) = s.binding else { continue };
        let auth = Authority::as_of(store, landing, s.position);
        if let Err(rule) = may_file(&auth, b) {
            out.findings.push(Finding::schema(&s.id, format!("D7: {rule}")));
            continue;
        }
        let Some(policy) = auth.policy(&s.namespace).cloned() else { continue };
        let mut keys = out.trusted.clone();
        if b.self_bound {
            keys.push(b);
        }
        match judge(store, s, &policy, &keys, &positions) {
            Ok(_) => out.trusted.push(b),
            Err(message) => out.findings.push(Finding::new(
                VerifyClass::L011,
                &s.id,
                format!("{message} — the binding is not trusted and is left out of allowed_signers"),
            )),
        }
    }
}

/// Judge one subject's sidecars against its policy. `Ok(Some(close))`: valid
/// but under a key closed after it.
fn judge<'a>(
    store: &Store,
    s: &Subject<'_>,
    policy: &Policy,
    keys: &[&'a KeyBinding],
    positions: &BTreeMap<String, Position>,
) -> Result<Option<&'a KeyBinding>, String> {
    let none_ok = policy.schemes.contains(&Scheme::None);
    let sidecars: Vec<&Sidecar> = store.sidecars.iter().filter(|c| c.ulid == s.ulid).collect();
    let mut close = None;
    for scheme in policy.schemes.iter().filter(|sc| **sc != Scheme::None) {
        if !sidecars.iter().any(|c| c.scheme == *scheme) && !none_ok {
            return Err(format!("`{}`'s policy requires a `{scheme}` signature; none is filed", s.namespace));
        }
    }
    for sidecar in sidecars {
        match verify_one(s, sidecar, keys, positions) {
            Verdict::Valid => {}
            Verdict::Closed(c) => close = Some(c),
            Verdict::Invalid(why) => return Err(format!("its `{}` signature does not hold: {why}", sidecar.scheme)),
        }
    }
    Ok(close)
}

fn verify_one<'a>(
    s: &Subject<'_>,
    sidecar: &Sidecar,
    keys: &[&'a KeyBinding],
    positions: &BTreeMap<String, Position>,
) -> Verdict<'a> {
    let mine: Vec<&'a KeyBinding> = keys
        .iter()
        .copied()
        .filter(|k| k.act.opens() && k.principal == s.signer && k.namespace == s.namespace)
        .collect();
    let key = match sidecar.scheme {
        Scheme::Ssh => ssh_key(s, sidecar, keys, &mine),
        Scheme::Dsse => dsse_key(s, sidecar, keys, &mine),
        Scheme::None => Err("`none` has no sidecar".to_string()),
    };
    let key = match key {
        Ok(k) => k,
        Err(why) => return Verdict::Invalid(why),
    };
    let opened = positions.get(&key.id.to_string()).copied();
    if opened.is_some_and(|o| !o.not_after(&s.position)) {
        return Verdict::Invalid(format!("its key {} was bound after the act", key.id));
    }
    let Some(close) = keys.iter().copied().find(|c| c.closes.as_ref() == Some(&key.id)) else { return Verdict::Valid };
    let closed_at = positions.get(&close.id.to_string()).copied();
    match closed_at {
        Some(c) if s.position.before(&c) => Verdict::Closed(close),
        _ => Verdict::Invalid(format!(
            "its key {} was closed by {}, and the act is not dated and landed before the close (D6)",
            key.id, close.id
        )),
    }
}

fn ssh_key<'a>(s: &Subject<'_>, sidecar: &Sidecar, keys: &[&'a KeyBinding], mine: &[&'a KeyBinding]) -> Result<&'a KeyBinding, String> {
    let fp = ssh::signer_fingerprint(&s.namespace, &sidecar.bytes, &s.bytes)?;
    let key = mine
        .iter()
        .copied()
        .find(|k| k.key.as_deref().and_then(ssh::fingerprint).as_deref() == Some(fp.as_str()))
        .ok_or_else(|| format!("signed by {fp}, which is no key bound to {} in `{}`", s.signer, s.namespace))?;
    let text = derive_from(keys).unwrap_or_default();
    ssh::verify(&text, s.signer.as_str(), &s.namespace, &sidecar.bytes, &s.bytes, &s.at)?;
    Ok(key)
}

fn dsse_key<'a>(s: &Subject<'_>, sidecar: &Sidecar, keys: &[&'a KeyBinding], mine: &[&'a KeyBinding]) -> Result<&'a KeyBinding, String> {
    let in_window: Vec<&'a KeyBinding> = mine
        .iter()
        .copied()
        .filter(|k| k.at <= s.at)
        .filter(|k| !keys.iter().any(|c| c.closes.as_ref() == Some(&k.id) && c.at <= s.at))
        .collect();
    let blobs: Vec<String> = in_window.iter().filter_map(|k| k.key.clone()).collect();
    let blob = dsse::verify(&sidecar.bytes, &s.bytes, &blobs)?;
    in_window.into_iter().find(|k| k.key.as_deref() == Some(blob.as_str())).ok_or_else(|| "no bound key".to_string())
}

/// `L012` and the review list, for acceptances under a since-closed key.
fn review_closed(
    store: &Store,
    closed: &[(&Subject<'_>, &KeyBinding)],
    verdicts: &BTreeMap<String, bool>,
    today: NaiveDate,
    out: &mut Outcome<'_>,
) {
    let auth = Authority::build(store);
    let acceptances: Vec<&crate::acceptance::Acceptance> = store.log.iter().flat_map(|l| l.file.acceptances.iter()).collect();
    for (s, close) in closed.iter().filter(|(s, _)| s.kind == Kind::Acceptance) {
        let Some(a) = acceptances.iter().find(|a| a.id.to_string() == s.id) else { continue };
        let affirmed = acceptances.iter().any(|later| {
            later.actor == a.actor
                && later.decision == a.decision
                && later.version == a.version
                && later.at > a.at
                && verdicts.get(&later.id.to_string()).copied().unwrap_or(false)
        });
        if affirmed {
            continue;
        }
        let deadline = auth
            .policy(&s.namespace)
            .and_then(|p| p.reaccept_within_days)
            .map(|d| close.at.date_naive() + Duration::days(i64::from(d)));
        if deadline.is_some_and(|d| today > d) {
            out.findings.push(Finding::new(
                VerifyClass::L012,
                &s.id,
                format!(
                    "signed under a key {} closed on {}; not re-accepted by the policy's deadline {} — no longer citable",
                    close.closes.as_ref().map(ToString::to_string).unwrap_or_default(),
                    close.at.date_naive(),
                    deadline.map(|d| d.to_string()).unwrap_or_default()
                ),
            ));
        } else {
            out.reaccept.push(Reaccept {
                acceptance: s.id.clone(),
                actor: a.actor.to_string(),
                key: close.closes.as_ref().map(ToString::to_string).unwrap_or_default(),
                closed_by: close.id.to_string(),
                deadline,
            });
        }
    }
}

#[path = "check_tests.rs"]
#[cfg(test)]
mod tests;
