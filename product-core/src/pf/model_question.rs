//! The authored open-question node: a question the graph cannot yet decide.
//!
//! Distinct from the *derived* facilitation prompts in [`super::questions`]
//! (recomputed from validation gaps on every call): an open question is raised
//! by a person, recorded next to the nodes it concerns, carried across the
//! What → How → Build phases, and answered — possibly by a How decision.

use serde::{Deserialize, Deserializer, Serialize};

/// The closed status vocabulary of an open question.
pub const QUESTION_STATUSES: [&str; 4] = ["open", "answered", "deferred", "wont-fix"];

/// The gates a question may block (empty = informational).
pub const QUESTION_GATES: [&str; 4] = ["what", "how", "build", "finalize"];

/// An authored open question (`kind=open-question`, ids conventionally `q-*`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
pub struct OpenQuestion {
    pub id: String,
    /// The question, phrased for the room.
    #[serde(default)]
    pub statement: String,
    /// The node ids the question is about (≥ 1; must resolve while open).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub concerns: Vec<String>,
    /// The bounded context the question belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Who raised it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raised_by: Option<String>,
    /// When it was raised (RFC-3339).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raised_at: Option<String>,
    /// `open` · `answered` · `deferred` · `wont-fix`.
    #[serde(default = "open_status")]
    pub status: String,
    /// The gate it blocks: `what` · `how` · `build` · `finalize`; empty (or
    /// `false`) means informational.
    #[serde(default, deserialize_with = "gate", skip_serializing_if = "String::is_empty")]
    #[schemars(with = "String")]
    pub blocking: String,
    /// The answer, required once the status leaves `open`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    /// The ids of what answered it — What nodes or How decisions/principles/patterns.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resolved_by: Vec<String>,
    /// When it was answered (RFC-3339).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answered_at: Option<String>,
    /// The workflow session that raised it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raised_in_session: Option<String>,
    /// The workflow session that answered it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answered_in_session: Option<String>,
}

impl Default for OpenQuestion {
    fn default() -> Self {
        Self {
            id: String::new(),
            statement: String::new(),
            concerns: Vec::new(),
            context: None,
            raised_by: None,
            raised_at: None,
            status: open_status(),
            blocking: String::new(),
            resolution: None,
            resolved_by: Vec::new(),
            answered_at: None,
            raised_in_session: None,
            answered_in_session: None,
        }
    }
}

impl OpenQuestion {
    /// True while the question is still undecided.
    pub fn is_open(&self) -> bool {
        self.status == "open"
    }

    /// True if the question blocks the named gate.
    pub fn blocks(&self, gate: &str) -> bool {
        self.blocking == gate
    }
}

fn open_status() -> String {
    "open".to_string()
}

/// Accept `blocking` as a gate name or a boolean: `false` is informational,
/// `true` blocks the last gate (`finalize`) — a question marked blocking must
/// block *something*.
fn gate<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
        Flag(bool),
        Name(String),
    }
    Ok(match Option::<Raw>::deserialize(d)? {
        None | Some(Raw::Flag(false)) => String::new(),
        Some(Raw::Flag(true)) => "finalize".to_string(),
        Some(Raw::Name(s)) if s.trim() == "false" => String::new(),
        Some(Raw::Name(s)) => s.trim().to_lowercase(),
    })
}
