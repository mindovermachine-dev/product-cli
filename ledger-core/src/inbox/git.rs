//! The git reads the inbox makes in each clone — no GitHub API
//! (registry PRD §10.3): branches, committed exports, pull-request labels,
//! push permission, and the worktree a branch is signed in.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::config::Repository;

/// Run git in `dir`; stdout on success, the first stderr line otherwise.
pub fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git").arg("-C").arg(dir).args(args).output().map_err(|e| format!("git: {e}"))?;
    if out.status.success() {
        return Ok(String::from_utf8_lossy(&out.stdout).into_owned());
    }
    let err = String::from_utf8_lossy(&out.stderr);
    Err(err.lines().find(|l| !l.trim().is_empty()).unwrap_or("git failed").trim().to_string())
}

/// The default branch's name: configured, else the remote's HEAD, else `main`.
pub fn default_branch(repo: &Repository) -> String {
    if let Some(b) = &repo.default_branch {
        return b.clone();
    }
    let head = format!("refs/remotes/{}/HEAD", repo.remote);
    git(&repo.path, &["symbolic-ref", "-q", "--short", &head])
        .ok()
        .and_then(|s| s.trim().strip_prefix(&format!("{}/", repo.remote)).map(str::to_string))
        .unwrap_or_else(|| "main".to_string())
}

/// Every remote-tracking branch of the clone's remote: name → commit.
pub fn remote_branches(repo: &Repository) -> Result<BTreeMap<String, String>, String> {
    let prefix = format!("refs/remotes/{}/", repo.remote);
    let text = git(&repo.path, &["for-each-ref", "--format=%(refname) %(objectname)", &prefix])?;
    Ok(text
        .lines()
        .filter_map(|l| l.split_once(' '))
        .filter_map(|(r, sha)| r.strip_prefix(&prefix).map(|n| (n.to_string(), sha.to_string())))
        .filter(|(n, _)| n != "HEAD")
        .collect())
}

/// The commit a revision names.
pub fn commit_of(dir: &Path, rev: &str) -> Result<String, String> {
    git(dir, &["rev-parse", "--verify", "-q", &format!("{rev}^{{commit}}")]).map(|s| s.trim().to_string())
}

/// Every committed export (`docs/decisions/*.nt`) at a revision, in path
/// order, concatenated: one store's N-Triples.
pub fn exports_at(dir: &Path, rev: &str) -> Result<String, String> {
    let names = git(dir, &["ls-tree", "-r", "--name-only", rev, "--", "docs/decisions/"])?;
    let mut out = String::new();
    for path in names.lines().map(str::trim).filter(|p| p.ends_with(".nt")) {
        out.push_str(&git(dir, &["show", &format!("{rev}:{path}")])?);
    }
    Ok(out)
}

/// Pull-request numbers by head commit, from `refs/pull/<n>/head` refs the
/// clone has fetched. Absent refs mean no labels, never a guess.
pub fn pull_requests(dir: &Path) -> BTreeMap<String, u32> {
    let Ok(text) = git(dir, &["for-each-ref", "--format=%(refname) %(objectname)", "refs/pull/"]) else {
        return BTreeMap::new();
    };
    text.lines()
        .filter_map(|l| l.split_once(' '))
        .filter_map(|(r, sha)| {
            let n = r.strip_prefix("refs/pull/")?.strip_suffix("/head")?.parse().ok()?;
            Some((sha.to_string(), n))
        })
        .collect()
}

/// Whether the clone's remote can be reached for a push to `branch`: `None`
/// when it can, the reason when it cannot. A forced dry run: it resolves the
/// push URL, connects and reads the ref advertisement, and sends nothing,
/// so nothing the receiving side checks on an update — write permission,
/// branch protection, hooks — is tested. `--force` keeps a merely stale
/// branch from reading as unreachable; `--no-verify` keeps the clone's own
/// `pre-push` hooks, which a dry run would otherwise run, out of a probe.
pub fn push_unreachable(repo: &Repository, commit: &str, branch: &str) -> Option<String> {
    let dst = format!("{commit}:refs/heads/{branch}");
    git(&repo.path, &["push", "--dry-run", "--force", "--no-verify", "--porcelain", &repo.remote, &dst]).err()
}

/// Where the inbox keeps its worktrees: `<git-common-dir>/ledger-inbox/`.
fn worktrees_dir(repo: &Repository) -> Result<PathBuf, String> {
    let common = git(&repo.path, &["rev-parse", "--git-common-dir"])?;
    Ok(repo.path.join(common.trim()).join("ledger-inbox"))
}

/// Remove every worktree the inbox made in this clone. A sitting discards
/// its own as it goes; this clears what a killed one left, signed but
/// unpushed commits included — reachable from no ref, they go at the next
/// `git gc`.
pub fn sweep(repo: &Repository) -> Result<(), String> {
    let dir = worktrees_dir(repo)?;
    let listing = git(&repo.path, &["worktree", "list", "--porcelain"])?;
    for path in listing.lines().filter_map(|l| l.strip_prefix("worktree ")).map(PathBuf::from) {
        if path.starts_with(&dir) || std::fs::canonicalize(&path).ok().zip(std::fs::canonicalize(&dir).ok()).is_some_and(|(p, d)| p.starts_with(d)) {
            let _ = git(&repo.path, &["worktree", "remove", "--force", &path.display().to_string()]);
        }
    }
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    git(&repo.path, &["worktree", "prune"]).map(|_| ())
}

/// A worktree of `rev` (detached) under the clone's git directory, made
/// fresh: the inbox signs and verifies a branch there without touching the
/// holder's own checkout.
pub fn worktree(repo: &Repository, branch: &str, rev: &str) -> Result<PathBuf, String> {
    let name: String = branch.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect();
    let path = worktrees_dir(repo)?.join(name);
    let at = path.display().to_string();
    if path.exists() {
        let _ = git(&repo.path, &["worktree", "remove", "--force", &at]);
        let _ = std::fs::remove_dir_all(&path);
    }
    let _ = git(&repo.path, &["worktree", "prune"]);
    git(&repo.path, &["worktree", "add", "--detach", "--force", &at, rev])?;
    Ok(path)
}

/// Remove a worktree the inbox made. A commit made there and never pushed
/// is then reachable from no ref, and goes at the clone's next `git gc`.
pub fn discard_worktree(repo: &Repository, path: &Path) {
    let _ = git(&repo.path, &["worktree", "remove", "--force", &path.display().to_string()]);
    let _ = std::fs::remove_dir_all(path);
    let _ = git(&repo.path, &["worktree", "prune"]);
}

/// Fetch the clone's remote, so a stale item is re-read before signing.
pub fn fetch(repo: &Repository) -> Result<(), String> {
    git(&repo.path, &["fetch", "--quiet", &repo.remote]).map(|_| ())
}
