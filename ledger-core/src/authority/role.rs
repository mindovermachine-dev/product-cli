//! Roles — what a holder of a grant may do (`.decisions/roles/<id>.yml`).
//!
//! A role is declared scope, like a set: a file of its own, written once,
//! naming the capabilities it carries from a **closed** vocabulary
//! (`ledger:CapabilityScheme` in `docs/ledger-authority/ledger-authority.ttl`).
//! A role grants nothing by itself — a [`super::grant::Grant`] gives one to
//! an identity over a scope. Every act the authority model polices is an
//! [`Act`], which names the capability it needs and the fallback limit that
//! withholds it.

use std::fmt;
use std::str::FromStr;

use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::identity::Identity;

/// The closed capability vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capability {
    AcceptDecision,
    SignOffPattern,
    WaiveInvalidation,
    GrantRole,
    RevokeGrant,
    DeclareUnavailability,
    RotateGenesis,
}

impl Capability {
    pub const ALL: &'static [Capability] = &[
        Self::AcceptDecision,
        Self::SignOffPattern,
        Self::WaiveInvalidation,
        Self::GrantRole,
        Self::RevokeGrant,
        Self::DeclareUnavailability,
        Self::RotateGenesis,
    ];

    /// The genesis (root) role's capabilities: acts on the authority
    /// structure, none of the three decision capabilities (D9 (f), ruled
    /// 2026-10-02). Accepting is a separate role, granted separately.
    pub const ROOT: &'static [Capability] =
        &[Self::GrantRole, Self::RevokeGrant, Self::DeclareUnavailability, Self::RotateGenesis];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::AcceptDecision => "accept-decision",
            Self::SignOffPattern => "sign-off-pattern",
            Self::WaiveInvalidation => "waive-invalidation",
            Self::GrantRole => "grant-role",
            Self::RevokeGrant => "revoke-grant",
            Self::DeclareUnavailability => "declare-unavailability",
            Self::RotateGenesis => "rotate-genesis",
        }
    }
}

impl FromStr for Capability {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL.iter().copied().find(|c| c.as_str() == s.trim()).ok_or_else(|| {
            let known: Vec<&str> = Self::ALL.iter().map(|c| c.as_str()).collect();
            format!("`{s}` is not a capability — the vocabulary is closed: {}", known.join(", "))
        })
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for Capability {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Capability {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
    }
}

/// One role file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Role {
    pub format: u32,
    /// Lowercase alphanumerics, dashes and dots — the set-id rule.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub owner: Identity,
    pub may: Vec<Capability>,
    pub created_at: NaiveDate,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl Role {
    /// Whether this role carries a capability.
    pub fn may(&self, capability: Capability) -> bool {
        self.may.contains(&capability)
    }

    /// The filename this role belongs in, relative to `roles/`.
    pub fn file_name(&self) -> String {
        format!("{}.yml", self.id)
    }
}

/// An act the authority model polices: which capability it needs, and
/// which fallback limit (if any) withholds it from a fallback grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    /// Accept a decision version.
    Accept,
    /// Revoke an acceptance — an act of the same authority as accepting.
    RevokeAcceptance,
    /// File a grant of a role.
    Grant,
    /// Revoke a grant.
    RevokeGrant,
    /// Declare another holder's unavailability (a holder's own needs none).
    DeclareUnavailability,
    /// Declare a new role file.
    DeclareRole,
}

impl Act {
    pub fn capability(self) -> Capability {
        match self {
            Self::Accept | Self::RevokeAcceptance => Capability::AcceptDecision,
            Self::Grant | Self::DeclareRole => Capability::GrantRole,
            Self::RevokeGrant => Capability::RevokeGrant,
            Self::DeclareUnavailability => Capability::DeclareUnavailability,
        }
    }

    /// The limit that withholds this act from a fallback grant.
    pub fn withheld_by(self) -> Option<super::grant::Limit> {
        use super::grant::Limit;
        match self {
            Self::Grant => Some(Limit::NoGrants),
            Self::RevokeGrant => Some(Limit::NoGrantRevocations),
            Self::DeclareRole => Some(Limit::NoRoleEdits),
            Self::Accept | Self::RevokeAcceptance | Self::DeclareUnavailability => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::RevokeAcceptance => "revoke an acceptance",
            Self::Grant => "grant a role",
            Self::RevokeGrant => "revoke a grant",
            Self::DeclareUnavailability => "declare unavailability",
            Self::DeclareRole => "declare a role",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_capability_vocabulary_is_closed_at_seven() {
        assert_eq!(Capability::ALL.len(), 7);
        for c in Capability::ALL {
            assert_eq!(c.as_str().parse::<Capability>(), Ok(*c));
        }
        let err = "edit-anything".parse::<Capability>().expect_err("closed");
        assert!(err.contains("closed"), "{err}");
    }

    #[test]
    fn a_role_file_parses_and_refuses_unknown_keys() {
        let text = "format: 6\nid: steward\nowner: o@x\nmay: [accept-decision, grant-role]\ncreated_at: 2026-10-02\n";
        let role: Role = serde_yaml::from_str(text).expect("parse");
        assert!(role.may(Capability::AcceptDecision) && !role.may(Capability::RevokeGrant));
        assert!(serde_yaml::from_str::<Role>(&format!("{text}members: []\n")).is_err());
    }
}
