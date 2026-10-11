//! Every stored version digest, re-derived under the current canonical form.
//!
//! Spec v1.6 (format 5) adds `key` and `exported` to the hashed field set.
//! They are omitted from the canonical object when absent, so a version
//! written before they existed must hash to exactly the digest it was filed
//! with: no existing acceptance may be invalidated by a format bump that
//! leaves `CANONICAL_FORM` at `v1`. This test is the proof, over every
//! fixture store and over this repository's own `.decisions/` log.

use std::path::{Path, PathBuf};

use ledger_core::changeset::ChangeSet;
use ledger_core::hash::version_hash;

/// The fixture whose stored hash is stale on purpose (it exercises `L007`).
const STALE_BY_DESIGN: &str = "l007";

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Every log directory of a store: `.decisions/ns/<ns>/log` (LP-3.34), or
/// the flat `.decisions/log` of a store not yet re-laid out.
fn log_dirs_of(root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = ledger_core::layout::namespaces(root)
        .into_iter()
        .map(|ns| ledger_core::layout::log_dir(root, &ns))
        .filter(|d| d.is_dir())
        .collect();
    let flat = root.join(".decisions/log");
    if flat.is_dir() {
        out.push(flat);
    }
    out
}

fn log_dirs() -> Vec<PathBuf> {
    let fixtures = workspace().join("ledger-cli/tests/fixtures");
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&fixtures)
        .expect("fixtures")
        .flatten()
        .filter(|e| e.file_name() != STALE_BY_DESIGN)
        .flat_map(|e| log_dirs_of(&e.path()))
        .collect();
    dirs.extend(log_dirs_of(&workspace()));
    dirs.sort();
    dirs
}

#[test]
fn every_filed_digest_is_unchanged_by_the_format_5_fields() {
    let mut checked = 0;
    let mut moved = Vec::new();
    for dir in log_dirs() {
        for entry in std::fs::read_dir(&dir).expect("log dir").flatten() {
            let text = std::fs::read_to_string(entry.path()).expect("read");
            let cs: ChangeSet = serde_yaml::from_str(&text).expect("parse");
            for v in &cs.versions {
                checked += 1;
                if version_hash(v) != v.hash {
                    moved.push(format!("{}: {}", entry.path().display(), v.decision));
                }
            }
        }
    }
    assert!(checked > 100, "the proof must cover the real store, not only fixtures: {checked}");
    assert!(moved.is_empty(), "digests moved under the current canonical form: {moved:#?}");
}
