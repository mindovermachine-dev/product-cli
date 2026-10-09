//! The two key notices of ruling 69: a key closed in one namespace yet open in another, or bound to two principals in two namespaces.
//!
//! Keys are per namespace (ruling 47, LP-6.32): a close ends a key in its
//! own namespace, and the same key bound elsewhere is another key there.
//! What a store-wide reading once refused is now reported, never a finding
//! (LP-8.31): the holder decides whether a key closed in `A` should be
//! closed in `B` too (`ledger identity revoke` there), and two principals
//! sharing one key across namespaces is theirs to know.

use std::collections::{BTreeMap, BTreeSet};

use crate::authority::key_close;
use crate::authority::KeyBinding;
use crate::store::Store;

/// A key closed in some namespaces and open in others.
#[derive(Debug, Default, Clone, serde::Serialize, PartialEq, Eq)]
pub struct KeySplit {
    pub holder: String,
    /// The key, as `<type> <blob>`.
    pub key: String,
    pub closed_in: Vec<String>,
    pub open_in: Vec<String>,
}

/// One key bound to different principals in different namespaces.
#[derive(Debug, Default, Clone, serde::Serialize, PartialEq, Eq)]
pub struct KeyShared {
    /// The key, as `<type> <blob>`.
    pub key: String,
    /// `(principal, namespace)` pairs, sorted.
    pub bound_to: Vec<(String, String)>,
}

fn key_of(b: &KeyBinding) -> Option<String> {
    Some(format!("{} {}", b.key_type.as_deref()?, b.key.as_deref()?))
}

/// Every key of a principal that is closed in one namespace and open in
/// another, over all filed bindings.
pub fn split(store: &Store) -> Vec<KeySplit> {
    let filed: Vec<&KeyBinding> = store.log.iter().flat_map(|l| l.file.key_bindings.iter()).collect();
    let mut by_key: BTreeMap<(String, String), (BTreeSet<String>, BTreeSet<String>)> = BTreeMap::new();
    for b in filed.iter().filter(|b| b.act.opens()) {
        let Some(key) = key_of(b) else { continue };
        let entry = by_key.entry((b.principal.to_string(), key)).or_default();
        match key_close::is_closed(&filed, b) {
            true => entry.0.insert(b.namespace.clone()),
            false => entry.1.insert(b.namespace.clone()),
        };
    }
    by_key
        .into_iter()
        .filter(|(_, (closed, open))| !closed.is_empty() && !open.is_empty())
        .map(|((holder, key), (closed, open))| KeySplit {
            holder,
            key,
            closed_in: closed.into_iter().filter(|ns| !open.contains(ns)).collect(),
            open_in: open.into_iter().collect(),
        })
        .filter(|s| !s.closed_in.is_empty())
        .collect()
}

/// Every key bound to more than one principal across namespaces (within
/// one namespace that is a schema fault, LP-4.37).
pub fn shared(store: &Store) -> Vec<KeyShared> {
    let mut by_key: BTreeMap<String, BTreeSet<(String, String)>> = BTreeMap::new();
    for b in store.log.iter().flat_map(|l| l.file.key_bindings.iter()).filter(|b| b.act.opens()) {
        let Some(key) = key_of(b) else { continue };
        by_key.entry(key).or_default().insert((b.principal.to_string(), b.namespace.clone()));
    }
    by_key
        .into_iter()
        .filter(|(_, bound)| bound.iter().map(|(p, _)| p).collect::<BTreeSet<_>>().len() > 1)
        .map(|(key, bound)| KeyShared { key, bound_to: bound.into_iter().collect() })
        .collect()
}
