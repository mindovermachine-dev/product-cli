//! What the record held at landing, it must still hold (D6's landing).
//!
//! **Landed entities are immutable.** Every entity a landed log file has
//! held on the verified commit's first-parent line must be present, and
//! identical to what landed, at the verified commit (the working tree
//! included). An entity is an item of an entity list, or the change-set's
//! header ([`crate::landed`]); re-declaring `format:` alone changes none,
//! but the declaration itself is compared along the file's history: the
//! one change it may undergo is a correction (LP-3.16, ruling 58).
//! Appending a new entity to a landed file is not a change to any other
//! one — it lands where it was appended ([`Landing::entity_index`]).
//!
//! Role files and signature files are one entity each: a landed role file
//! never changes (Session A close-out §3 question 8, ruled write-once), and
//! a landed sidecar never changes or goes.
//!
//! These are the cases the earlier rules named: a removed policy (opting
//! in is one-way, D5 (c)), a removed revocation or key close (deleting it
//! would revive what it ended), an edited role. All are `L007` by the
//! `L010` amendment mechanism: the stored content no longer matches content
//! the record already fixed.

use std::collections::BTreeMap;
use std::path::Path;

use serde_yaml::Value;

use crate::finding::{Finding, VerifyClass};
use crate::landed::entities;
use crate::landing::{content_at, file_versions, touched_after_landing, Landing};
use crate::store::Store;

/// Every history finding, when git answered.
pub fn findings(store: &Store, landing: &Landing) -> Vec<Finding> {
    if !landing.available {
        return Vec::new();
    }
    let mut out = Vec::new();
    for path in touched_after_landing(&store.root) {
        let versions = file_versions(&store.root, "HEAD", &path);
        let Some(first) = versions.first() else { continue };
        let now = std::fs::read_to_string(store.root.join(&path)).ok();
        if crate::layout::is_log(&path) {
            out.extend(log_file(&store.root, &path, &versions, now.as_deref()));
            out.extend(format_changes(&store.root, &path, &versions, now.as_deref()));
        } else if let Some(landed) = content_at(&store.root, first, &path) {
            out.extend(whole_file(&path, first, &landed, now.as_deref()));
        }
    }
    out
}

/// Each entity any landed version of the file held, against the file now.
fn log_file(root: &Path, path: &str, versions: &[String], now: Option<&str>) -> Vec<Finding> {
    let mut held: BTreeMap<String, (Value, &str)> = BTreeMap::new();
    for commit in versions {
        let Some(text) = content_at(root, commit, path) else { continue };
        for (key, value) in entities(&text).unwrap_or_default() {
            held.entry(key).or_insert((value, commit.as_str()));
        }
    }
    let current = now.and_then(entities).unwrap_or_default();
    held.into_iter()
        .filter_map(|(key, (value, commit))| {
            let what = match current.get(&key) {
                None => "is gone",
                Some(v) if *v != value => "has changed",
                Some(_) => return None,
            };
            let (list, id) = key.split_once('/').unwrap_or((key.as_str(), ""));
            let subject = if id.is_empty() { path.to_string() } else { id.to_string() };
            Some(Finding::new(
                VerifyClass::L007,
                &subject,
                format!(
                    "{} landed in {} ({path}) and {what} — a landed entity is never edited or removed{}",
                    entity_name(list),
                    short(commit),
                    consequence(list)
                ),
            ))
        })
        .collect()
}

/// Each change of the file's `format:` declaration along its history, the
/// working tree last (ruling 58). The one change allowed is a correction
/// (LP-3.16): a raise, to exactly the lowest format the content needs, with
/// no entity changed in the same step. Anything else is `L007`.
fn format_changes(root: &Path, path: &str, versions: &[String], now: Option<&str>) -> Vec<Finding> {
    let mut steps: Vec<(String, String)> =
        versions.iter().filter_map(|c| content_at(root, c, path).map(|t| (short(c).to_string(), t))).collect();
    if let Some(now) = now {
        steps.push(("the working tree".to_string(), now.to_string()));
    }
    steps
        .windows(2)
        .filter_map(|pair| {
            let [(from_at, from), (to_at, to)] = pair else { return None };
            let (was, is) = (declared(from)?, declared(to)?);
            if was == is || is_correction(from, to, was, is) {
                return None;
            }
            Some(Finding::new(
                VerifyClass::L007,
                path,
                format!(
                    "its format declaration was {was} at {from_at} and is {is} at {to_at} — a landed declaration only rises, \
                     to the lowest format the content needs, with nothing else changed (LP-3.16)"
                ),
            ))
        })
        .collect()
}

/// A raise to exactly what the content needs, and nothing else changed.
fn is_correction(from: &str, to: &str, was: u64, is: u64) -> bool {
    let needed = serde_yaml::from_str::<crate::changeset::ChangeSet>(to).ok().map(|cs| u64::from(crate::format::needed_for(&cs)));
    is > was && needed == Some(is) && entities(from) == entities(to)
}

/// The `format:` a file's text declares.
fn declared(text: &str) -> Option<u64> {
    serde_yaml::from_str::<Value>(text).ok()?.get("format")?.as_u64()
}

/// A role file or a sidecar: the file is the entity.
fn whole_file(path: &str, commit: &str, landed: &str, now: Option<&str>) -> Option<Finding> {
    let same = match now {
        None => false,
        Some(now) if crate::layout::is_role(path) => without_format(now) == without_format(landed),
        Some(now) => now == landed,
    };
    if same {
        return None;
    }
    let what = if now.is_some() { "changed" } else { "removed" };
    let rule = if crate::layout::is_role(path) {
        "roles are write-once; a new role and new grants supersede"
    } else {
        "a landed signature is never edited or removed"
    };
    Some(Finding::new(VerifyClass::L007, path, format!("{what} since it landed in {} — {rule}", short(commit))))
}

fn without_format(text: &str) -> Option<Value> {
    let mut v: Value = serde_yaml::from_str(text).ok()?;
    if let Value::Mapping(m) = &mut v {
        m.remove("format");
    }
    Some(v)
}

fn entity_name(list: &str) -> String {
    match list {
        crate::landed::HEADER => "the change-set header".to_string(),
        "key_bindings" => "a key binding".to_string(),
        "policies" => "a policy".to_string(),
        "revocations" => "a revocation".to_string(),
        other => format!("an entry of `{other}`"),
    }
}

fn consequence(list: &str) -> &'static str {
    match list {
        "policies" => "; putting a namespace under policy is one-way",
        "revocations" => "; removing a revocation would revive what it ended",
        "key_bindings" => "; removing a key close would reopen the key",
        _ => "",
    }
}

fn short(commit: &str) -> &str {
    commit.get(..12).unwrap_or(commit)
}
