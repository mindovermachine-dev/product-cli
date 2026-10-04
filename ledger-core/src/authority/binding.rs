//! Key bindings — `identity add | rotate | revoke`, the trust root's log entries.
//!
//! The trust root is a governed projection (#65): `allowed_signers` is
//! derived from these entries and never edited (see [`super::signers`]).
//! Each binding is append-only. `add` opens a key's window at its `at`;
//! `rotate` closes one binding's window and opens a new key's in the same
//! act; `revoke` only closes. A window is never reopened.
//!
//! **Bootstrap.** The first binding in a namespace is the genesis holder's,
//! marked `self_bound` and carrying the genesis grant's `external_ref` as
//! its `mandate` — the out-of-band authority for a key nothing inside the
//! ledger could yet vouch for.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::hash::VersionHash;
use crate::id::KeyBindingId;
use crate::identity::Identity;

/// Which identity act a binding records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingAct {
    Add,
    Rotate,
    Revoke,
}

impl BindingAct {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Rotate => "rotate",
            Self::Revoke => "revoke",
        }
    }

    /// Whether the act opens a key's window (and so carries a key).
    pub fn opens(self) -> bool {
        matches!(self, Self::Add | Self::Rotate)
    }
}

impl FromStr for BindingAct {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "add" => Ok(Self::Add),
            "rotate" => Ok(Self::Rotate),
            "revoke" => Ok(Self::Revoke),
            other => Err(format!("`{other}` is not an identity act — add | rotate | revoke")),
        }
    }
}

impl fmt::Display for BindingAct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for BindingAct {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for BindingAct {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
    }
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// One key-binding entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyBinding {
    pub id: KeyBindingId,
    pub act: BindingAct,
    pub principal: Identity,
    pub namespace: String,
    /// The OpenSSH key type (`ssh-ed25519`, `sk-ssh-ed25519@openssh.com`, …)
    /// — `add`/`rotate` only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_type: Option<String>,
    /// The base64 public key — `add`/`rotate` only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The binding whose window this act closes — `rotate`/`revoke` only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closes: Option<KeyBindingId>,
    /// The genesis bootstrap marker; `add` only.
    #[serde(default, skip_serializing_if = "is_false")]
    pub self_bound: bool,
    /// The genesis grant's `external_ref` — only with `self_bound`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mandate: Option<String>,
    pub by: Identity,
    /// The grant `by` acts under when it is not the principal (format 7,
    /// D9 (a)): the genesis holder vouching for a first key (D7).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub under: Option<crate::id::GrantId>,
    pub at: DateTime<Utc>,
    /// `ledger.identity-binding.v1` over the closed payload; `L007`.
    pub hash: VersionHash,
}

impl KeyBinding {
    /// Whether this is a hardware-backed (`sk-`) key type.
    pub fn is_security_key(&self) -> bool {
        self.key_type.as_deref().is_some_and(|t| t.starts_with("sk-"))
    }
}
