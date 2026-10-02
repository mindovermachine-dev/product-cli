//! Rules over decision keys — the names generated types are built from.
//!
//! Two classes, both added at spec v1.6 by the `L010` amendment mechanism
//! and both file-gate classes rather than graph-only (PRD §0 item 2): an
//! outside implementation importing this format must reproduce them,
//! because a generated type name depends on each.
//!
//! - `L013` — a key, once given, is immutable along a decision's version
//!   chain: a version must carry the key its `parent` (and its
//!   `merged_from`) carries. Giving a keyless decision a key is allowed; it
//!   is a new version, so it needs re-acceptance like any other.
//! - `L014` — among the *live* decisions of one namespace (those no other
//!   decision's latest version supersedes), the latest versions' keys are
//!   unique. A superseded decision's key is free for its successor.

use std::collections::BTreeMap;

use crate::finding::{Finding, VerifyClass};
use crate::version::VersionRaw;

use super::state::supersession;
use super::view::View;

/// `L013` — a version whose key differs from a predecessor's key.
pub fn key_changed(view: &View) -> Vec<Finding> {
    let by_hash: BTreeMap<(String, String), &VersionRaw> = view
        .versions
        .iter()
        .map(|v| ((v.raw.decision.to_string(), v.raw.hash.to_string()), v.raw))
        .collect();
    let mut out = Vec::new();
    for v in &view.versions {
        let raw = v.raw;
        let decision = raw.decision.to_string();
        for (edge, pred) in [("parent", &raw.parent), ("merged_from", &raw.merged_from)] {
            let Some(pred) = pred else { continue };
            let Some(prior) = by_hash.get(&(decision.clone(), pred.to_string())) else { continue };
            let Some(was) = &prior.key else { continue };
            if raw.key.as_ref() != Some(was) {
                let now = raw.key.as_ref().map_or("no key".to_string(), |k| format!("`{k}`"));
                out.push(Finding::new(
                    VerifyClass::L013,
                    &decision,
                    format!(
                        "version {} carries {now}, but its {edge} {} carries `{was}` — a key is immutable once given (spec v1.6)",
                        raw.hash.short(),
                        pred.short()
                    ),
                ));
            }
        }
    }
    out
}

/// `L014` — two live decisions of one namespace naming the same key.
pub fn key_collision(view: &View) -> Vec<Finding> {
    let superseded = supersession(view);
    let mut holders: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for v in view.latest_versions() {
        let Some(key) = &v.raw.key else { continue };
        let decision = v.raw.decision.to_string();
        if superseded.contains_key(&decision) {
            continue;
        }
        holders
            .entry((v.raw.decision.namespace().to_string(), key.to_string()))
            .or_default()
            .push(decision);
    }
    holders
        .into_iter()
        .filter(|(_, decisions)| decisions.len() > 1)
        .flat_map(|((namespace, key), decisions)| {
            let all = decisions.join(", ");
            decisions.into_iter().map(move |d| {
                Finding::new(
                    VerifyClass::L014,
                    &d,
                    format!(
                        "key `{key}` names {all} in namespace `{namespace}` — one live decision per key, or a generated type name names two (spec v1.6)"
                    ),
                )
            })
        })
        .collect()
}

#[path = "keys_tests.rs"]
#[cfg(test)]
mod tests;
