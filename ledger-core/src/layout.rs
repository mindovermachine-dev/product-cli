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
    fn the_relative_directories_spell_the_layout() {
        assert_eq!(relative_dir("a.ns", Kind::Log), ".decisions/ns/a.ns/log");
        assert_eq!(relative_dir("a.ns", Kind::Sig), ".decisions/ns/a.ns/sig");
    }
}
