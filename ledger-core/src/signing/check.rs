//! The signature gate: `L011`, `L012`, the trusted bindings (spec v1.8).
//!
//! **Requirement.** The policy in force for the entity's namespace at the
//! entity's position (D6) says which schemes are required; a policy change
//! is judged under the policy it replaces. An entity before its namespace's
//! first policy is not checked (D5 (c)) — except a key binding, which is
//! never exempt: it is judged under that first policy (ruled 2026-10-05).
//! With `none` listed, an entity may
//! carry no sidecar; otherwise every listed scheme needs a sidecar that
//! verifies. A sidecar that is present always has to verify.
//!
//! **Trust.** Key bindings are judged first, in landing order: a binding
//! counts only when D7 allows its filer ([`crate::authority::filing`]) and,
//! where policy requires it, its signature verifies against the bindings
//! already trusted (a self-bound binding against the key it binds). The
//! trusted bindings are what `allowed_signers` is derived from, so an
//! unsigned binding for an existing holder never reaches the trust root.
//! A binding before its namespace's first policy is judged under that
//! policy; one in a namespace no policy governs is a schema fault. Every
//! filed binding ends trusted or with a finding — none is left silent.
//!
//! **Closed keys** (ruling 12, D6). A close ends the key, not the binding
//! (ruled 2026-10-06): closing any binding of a principal's key closes it in
//! every namespace, and every check asks over all bindings of the matched
//! key ([`key_close`]). An entity signed by a closed key is judged by order:
//! dated after the close fails `-Overify-time`, so `L011`; landed after the
//! close, whatever its date, is `L011`, naming the close and its namespace;
//! dated *and* landed before it, an acceptance goes to the re-acceptance
//! review (`L012`, `review.rs`). A key already open in a namespace is never
//! bound there twice: a second binding is a schema fault.

use std::collections::BTreeMap;

use chrono::NaiveDate;

use crate::authority::filing::may_file;
use crate::authority::key_close;
use crate::authority::signers::{derive_from, filed};
use crate::authority::{Authority, KeyBinding, Policy, Scheme};
use crate::finding::{Finding, VerifyClass};
use crate::landing::{Landing, Position};
use crate::store::Store;

use super::review::review_closed;
pub use super::review::Reaccept;
use super::subject::{signable_ulids, subjects, Kind, Subject};
use super::{dsse, ssh, Sidecar};

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
    if needs_ssh(store, landing, &all) {
        if let Err(why) = ssh::preflight() {
            out.findings.push(Finding::new(VerifyClass::L011, "ssh-keygen", why));
        }
    }
    trust_bindings(store, landing, &all, &mut out);
    let positions: BTreeMap<String, Position> = all.iter().map(|s| (s.id.clone(), s.position)).collect();
    let mut verdicts: BTreeMap<String, bool> = BTreeMap::new();
    let mut closed: Vec<(&Subject<'_>, &KeyBinding)> = Vec::new();
    for s in all.iter().filter(|s| s.kind != Kind::Binding) {
        if let Some(p) = first_policy(s) {
            if let Some(Err(message)) = judge_first_policy(store, s, p, &out.trusted, &positions) {
                out.findings.push(Finding::new(VerifyClass::L011, &s.id, message));
            }
            continue;
        }
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

/// Whether any check in this store needs `ssh-keygen`: an `ssh` sidecar,
/// or an entity whose policy requires `ssh`. A first policy needs one only
/// through a sidecar or a key its author held (judged in the loop).
fn needs_ssh(store: &Store, landing: &Landing, all: &[Subject<'_>]) -> bool {
    store.sidecars.iter().any(|c| c.scheme == Scheme::Ssh)
        || all
            .iter()
            .filter(|s| first_policy(s).is_none())
            .any(|s| governing(store, landing, s).is_some_and(|p| p.schemes.contains(&Scheme::Ssh)))
}

/// The subject's policy, when it is a namespace's first (it replaces none).
fn first_policy<'a>(s: &Subject<'a>) -> Option<&'a Policy> {
    s.policy.filter(|p| p.replaces.is_none())
}

/// A namespace's first policy (#96): signed under its own schemes when its
/// author held a live trusted key at its position — in this namespace or
/// another, which for this check stands here, as [`carried_over`] does for
/// a binding. A sidecar that names it is verified whatever. `None`: neither,
/// so nothing is required.
fn judge_first_policy(
    store: &Store,
    s: &Subject<'_>,
    p: &Policy,
    trusted: &[&KeyBinding],
    positions: &BTreeMap<String, Position>,
) -> Option<Result<(), String>> {
    let at = |id: &crate::id::KeyBindingId| positions.get(&id.to_string()).copied();
    let closed = |k: &KeyBinding| {
        key_close::closes_among(trusted, &filed(store), k).iter().any(|c| at(&c.id).is_some_and(|c| !s.position.before(&c)))
    };
    let held = trusted.iter().any(|k| {
        k.principal == s.signer && k.act.opens() && at(&k.id).is_some_and(|o| o.not_after(&s.position)) && !closed(k)
    });
    if !held && !store.sidecars.iter().any(|c| c.ulid == s.ulid) {
        return None;
    }
    let here: Vec<KeyBinding> =
        trusted.iter().filter(|k| k.principal == s.signer).map(|k| KeyBinding { namespace: s.namespace.clone(), ..(*k).clone() }).collect();
    let keys: Vec<&KeyBinding> = here.iter().collect();
    Some(judge(store, s, p, &keys, positions).map(|_| ()))
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
        if let Some(open) = key_close::already_open(&out.trusted, b) {
            let rule = format!("{} already has this key open in `{}` ({}) — a key is bound once per namespace", b.principal, b.namespace, open.id);
            out.findings.push(Finding::schema(&s.id, rule));
            continue;
        }
        let auth = Authority::as_of(store, landing, s.position);
        if let Err(rule) = may_file(&auth, b) {
            out.findings.push(Finding::schema(&s.id, format!("D7: {rule}")));
            continue;
        }
        // A binding is never exempt as a pre-policy act (ruled 2026-10-05):
        // before its namespace's first policy it is judged under that first
        // policy's requirement. In a namespace no policy governs at all it is
        // already a schema fault (`authority/references.rs`) and stays
        // untrusted.
        let policy = auth.policy(&s.namespace).cloned().or_else(|| first_policy_of(store, &s.namespace));
        let Some(policy) = policy else { continue };
        let mut keys = out.trusted.clone();
        if b.self_bound {
            keys.push(b);
        }
        let elsewhere = carried_over(&out.trusted, b);
        keys.extend(elsewhere.iter());
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

/// A namespace's first policy (it replaces none), wherever it landed.
fn first_policy_of(store: &Store, namespace: &str) -> Option<Policy> {
    Authority::build(store).policies.iter().find(|p| p.namespace == namespace && p.replaces.is_none()).map(|p| (*p).clone())
}

/// The genesis holder's first key in a later namespace is their own `add`,
/// signed by a key of theirs already trusted in another namespace (D7). For
/// that check alone, those keys stand in this namespace: the signature is
/// made in `ledger-accept@<this namespace>`, and `allowed_signers` scopes
/// each line to its own. `may_file` has already ruled who may file it.
fn carried_over(trusted: &[&KeyBinding], b: &KeyBinding) -> Vec<KeyBinding> {
    let own_add = b.act == crate::authority::BindingAct::Add && !b.self_bound && b.by == b.principal;
    let here = trusted.iter().any(|k| k.principal == b.by && k.namespace == b.namespace && k.act.opens());
    if !own_add || here {
        return Vec::new();
    }
    trusted
        .iter()
        .filter(|k| k.principal == b.by && k.act.opens())
        .map(|k| KeyBinding { namespace: b.namespace.clone(), ..(*k).clone() })
        .collect()
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
    // `none` is exclusive: a `[none]` policy requires nothing, any other
    // requires every scheme it lists. A sidecar that is present is always
    // verified.
    let sidecars: Vec<&Sidecar> = store.sidecars.iter().filter(|c| c.ulid == s.ulid).collect();
    let mut close = None;
    for scheme in policy.schemes.iter().filter(|sc| **sc != Scheme::None) {
        if !sidecars.iter().any(|c| c.scheme == *scheme) {
            return Err(format!("`{}`'s policy requires a `{scheme}` signature; none is filed", s.namespace));
        }
    }
    let filed = filed(store);
    for sidecar in sidecars {
        match verify_one(s, sidecar, keys, (&filed, positions)) {
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
    (filed, positions): (&[&KeyBinding], &BTreeMap<String, Position>),
) -> Verdict<'a> {
    let mine: Vec<&'a KeyBinding> = keys
        .iter()
        .copied()
        .filter(|k| k.act.opens() && k.principal == s.signer && k.namespace == s.namespace)
        .collect();
    // The matched binding, and whether the signature holds cryptographically
    // — asked after the key's closes, so a closed key's finding names its close.
    let key = match sidecar.scheme {
        Scheme::Ssh => ssh_key(s, sidecar, keys, &mine, (filed, positions)),
        Scheme::Dsse => dsse_key(s, sidecar, keys, &mine).map(|k| (k, Ok(()))),
        Scheme::None => Err("`none` has no sidecar".to_string()),
    };
    let (key, holds) = match key {
        Ok(k) => k,
        Err(why) => return Verdict::Invalid(why),
    };
    let opened = positions.get(&key.id.to_string()).copied();
    if opened.is_some_and(|o| !o.not_after(&s.position)) {
        return Verdict::Invalid(format!("its key {} was bound after the act", key.id));
    }
    // A close ends the key, not the binding (ruled 2026-10-06): every close
    // of any binding of this key, in any namespace, counts.
    let closes = key_close::closes_among(keys, filed, key);
    let at = |c: &KeyBinding| positions.get(&c.id.to_string()).copied();
    if let Some(close) = closes.iter().copied().find(|c| !at(c).is_some_and(|p| s.position.before(&p))) {
        return Verdict::Invalid(format!(
            "its key {} was closed in `{}` by {}, and the act is not dated and landed before the close (D6) — a close ends the key in every namespace, so it signs nothing in `{}`",
            key.id, close.namespace, close.id, s.namespace
        ));
    }
    if let Err(why) = holds {
        return Verdict::Invalid(why);
    }
    let earliest = closes.into_iter().reduce(|a, b| match (at(a), at(b)) {
        (Some(pa), Some(pb)) if pb.before(&pa) => b,
        _ => a,
    });
    earliest.map_or(Verdict::Valid, Verdict::Closed)
}

/// Whether `k` was bound no later than the act.
fn opened_by(positions: &BTreeMap<String, Position>, k: &KeyBinding, s: &Subject<'_>) -> bool {
    positions.get(&k.id.to_string()).is_none_or(|o| o.not_after(&s.position))
}

/// The binding a sidecar's key matches — of every binding of that key, one
/// opened no later than the act when there is one. Whether the key is
/// closed is asked of all of them ([`key_close`]), never this one alone.
fn ssh_key<'a>(
    s: &Subject<'_>,
    sidecar: &Sidecar,
    keys: &[&'a KeyBinding],
    mine: &[&'a KeyBinding],
    (filed, positions): (&[&KeyBinding], &BTreeMap<String, Position>),
) -> Result<(&'a KeyBinding, Result<(), String>), String> {
    ssh::preflight().map_err(|why| format!("could not be checked — {why}"))?;
    let fp = ssh::signer_fingerprint(&s.namespace, &sidecar.bytes, &s.bytes)?;
    let key = mine
        .iter()
        .copied()
        .filter(|k| k.key.as_deref().and_then(ssh::fingerprint).as_deref() == Some(fp.as_str()))
        .reduce(|a, b| if opened_by(positions, a, s) { a } else { b })
        .ok_or_else(|| format!("signed by {fp}, which is no key bound to {} in `{}`", s.signer, s.namespace))?;
    let text = derive_from(keys, filed).unwrap_or_default();
    Ok((key, ssh::verify(&text, s.signer.as_str(), &s.namespace, &sidecar.bytes, &s.bytes, &s.at)))
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

#[path = "check_tests.rs"]
#[cfg(test)]
mod tests;
