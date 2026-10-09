//! The re-acceptance review of acceptances signed under a since-closed key (`L012`).
//!
//! An acceptance dated *and* landed before its key's close is a review item
//! ("needs re-acceptance") until a later valid acceptance of the same
//! version by the same actor affirms it, and `L012` once the policy's
//! re-acceptance deadline has passed. The close is the key's earliest in
//! its own namespace (ruled 2026-10-06; per namespace by ruling 47), as the
//! signature check found it.

use std::collections::BTreeMap;

use chrono::{Duration, NaiveDate};
use serde::Serialize;

use crate::authority::{Authority, KeyBinding};
use crate::finding::{Finding, VerifyClass};
use crate::store::Store;

use super::check::Outcome;
use super::subject::{Kind, Subject};

/// An acceptance signed by a key closed after it, awaiting affirmation.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Reaccept {
    pub acceptance: String,
    pub actor: String,
    pub key: String,
    pub closed_by: String,
    /// After this date the acceptance stops being citable (`L012`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<NaiveDate>,
}

/// `L012` and the review list, for acceptances under a since-closed key.
pub(super) fn review_closed(
    store: &Store,
    closed: &[(&Subject<'_>, &KeyBinding)],
    verdicts: &BTreeMap<String, bool>,
    today: NaiveDate,
    out: &mut Outcome<'_>,
) {
    let auth = Authority::build(store);
    let acceptances: Vec<&crate::acceptance::Acceptance> = store.log.iter().flat_map(|l| l.file.acceptances.iter()).collect();
    for (s, close) in closed.iter().filter(|(s, _)| s.kind == Kind::Acceptance) {
        let Some(a) = acceptances.iter().find(|a| a.id.to_string() == s.id) else { continue };
        let affirmed = acceptances.iter().any(|later| {
            later.actor == a.actor
                && later.decision == a.decision
                && later.version == a.version
                && later.at > a.at
                && verdicts.get(&later.id.to_string()).copied().unwrap_or(false)
        });
        if affirmed {
            continue;
        }
        let deadline = auth
            .policy(&s.namespace)
            .and_then(|p| p.reaccept_within_days)
            .map(|d| close.at.date_naive() + Duration::days(i64::from(d)));
        if deadline.is_some_and(|d| today > d) {
            out.findings.push(Finding::new(
                VerifyClass::L012,
                &s.id,
                format!(
                    "signed under a key {} closed on {}; not re-accepted by the policy's deadline {} — no longer citable",
                    close.closes.as_ref().map(ToString::to_string).unwrap_or_default(),
                    close.at.date_naive(),
                    deadline.map(|d| d.to_string()).unwrap_or_default()
                ),
            ));
        } else {
            out.reaccept.push(Reaccept {
                acceptance: s.id.clone(),
                actor: a.actor.to_string(),
                key: close.closes.as_ref().map(ToString::to_string).unwrap_or_default(),
                closed_by: close.id.to_string(),
                deadline,
            });
        }
    }
}
