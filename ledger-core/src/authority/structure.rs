//! Single-entry schema rules for the authority records — the parse gate's share.
//!
//! What one record must say about itself, judged without reading any other
//! record: a primary grant carries no limits; the genesis grant is
//! self-granted, scope `*`, primary, with an external reference, and only
//! the genesis carries one; a `ns:` scope names the namespace the grant is
//! filed in (ruling 47); an interval ends after it starts; a key-binding
//! act carries exactly the fields its act defines; a policy names at least
//! one scheme. Every fault is a `SCHEMA` finding — the shapes in
//! `ledger/spec/authority/ledger-authority-shapes.ttl`, enforced where an
//! outside implementation must enforce them too.

use crate::changeset::ChangeSet;
use crate::finding::Finding;
use crate::set::DecisionSet;
use crate::store::Store;

use super::binding::{BindingAct, KeyBinding};
use super::grant::{Grant, GrantScope};
use super::role::Role;

/// Faults in one role file: format, id, filename, duplicates, capabilities.
pub fn role_faults(role: &Role, stem: &str, store: &Store) -> Vec<String> {
    let mut out = Vec::new();
    if role.format < crate::format::AUTHORITY_FORMAT {
        out.push("a role file is a format 6 record — declare `format: 6`".to_string());
    }
    if let Err(e) = DecisionSet::validate_id(&role.id) {
        out.push(e);
    }
    if stem != role.id {
        out.push(format!("declares id `{}` but is filed as `{stem}`", role.id));
    }
    if store.roles.iter().any(|r| r.id == role.id && r.namespace == role.namespace) {
        out.push(format!("role `{}` is declared twice in `{}`", role.id, role.namespace));
    }
    if role.may.is_empty() {
        out.push("a role that may do nothing grants nothing — `may` names at least one capability".to_string());
    }
    out
}

/// Faults every authority record in one change-set carries on its own,
/// read under the namespace `ns` whose directory holds the file.
pub fn entry_faults(cs: &ChangeSet, ns: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    for g in &cs.grants {
        out.extend(grant_faults(g).into_iter().map(|m| Finding::schema(&g.id.to_string(), m)));
        if let GrantScope::Namespace(other) = &g.scope {
            if other != ns {
                out.push(Finding::schema(&g.id.to_string(), format!(
                    "is scoped `ns:{other}` but filed under `{ns}` — a scope is read inside its own namespace, so the grant could never have effect (LP-6.16, ruling 47); `*` or `ns:{ns}` is the whole of it"
                )));
            }
        }
    }
    for u in &cs.unavailabilities {
        if u.until.is_some_and(|until| until <= u.from) {
            out.push(Finding::schema(&u.id.to_string(), "`until` must fall after `from`"));
        }
    }
    for b in &cs.key_bindings {
        out.extend(binding_faults(b).into_iter().map(|m| Finding::schema(&b.id.to_string(), m)));
    }
    for p in &cs.policies {
        if p.schemes.is_empty() {
            out.push(Finding::schema(&p.id.to_string(), "a policy names at least one scheme"));
        }
        // `none` is exclusive (ruled 2026-10-04, amending D4): governed and
        // unsigned, never "signed, optionally".
        if p.schemes.contains(&super::Scheme::None) && p.schemes.len() > 1 {
            out.push(Finding::schema(
                &p.id.to_string(),
                "`none` is exclusive — a policy is governed and unsigned (`none` alone), or lists the schemes it requires",
            ));
        }
        if p.reaccept_within_days == Some(0) {
            out.push(Finding::schema(&p.id.to_string(), "a re-acceptance deadline is at least one day"));
        }
    }
    out
}

fn grant_faults(g: &Grant) -> Vec<&'static str> {
    let mut out = Vec::new();
    if g.order.is_primary() && !g.limits.is_empty() {
        out.push("a primary grant must not carry limits");
    }
    if g.genesis {
        let shaped = g.holder == g.granted_by
            && g.scope == GrantScope::All
            && g.order.is_primary()
            && g.external_ref.as_deref().is_some_and(|r| !r.trim().is_empty());
        if !shaped {
            out.push("a genesis grant must be self-granted, scope *, order primary, with an external ref");
        }
        if g.supersedes.is_some() {
            out.push("a genesis grant supersedes nothing — rotating the genesis is its own act");
        }
    } else if g.external_ref.is_some() {
        out.push("`external_ref` is permitted on the genesis grant only");
    }
    out
}

fn binding_faults(b: &KeyBinding) -> Vec<&'static str> {
    let mut out = Vec::new();
    let keyed = b.key.is_some() && b.key_type.is_some();
    let spaced = |v: &Option<String>| v.as_deref().is_some_and(|s| s.is_empty() || s.contains(char::is_whitespace));
    if b.act.opens() && !keyed {
        out.push("`add` and `rotate` carry `key_type` and `key`");
    }
    if !b.act.opens() && (b.key.is_some() || b.key_type.is_some()) {
        out.push("`revoke` closes a window and carries no key");
    }
    if spaced(&b.key) || spaced(&b.key_type) {
        out.push("`key_type` and `key` are single tokens (`ssh-ed25519`, base64)");
    }
    match (b.act, &b.closes) {
        (BindingAct::Add, Some(_)) => out.push("`add` opens a window and closes none"),
        (BindingAct::Rotate | BindingAct::Revoke, None) => {
            out.push("`rotate` and `revoke` name the binding they close")
        }
        _ => {}
    }
    if b.self_bound && (b.act != BindingAct::Add || b.mandate.is_none()) {
        out.push("a self-bound binding is an `add` carrying the genesis mandate");
    }
    if !b.self_bound && b.mandate.is_some() {
        out.push("`mandate` is carried only by the self-bound genesis binding");
    }
    out
}
