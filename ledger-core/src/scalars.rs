//! Plain scalars in hashed content that are not text (rulings 55, 56; LP-3.4, LP-4.18).
//!
//! Hashed content is strings only. A string field is typed `String`, and a
//! YAML loader hands such a field a plain scalar as its source text, so the
//! typed record cannot say whether `statement: 1.5` was written as text or
//! as a number. A second reading of the file as an untyped
//! [`serde_yaml::Value`] can: a quoted scalar is always a string there, and
//! a plain one is resolved, so `1.5`, `1e3` and `.inf` arrive as floats and
//! `null` and `~` as null. Two faults follow, both `SCHEMA`:
//!
//! - a plain scalar that resolves to a float anywhere in a hashed entity
//!   (LP-4.18 step 7). A second implementation with a typed loader would
//!   read it as a number and hash something else, or refuse it;
//! - an explicit null in a required string field (LP-4.18 step 3). Null is
//!   absent, and a required field cannot be absent.
//!
//! No digest moves: no input that verified hashes differently. A file that
//! verified may now be refused.

use serde_yaml::Value;

use crate::finding::Finding;

/// The entity lists whose items are hashed or signed: every list but the
/// decision identity objects.
const HASHED_LISTS: &[&str] = &[
    "versions", "acceptances", "revocations", "grants", "grant_acceptances", "unavailabilities", "availabilities",
    "key_bindings", "policies",
];

/// The required string fields of each list, typed `String` in the wire
/// structs: an explicit null there would be hashed as the text `null`.
const REQUIRED_STRINGS: &[(&str, &[&str])] = &[
    ("versions", &["set", "statement"]),
    ("revocations", &["reason"]),
    ("grants", &["role"]),
    ("key_bindings", &["namespace"]),
    ("policies", &["namespace", "accept_role"]),
];

/// Every float and every explicit null in a required string field, in one
/// change-set file's text. Nothing when the text is not YAML: the typed
/// parse reports that.
pub fn faults(label: &str, text: &str) -> Vec<Finding> {
    let Ok(Value::Mapping(map)) = serde_yaml::from_str::<Value>(text) else { return Vec::new() };
    let mut out = Vec::new();
    for list in HASHED_LISTS {
        let Some(Value::Sequence(items)) = map.get(*list) else { continue };
        let required = REQUIRED_STRINGS.iter().find(|(l, _)| l == list).map(|(_, f)| *f).unwrap_or_default();
        for (i, item) in items.iter().enumerate() {
            let at = format!("{list}[{i}]");
            floats(item, &at, &mut out, label);
            for field in required {
                if matches!(item.get(*field), Some(Value::Null)) {
                    out.push(Finding::schema(label, format!(
                        "`{at}.{field}` is an explicit null, which is absent, and `{field}` is required (LP-3.4)"
                    )));
                }
            }
        }
    }
    out
}

fn floats(value: &Value, at: &str, out: &mut Vec<Finding>, label: &str) {
    match value {
        Value::Number(n) if n.is_f64() => out.push(Finding::schema(label, format!(
            "`{at}` is a plain scalar YAML reads as the float {n} — hashed content is strings only; quote it (LP-3.4, LP-4.18)"
        ))),
        Value::Sequence(items) => {
            for (i, v) in items.iter().enumerate() {
                floats(v, &format!("{at}[{i}]"), out, label);
            }
        }
        Value::Mapping(m) => {
            for (k, v) in m {
                floats(v, &format!("{at}.{}", k.as_str().unwrap_or("?")), out, label);
            }
        }
        Value::Tagged(t) => floats(&t.value, at, out, label),
        _ => {}
    }
}

#[path = "scalars_tests.rs"]
#[cfg(test)]
mod tests;
