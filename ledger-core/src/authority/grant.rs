//! Grants — a role given to one identity over one scope, at one order.
//!
//! A grant is permanent once filed: a change is a new grant that
//! `supersedes` it, and an end is a [`super::revocation::Revocation`]. It
//! is *live* only once its holder files a [`GrantAcceptance`] signing the
//! grant's hash, and *active* at an instant only while no unavailability
//! covers it. The genesis grant is the store's trust root: self-granted,
//! scope `*`, order `primary`, carrying the external reference to the
//! mandate that exists outside the tool.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::hash::VersionHash;
use crate::id::{GrantAcceptanceId, GrantId};
use crate::identity::Identity;

/// What a grant covers: `*`, `ns:<namespace>`, `set:<set-id>`, or
/// `pattern:<id>` (a literal, because sets and patterns may live elsewhere).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GrantScope {
    All,
    Namespace(String),
    Set(String),
    Pattern(String),
}

impl FromStr for GrantScope {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = s.trim();
        if t == "*" {
            return Ok(Self::All);
        }
        let (kind, payload) = t
            .split_once(':')
            .ok_or_else(|| format!("`{t}` is not a scope — expected `*`, `ns:…`, `set:…` or `pattern:…`"))?;
        if payload.is_empty() || payload.chars().any(char::is_whitespace) {
            return Err(format!("`{t}` has an empty or spaced payload"));
        }
        match kind {
            "ns" => Ok(Self::Namespace(payload.to_string())),
            "set" if payload.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-') => {
                Ok(Self::Set(payload.to_string()))
            }
            "set" => Err(format!("`{t}`: a set id is lowercase alphanumerics and dashes")),
            "pattern" => Ok(Self::Pattern(payload.to_string())),
            other => Err(format!("`{other}:` is not a scope kind — expected ns, set or pattern")),
        }
    }
}

impl fmt::Display for GrantScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::All => f.write_str("*"),
            Self::Namespace(n) => write!(f, "ns:{n}"),
            Self::Set(s) => write!(f, "set:{s}"),
            Self::Pattern(p) => write!(f, "pattern:{p}"),
        }
    }
}

/// `primary`, or `fallback-N` for N ≥ 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Order(u32);

impl Order {
    pub const PRIMARY: Order = Order(0);

    /// The derived rank: primary 0, fallback-N N. Projection only.
    pub fn rank(self) -> u32 {
        self.0
    }

    pub fn is_primary(self) -> bool {
        self.0 == 0
    }

    /// `fallback-n`; `fallback(0)` is the primary order.
    pub fn fallback(n: u32) -> Self {
        Self(n)
    }
}

impl FromStr for Order {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = s.trim();
        if t == "primary" {
            return Ok(Self::PRIMARY);
        }
        t.strip_prefix("fallback-")
            .filter(|n| !n.starts_with('0'))
            .and_then(|n| n.parse::<u32>().ok())
            .filter(|n| *n >= 1)
            .map(Self)
            .ok_or_else(|| format!("`{t}` is not an order — expected `primary` or `fallback-N`, N ≥ 1"))
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            f.write_str("primary")
        } else {
            write!(f, "fallback-{}", self.0)
        }
    }
}

/// What a fallback grant may not do (`ledger:LimitScheme`, closed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Limit {
    NoGrants,
    NoGrantRevocations,
    NoGenesis,
    NoRoleEdits,
}

impl Limit {
    pub const ALL: &'static [Limit] =
        &[Self::NoGrants, Self::NoGrantRevocations, Self::NoGenesis, Self::NoRoleEdits];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoGrants => "no-grants",
            Self::NoGrantRevocations => "no-grant-revocations",
            Self::NoGenesis => "no-genesis",
            Self::NoRoleEdits => "no-role-edits",
        }
    }
}

impl FromStr for Limit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|l| l.as_str() == s.trim())
            .ok_or_else(|| format!("`{s}` is not a limit — no-grants | no-grant-revocations | no-genesis | no-role-edits"))
    }
}

impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

macro_rules! string_serde {
    ($($ty:ty),*) => {$(
        impl Serialize for $ty {
            fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_str(&self.to_string())
            }
        }
        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
            }
        }
    )*};
}
string_serde!(GrantScope, Order, Limit);

fn is_false(b: &bool) -> bool {
    !*b
}

/// One grant, as it appears in a log file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grant {
    pub id: GrantId,
    pub role: String,
    pub scope: GrantScope,
    pub holder: Identity,
    pub granted_by: Identity,
    pub order: Order,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limits: Vec<Limit>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub genesis: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<GrantId>,
    /// The grant the grantor acts under (format 7, D9 (a)); absent on the
    /// genesis grant. Hashed when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub under: Option<GrantId>,
    pub at: DateTime<Utc>,
    /// `ledger.authority-grant.v1` over the closed payload; checked by `L007`.
    pub hash: VersionHash,
}

/// The holder's acceptance of a grant: it signs the grant's hash.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrantAcceptance {
    pub id: GrantAcceptanceId,
    pub grant: GrantId,
    pub signs: VersionHash,
    pub actor: Identity,
    pub at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scopes_round_trip_and_unknown_kinds_are_refused() {
        for s in ["*", "ns:hafeok.ledger", "set:ledger-design", "pattern:slice/x"] {
            assert_eq!(s.parse::<GrantScope>().expect(s).to_string(), s);
        }
        for bad in ["team:foo", "ns:", "set:Upper", "everything"] {
            assert!(bad.parse::<GrantScope>().is_err(), "{bad}");
        }
    }

    #[test]
    fn orders_are_primary_or_a_positive_fallback() {
        assert_eq!("primary".parse::<Order>().expect("primary").rank(), 0);
        assert_eq!("fallback-2".parse::<Order>().expect("fb").rank(), 2);
        for bad in ["fallback-0", "fallback-01", "secondary", "fallback-"] {
            assert!(bad.parse::<Order>().is_err(), "{bad}");
        }
    }

    #[test]
    fn the_limit_vocabulary_is_closed_at_four() {
        assert_eq!(Limit::ALL.len(), 4);
        assert!("no-edits".parse::<Limit>().is_err());
    }
}
