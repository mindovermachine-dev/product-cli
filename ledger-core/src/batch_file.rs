//! The batch selection file: the hand-off `ledger inbox` writes and
//! `ledger accept --batch` signs (#86).
//!
//! An enumerated list of rows — repository, branch, decision, version hash,
//! and the grant each is accepted under (D9) — pinned by a manifest digest
//! exactly as `accept --set|--group --confirm` pins its selection
//! ([`crate::batch::MANIFEST_FORM`]). One digest covers every row of the
//! file, across repositories and branches, so a holder confirms a sitting
//! once; each clone then signs its own rows, one acceptance and one
//! signature per row. There is no signature over the file and none over
//! more than one acceptance.

use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::batch::MANIFEST_FORM;
use crate::canon::put;
use crate::hash::domain_hash;

/// The file's own form tag: what it is, and which reading applies.
pub const BATCH_FORM: &str = "ledger.acceptance-batch.v1";

/// How a row with no grant enters the manifest: a namespace with no policy
/// names none.
pub const NO_GRANT: &str = "-";

/// The whole selection file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchFile {
    pub form: String,
    /// Whose acceptances these are. A clone signing as anyone else refuses.
    pub actor: String,
    /// The role the whole batch is made as (D9 (b)), when one is named.
    #[serde(rename = "as", default, skip_serializing_if = "Option::is_none")]
    pub as_role: Option<String>,
    pub rows: Vec<Row>,
}

/// One acceptance to make.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    /// A label for the clone the row belongs to; the writer chooses it.
    pub repository: String,
    /// The branch the decision is proposed on, when the writer knows it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    pub decision: String,
    /// The exact version hash the acceptance signs.
    pub version: String,
    /// The grant the row is accepted under, as the writer resolved it. The
    /// clone that signs the row resolves it again and refuses a mismatch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grant: Option<String>,
}

impl Row {
    /// Whether this row belongs to the checkout `repository` / `branch`.
    pub fn is_for(&self, repository: &str, branch: Option<&str>) -> bool {
        self.repository == repository && (branch.is_none() || self.branch.as_deref() == branch)
    }
}

/// Read and check a selection file.
pub fn read(path: &Path) -> Result<BatchFile, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let file: BatchFile = serde_yaml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    check(&file).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(file)
}

/// Write a selection file.
pub fn write(path: &Path, file: &BatchFile) -> Result<(), String> {
    check(file)?;
    let text = serde_yaml::to_string(file).map_err(|e| e.to_string())?;
    product_core::fileops::write_file_atomic(path, &text).map_err(|e| e.to_string())
}

/// The shape every file must have.
pub fn check(file: &BatchFile) -> Result<(), String> {
    if file.form != BATCH_FORM {
        return Err(format!("`form` is `{}`, not `{BATCH_FORM}`", file.form));
    }
    if file.rows.is_empty() {
        return Err("a batch names at least one row — an empty batch is a misread selection".to_string());
    }
    let mut seen = BTreeSet::new();
    for r in &file.rows {
        if !seen.insert((&r.repository, &r.branch, &r.decision)) {
            return Err(format!("{} is listed twice for {}", r.decision, r.repository));
        }
    }
    Ok(())
}

/// The manifest digest over the whole file: `sha256(MANIFEST_FORM || 0x0A
/// || canonical JSON)` over `{selector: "batch", actor, as, rows}`, each row
/// `[repository, branch, decision, version, grant]` (absent branch `""`,
/// absent grant [`NO_GRANT`]), rows sorted. Every row of every repository is
/// inside it, grants included.
pub fn manifest(file: &BatchFile) -> String {
    let mut m = Map::new();
    put(&mut m, "selector", Some("batch".to_string()));
    put(&mut m, "actor", Some(file.actor.clone()));
    put(&mut m, "as", file.as_role.clone());
    let mut rows: Vec<[String; 5]> = file
        .rows
        .iter()
        .map(|r| {
            [
                r.repository.clone(),
                r.branch.clone().unwrap_or_default(),
                r.decision.clone(),
                r.version.clone(),
                r.grant.clone().unwrap_or_else(|| NO_GRANT.to_string()),
            ]
        })
        .collect();
    rows.sort();
    let array = rows.into_iter().map(|r| Value::Array(r.into_iter().map(Value::String).collect())).collect();
    m.insert("rows".to_string(), Value::Array(array));
    domain_hash(MANIFEST_FORM, Value::Object(m).to_string().as_bytes())
}

#[path = "batch_file_tests.rs"]
#[cfg(test)]
mod tests;
