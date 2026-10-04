//! Loading `.decisions/` — the log is the source of truth (§5).
//!
//! Files in git are the log; nothing here is derived from an index, because
//! L0 has no index. Loading never hard-fails on one bad entry: each file
//! that will not parse becomes a named `SCHEMA` finding so `verify` reports
//! the whole store in one pass rather than one file per run. Same discipline
//! as the `ddd` store, for the same reason.

use std::path::{Path, PathBuf};

use crate::changeset::ChangeSet;
use crate::finding::Finding;
use crate::format;
use crate::set::DecisionSet;
use crate::STORE_DIR;

/// A log file with the path it came from, so findings can name it.
#[derive(Debug, Clone)]
pub struct LoggedChangeSet {
    pub path: PathBuf,
    pub file: ChangeSet,
}

/// The parsed store plus every schema fault met while loading it.
#[derive(Debug, Default)]
pub struct Store {
    /// The repo root holding `.decisions/` — what [`crate::blame`] needs.
    pub root: PathBuf,
    pub dir: PathBuf,
    pub sets: Vec<DecisionSet>,
    /// Role files (`roles/<id>.yml`, format 6) — declared scope, like sets.
    pub roles: Vec<crate::authority::Role>,
    pub log: Vec<LoggedChangeSet>,
    /// Signature sidecars (`sig/<ulid>.<scheme>.sig`, spec v1.8).
    pub sidecars: Vec<crate::signing::Sidecar>,
    pub schema_findings: Vec<Finding>,
}

impl Store {
    /// The set with this id, when one is declared.
    pub fn set(&self, id: &str) -> Option<&DecisionSet> {
        self.sets.iter().find(|s| s.id == id)
    }

    /// The role with this id, when one is declared.
    pub fn role(&self, id: &str) -> Option<&crate::authority::Role> {
        self.roles.iter().find(|r| r.id == id)
    }

    /// How many entries loaded cleanly, for the summary line.
    pub fn entry_count(&self) -> usize {
        self.log.iter().map(|c| c.file.entry_count()).sum::<usize>()
            + self.sets.len()
            + self.roles.len()
    }
}

/// Walk upward from `start` to the first directory containing `.decisions/`.
pub fn find_root(start: &Path) -> Option<PathBuf> {
    let mut cur = Some(start);
    while let Some(dir) = cur {
        if dir.join(STORE_DIR).is_dir() {
            return Some(dir.to_path_buf());
        }
        cur = dir.parent();
    }
    None
}

/// Load the store rooted at `repo_root`.
pub fn load(repo_root: &Path) -> Store {
    let dir = repo_root.join(STORE_DIR);
    let mut store = Store {
        root: repo_root.to_path_buf(),
        dir: dir.clone(),
        ..Store::default()
    };
    load_sets(&dir.join("sets"), &mut store);
    load_roles(&dir.join("roles"), &mut store);
    load_log(&dir.join("log"), &mut store);
    let (sidecars, faults) = crate::signing::load(&dir.join(crate::signing::SIG_DIR));
    store.sidecars = sidecars;
    store.schema_findings.extend(faults);
    store.sets.sort_by(|a, b| a.id.cmp(&b.id));
    store.roles.sort_by(|a, b| a.id.cmp(&b.id));
    store.log.sort_by(|a, b| a.file.id.cmp(&b.file.id));
    store
}

fn load_sets(dir: &Path, store: &mut Store) {
    for path in yaml_files(dir) {
        match std::fs::read_to_string(&path) {
            Ok(text) => take_set(store, &file_label(&path), &stem(&path), &text),
            Err(e) => store.schema_findings.push(Finding::schema(&file_label(&path), e.to_string())),
        }
    }
}

fn load_roles(dir: &Path, store: &mut Store) {
    for path in yaml_files(dir) {
        match std::fs::read_to_string(&path) {
            Ok(text) => take_role(store, &file_label(&path), &stem(&path), &text),
            Err(e) => store.schema_findings.push(Finding::schema(&file_label(&path), e.to_string())),
        }
    }
}

/// Parse one role file's text into the store. Shared like [`take_set`].
pub(crate) fn take_role(store: &mut Store, label: &str, stem: &str, text: &str) {
    match serde_yaml::from_str::<crate::authority::Role>(text) {
        Ok(role) => {
            check_format(label, role.format, store);
            let faults = crate::authority::structure::role_faults(&role, stem, store);
            store.schema_findings.extend(faults.into_iter().map(|m| Finding::schema(label, m)));
            store.roles.push(role);
        }
        Err(e) => store.schema_findings.push(parse_fault("role", label, &e.to_string())),
    }
}

fn load_log(dir: &Path, store: &mut Store) {
    for path in yaml_files(dir) {
        match std::fs::read_to_string(&path) {
            Ok(text) => take_log(store, path.clone(), &file_label(&path), &stem(&path), &text),
            Err(e) => store.schema_findings.push(Finding::schema(&file_label(&path), e.to_string())),
        }
    }
}

/// Parse one set file's text into the store. Shared by the disk loader and
/// the at-revision loader, so both readings apply identical rules.
pub(crate) fn take_set(store: &mut Store, label: &str, stem: &str, text: &str) {
    match serde_yaml::from_str::<DecisionSet>(text) {
        Ok(set) => {
            check_format(label, set.format, store);
            if let Err(e) = DecisionSet::validate_id(&set.id) {
                store.schema_findings.push(Finding::schema(label, e));
            }
            if stem != set.id {
                store.schema_findings.push(Finding::schema(
                    label,
                    format!("declares id `{}` but is filed as `{stem}`", set.id),
                ));
            }
            if store.sets.iter().any(|s| s.id == set.id) {
                store
                    .schema_findings
                    .push(Finding::schema(label, format!("set `{}` is declared twice", set.id)));
            }
            store.sets.push(set);
        }
        Err(e) => store.schema_findings.push(parse_fault("set", label, &e.to_string())),
    }
}

/// Parse one change-set file's text into the store. Shared like [`take_set`].
pub(crate) fn take_log(store: &mut Store, path: PathBuf, label: &str, stem: &str, text: &str) {
    match serde_yaml::from_str::<ChangeSet>(text) {
        Ok(mut file) => {
            check_format(label, file.format, store);
            let at_hashed = file.format >= format::SIGNING_FORMAT;
            for p in &mut file.policies {
                p.at_hashed = at_hashed;
            }
            if stem != file.id.ulid() {
                store.schema_findings.push(Finding::schema(
                    label,
                    format!("declares id `{}` but is filed as `{stem}.yml`", file.id),
                ));
            }
            if store.log.iter().any(|c| c.file.id == file.id) {
                store.schema_findings.push(Finding::schema(
                    label,
                    format!("change-set `{}` appears twice", file.id),
                ));
            }
            for fault in file.acceptances.iter().flat_map(|a| a.schema_faults()) {
                store.schema_findings.push(fault);
            }
            for r in &file.revocations {
                let faults = r.shape_faults(file.format);
                store.schema_findings.extend(faults.into_iter().map(|m| Finding::schema(&r.subject(), m)));
            }
            store.schema_findings.extend(format_faults(label, &file));
            store.log.push(LoggedChangeSet { path, file });
        }
        Err(e) => store.schema_findings.push(parse_fault("change-set", label, &e.to_string())),
    }
}

/// A file must declare the format that defines every field it uses —
/// a lower-format file carrying a later field is a schema fault.
fn format_faults(label: &str, file: &ChangeSet) -> Vec<Finding> {
    let uses = |pred: &dyn Fn(&crate::version::VersionRaw) -> bool| file.versions.iter().any(pred);
    let authority = file.authority_count() > 0;
    let under = file.acceptances.iter().any(|a| a.under.is_some())
        || file.revocations.iter().any(|r| r.under.is_some())
        || file.grants.iter().any(|g| g.under.is_some())
        || file.key_bindings.iter().any(|b| b.under.is_some());
    let rules: [(u32, bool, &str); 5] = [
        (
            format::MERGE_FORMAT,
            uses(&|v| v.merged_from.is_some()),
            "carries `merged_from`, a format 2 field — declare `format: 2`",
        ),
        (
            format::CONTRACT_FORMAT,
            uses(&|v| v.discharge.iter().any(|d| d.is_contract())),
            "carries a `contract:` discharge pointer, a format 3 scheme — declare `format: 3`",
        ),
        (
            format::KEY_FORMAT,
            uses(&|v| v.key.is_some() || v.exported),
            "carries a version `key` or `exported`, format 5 fields — declare `format: 5`",
        ),
        (
            format::AUTHORITY_FORMAT,
            authority,
            "carries authority records (grants, bindings, policy …), format 6 entries — declare `format: 6`",
        ),
        (
            format::SIGNING_FORMAT,
            under,
            "carries `under`, a format 7 field — declare `format: 7`",
        ),
    ];
    rules
        .into_iter()
        .filter(|(needed, used, _)| *used && file.format < *needed)
        .map(|(_, _, message)| Finding::schema(label, message))
        .collect()
}

fn check_format(label: &str, declared: u32, store: &mut Store) {
    if !format::is_supported(declared) {
        store
            .schema_findings
            .push(Finding::schema(label, format::unsupported_message(declared)));
    }
}

/// Every `.yml`/`.yaml` file in `dir`, in a stable order. The PRD writes
/// `.yml`; `.yaml` is read too so a store hand-authored either way loads.
fn yaml_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| p.extension().is_some_and(|e| e == "yml" || e == "yaml"))
        .collect();
    out.sort();
    out
}

fn stem(path: &Path) -> String {
    path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

fn file_label(path: &Path) -> String {
    path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

fn parse_fault(kind: &str, label: &str, err: &str) -> Finding {
    Finding::schema(label, format!("{kind} file does not parse: {err}"))
}

#[path = "store_tests.rs"]
#[cfg(test)]
mod tests;
