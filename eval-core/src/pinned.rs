//! A set of declared facts, with a digest that fixes them.
//!
//! One construction, three uses: the coordinates a run happened at, the
//! arrangement that answered, and the context a judge was shown. Each is a
//! claim about what was declared, kept beside the digest so a reader can check
//! it rather than take it.
//!
//! §6 of the format doc states the canonical form normatively. Nothing here
//! decides whether the declaration was *adequate* — that is a normative ruling
//! with a named owner, and no digest substitutes for it.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::digest;

/// Declared facts, and the digest that pins them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pinned {
    pub digest: String,
    /// What the digest covers, kept so a reader can check it.
    pub shown: BTreeMap<String, String>,
}

impl Pinned {
    /// Pin a set of declared facts.
    pub fn new(shown: BTreeMap<String, String>) -> Self {
        Self { digest: digest::pin(&shown), shown }
    }

    /// Pin from pairs, for a caller that has them to hand.
    pub fn of<K: Into<String>, V: Into<String>>(pairs: impl IntoIterator<Item = (K, V)>) -> Self {
        Self::new(pairs.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }

    /// Whether the digest still matches what this says it covered.
    ///
    /// A record that carries its own check is one a reader can disbelieve
    /// without reconstructing the act.
    pub fn holds(&self) -> bool {
        digest::holds(&self.shown, &self.digest)
    }

    /// The digest's first bytes, enough to tell two apart on sight.
    pub fn short(&self) -> String {
        let body = self.digest.strip_prefix("sha256:").unwrap_or(&self.digest);
        body.chars().take(12).collect()
    }
}

#[path = "pinned_tests.rs"]
#[cfg(test)]
mod tests;
