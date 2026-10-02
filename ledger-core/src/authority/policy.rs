//! Namespace policy — what a namespace promises those who cite it (#65, D1).
//!
//! When a signature is required is a property of the namespace, not of a
//! decision's tier: the namespace is inside every decision id, so the
//! requirement cannot be shopped out per decision. A policy is a log entry;
//! changing it files a new one that names the hash of the policy it
//! `replaces`, so the policy in force is the tip of that chain. The first
//! policy of a namespace is what `init --namespace` files.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::hash::VersionHash;
use crate::id::PolicyId;
use crate::identity::Identity;

/// A signature scheme a policy may require (#65, D4). Closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scheme {
    Ssh,
    Dsse,
    /// Pre-v2 stores only, and only where the policy lists it.
    None,
}

impl Scheme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ssh => "ssh",
            Self::Dsse => "dsse",
            Self::None => "none",
        }
    }
}

impl FromStr for Scheme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "ssh" => Ok(Self::Ssh),
            "dsse" => Ok(Self::Dsse),
            "none" => Ok(Self::None),
            other => Err(format!("`{other}` is not a signature scheme — ssh | dsse | none")),
        }
    }
}

impl fmt::Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for Scheme {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Scheme {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
    }
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// One version of a namespace's policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub id: PolicyId,
    pub namespace: String,
    /// The schemes an acceptance in this namespace must be signed under.
    pub schemes: Vec<Scheme>,
    /// Whether keys must be hardware-backed (`-sk`).
    #[serde(default, skip_serializing_if = "is_false")]
    pub require_sk: bool,
    /// The role whose grants carry `accept-decision` here.
    pub accept_role: String,
    /// Days after a key's window closes within which its acceptances must
    /// be re-accepted or affirmed before they stop being citable. Absent
    /// means no deadline (ruling 12).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reaccept_within_days: Option<u32>,
    /// The hash of the policy in force before this one; absent on the first.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaces: Option<VersionHash>,
    pub by: Identity,
    pub at: DateTime<Utc>,
    /// `ledger.namespace-policy.v1` over the closed payload; `L007`.
    pub hash: VersionHash,
}
