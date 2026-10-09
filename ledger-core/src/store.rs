//! Loading `.decisions/` — the log is the source of truth (§5).
//!
//! Files in git are the log; nothing here is derived from an index, because
//! L0 has no index. Loading never hard-fails on one bad entry: each file
//! that will not parse becomes a named `SCHEMA` finding so `verify` reports
//! the whole store in one pass rather than one file per run. Same discipline
//! as the `ddd` store, for the same reason.
//!
//! **One directory per namespace** (LP-3.34, rulings 62 and 63). Each
//! namespace's sets, roles, change-sets and sidecars are read from
//! `.decisions/ns/<namespace>/`, and a file's namespace is its directory
//! ([`crate::layout`]). The flat paths of revision v1.8 are read nowhere:
//! a file there, or anywhere under `.decisions/` outside `ns/` and `index/`,
//! is a schema fault.

use std::path::{Path, PathBuf};

use crate::changeset::ChangeSet;
use crate::finding::Finding;
use crate::format;
use crate::layout;
use crate::set::DecisionSet;
use crate::STORE_DIR;

/// A log file with the path it came from, so findings can name it, and
/// the namespace whose directory holds it (LP-3.34).
#[derive(Debug, Clone)]
pub struct LoggedChangeSet {
    pub namespace: String,
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
    /// The set with this id declared in `namespace`, when one is: a version
    /// names a set of its own namespace (LP-5.22).
    pub fn set_in(&self, namespace: &str, id: &str) -> Option<&DecisionSet> {
        self.sets.iter().find(|s| s.id == id && s.namespace == namespace)
    }

    /// The namespaces the store has a directory for, sorted.
    pub fn namespaces(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .sets
            .iter()
            .map(|s| s.namespace.clone())
            .chain(self.roles.iter().map(|r| r.namespace.clone()))
            .chain(self.log.iter().map(|l| l.namespace.clone()))
            .chain(self.sidecars.iter().map(|c| c.namespace.clone()))
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// The role with this id declared under `namespace`'s `roles/`, when one
    /// is: roles are per namespace (LP-5.19, ruling 47).
    pub fn role_in(&self, namespace: &str, id: &str) -> Option<&crate::authority::Role> {
        self.roles.iter().find(|r| r.id == id && r.namespace == namespace)
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

/// Load the store rooted at `repo_root`: every namespace directory under
/// `.decisions/ns/`, then the faults for anything under `.decisions/` that
/// the layout does not hold (LP-3.34).
pub fn load(repo_root: &Path) -> Store {
    let dir = repo_root.join(STORE_DIR);
    let mut store = Store {
        root: repo_root.to_path_buf(),
        dir: dir.clone(),
        ..Store::default()
    };
    for namespace in layout::namespaces(repo_root) {
        load_namespace(repo_root, &namespace, &mut store);
    }
    store.schema_findings.extend(layout_faults(&dir));
    store.sets.sort_by(|a, b| (&a.namespace, &a.id).cmp(&(&b.namespace, &b.id)));
    store.roles.sort_by(|a, b| (&a.namespace, &a.id).cmp(&(&b.namespace, &b.id)));
    store.log.sort_by(|a, b| a.file.id.cmp(&b.file.id));
    store
}

/// One namespace's directory: its sets, roles, change-sets and sidecars.
fn load_namespace(root: &Path, namespace: &str, store: &mut Store) {
    if let Err(e) = crate::id::validate_namespace(namespace) {
        store.schema_findings.push(Finding::schema(&format!("{}/{}", layout::NS_DIR, namespace), format!("is not a namespace directory: {e}")));
    }
    for path in yaml_files(&layout::sets_dir(root, namespace)) {
        match std::fs::read_to_string(&path) {
            Ok(text) => take_set(store, namespace, &file_label(&path), &stem(&path), &text),
            Err(e) => store.schema_findings.push(Finding::schema(&file_label(&path), e.to_string())),
        }
    }
    for path in yaml_files(&layout::roles_dir(root, namespace)) {
        match std::fs::read_to_string(&path) {
            Ok(text) => take_role(store, namespace, &file_label(&path), &stem(&path), &text),
            Err(e) => store.schema_findings.push(Finding::schema(&file_label(&path), e.to_string())),
        }
    }
    for path in yaml_files(&layout::log_dir(root, namespace)) {
        match std::fs::read_to_string(&path) {
            Ok(text) => take_log(store, namespace, path.clone(), &file_label(&path), &stem(&path), &text),
            Err(e) => store.schema_findings.push(Finding::schema(&file_label(&path), e.to_string())),
        }
    }
    let (sidecars, faults) = crate::signing::load(&layout::sig_dir(root, namespace), namespace);
    store.sidecars.extend(sidecars);
    store.schema_findings.extend(faults);
}

/// A schema fault for every file under `.decisions/` the layout does not
/// hold: the flat paths of revision v1.8 (`sets/`, `roles/`, `log/`,
/// `sig/`, `allowed_signers`) and anything else outside `ns/` and `index/`
/// (LP-3.34, ruling 63). A file directly under `ns/` is one too: a
/// namespace is a directory.
fn layout_faults(dir: &Path) -> Vec<Finding> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return out };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        let name = file_label(&path);
        if name == layout::NS_DIR && path.is_dir() {
            let Ok(inner) = std::fs::read_dir(&path) else { continue };
            for stray in inner.flatten().map(|e| e.path()).filter(|p| !p.is_dir()) {
                out.push(stray_fault(&format!("{STORE_DIR}/{}/{}", layout::NS_DIR, file_label(&stray))));
            }
            continue;
        }
        if name == layout::INDEX_DIR || name == ".gitattributes" {
            continue;
        }
        if path.is_dir() {
            for file in files_under(&path) {
                out.push(stray_fault(&format!("{STORE_DIR}/{name}/{file}")));
            }
        } else {
            out.push(stray_fault(&format!("{STORE_DIR}/{name}")));
        }
    }
    out
}

fn stray_fault(path: &str) -> Finding {
    Finding::schema(
        path,
        "is outside the layout — every namespace lives under `.decisions/ns/<namespace>/` with its own \
         `sets/`, `roles/`, `log/`, `sig/` and `allowed_signers`, and the flat paths of revision v1.8 are \
         not read (LP-3.34, ruling 63)",
    )
}

/// Every file under `dir`, recursively, as paths relative to `dir`.
fn files_under(dir: &Path) -> Vec<String> {
    fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for path in entries.flatten().map(|e| e.path()) {
            if path.is_dir() {
                walk(&path, base, out);
            } else {
                out.push(path.strip_prefix(base).unwrap_or(&path).to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

/// Parse one role file's text into the store. Shared like [`take_set`].
pub(crate) fn take_role(store: &mut Store, namespace: &str, label: &str, stem: &str, text: &str) {
    match serde_yaml::from_str::<crate::authority::Role>(text) {
        Ok(mut role) => {
            role.namespace = namespace.to_string();
            check_format(label, role.format, store);
            let faults = crate::authority::structure::role_faults(&role, stem, store);
            store.schema_findings.extend(faults.into_iter().map(|m| Finding::schema(label, m)));
            store.roles.push(role);
        }
        Err(e) => store.schema_findings.push(parse_fault("role", label, &e.to_string())),
    }
}

/// Parse one set file's text into the store. Shared by the disk loader and
/// the at-revision loader, so both readings apply identical rules. A set id
/// is unique within its namespace (LP-3.36).
pub(crate) fn take_set(store: &mut Store, namespace: &str, label: &str, stem: &str, text: &str) {
    match serde_yaml::from_str::<DecisionSet>(text) {
        Ok(mut set) => {
            set.namespace = namespace.to_string();
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
            if store.sets.iter().any(|s| s.id == set.id && s.namespace == namespace) {
                store
                    .schema_findings
                    .push(Finding::schema(label, format!("set `{}` is declared twice in `{namespace}`", set.id)));
            }
            store.sets.push(set);
        }
        Err(e) => store.schema_findings.push(parse_fault("set", label, &e.to_string())),
    }
}

/// Parse one change-set file's text into the store. Shared like [`take_set`].
pub(crate) fn take_log(store: &mut Store, namespace: &str, path: PathBuf, label: &str, stem: &str, text: &str) {
    match serde_yaml::from_str::<ChangeSet>(text) {
        Ok(file) => {
            check_format(label, file.format, store);
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
            store.schema_findings.extend(crate::scalars::faults(label, text));
            store.log.push(LoggedChangeSet { namespace: namespace.to_string(), path, file });
        }
        Err(e) => store.schema_findings.push(parse_fault("change-set", label, &e.to_string())),
    }
}

/// A file must declare the format that defines every field it uses —
/// a lower-format file carrying a later field is a schema fault.
fn format_faults(label: &str, file: &ChangeSet) -> Vec<Finding> {
    version_field_rules(file)
        .into_iter()
        .chain(entity_rules(file))
        .filter(|(needed, used, _)| *used && file.format < *needed)
        .map(|(_, _, message)| Finding::schema(label, message))
        .collect()
}

/// One declare-what-you-need row: the format a use needs, whether the file
/// makes it, and the fault when it declares less.
type FormatRule = (u32, bool, &'static str);

/// The rows for fields a version carries (formats 2–5).
fn version_field_rules(file: &ChangeSet) -> [FormatRule; 4] {
    let uses = |pred: &dyn Fn(&crate::version::VersionRaw) -> bool| file.versions.iter().any(pred);
    [
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
            format::REVISIT_FORMAT,
            uses(&|v| !v.revisit_if.is_empty()),
            "carries `revisit_if`, a format 4 field — declare `format: 4`",
        ),
        (
            format::KEY_FORMAT,
            uses(&|v| v.key.is_some() || v.exported),
            "carries a version `key` or `exported`, format 5 fields — declare `format: 5`",
        ),
    ]
}

/// The rows for entities beside the versions (formats 6–7).
fn entity_rules(file: &ChangeSet) -> [FormatRule; 3] {
    [
        (
            format::AUTHORITY_FORMAT,
            file.authority_count() > 0,
            "carries authority records (grants, bindings, policy …), format 6 entries — declare `format: 6`",
        ),
        (
            format::SIGNING_FORMAT,
            carries_under(file),
            "carries `under`, a format 7 field — declare `format: 7`",
        ),
        (
            format::SIGNING_FORMAT,
            !file.policies.is_empty(),
            "carries a namespace policy, a format 7 entry (its `at` is hashed, D8) — declare `format: 7`",
        ),
    ]
}

/// Whether any entity in the file names the grant it was made under.
fn carries_under(file: &ChangeSet) -> bool {
    file.acceptances.iter().any(|a| a.under.is_some())
        || file.revocations.iter().any(|r| r.under.is_some())
        || file.grants.iter().any(|g| g.under.is_some())
        || file.key_bindings.iter().any(|b| b.under.is_some())
        || file.policies.iter().any(|p| p.under.is_some())
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
