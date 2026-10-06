//! A close ends the key, not the binding (ruled 2026-10-06).
//!
//! Closing any binding of a principal's key — by `rotate` or `revoke`, in
//! any namespace — closes that key in every namespace of the store, from the
//! close's position (D6). The key is the principal and the key blob; every
//! check that asks whether a key is closed asks it here, over every binding
//! of that key, never only the binding a signature happened to match.

use super::binding::KeyBinding;

/// Whether `a` and `b` open the same principal's same key.
pub fn same_key(a: &KeyBinding, b: &KeyBinding) -> bool {
    a.act.opens() && b.act.opens() && a.principal == b.principal && a.key.is_some() && a.key == b.key
}

/// Every close among `bindings` that ends `b`'s key: a close of any opening
/// binding of the same principal and key, in any namespace.
pub fn closes_of<'a>(bindings: &[&'a KeyBinding], b: &KeyBinding) -> Vec<&'a KeyBinding> {
    closes_among(bindings, bindings, b)
}

/// Every close among `closes` that ends `b`'s key, its target looked up in
/// `filed` — every binding filed, trusted or not: a close of any binding of
/// the key ends it, a hand-filed duplicate's included.
pub fn closes_among<'a>(closes: &[&'a KeyBinding], filed: &[&KeyBinding], b: &KeyBinding) -> Vec<&'a KeyBinding> {
    closes
        .iter()
        .copied()
        .filter(|c| c.closes.as_ref().is_some_and(|id| filed.iter().any(|o| o.id == *id && same_key(o, b))))
        .collect()
}

/// Whether some close among `bindings` ends `b`'s key.
pub fn is_closed(bindings: &[&KeyBinding], b: &KeyBinding) -> bool {
    !closes_of(bindings, b).is_empty()
}

/// The binding among `bindings` that already opens `b`'s key in `b`'s
/// namespace and is still open — `b` would bind it twice. A key is bound
/// once per namespace (ruled 2026-10-06).
pub fn already_open<'a>(bindings: &[&'a KeyBinding], b: &KeyBinding) -> Option<&'a KeyBinding> {
    bindings.iter().copied().find(|o| o.id != b.id && o.namespace == b.namespace && same_key(o, b) && !is_closed(bindings, o))
}

#[path = "key_close_tests.rs"]
#[cfg(test)]
mod tests;
