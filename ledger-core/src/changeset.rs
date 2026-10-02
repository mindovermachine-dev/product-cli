//! The change-set — the append-only unit one file holds (§5).
//!
//! Git commits snapshot a whole tree; a change-set groups the version
//! transitions of one *act*, which is what decisions actually have (one act
//! touches 4 of 200). Each lands as `.decisions/log/<ulid>.yml`, written
//! once and never edited: a correction is a new version, a reversal is a
//! revocation. That is the property a rebase cannot destroy, and the reason
//! the log — not the graph — is the source of truth.
//!
//! The `Decision` record here carries only what identity needs. Its
//! namespace is not restated: it is already inside the id, and a second
//! spelling of the same fact is a second thing that can disagree.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::acceptance::{Acceptance, Revocation};
use crate::authority::{
    Availability, Grant, GrantAcceptance, KeyBinding, Policy, Unavailability,
};
use crate::id::{ChangeSetId, DecisionId};
use crate::identity::Identity;
use crate::version::VersionRaw;

/// A decision's identity object, first introduced by some change-set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRecord {
    /// `dec:<namespace>/<ulid>` — stable forever. Supersession mints a new
    /// id with a `supersedes` edge; it never mutates or reuses this one.
    pub id: DecisionId,
    pub created_at: DateTime<Utc>,
    pub created_by: Identity,
}

/// One log file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeSet {
    pub format: u32,
    pub id: ChangeSetId,
    pub created_at: DateTime<Utc>,
    /// Who performed the act. Not an acceptor: a model may author a
    /// change-set, and §9.3 is about who *signs*, not who types.
    pub created_by: Identity,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parents: Vec<ChangeSetId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decisions: Vec<DecisionRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub versions: Vec<VersionRaw>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acceptances: Vec<Acceptance>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revocations: Vec<Revocation>,
    /// The authority records (format 6, `docs/ledger-authority/`): grants,
    /// their acceptances, unavailability and its end, key bindings, and
    /// namespace policy versions. Revocations of grants share
    /// `revocations` with revocations of acceptances — one entity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grants: Vec<Grant>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grant_acceptances: Vec<GrantAcceptance>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unavailabilities: Vec<Unavailability>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub availabilities: Vec<Availability>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub key_bindings: Vec<KeyBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policies: Vec<Policy>,
}

impl ChangeSet {
    /// An empty change-set for one act — every list empty.
    pub fn empty(
        format: u32,
        id: ChangeSetId,
        created_at: DateTime<Utc>,
        created_by: Identity,
        note: Option<String>,
    ) -> Self {
        Self {
            format,
            id,
            created_at,
            created_by,
            parents: Vec::new(),
            note,
            decisions: Vec::new(),
            versions: Vec::new(),
            acceptances: Vec::new(),
            revocations: Vec::new(),
            grants: Vec::new(),
            grant_acceptances: Vec::new(),
            unavailabilities: Vec::new(),
            availabilities: Vec::new(),
            key_bindings: Vec::new(),
            policies: Vec::new(),
        }
    }

    /// How many entries the act filed, of every kind.
    pub fn entry_count(&self) -> usize {
        self.decisions.len()
            + self.versions.len()
            + self.acceptances.len()
            + self.revocations.len()
            + self.authority_count()
    }

    /// How many authority records (format 6) the act filed.
    pub fn authority_count(&self) -> usize {
        self.grants.len()
            + self.grant_acceptances.len()
            + self.unavailabilities.len()
            + self.availabilities.len()
            + self.key_bindings.len()
            + self.policies.len()
    }

    /// The filename this change-set belongs in, relative to `log/`.
    pub fn file_name(&self) -> String {
        format!("{}.yml", self.id.ulid())
    }

    /// Whether the act recorded anything at all. An empty change-set is not
    /// a fault, but it is nothing, and saying so is cheaper than wondering.
    pub fn is_empty(&self) -> bool {
        self.entry_count() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ULID: &str = "01K2C4YQJ3F8M0PT5W7NZ9RDXV";

    fn minimal() -> String {
        format!(
            "format: 1\nid: cs:{ULID}\ncreated_at: 2026-08-10T09:14:22Z\ncreated_by: emk@delegate.dk\n"
        )
    }

    #[test]
    fn a_minimal_change_set_parses_with_every_list_defaulting_empty() {
        let cs: ChangeSet = serde_yaml::from_str(&minimal()).expect("parse");
        assert!(cs.is_empty());
        assert!(cs.parents.is_empty() && cs.versions.is_empty());
        assert_eq!(cs.file_name(), format!("{ULID}.yml"));
    }

    #[test]
    fn an_unknown_key_is_rejected_rather_than_silently_dropped() {
        let text = format!("{}slices: []\n", minimal());
        let err = serde_yaml::from_str::<ChangeSet>(&text).expect_err("unknown field");
        assert!(err.to_string().contains("unknown field"), "{err}");
    }

    #[test]
    fn a_decision_record_carries_no_second_spelling_of_its_namespace() {
        let text = format!(
            "{}decisions:\n  - id: dec:hafeok.ledger/{ULID}\n    namespace: hafeok.ledger\n    created_at: 2026-08-10T09:14:22Z\n    created_by: emk@delegate.dk\n",
            minimal()
        );
        assert!(serde_yaml::from_str::<ChangeSet>(&text).is_err(), "namespace is in the id");
    }

    #[test]
    fn a_change_set_records_its_parents() {
        let text = format!("{}parents: [cs:{ULID}]\n", minimal());
        let cs: ChangeSet = serde_yaml::from_str(&text).expect("parse");
        assert_eq!(cs.parents.len(), 1);
    }
}
