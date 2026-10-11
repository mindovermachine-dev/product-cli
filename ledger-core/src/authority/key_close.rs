//! A close ends the key, not the binding — in its own namespace (ruled
//! 2026-10-06; per namespace by ruling 47, LP-6.32).
//!
//! Closing any binding of a principal's key in a namespace — by `rotate` or
//! `revoke` — closes that key in that namespace, from the close's position
//! (D6). The key is the principal, the key blob and the namespace; every
//! check that asks whether a key is closed asks it here, over every binding
//! of that key in the namespace, never only the binding a signature
//! happened to match. The same key bound in another namespace is another
//! key there, judged by that namespace alone: a key closed in one and open
//! in another is a repository notice (ruling 69), never a finding. Which
//! keys may be bound at all — once per namespace, never once closed there,
//! never another principal's there — is [`refusal`].

use super::binding::KeyBinding;

/// Whether `a` and `b` open the same principal's same key in the same
/// namespace (ruling 47: a key is bound, closed and trusted per namespace).
pub fn same_key(a: &KeyBinding, b: &KeyBinding) -> bool {
    a.act.opens() && b.act.opens() && a.principal == b.principal && a.namespace == b.namespace && a.key.is_some() && a.key == b.key
}

/// Every close among `bindings` that ends `b`'s key: a close of any opening
/// binding of the same principal and key in `b`'s namespace.
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
/// once per namespace (ruled 2026-10-06, ruling 1).
pub fn already_open<'a>(bindings: &[&'a KeyBinding], b: &KeyBinding) -> Option<&'a KeyBinding> {
    bindings.iter().copied().find(|o| o.id != b.id && o.namespace == b.namespace && same_key(o, b) && !is_closed(bindings, o))
}

/// Why `b` may not open its key, judged against the bindings `bound` before
/// it (trusted, in landing order), a close's target looked up in `filed`
/// (ruled 2026-10-06). `None`: it may.
///
/// Each rule looks at the binding's own namespace (ruling 47, LP-4.37):
///
/// - **A key belongs to one principal** (ruling 4): a key that is, or was,
///   bound to a different principal in the namespace is not bound again
///   there. The same key bound to two principals in two namespaces is a
///   repository notice.
/// - **A closed key is never bound again** (ruling 3): a key closed for its
///   principal in the namespace is not bound again there — the genesis
///   holder vouching for it as someone's first key included.
/// - **A key is bound once per namespace** (ruling 1).
pub fn refusal(bound: &[&KeyBinding], filed: &[&KeyBinding], b: &KeyBinding) -> Option<String> {
    if !b.act.opens() || b.key.is_none() {
        return None;
    }
    if let Some(other) = bound.iter().find(|o| o.act.opens() && o.namespace == b.namespace && o.key == b.key && o.principal != b.principal) {
        return Some(format!(
            "this key is bound to {} ({} in `{}`) — a key belongs to one principal, so it is never bound to {} there",
            other.principal, other.id, other.namespace, b.principal
        ));
    }
    if let Some(close) = closes_among(bound, filed, b).first() {
        return Some(format!(
            "this key was closed for {} in `{}` by {} — a closed key is never bound again there",
            b.principal, close.namespace, close.id
        ));
    }
    already_open(bound, b).map(|open| {
        format!("{} already has this key open in `{}` ({}) — a key is bound once per namespace", b.principal, b.namespace, open.id)
    })
}

#[path = "key_close_tests.rs"]
#[cfg(test)]
mod tests;
