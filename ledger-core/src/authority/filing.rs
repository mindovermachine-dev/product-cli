//! Who may file a key binding (D7, ruled 2026-10-02) — one rule, two callers.
//!
//! `ledger identity add | rotate | revoke` refuses with it before writing,
//! and `verify` re-judges every filed binding with it against the authority
//! as it stood before the binding (`Authority::as_of`), so a hand-written
//! binding file meets the same rule as the verb.
//!
//! - **The genesis holder's first binding in the store** is self-bound:
//!   filed by the genesis holder for itself, carrying the genesis grant's
//!   `external_ref` as its mandate, signed by the key it binds. Once per
//!   store: in any later namespace the genesis holder's first binding is
//!   their own `add`, signed by a key of theirs already trusted elsewhere.
//! - **A principal's first key** (no open window in the namespace) is filed
//!   and signed by the genesis holder: `by` is the genesis holder,
//!   `principal` the new holder, `under` the genesis grant. Recovery from a
//!   lost key is the same act after a revoke.
//! - **A further `add`** is the principal's own, signed by a live key.
//! - **`rotate`** is the principal's own, on its own open window, signed by
//!   the key it closes.
//! - **`revoke`** is the principal's (its live key) or the genesis holder's
//!   (under the genesis grant).

use super::binding::{BindingAct, KeyBinding};
use super::view::Authority;

/// Whether `b` was filed by a party D7 allows, judged against `auth` as it
/// stood before `b`. `Err` names the rule broken.
pub fn may_file(auth: &Authority<'_>, b: &KeyBinding) -> Result<(), String> {
    let genesis = auth.genesis().filter(|g| auth.is_available(g, b.at));
    let by_genesis = genesis.is_some_and(|g| g.holder == b.by);
    let expected_under = (b.by != b.principal).then(|| genesis.map(|g| g.id.to_string())).flatten();
    if b.under.as_ref().map(ToString::to_string) != expected_under {
        return Err(match &expected_under {
            Some(g) => format!("filed for {} by {}, so it names the genesis grant {g} as `under`", b.principal, b.by),
            None => "a principal's act on its own keys names no `under`".to_string(),
        });
    }
    let open_in_ns = |who| open_window(auth, b, who, Some(&b.namespace));
    match (b.act, &b.closes) {
        (BindingAct::Add, _) if b.self_bound => self_bound(auth, b, genesis.and_then(|g| g.external_ref.clone()), by_genesis),
        (BindingAct::Add, _) if b.by == b.principal => match open_in_ns(&b.principal) {
            true => Ok(()),
            false if by_genesis && open_window(auth, b, &b.principal, None) => Ok(()),
            false => Err(format!(
                "{} has no live key in `{}`: a principal's first key is filed by the genesis holder (D7)",
                b.principal, b.namespace
            )),
        },
        (BindingAct::Add, _) if by_genesis => match open_in_ns(&b.principal) {
            false => Ok(()),
            true => Err(format!("{} already has a live key in `{}` — a further key is the principal's own act", b.principal, b.namespace)),
        },
        (BindingAct::Add, _) => Err(format!("{} binds a key for {}: only the genesis holder files another's first key", b.by, b.principal)),
        (act, Some(closed)) => closing(auth, b, act, closed, by_genesis),
        (_, None) => Err(format!("`{}` names the binding it closes", b.act)),
    }
}

/// Whether `who` holds an open key window (other than `b` itself) in `ns`,
/// or in any namespace when `ns` is `None`.
fn open_window(auth: &Authority<'_>, b: &KeyBinding, who: &crate::identity::Identity, ns: Option<&str>) -> bool {
    auth.bindings.iter().any(|o| {
        o.act.opens()
            && ns.is_none_or(|ns| o.namespace == ns)
            && o.principal == *who
            && o.id != b.id
            && !auth.bindings.iter().any(|c| c.closes.as_ref() == Some(&o.id))
    })
}

/// A self-bound binding: the genesis holder's first in the store, under
/// the genesis mandate.
fn self_bound(auth: &Authority<'_>, b: &KeyBinding, mandate: Option<String>, by_genesis: bool) -> Result<(), String> {
    let first = auth.bindings.iter().all(|o| o.principal != b.principal || o.id == b.id);
    if by_genesis && b.principal == b.by && first && b.mandate == mandate {
        return Ok(());
    }
    Err("a self-bound binding is the genesis holder's first in the store, under the genesis mandate — \
         in a later namespace their first key is signed by a key of theirs already trusted"
        .into())
}

fn closing(
    auth: &Authority<'_>,
    b: &KeyBinding,
    act: BindingAct,
    closed: &crate::id::KeyBindingId,
    by_genesis: bool,
) -> Result<(), String> {
    let open = auth
        .bindings
        .iter()
        .find(|o| o.id == *closed && o.act.opens())
        .ok_or_else(|| format!("{closed} opens no key window"))?;
    if auth.bindings.iter().any(|c| c.closes.as_ref() == Some(closed) && c.id != b.id) {
        return Err(format!("{closed}'s window is already closed"));
    }
    if open.principal != b.principal || open.namespace != b.namespace {
        return Err(format!("{closed} binds {}'s key in `{}`", open.principal, open.namespace));
    }
    let allowed = b.by == b.principal || (act == BindingAct::Revoke && by_genesis);
    if !allowed {
        return Err(format!("{closed} binds {}'s key; {} may not {act} it", open.principal, b.by));
    }
    Ok(())
}

#[path = "filing_tests.rs"]
#[cfg(test)]
mod tests;
