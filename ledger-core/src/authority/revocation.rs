//! Revocation — one entity that revokes a grant or an acceptance (ruling 3).
//!
//! The revoked record is never edited: a revocation names it with
//! `revokes`, carries its own reason, attribution and time, and is hashed
//! over the closed payload `{revokes, actor, at, reason}` under
//! `ledger.revocation.v1`, so a signature over that payload (Session B)
//! covers fixed content and the revoked record's own signature stays valid.
//!
//! Two wire shapes share the `revocations:` list. The **format 6** shape is
//! `{id, revokes, actor, at, reason, hash}`. The **legacy** shape of
//! formats 1–5 is `{acceptance, at, by, reason}` — no id, acceptances only.
//! A file carries the shape its declared format defines, and the loader
//! refuses the other (`SCHEMA`). Readers go through [`Revocation::target`]
//! and [`Revocation::actor`], which answer for both.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::hash::VersionHash;
use crate::id::{AcceptanceId, GrantId, RevocationId};
use crate::identity::Identity;

/// What a revocation revokes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Revocable {
    Grant(GrantId),
    Acceptance(AcceptanceId),
}

impl FromStr for Revocable {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().starts_with("grant:") {
            s.trim().parse().map(Self::Grant)
        } else if s.trim().starts_with("acc:") {
            s.trim().parse().map(Self::Acceptance)
        } else {
            Err(format!("`{s}` is not revocable — a revocation names a `grant:` or an `acc:`"))
        }
    }
}

impl fmt::Display for Revocable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Grant(g) => write!(f, "{g}"),
            Self::Acceptance(a) => write!(f, "{a}"),
        }
    }
}

impl Serialize for Revocable {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Revocable {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
    }
}

/// One revocation, in either wire shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Revocation {
    /// `rev:<ulid>` — format 6.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<RevocationId>,
    /// The grant or acceptance revoked — format 6.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revokes: Option<Revocable>,
    /// The acceptance revoked — legacy shape (formats 1–5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance: Option<AcceptanceId>,
    pub at: DateTime<Utc>,
    /// Who revoked — format 6.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<Identity>,
    /// Who revoked — legacy shape.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<Identity>,
    pub reason: String,
    /// `ledger.revocation.v1` over the closed payload — format 6.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<VersionHash>,
}

impl Revocation {
    /// What is revoked, whichever shape the file used.
    pub fn target(&self) -> Option<Revocable> {
        self.revokes.clone().or_else(|| self.acceptance.clone().map(Revocable::Acceptance))
    }

    /// Who revoked, whichever shape the file used.
    pub fn actor(&self) -> Option<&Identity> {
        self.actor.as_ref().or(self.by.as_ref())
    }

    /// Whether this is the format-6 entity (it carries an id).
    pub fn is_entity(&self) -> bool {
        self.id.is_some()
    }

    /// The acceptance this revokes, if it revokes one.
    pub fn revoked_acceptance(&self) -> Option<AcceptanceId> {
        match self.target()? {
            Revocable::Acceptance(a) => Some(a),
            Revocable::Grant(_) => None,
        }
    }

    /// The display subject: its id, else (legacy) the acceptance it names.
    pub fn subject(&self) -> String {
        self.id
            .as_ref()
            .map(ToString::to_string)
            .or_else(|| self.target().map(|t| t.to_string()))
            .unwrap_or_default()
    }

    /// Shape faults for a file declaring `format`: exactly one shape, and
    /// the one that format defines.
    pub fn shape_faults(&self, format: u32) -> Vec<String> {
        let entity = self.id.is_some() || self.revokes.is_some() || self.actor.is_some() || self.hash.is_some();
        let legacy = self.acceptance.is_some() || self.by.is_some();
        let mut out = Vec::new();
        if entity && legacy {
            out.push("mixes the format-6 shape (id, revokes, actor, hash) with the legacy one (acceptance, by)".to_string());
        } else if format >= crate::format::AUTHORITY_FORMAT {
            if !(self.id.is_some() && self.revokes.is_some() && self.actor.is_some() && self.hash.is_some()) {
                out.push("a format-6 revocation carries id, revokes, actor and hash".to_string());
            }
        } else if entity {
            out.push("carries the format-6 revocation shape — declare `format: 6`".to_string());
        } else if !(self.acceptance.is_some() && self.by.is_some()) {
            out.push("a legacy revocation carries acceptance and by".to_string());
        }
        if self.reason.trim().is_empty() {
            out.push("a revocation carries its reason".to_string());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEGACY: &str = "acceptance: acc:01K2C4YQJ3F8M0PT5W7NZ9RDXX\nat: 2026-08-11T09:00:00Z\nby: o@x\nreason: wrong version\n";
    const ENTITY: &str = "id: rev:01K2C4YQJ3F8M0PT5W7NZ9RDXY\nrevokes: grant:01K2C4YQJ3F8M0PT5W7NZ9RDXX\nat: 2026-10-02T09:00:00Z\nactor: o@x\nreason: moved\nhash: sha256:0000000000000000000000000000000000000000000000000000000000000000\n";

    #[test]
    fn both_shapes_answer_target_and_actor() {
        let legacy: Revocation = serde_yaml::from_str(LEGACY).expect("legacy");
        assert!(matches!(legacy.target(), Some(Revocable::Acceptance(_))));
        assert_eq!(legacy.actor().map(Identity::as_str), Some("o@x"));
        assert!(legacy.shape_faults(1).is_empty());
        let entity: Revocation = serde_yaml::from_str(ENTITY).expect("entity");
        assert!(matches!(entity.target(), Some(Revocable::Grant(_))));
        assert!(entity.shape_faults(6).is_empty());
    }

    #[test]
    fn each_format_takes_only_its_own_shape() {
        let legacy: Revocation = serde_yaml::from_str(LEGACY).expect("legacy");
        assert!(!legacy.shape_faults(6).is_empty(), "format 6 has no legacy shape");
        let entity: Revocation = serde_yaml::from_str(ENTITY).expect("entity");
        assert!(!entity.shape_faults(5).is_empty(), "format 5 has no entity shape");
    }
}
