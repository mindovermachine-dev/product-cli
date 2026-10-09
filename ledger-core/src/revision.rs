//! Loading `.decisions/` as it stood at a git revision.
//!
//! The store-at-revision is the merge base's machinery (L3): semantic diff
//! compares two of these, and reconciliation loads the common ancestor
//! through one. Nothing is checked out — files are read straight from the
//! object store (`git show <rev>:<path>`), parsed by the same
//! [`crate::store`] rules the disk loader applies, so a store loaded at a
//! revision and the same tree checked out on disk read identically.

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
    for path in tree_paths(root, &commit)? {
        let Some((namespace, kind)) = crate::layout::classify(&path) else { continue };
        let Some(text) = show(root, &commit, &path)? else { continue };
        let label = format!("{rev}:{path}");
        let stem = file_stem(&path);
        match kind {
            Kind::Set => crate::store::take_set(&mut store, &namespace, &label, &stem, &text),
            Kind::Role => crate::store::take_role(&mut store, &namespace, &label, &stem, &text),
            Kind::Log => crate::store::take_log(&mut store, &namespace, PathBuf::from(&label), &label, &stem, &text),
            Kind::Sig | Kind::Signers => {}
        }
    }
    store.sets.sort_by(|a, b| a.id.cmp(&b.id));
    store.roles.sort_by(|a, b| a.id.cmp(&b.id));
    store.log.sort_by(|a, b| a.file.id.cmp(&b.file.id));
    Ok(store)
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
        let Some((namespace, Kind::Sig)) = crate::layout::classify(path) else { continue };
        let name = path.rsplit('/').next().unwrap_or_default().to_string();
        if store.sidecars.iter().any(|c| c.file == name && c.namespace == namespace) {
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
