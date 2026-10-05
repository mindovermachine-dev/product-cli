//! What a landed file holds, entity by entity (D6, landed files immutable).
//!
//! A change-set file is read as YAML, not as a [`crate::changeset::ChangeSet`],
//! so the comparison is over what was written and not over what a loader
//! makes of it. Each item of each entity list is keyed `<list>/<id>`:
//!
//! - `id` when the item has one (every entity since format 6, and
//!   decisions, acceptances, grants, bindings, policies before it);
//! - `hash` for a version (its id is its digest);
//! - `acceptance` for a legacy revocation (one per acceptance revoked);
//! - otherwise the item's own canonical text.
//!
//! The change-set's header fields (`id`, `created_at`, `created_by`,
//! `parents`, `note`) are one entity, keyed `changeset`. `format` is not an
//! entity: re-declaring a file's format alone changes nothing it holds.

use std::collections::BTreeMap;

use serde_yaml::{Mapping, Value};

/// Every entity in a change-set file's text, by key. `None` when the text
/// is not a YAML mapping.
pub fn entities(text: &str) -> Option<BTreeMap<String, Value>> {
    let Value::Mapping(map) = serde_yaml::from_str::<Value>(text).ok()? else { return None };
    let mut out = BTreeMap::new();
    let mut header = Mapping::new();
    for (k, v) in map {
        let Some(name) = k.as_str() else { continue };
        match (name, v) {
            ("format", _) => {}
            (list, Value::Sequence(items)) => {
                for item in items {
                    out.insert(format!("{list}/{}", item_key(&item)), item);
                }
            }
            (_, v) => {
                header.insert(k, v);
            }
        }
    }
    if !header.is_empty() {
        out.insert(HEADER.to_string(), Value::Mapping(header));
    }
    Some(out)
}

/// The header entity's key.
pub const HEADER: &str = "changeset";

/// The key of one entity in `list` with `id` — what [`entities`] files it
/// under, for callers that hold the typed record.
pub fn key(list: &str, id: &str) -> String {
    format!("{list}/{id}")
}

/// The key of a revocation, legacy or entity.
pub fn revocation_key(r: &crate::authority::Revocation) -> String {
    match (&r.id, &r.acceptance) {
        (Some(id), _) => key("revocations", &id.to_string()),
        (None, Some(acc)) => key("revocations", &acc.to_string()),
        (None, None) => key("revocations", ""),
    }
}

fn item_key(item: &Value) -> String {
    for field in ["id", "hash", "acceptance"] {
        if let Some(s) = item.get(field).and_then(Value::as_str) {
            return s.to_string();
        }
    }
    serde_yaml::to_string(item).unwrap_or_default()
}

#[path = "landed_tests.rs"]
#[cfg(test)]
mod tests;
