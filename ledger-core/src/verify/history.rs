//! What the record held at landing and must still hold (D6's landing).
//!
//! - **Roles are write-once** (Session A close-out §3 question 8, ruled
//!   2026-10-02). A grant names its role by id only, so a role file edited
//!   after it landed would change what every grant of it gives without
//!   touching a hash. `verify` fails a landed role file whose content
//!   differs from its content at its landing commit.
//! - **Opting in is one-way** (D5 (c)). A namespace that had a policy at an
//!   earlier first-parent commit and has none at the verified commit fails:
//!   removing a policy would turn every later act back into an unchecked
//!   pre-policy one.
//!
//! Both are reported under `L007` by the `L010` amendment mechanism: the
//! stored content no longer matches content the record already fixed.

use crate::changeset::ChangeSet;
use crate::finding::{Finding, VerifyClass};
use crate::landing::{content_at, Landing};
use crate::store::Store;

/// Every history finding, when git answered.
pub fn findings(store: &Store, landing: &Landing) -> Vec<Finding> {
    if !landing.available {
        return Vec::new();
    }
    let touched = crate::landing::touched_after_landing(&store.root);
    let mut out = roles_changed(store, landing, &touched);
    out.extend(policies_removed(store, landing, &touched));
    out
}

type Touched = std::collections::BTreeSet<String>;

fn roles_changed(store: &Store, landing: &Landing, touched: &Touched) -> Vec<Finding> {
    landing
        .landed_under(".decisions/roles/")
        .filter(|(path, _)| touched.contains(*path))
        .filter_map(|(path, commit)| {
            let landed = content_at(&store.root, commit, path)?;
            let now = std::fs::read_to_string(store.root.join(path)).ok();
            (now.as_deref() != Some(landed.as_str())).then(|| {
                Finding::new(
                    VerifyClass::L007,
                    path,
                    format!(
                        "role file {} since it landed in {} — roles are write-once; a new role and new grants supersede",
                        if now.is_some() { "changed" } else { "removed" },
                        short(commit)
                    ),
                )
            })
        })
        .collect()
}

fn policies_removed(store: &Store, landing: &Landing, touched: &Touched) -> Vec<Finding> {
    let auth = crate::authority::Authority::build(store);
    let mut out = Vec::new();
    for (path, commit) in landing.landed_under(".decisions/log/").filter(|(p, _)| touched.contains(*p)) {
        let now = std::fs::read_to_string(store.root.join(path)).ok();
        let Some(landed) = content_at(&store.root, commit, path) else { continue };
        if now.as_deref() == Some(landed.as_str()) {
            continue;
        }
        let Ok(cs) = serde_yaml::from_str::<ChangeSet>(&landed) else { continue };
        for ns in cs.policies.iter().map(|p| p.namespace.as_str()).filter(|ns| auth.policy(ns).is_none()) {
            out.push(Finding::new(
                VerifyClass::L007,
                path,
                format!("`{ns}` had a policy when {} landed and has none now — putting a namespace under policy is one-way", short(commit)),
            ));
        }
    }
    out
}

fn short(commit: &str) -> &str {
    commit.get(..12).unwrap_or(commit)
}
