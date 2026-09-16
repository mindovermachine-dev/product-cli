//! Where runs are kept, beside but never among what judged them.
//!
//! The key layout lives here and nowhere else, so a backend added later
//! addresses bodies without knowing what a run record is. Disk and object
//! storage see the same keys; only [`Blobs`] differs between them.

use product_core::error::{ProductError, Result};

use crate::blobs::Blobs;
use crate::judgement::Judgement;
use crate::run::RunRecord;

/// The key prefix runs are kept under.
pub const RUNS: &str = "runs";

/// The key prefix judgments are kept under.
pub const JUDGEMENTS: &str = "judgements";

/// The key one run record is kept at.
pub fn run_key(id: &str) -> String {
    format!("{RUNS}/{id}.json")
}

/// The key prefix one run's judgments are kept under.
///
/// A prefix per run, because a run is judged more than once: by a second
/// model, by the same model later, by a bigger one when the question turns out
/// to matter. A single verdict per run would be one nobody could argue with.
pub fn judgements_prefix(run: &str) -> String {
    format!("{JUDGEMENTS}/{run}")
}

/// The key one judgment is kept at.
///
/// Named by the judge and the context digest, so re-asking the same judge the
/// same question replaces in place rather than accreting copies, while a
/// genuinely new occasion differs in one of them and lands beside the first.
pub fn judgement_key(judgement: &Judgement) -> String {
    let slug = slug(&judgement.judge.model);
    let short = judgement.context.short();
    format!("{}/{slug}.{short}.json", judgements_prefix(&judgement.judges))
}

/// Records kept in whatever [`Blobs`] was configured.
pub struct EvalStore<B: Blobs> {
    blobs: B,
}

impl<B: Blobs> EvalStore<B> {
    /// A store over a backend.
    pub fn new(blobs: B) -> Self {
        Self { blobs }
    }

    /// The backend beneath, for a caller that needs to say where it wrote.
    pub fn blobs(&self) -> &B {
        &self.blobs
    }

    /// File one run record, returning a locator.
    pub fn write_run(&self, record: &RunRecord) -> Result<String> {
        self.blobs.put(&run_key(&record.id), &body(record)?)
    }

    /// File one judgment, returning a locator.
    pub fn write_judgement(&self, judgement: &Judgement) -> Result<String> {
        self.blobs.put(&judgement_key(judgement), &body(judgement)?)
    }

    /// Read one run record back.
    pub fn read_run(&self, id: &str) -> Result<RunRecord> {
        let key = run_key(id);
        let text = self
            .blobs
            .get(&key)?
            .ok_or_else(|| ProductError::NotFound(format!("no run `{id}`")))?;
        serde_json::from_str(&text).map_err(|e| ProductError::ConfigError(format!("{key}: {e}")))
    }

    /// Every run in the store, oldest first.
    ///
    /// An unreadable entry is skipped rather than raised: a store is
    /// measurement, and one corrupt record must not hide the rest.
    pub fn read_runs(&self) -> Vec<RunRecord> {
        let mut found: Vec<RunRecord> = self.read_under(RUNS);
        found.sort_by_key(|r| r.ran_at);
        found
    }

    /// Every judgment of one run, oldest first.
    pub fn read_judgements(&self, run: &str) -> Vec<Judgement> {
        let mut found: Vec<Judgement> = self.read_under(&judgements_prefix(run));
        found.sort_by_key(|j| j.judged_at);
        found
    }

    /// Parse every body under a prefix, skipping what will not parse.
    fn read_under<T: serde::de::DeserializeOwned>(&self, prefix: &str) -> Vec<T> {
        self.blobs
            .list(prefix)
            .into_iter()
            .filter(|key| key.ends_with(".json"))
            .filter_map(|key| self.blobs.get(&key).ok().flatten())
            .filter_map(|text| serde_json::from_str(&text).ok())
            .collect()
    }
}

/// A record as the bytes that land in the store.
fn body<T: serde::Serialize>(record: &T) -> Result<String> {
    serde_json::to_string_pretty(record)
        .map(|text| format!("{text}\n"))
        .map_err(|e| ProductError::Internal(format!("a record would not encode: {e}")))
}

/// A model name reduced to something a key accepts.
fn slug(model: &str) -> String {
    model
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect()
}

#[path = "store_tests.rs"]
#[cfg(test)]
mod tests;
