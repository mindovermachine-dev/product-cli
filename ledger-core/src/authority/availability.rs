//! Unavailability — a filed interval during which a grant's holder does not hold authority.
//!
//! Closed by its own `until`, or early by an [`Availability`] the holder
//! files (never by editing the interval: no node gains triples after it is
//! created). While an interval covers an instant, the grant is not active,
//! and the next fallback in order may act.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::id::{AvailabilityId, GrantId, UnavailabilityId};
use crate::identity::Identity;

/// Who declared the interval, relative to the grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Basis {
    /// The holder, of themselves.
    SelfDeclared,
    /// The identity that granted it.
    Grantor,
    /// The first fallback of the genesis role, against the genesis grant.
    FallbackOfGenesis,
}

impl Basis {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SelfDeclared => "self",
            Self::Grantor => "grantor",
            Self::FallbackOfGenesis => "fallback-of-genesis",
        }
    }
}

impl FromStr for Basis {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "self" => Ok(Self::SelfDeclared),
            "grantor" => Ok(Self::Grantor),
            "fallback-of-genesis" => Ok(Self::FallbackOfGenesis),
            other => Err(format!("`{other}` is not a basis — self | grantor | fallback-of-genesis")),
        }
    }
}

impl fmt::Display for Basis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for Basis {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Basis {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
    }
}

/// One unavailability interval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unavailability {
    pub id: UnavailabilityId,
    pub grant: GrantId,
    pub from: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<DateTime<Utc>>,
    pub basis: Basis,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub by: Identity,
    pub at: DateTime<Utc>,
}

/// The holder ending an unavailability early.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Availability {
    pub id: AvailabilityId,
    pub ends: UnavailabilityId,
    pub available_at: DateTime<Utc>,
    pub by: Identity,
    pub at: DateTime<Utc>,
}
