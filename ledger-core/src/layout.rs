//! The store layout: one directory per namespace (LP-3.34, rulings 62, 63).
//!
//! Every store holds each namespace under `.decisions/ns/<namespace>/`, with
//! its own `sets/`, `roles/`, `log/`, `sig/` and derived `allowed_signers`.
//! A file's namespace is its directory. The root `.decisions/` holds only
//! `ns/` and the uncommitted `index/` cache; a file at the flat paths of
//! revision v1.8 (`.decisions/sets/`, `.decisions/roles/`, `.decisions/log/`,
//! `.decisions/sig/`, `.decisions/allowed_signers`), or anywhere else under
//! `.decisions/` outside `ns/` and `index/`, is a schema fault at the verified
//! commit and in the working tree — so is a store with no `ns/` directory
//! and any of those paths. The layout is specification revision v1.9, with
//! no format number (ruling 66): no file's content changes.
//!
//! Every reader of a repo-relative path classifies it here, so the one
//! place that knows the layout is this module.

use std::path::{Path, PathBuf};

use crate::STORE_DIR;

/// The directory under `.decisions/` that holds the namespaces.
pub const NS_DIR: &str = "ns";
/// The rebuildable cache, never committed (LP-3.13).
pub const INDEX_DIR: &str = "index";
pub const SETS_DIR: &str = "sets";
pub const ROLES_DIR: &str = "roles";
pub const LOG_DIR: &str = "log";
pub const SIG_DIR: &str = "sig";
/// The derived trust file's name, in each namespace's directory.
pub const SIGNERS_FILE: &str = "allowed_signers";

/// The directories whose files land and are held immutable (`L007`): the
/// log, role files and signature sidecars of every namespace, as git
/// pathspecs. Set files are not tracked: a floor is raised in place.
pub const TRACKED: &[&str] = &[".decisions/ns/*/log/*", ".decisions/ns/*/roles/*", ".decisions/ns/*/sig/*"];

/// The tracked directories of revision v1.8's flat layout, as git
/// pathspecs. Read in history only, by the legacy capability (LP-3.35,
/// rulings 82 and 97): never at the verified commit.
pub const FLAT_TRACKED: &[&str] = &[".decisions/log", ".decisions/roles", ".decisions/sig"];

/// What kind of file a path under a namespace directory is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Set,
    Role,
    Log,
    Sig,
    Signers,
}

/// `.decisions/ns/<namespace>` under `root`.
pub fn namespace_dir(root: &Path, namespace: &str) -> PathBuf {
    root.join(STORE_DIR).join(NS_DIR).join(namespace)
}

/// `.decisions/ns/<namespace>/log` under `root`.
pub fn log_dir(root: &Path, namespace: &str) -> PathBuf {
    namespace_dir(root, namespace).join(LOG_DIR)
}

/// `.decisions/ns/<namespace>/sets` under `root`.
pub fn sets_dir(root: &Path, namespace: &str) -> PathBuf {
    namespace_dir(root, namespace).join(SETS_DIR)
}

/// `.decisions/ns/<namespace>/roles` under `root`.
pub fn roles_dir(root: &Path, namespace: &str) -> PathBuf {
    namespace_dir(root, namespace).join(ROLES_DIR)
}

/// `.decisions/ns/<namespace>/sig` under `root`.
pub fn sig_dir(root: &Path, namespace: &str) -> PathBuf {
    namespace_dir(root, namespace).join(SIG_DIR)
}

/// `.decisions/ns/<namespace>/allowed_signers` under `root`.
pub fn signers_path(root: &Path, namespace: &str) -> PathBuf {
    namespace_dir(root, namespace).join(SIGNERS_FILE)
}

/// The repo-relative directory `.decisions/ns/<namespace>/<kind>`.
pub fn relative_dir(namespace: &str, kind: Kind) -> String {
    let sub = match kind {
        Kind::Set => SETS_DIR,
        Kind::Role => ROLES_DIR,
        Kind::Log => LOG_DIR,
        Kind::Sig => SIG_DIR,
        Kind::Signers => "",
    };
    format!("{STORE_DIR}/{NS_DIR}/{namespace}/{sub}")
}

/// The namespace and kind of a repo-relative path under the layout:
/// `.decisions/ns/<ns>/<sets|roles|log|sig>/<file>`, or
/// `.decisions/ns/<ns>/allowed_signers`. `None` for anything else.
pub fn classify(path: &str) -> Option<(String, Kind)> {
    let rest = path.strip_prefix(&format!("{STORE_DIR}/{NS_DIR}/"))?;
    let (namespace, rest) = rest.split_once('/')?;
    if namespace.is_empty() {
        return None;
    }
    if rest == SIGNERS_FILE {
        return Some((namespace.to_string(), Kind::Signers));
    }
    let (sub, file) = rest.split_once('/')?;
    if file.is_empty() || file.contains('/') {
        return None;
    }
    let kind = match sub {
        SETS_DIR => Kind::Set,
        ROLES_DIR => Kind::Role,
        LOG_DIR => Kind::Log,
        SIG_DIR => Kind::Sig,
        _ => return None,
    };
    Some((namespace.to_string(), kind))
}

/// The kind of a repo-relative path of the flat layout:
/// `.decisions/<sets|roles|log|sig>/<file>`. `None` for anything else, a
/// path of the namespace layout included: `ns` is no directory of the flat
/// layout, so no path matches both.
pub fn classify_flat(path: &str) -> Option<Kind> {
    let rest = path.strip_prefix(&format!("{STORE_DIR}/"))?;
    let (sub, file) = rest.split_once('/')?;
    if file.is_empty() || file.contains('/') {
        return None;
    }
    match sub {
        SETS_DIR => Some(Kind::Set),
        ROLES_DIR => Some(Kind::Role),
        LOG_DIR => Some(Kind::Log),
        SIG_DIR => Some(Kind::Sig),
        _ => None,
    }
}

/// The flat path a namespaced tracked file had before the re-layout: the
/// same kind and file name directly under `.decisions/`. A file's stem is
/// its id (LP-3.12), so the name follows the entity across the move.
pub fn flat_counterpart(path: &str) -> Option<String> {
    let (_, kind) = classify(path)?;
    let sub = match kind {
        Kind::Set => SETS_DIR,
        Kind::Role => ROLES_DIR,
        Kind::Log => LOG_DIR,
        Kind::Sig => SIG_DIR,
        Kind::Signers => return None,
    };
    let file = path.rsplit('/').next()?;
    Some(format!("{STORE_DIR}/{sub}/{file}"))
}

/// Where a flat file went: every repo-relative path under a namespace's
/// directory at `root` that holds a file of the same kind and name.
pub fn moved_to(root: &Path, flat: &str) -> Vec<String> {
    let Some(kind) = classify_flat(flat) else { return Vec::new() };
    let Some(file) = flat.rsplit('/').next() else { return Vec::new() };
    namespaces(root)
        .iter()
        .filter(|ns| {
            let dir = match kind {
                Kind::Set => sets_dir(root, ns),
                Kind::Role => roles_dir(root, ns),
                Kind::Log => log_dir(root, ns),
                Kind::Sig => sig_dir(root, ns),
                Kind::Signers => return false,
            };
            dir.join(file).is_file()
        })
        .map(|ns| format!("{}/{file}", relative_dir(ns, kind)))
        .collect()
}

/// Whether some namespace's directory under `root` holds a file of the
/// same kind and name as the flat path.
pub fn moved_to_a_namespace(root: &Path, flat: &str) -> bool {
    !moved_to(root, flat).is_empty()
}

/// The oldest first-parent commit of any of `revs` that touched a flat
/// tracked path: the history predates v1.9 (LP-3.35). `None` when no
/// commit did, or when git cannot answer.
pub fn flat_history(root: &Path, revs: &[&str]) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "--first-parent", "--reverse", "--format=%H"])
        .args(revs)
        .arg("--")
        .args(FLAT_TRACKED)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout).lines().map(str::trim).find(|l| !l.is_empty()).map(str::to_string)
}

/// Whether a repo-relative path is a change-set file of some namespace.
pub fn is_log(path: &str) -> bool {
    classify(path).is_some_and(|(_, k)| k == Kind::Log)
}

/// Whether a repo-relative path is a role file of some namespace.
pub fn is_role(path: &str) -> bool {
    classify(path).is_some_and(|(_, k)| k == Kind::Role)
}

/// The namespaces present under `.decisions/ns/`, sorted: every directory
/// there. Entries that are not directories are the loader's to report.
pub fn namespaces(root: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(root.join(STORE_DIR).join(NS_DIR)) else { return Vec::new() };
    let mut out: Vec<String> =
        entries.flatten().filter(|e| e.path().is_dir()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_under_a_namespace_directory_classifies_by_its_subdirectory() {
        assert_eq!(classify(".decisions/ns/a.ns/log/01K.yml"), Some(("a.ns".into(), Kind::Log)));
        assert_eq!(classify(".decisions/ns/a.ns/roles/steward.yml"), Some(("a.ns".into(), Kind::Role)));
        assert_eq!(classify(".decisions/ns/a.ns/sig/01K.ssh.sig"), Some(("a.ns".into(), Kind::Sig)));
        assert_eq!(classify(".decisions/ns/a.ns/sets/design.yml"), Some(("a.ns".into(), Kind::Set)));
        assert_eq!(classify(".decisions/ns/a.ns/allowed_signers"), Some(("a.ns".into(), Kind::Signers)));
    }

    #[test]
    fn the_flat_paths_and_anything_else_classify_as_nothing() {
        for flat in [".decisions/log/01K.yml", ".decisions/roles/steward.yml", ".decisions/sig/x.ssh.sig", ".decisions/allowed_signers"] {
            assert_eq!(classify(flat), None, "{flat}");
        }
        assert_eq!(classify(".decisions/ns/a.ns/log"), None, "a directory, not a file");
        assert_eq!(classify(".decisions/ns/a.ns/pins/b/x.nt"), None, "pins are not read here");
        assert_eq!(classify(".decisions/ns//log/x.yml"), None);
        assert!(is_log(".decisions/ns/a/log/x.yml") && !is_log(".decisions/ns/a/roles/x.yml"));
        assert!(is_role(".decisions/ns/a/roles/x.yml"));
    }

    #[test]
    fn a_flat_path_classifies_by_kind_and_names_its_namespaced_counterpart() {
        assert_eq!(classify_flat(".decisions/log/01K.yml"), Some(Kind::Log));
        assert_eq!(classify_flat(".decisions/roles/steward.yml"), Some(Kind::Role));
        assert_eq!(classify_flat(".decisions/sig/01K.ssh.sig"), Some(Kind::Sig));
        assert_eq!(classify_flat(".decisions/sets/design.yml"), Some(Kind::Set));
        assert_eq!(classify_flat(".decisions/ns/a/log/01K.yml"), None, "no path matches both patterns");
        assert_eq!(classify_flat(".decisions/allowed_signers"), None);
        assert_eq!(flat_counterpart(".decisions/ns/a.ns/log/01K.yml").as_deref(), Some(".decisions/log/01K.yml"));
        assert_eq!(flat_counterpart(".decisions/ns/a.ns/sig/01K.ssh.sig").as_deref(), Some(".decisions/sig/01K.ssh.sig"));
        assert_eq!(flat_counterpart(".decisions/ns/a.ns/allowed_signers"), None);
    }

    #[test]
    fn flat_history_names_the_oldest_flat_commit_and_nothing_in_a_clean_history() {
        use crate::testkit::git;
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        git(root, &["init", "-q", "--initial-branch=main"]);
        git(root, &["config", "user.email", "fixture-human@example"]);
        git(root, &["config", "user.name", "Fixture"]);
        let file = |rel: &str| {
            let p = root.join(rel);
            std::fs::create_dir_all(p.parent().expect("dir")).expect("mkdir");
            std::fs::write(&p, rel).expect("write");
            git(root, &["add", "-A"]);
            git(root, &["commit", "-q", "-m", rel]);
        };
        file(".decisions/ns/a.ns/log/x.yml");
        assert_eq!(flat_history(root, &["HEAD"]), None, "a v1.9 history holds no flat path");
        file(".decisions/log/y.yml");
        let first = String::from_utf8_lossy(&git(root, &["rev-parse", "HEAD"]).stdout).trim().to_string();
        file(".decisions/log/z.yml");
        assert_eq!(flat_history(root, &["HEAD"]).as_deref(), Some(first.as_str()), "the oldest flat commit");
        assert!(moved_to_a_namespace(root, ".decisions/log/x.yml"));
        assert!(!moved_to_a_namespace(root, ".decisions/log/y.yml"));
    }

    #[test]
    fn the_relative_directories_spell_the_layout() {
        assert_eq!(relative_dir("a.ns", Kind::Log), ".decisions/ns/a.ns/log");
        assert_eq!(relative_dir("a.ns", Kind::Sig), ".decisions/ns/a.ns/sig");
    }
}
