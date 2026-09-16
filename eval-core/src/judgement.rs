//! What a model said about a run, filed with who said it over what.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::digest;
use crate::run::Metric;

/// The form a judgment is written in.
pub const JUDGEMENT_FORM: &str = "eval.judgement.v1";

/// A model's assessment of one run, on one occasion, over a pinned context.
///
/// Asking a model to assess a run is itself an act: it happens at a time, by a
/// named model, over a particular context, and it is not reproducible. A
/// verdict is filed with all three or it is not filed — any two leave a number
/// that reads as fact and cannot be checked.
///
/// **Never a ratification.** There is no principal field and a machine could
/// not fill one. A judgment is evidence a person may read before deciding; it
/// decides nothing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Judgement {
    pub form: String,
    /// The run this judges, by its id.
    pub judges: String,
    pub judged_at: DateTime<Utc>,
    pub judge: Judge,
    pub context: JudgementContext,
    pub verdicts: Vec<Metric>,
    /// Always true, and carried rather than implied.
    ///
    /// A reader who finds this file should not have to know the directory
    /// layout to learn that nothing in it was decided by anyone.
    pub ratifies_nothing: bool,
}

impl Judgement {
    /// A judgment of one run, over a context pinned as it is built.
    pub fn new(
        judges: impl Into<String>,
        judge: Judge,
        shown: BTreeMap<String, String>,
        verdicts: Vec<Metric>,
    ) -> Self {
        Self {
            form: JUDGEMENT_FORM.to_string(),
            judges: judges.into(),
            judged_at: Utc::now(),
            judge,
            context: JudgementContext::pin(shown),
            verdicts,
            ratifies_nothing: true,
        }
    }

    /// Whether the record still covers what it says it covered.
    pub fn context_holds(&self) -> bool {
        self.context.holds()
    }
}

/// Who gave the verdict. A machine, named as one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Judge {
    pub model: String,
    /// Where it was asked. Host only — the key is never written.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_host: Option<String>,
    /// How the judge is named, in a shape that reads as a machine.
    pub identity: String,
}

impl Judge {
    /// Name a model as the machine it is.
    ///
    /// A `model:` prefix rather than an address that could pass for a person's.
    /// Elsewhere in this workspace a machine principal is refused outright; a
    /// store that merely observes does not get to be vaguer about it.
    pub fn new(model: impl Into<String>, endpoint_host: Option<String>) -> Self {
        let model = model.into();
        Self { identity: format!("model:{model}"), model, endpoint_host }
    }
}

/// Exactly what the judge was shown, and a digest that pins it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JudgementContext {
    pub digest: String,
    /// What the digest covers, kept so a reader can check it.
    pub shown: BTreeMap<String, String>,
}

impl JudgementContext {
    /// Pin a context by digesting what it shows.
    pub fn pin(shown: BTreeMap<String, String>) -> Self {
        Self { digest: digest::pin(&shown), shown }
    }

    /// Whether the digest still matches what this context says it showed.
    pub fn holds(&self) -> bool {
        digest::holds(&self.shown, &self.digest)
    }

    /// The digest's first bytes, enough to tell two contexts apart on disk.
    pub fn short(&self) -> String {
        let body = self.digest.strip_prefix("sha256:").unwrap_or(&self.digest);
        body.chars().take(12).collect()
    }
}

#[path = "judgement_tests.rs"]
#[cfg(test)]
mod tests;
