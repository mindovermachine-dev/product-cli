//! The committed export — one namespace of the log, as sorted N-Triples.
//!
//! The generator in `DecisionDriven.Analyzers` reads `docs/decisions/<ns>.nt`
//! rather than the log, so the export is a read model that is *committed*.
//! A committed read model can go stale or be edited by hand; `verify
//! --export` re-derives every export from the log and compares bytes, so the
//! file the generator reads is provably the log's own statement. The export
//! carries no fact the log does not: it is the same triples the index holds
//! (`ledger:set` stays the set IRI), restricted to one namespace.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::changeset::ChangeSet;
use crate::store::{LoggedChangeSet, Store};

use super::ntriples;

/// Where committed exports live, relative to the repo root.
pub const EXPORT_DIR: &str = "docs/decisions";

/// The analyzers' citation projection shares the directory; it is not an
/// export of the log and is never compared against one.
const CITATIONS_SUFFIX: &str = ".citations.nt";

/// One reason a committed export does not match the log. A distinct stage,
/// outside the file gate's closed classes, reported by the same `verify`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportFinding {
    pub subject: String,
    pub message: String,
}

/// Every namespace the log speaks: the namespaces of its decision ids.
pub fn namespaces(store: &Store) -> BTreeSet<String> {
    let from_records = store.log.iter().flat_map(|c| c.file.decisions.iter().map(|d| &d.id));
    let from_versions = store.log.iter().flat_map(|c| c.file.versions.iter().map(|v| &v.decision));
    from_records.chain(from_versions).map(|id| id.namespace().to_string()).collect()
}

/// The export of one namespace. `None` when the log does not speak it.
pub fn export(store: &Store, namespace: &str) -> Option<String> {
    namespaces(store).contains(namespace).then(|| ntriples::emit(&select(store, namespace)))
}

/// The committed path for a namespace's export.
pub fn default_path(root: &Path, namespace: &str) -> PathBuf {
    root.join(EXPORT_DIR).join(format!("{namespace}.nt"))
}

/// Write an export atomically, creating `docs/decisions/` on first use.
pub fn write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    product_core::fileops::write_file_atomic(path, text).map_err(|e| e.to_string())
}

/// The store restricted to one namespace: its decisions, their versions,
/// the acceptances of those decisions, the revocations of those
/// acceptances, the sets those versions name, and the change-sets that
/// filed any of it (each holding only what belongs here). The authority
/// records that reach the namespace come too (spec v1.7): its policy
/// versions and key bindings, the grants whose scope covers it (`*`, its
/// `ns:`, or a set its versions name) with their acceptances,
/// unavailability, availability and revocations, and the roles those
/// grants and policies name.
pub fn select(store: &Store, namespace: &str) -> Store {
    let ours = |id: &crate::id::DecisionId| id.namespace() == namespace;
    let named: BTreeSet<String> = store
        .log
        .iter()
        .flat_map(|c| c.file.versions.iter())
        .filter(|v| ours(&v.decision))
        .map(|v| v.set.clone())
        .collect();
    let reach = Reach::of(store, namespace, &named);
    let log: Vec<LoggedChangeSet> = store
        .log
        .iter()
        .map(|logged| LoggedChangeSet {
            path: logged.path.clone(),
            file: restrict(&logged.file, &ours, &reach),
        })
        .filter(|logged| !logged.file.is_empty())
        .collect();
    let roles = reach.roles(&log);
    Store {
        root: store.root.clone(),
        dir: store.dir.clone(),
        sets: store.sets.iter().filter(|s| named.contains(&s.id)).cloned().collect(),
        roles: store.roles.iter().filter(|r| roles.contains(&r.id)).cloned().collect(),
        log,
        sidecars: store.sidecars.clone(),
        schema_findings: Vec::new(),
    }
}

/// Which records' ids reach one namespace.
struct Reach<'n> {
    namespace: &'n str,
    acceptances: BTreeSet<String>,
    grants: BTreeSet<String>,
    intervals: BTreeSet<String>,
}

impl<'n> Reach<'n> {
    fn of(store: &Store, namespace: &'n str, named: &BTreeSet<String>) -> Self {
        use crate::authority::GrantScope;
        let files = || store.log.iter().map(|c| &c.file);
        let acceptances = files()
            .flat_map(|c| c.acceptances.iter())
            .filter(|a| a.decision.namespace() == namespace)
            .map(|a| a.id.to_string())
            .collect();
        let grants: BTreeSet<String> = files()
            .flat_map(|c| c.grants.iter())
            .filter(|g| match &g.scope {
                GrantScope::All => true,
                GrantScope::Namespace(n) => n == namespace,
                GrantScope::Set(s) => named.contains(s),
                GrantScope::Pattern(_) => false,
            })
            .map(|g| g.id.to_string())
            .collect();
        let intervals = files()
            .flat_map(|c| c.unavailabilities.iter())
            .filter(|u| grants.contains(&u.grant.to_string()))
            .map(|u| u.id.to_string())
            .collect();
        Self { namespace, acceptances, grants, intervals }
    }

    /// The roles the kept grants and policies name.
    fn roles(&self, log: &[LoggedChangeSet]) -> BTreeSet<String> {
        let files = || log.iter().map(|c| &c.file);
        files()
            .flat_map(|c| c.grants.iter().map(|g| g.role.clone()))
            .chain(files().flat_map(|c| c.policies.iter().map(|p| p.accept_role.clone())))
            .collect()
    }
}

fn restrict(
    cs: &ChangeSet,
    ours: &impl Fn(&crate::id::DecisionId) -> bool,
    reach: &Reach<'_>,
) -> ChangeSet {
    use crate::authority::Revocable;
    let mut kept = cs.clone();
    kept.decisions.retain(|d| ours(&d.id));
    kept.versions.retain(|v| ours(&v.decision));
    kept.acceptances.retain(|a| ours(&a.decision));
    kept.revocations.retain(|r| match r.target() {
        Some(Revocable::Acceptance(a)) => reach.acceptances.contains(&a.to_string()),
        Some(Revocable::Grant(g)) => reach.grants.contains(&g.to_string()),
        None => false,
    });
    kept.grants.retain(|g| reach.grants.contains(&g.id.to_string()));
    kept.grant_acceptances.retain(|ga| reach.grants.contains(&ga.grant.to_string()));
    kept.unavailabilities.retain(|u| reach.intervals.contains(&u.id.to_string()));
    kept.availabilities.retain(|a| reach.intervals.contains(&a.ends.to_string()));
    kept.key_bindings.retain(|b| b.namespace == reach.namespace);
    kept.policies.retain(|p| p.namespace == reach.namespace);
    kept
}

/// `verify --export`: every committed export equals the log's, byte for
/// byte, and every namespace the log speaks has one. Finding nothing to
/// compare is itself a finding — a check that silently ran over nothing
/// would read as a passing one.
pub fn check(root: &Path, store: &Store) -> Vec<ExportFinding> {
    let spoken = namespaces(store);
    let committed = committed_exports(root);
    let mut out = Vec::new();
    if committed.is_empty() {
        out.push(finding(
            EXPORT_DIR,
            "no committed export found — `ledger export --format ntriples` writes one per namespace",
        ));
        return out;
    }
    for (namespace, path) in &committed {
        out.extend(compare(store, namespace, path));
    }
    let present: BTreeSet<&String> = committed.iter().map(|(ns, _)| ns).collect();
    for namespace in spoken.iter().filter(|ns| !present.contains(ns)) {
        out.push(finding(
            &relative(root, &default_path(root, namespace)),
            &format!("the log speaks namespace `{namespace}` but no export of it is committed — {}", regenerate(namespace)),
        ));
    }
    out
}

/// One committed file against the log's export of its namespace.
fn compare(store: &Store, namespace: &str, path: &Path) -> Option<ExportFinding> {
    let label = relative(&store.root, path);
    let Some(expected) = export(store, namespace) else {
        return Some(finding(&label, &format!("exports namespace `{namespace}`, which the log does not speak")));
    };
    let actual = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) => return Some(finding(&label, &format!("could not be read: {e}"))),
    };
    if actual == expected {
        return None;
    }
    let want: BTreeSet<&str> = expected.lines().collect();
    let have: BTreeSet<&str> = actual.lines().collect();
    let extra = have.difference(&want).count();
    let missing = want.difference(&have).count();
    let message = if extra == 0 && missing == 0 {
        format!("carries the log's triples but not its bytes (order, spacing or line endings) — {}", regenerate(namespace))
    } else {
        format!(
            "does not match the log: {extra} line(s) the log does not produce, {missing} line(s) it lacks — {}",
            regenerate(namespace)
        )
    };
    Some(finding(&label, &message))
}

/// `(namespace, path)` for every committed export, by file stem.
fn committed_exports(root: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(root.join(EXPORT_DIR)) else {
        return Vec::new();
    };
    let mut out: Vec<(String, PathBuf)> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?.to_string();
            if name.ends_with(CITATIONS_SUFFIX) {
                return None;
            }
            let namespace = name.strip_suffix(".nt")?.to_string();
            Some((namespace, path))
        })
        .collect();
    out.sort();
    out
}

fn regenerate(namespace: &str) -> String {
    format!("regenerate with `ledger export --format ntriples --namespace {namespace}`")
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).display().to_string()
}

fn finding(subject: &str, message: &str) -> ExportFinding {
    ExportFinding { subject: subject.to_string(), message: message.to_string() }
}
