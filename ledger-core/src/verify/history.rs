//! What the record held at landing, it must still hold (D6's landing).
//!
//! **Landed entities are immutable.** Every entity a landed log file has
//! held on the verified commit's first-parent line must be present, and
//! identical to what landed, at the verified commit (the working tree
//! included). An entity is an item of an entity list, or the change-set's
//! header ([`crate::landed`]); re-declaring `format:` alone changes none.
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
        if path.starts_with(".decisions/log/") {
            out.extend(log_file(&store.root, &path, &versions, now.as_deref()));
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

/// A role file or a sidecar: the file is the entity.
fn whole_file(path: &str, commit: &str, landed: &str, now: Option<&str>) -> Option<Finding> {
    let same = match now {
        None => false,
        Some(now) if path.starts_with(".decisions/roles/") => without_format(now) == without_format(landed),
        Some(now) => now == landed,
    };
    if same {
        return None;
    }
    let what = if now.is_some() { "changed" } else { "removed" };
    let rule = if path.starts_with(".decisions/roles/") {
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
