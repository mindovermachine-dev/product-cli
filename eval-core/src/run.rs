//! What a model did, recorded at the time, before anything assessed it.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::declaration::{Attribution, Declaration};
use crate::pinned::Pinned;

/// The form a run record is written in.
pub const RUN_FORM: &str = "eval.run-record.v1";

/// One run of a model, as the tool that ran it observed it.
///
/// An observation, never authority. It is not signed, names no principal, and
/// no gate reads it. What it is *for* is the pair `proposed`/`kept`: the
/// distance between what a model put forward and what a person kept is a human
/// judgment on model output, collected on every run, supplied by no model and
/// costing nothing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRecord {
    pub form: String,
    /// The run's identity, supplied by the tool that ran it.
    pub id: String,
    /// Which tool ran it, so one store can hold several.
    pub tool: String,
    /// What was attempted. An address, not prose.
    pub task: String,
    /// What it was attempted on.
    pub subject: String,
    pub ran_at: DateTime<Utc>,
    /// The model asked, where one was.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Where it was asked. Host only — the key is never written.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_host: Option<String>,
    pub duration_ms: u64,
    /// What the model put forward.
    #[serde(default)]
    pub proposed: Vec<String>,
    /// What a person kept of it.
    ///
    /// A tool with no human in the loop writes this equal to `proposed` and
    /// says so by their being equal, rather than by leaving the field out.
    #[serde(default)]
    pub kept: Vec<String>,
    /// The model's reply, whole.
    ///
    /// A run that proposed nothing is the interesting case, and a length
    /// cannot tell a model that answered "nothing" from one whose answer could
    /// not be read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply: Option<String>,
    /// Deterministic observations. Arithmetic, reproducible, never a verdict.
    #[serde(default)]
    pub metrics: Vec<Metric>,
    /// **Where this run happened** — the coordinates behaviour is compared at.
    ///
    /// The task instance together with the ground the caller declared. Two runs
    /// at the same address were asked the same question; runs at different
    /// addresses are not comparable, and saying so is the point.
    ///
    /// Optional, because a caller that declares no coordinates can still record
    /// what it did. Its runs simply cannot be read against each other, and the
    /// absence of the field is how a reader learns that rather than guessing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<Pinned>,
    /// **What answered** — the worker version, and whatever else was arranged.
    ///
    /// Kept apart from the address so the two can vary independently. Same
    /// address and same arrangement isolates run-to-run variance; same address
    /// and a changed arrangement is drift in the arrangement rather than in the
    /// world.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arrangement: Option<Pinned>,
    /// **What the run said it was doing, before it did it.**
    ///
    /// Absent means the run predicted nothing, so nothing it did can be
    /// contradicted by what it said it would do. That is itself a finding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared: Option<Declaration>,
    /// The ground actually consulted.
    ///
    /// Read against [`Declaration::ground`]: declared and never read, or read
    /// and never declared, are different faults with different fixes.
    #[serde(default)]
    pub ground_read: Vec<String>,
    /// What the act claimed, and what each claim rests on.
    #[serde(default)]
    pub attributions: Vec<Attribution>,
}

impl RunRecord {
    /// A run record in this crate's form.
    pub fn new(
        id: impl Into<String>,
        tool: impl Into<String>,
        task: impl Into<String>,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            form: RUN_FORM.to_string(),
            id: id.into(),
            tool: tool.into(),
            task: task.into(),
            subject: subject.into(),
            ran_at: Utc::now(),
            model: None,
            endpoint_host: None,
            duration_ms: 0,
            proposed: Vec::new(),
            kept: Vec::new(),
            reply: None,
            metrics: Vec::new(),
            address: None,
            arrangement: None,
            declared: None,
            ground_read: Vec::new(),
            attributions: Vec::new(),
        }
    }

    /// Record what the run declared before acting.
    pub fn declaring(mut self, declaration: Declaration) -> Self {
        self.declared = Some(declaration);
        self
    }

    /// Record the ground actually consulted.
    pub fn having_read(mut self, ground: Vec<String>) -> Self {
        self.ground_read = ground;
        self
    }

    /// Declare where this run happened.
    ///
    /// The grain is the caller's to choose and is recorded rather than assumed:
    /// what counts as "the same question" is a judgement nobody else can make,
    /// and leaving it implicit is how comparison quietly compares unlike things.
    pub fn at(mut self, coordinates: Pinned) -> Self {
        self.address = Some(coordinates);
        self
    }

    /// Declare what answered.
    pub fn arranged_as(mut self, arrangement: Pinned) -> Self {
        self.arrangement = Some(arrangement);
        self
    }

    /// The address's digest, when one was declared.
    pub fn address_digest(&self) -> Option<&str> {
        self.address.as_ref().map(|p| p.digest.as_str())
    }

    /// The arrangement's digest, when one was declared.
    pub fn arrangement_digest(&self) -> Option<&str> {
        self.arrangement.as_ref().map(|p| p.digest.as_str())
    }

    /// The host of an endpoint, or nothing when it is not a URL.
    ///
    /// Host rather than the whole URL: a query string is where a key ends up
    /// when somebody is in a hurry, and a store is meant to be readable by
    /// people who did not run it.
    pub fn host_of(endpoint: &str) -> Option<String> {
        let rest = endpoint.split_once("://")?.1;
        let authority = rest.split(['/', '?', '#']).next()?;
        let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
        let host = host.rsplit_once(':').map_or(host, |(h, port)| {
            if port.chars().all(|c| c.is_ascii_digit()) { h } else { host }
        });
        (!host.is_empty()).then(|| host.to_string())
    }

    /// Whether a person moved the draft at all.
    ///
    /// Not a score of the model: a reviewer may amend a good proposal. Over
    /// many runs it is the only measure grounded in a human judgment.
    pub fn was_amended(&self) -> bool {
        self.proposed != self.kept
    }
}

/// One measurement or one verdict, in the shape both are written in.
///
/// `value` is text whatever the measured type. A store that must be migrated
/// before it can be read is a store nobody reads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metric {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// One of `Unknown`, `Inconclusive`, `Unacceptable`, `Poor`, `Average`,
    /// `Good`, `Exceptional`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<String>,
    #[serde(default)]
    pub diagnostics: Vec<String>,
}

impl Metric {
    /// A metric with a value and the reason it has that value.
    pub fn new(name: impl Into<String>, value: Option<String>, reason: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value,
            reason: Some(reason.into()),
            rating: None,
            diagnostics: Vec::new(),
        }
    }

    /// How to read it. Never a pass or a fail: nothing here gates.
    pub fn rated(mut self, rating: impl Into<String>) -> Self {
        self.rating = Some(rating.into());
        self
    }
}

#[path = "run_tests.rs"]
#[cfg(test)]
mod tests;
