//! Loading `.decisions/` as it stood at a git revision.
//!
//! The store-at-revision is the merge base's machinery (L3): semantic diff
//! compares two of these, and reconciliation loads the common ancestor
//! through one. Nothing is checked out — files are read straight from the
//! object store (`git show <rev>:<path>`), parsed by the same
//! [`crate::store`] rules the disk loader applies, so a store loaded at a
//! revision and the same tree checked out on disk read identically.
//!
//! A revision before v1.9 holds the flat layout. The reference
//! implementation carries the legacy capability (LP-3.35), so a flat
//! revision is read too — its change-sets homed by the namespace their
//! entities name, its sets by the versions that name them — for `diff`,
//! `merge`, and the base overlay of the re-layout's own pull request. A
//! verifier run without the capability refuses such a history before it
//! gets here.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::layout::Kind;
use crate::store::Store;
use crate::STORE_DIR;

/// Run git in `root`, returning stdout on success.
fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| format!("could not run git: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("git {} failed: {}", args.first().unwrap_or(&""), err.trim()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Resolve a revision to its full commit id, or say why not.
pub fn resolve(root: &Path, rev: &str) -> Result<String, String> {
    let spec = format!("{rev}^{{commit}}");
    git(root, &["rev-parse", "--verify", "--quiet", &spec])
        .map(|s| s.trim().to_string())
        .map_err(|_| format!("`{rev}` does not name a commit in this repository"))
}

/// Load the store as it stood at `rev`. The returned store's `root` is the
/// live repo root (so git-dependent checks still know where they are), but
/// every file label carries the revision so findings name what was read.
pub fn load_at(root: &Path, rev: &str) -> Result<Store, String> {
    let commit = resolve(root, rev)?;
    let mut store = Store {
        root: root.to_path_buf(),
        dir: root.join(STORE_DIR),
        ..Store::default()
    };
    let mut flat_sets: Vec<(String, String, String)> = Vec::new();
    for path in tree_paths(root, &commit)? {
        let label = format!("{rev}:{path}");
        let stem = file_stem(&path);
        let Some(text) = show(root, &commit, &path)? else { continue };
        if let Some((namespace, kind)) = crate::layout::classify(&path) {
            take_namespaced(&mut store, &namespace, kind, &label, &stem, &text);
        } else if let Some(kind) = crate::layout::classify_flat(&path) {
            take_flat(&mut store, kind, &label, &stem, &text, &mut flat_sets);
        }
    }
    home_flat_sets(&mut store, flat_sets);
    store.sets.sort_by(|a, b| a.id.cmp(&b.id));
    store.roles.sort_by(|a, b| a.id.cmp(&b.id));
    store.log.sort_by(|a, b| a.file.id.cmp(&b.file.id));
    Ok(store)
}

/// One file of the layout at a revision, by its directory.
fn take_namespaced(store: &mut Store, namespace: &str, kind: Kind, label: &str, stem: &str, text: &str) {
    match kind {
        Kind::Set => crate::store::take_set(store, namespace, label, stem, text),
        Kind::Role => crate::store::take_role(store, namespace, label, stem, text),
        Kind::Log => crate::store::take_log(store, namespace, PathBuf::from(label), label, stem, text),
        Kind::Sig | Kind::Signers => {}
    }
}

/// One file of the flat layout of revision v1.8 at a revision, homed by
/// content (LP-3.35): a change-set by the namespace its entities name, a
/// set later by the versions that name it, a role by nothing.
fn take_flat(store: &mut Store, kind: Kind, label: &str, stem: &str, text: &str, flat_sets: &mut Vec<(String, String, String)>) {
    match kind {
        Kind::Log => {
            let namespace = serde_yaml::from_str::<crate::changeset::ChangeSet>(text)
                .ok()
                .and_then(|cs| cs.namespace_hint().map(str::to_string))
                .unwrap_or_default();
            crate::store::take_log(store, &namespace, PathBuf::from(label), label, stem, text);
        }
        Kind::Set => flat_sets.push((label.to_string(), stem.to_string(), text.to_string())),
        Kind::Role => crate::store::take_role(store, "", label, stem, text),
        Kind::Sig | Kind::Signers => {}
    }
}

/// A flat set belongs to the namespace whose versions name it.
fn home_flat_sets(store: &mut Store, flat_sets: Vec<(String, String, String)>) {
    for (label, stem, text) in flat_sets {
        let namespace = store
            .log
            .iter()
            .find(|l| l.file.versions.iter().any(|v| v.set == stem))
            .map(|l| l.namespace.clone())
            .unwrap_or_default();
        crate::store::take_set(store, &namespace, &label, &stem, &text);
    }
}

/// Add to `store` the log files `base` carries that the working tree does
/// not (D6): the log is append-only, so the union is what the merge into
/// `base` would hold, and `verify --base` then judges the merge's store.
/// Returns how many change-sets came from the base.
pub fn overlay_base(store: &mut Store, base: &str) -> Result<usize, String> {
    let at_base = load_at(&store.root, base)?;
    let have: std::collections::BTreeSet<String> = store.log.iter().map(|l| l.file.id.to_string()).collect();
    let mut added = 0;
    for logged in at_base.log.into_iter().filter(|l| !have.contains(&l.file.id.to_string())) {
        let path = crate::layout::log_dir(&store.root, &logged.namespace).join(logged.file.file_name());
        store.log.push(crate::store::LoggedChangeSet { namespace: logged.namespace, path, file: logged.file });
        added += 1;
    }
    store.log.sort_by(|a, b| a.file.id.cmp(&b.file.id));
    let commit = resolve(&store.root, base)?;
    let listing = git(&store.root, &["ls-tree", "-r", "--name-only", &commit, "--", STORE_DIR]).unwrap_or_default();
    for path in listing.lines().map(str::trim).filter(|p| !p.is_empty()) {
        let name = path.rsplit('/').next().unwrap_or_default().to_string();
        let namespace = match (crate::layout::classify(path), crate::layout::classify_flat(path)) {
            (Some((namespace, Kind::Sig)), _) => namespace,
            // A flat sidecar (LP-3.35) is homed by the entity it signs.
            (None, Some(Kind::Sig)) => sidecar_home(store, &name).unwrap_or_default(),
            _ => continue,
        };
        if store.sidecars.iter().any(|c| c.file == name) {
            continue;
        }
        let Ok((ulid, scheme)) = crate::signing::Sidecar::parse_name(&name) else { continue };
        let Ok(out) = Command::new("git").arg("-C").arg(&store.root).args(["show", &format!("{commit}:{path}")]).output() else { continue };
        if out.status.success() {
            store.sidecars.push(crate::signing::Sidecar { namespace, ulid, scheme, file: name, bytes: out.stdout });
        }
    }
    Ok(added)
}

/// The namespace of the signable entity a sidecar `<ulid>.<scheme>.sig`
/// names, among the store's change-sets.
fn sidecar_home(store: &Store, name: &str) -> Option<String> {
    let ulid = name.split('.').next()?;
    store
        .log
        .iter()
        .find(|l| {
            let cs = &l.file;
            cs.acceptances.iter().any(|a| a.id.ulid() == ulid)
                || cs.revocations.iter().any(|r| r.id.as_ref().is_some_and(|i| i.ulid() == ulid))
                || cs.key_bindings.iter().any(|b| b.id.ulid() == ulid)
                || cs.policies.iter().any(|p| p.id.ulid() == ulid)
        })
        .map(|l| l.namespace.clone())
}

/// Every `.yml`/`.yaml` path under `.decisions/` in the revision's tree.
/// A revision with no store at all is an empty store, not an error — a
/// branch that predates `ledger init` diffs as "everything was added".
fn tree_paths(root: &Path, commit: &str) -> Result<Vec<String>, String> {
    let listing = match git(root, &["ls-tree", "-r", "--name-only", commit, "--", STORE_DIR]) {
        Ok(text) => text,
        Err(_) => return Ok(Vec::new()),
    };
    let mut paths: Vec<String> = listing
        .lines()
        .map(str::trim)
        .filter(|l| l.ends_with(".yml") || l.ends_with(".yaml"))
        .map(str::to_string)
        .collect();
    paths.sort();
    Ok(paths)
}

/// One blob's content at the revision. `None` when git cannot produce it
/// (e.g. the path is a submodule entry) — skipped, never fatal.
fn show(root: &Path, commit: &str, path: &str) -> Result<Option<String>, String> {
    match git(root, &["show", &format!("{commit}:{path}")]) {
        Ok(text) => Ok(Some(text)),
        Err(_) => Ok(None),
    }
}

fn file_stem(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_revision_is_a_named_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let err = load_at(dir.path(), "no-such-rev").expect_err("no repo");
        assert!(err.contains("does not name a commit"), "{err}");
    }
}
