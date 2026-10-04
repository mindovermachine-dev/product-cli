//! Landing — when an entity entered the record (D6, ruled 2026-10-02).
//!
//! An entity's landing commit is the first commit on the first-parent
//! history of the verified commit whose version of its file contains it.
//! For a file never touched after it was added, that is the file's adding
//! commit. For a file git reports modified or deleted since
//! ([`touched_after_landing`]), that file's own first-parent history is
//! walked and each entity lands at the first version that holds it
//! ([`crate::landed`]); an entity no commit holds — appended in the working
//! tree — lands at the tip. Not `git log -S`: a pickaxe matches an id
//! wherever the string appears.
//!
//! **Order.** A landing is a first-parent index. Entities on an unmerged
//! branch land at the tip, after everything on the base, all at once — the
//! merge commit is where they land on the default branch. So that a local
//! `verify` on a branch gives the result the merge would, a [`Landing`] can
//! be computed against a base ref (`ledger verify --base <ref>`; by default
//! `origin/HEAD` when the clone has it): a file on the base's first-parent
//! line keeps the base's index, and everything else lands at the tip. On a
//! merge ref the merge commit's own first-parent line is the base line, so
//! the two agree.
//!
//! **Before.** An act is before a terminating entry when it landed no
//! later and its `at` is earlier ([`Position::before`]); in the same commit,
//! `at` decides. Without git every entity is at the tip and `at` alone
//! decides — the export-only verifier's reading, stated as its limit.
//!
//! Assumption, stated rather than proved: the default branch's history is
//! not rewritten. A ruleset holds that; `verify` cannot.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use chrono::{DateTime, Utc};

/// The directories whose files land: the log, role files, signatures.
const TRACKED: &[&str] = &[".decisions/log", ".decisions/roles", ".decisions/sig"];

/// Where each file of the store landed.
#[derive(Debug, Clone, Default)]
pub struct Landing {
    /// Whether git answered. When it did not, everything is at the tip.
    pub available: bool,
    /// Repo-relative path → (first-parent index, landing commit).
    landed: BTreeMap<String, (usize, String)>,
    /// For touched files only: (path, entity key) → first-parent index of
    /// the first version that holds the entity.
    entities: BTreeMap<(String, String), usize>,
    /// The files whose entities land one by one.
    touched: std::collections::BTreeSet<String>,
    /// The index of the tip: one past the last landed commit.
    tip: usize,
}

/// An entity's place in the record: landing index, then its own `at`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub index: usize,
    pub at: DateTime<Utc>,
}

impl Position {
    /// D6's "before": landed no later, and dated strictly earlier. Landed
    /// earlier but dated later is not before; landed later never is.
    pub fn before(&self, other: &Position) -> bool {
        self.index <= other.index && self.at < other.at
    }

    /// D6's "not after", for enabling entries (a key added, a grant
    /// accepted): landed no later and dated no later.
    pub fn not_after(&self, other: &Position) -> bool {
        self.index <= other.index && self.at <= other.at
    }
}

impl Landing {
    /// Nothing landed: every file is at the tip (no git, or not asked).
    pub fn unknown() -> Self {
        Self::default()
    }

    /// Resolve every tracked file's landing on `HEAD`, against `base` when
    /// given (see the module doc). A `base` that does not resolve is an
    /// error, never silently ignored: the result would differ from CI's.
    pub fn compute(root: &Path, base: Option<&str>) -> Result<Self, String> {
        if !git_ok(root, &["rev-parse", "--verify", "-q", "HEAD"]) {
            return Ok(Self::unknown());
        }
        let head = first_parent_adds(root, "HEAD")?;
        let touched = touched_after_landing(root);
        let Some(base) = base else {
            let tip = head.commits;
            let entities = entity_landings(root, "HEAD", &touched)?;
            return Ok(Self { available: true, landed: head.adds, entities, touched, tip });
        };
        if !git_ok(root, &["rev-parse", "--verify", "-q", &format!("{base}^{{commit}}")]) {
            return Err(format!("`{base}` does not name a commit — landing is computed against the base"));
        }
        let on_base = first_parent_adds(root, base)?;
        let tip = on_base.commits;
        // The merge's reading: the base's files keep the base's order (the
        // caller overlays the ones a branch checkout lacks, see
        // `revision::overlay_base`), and the branch's own land at the tip.
        let mut landed = on_base.adds;
        for (path, (_, commit)) in head.adds {
            landed.entry(path).or_insert((tip, commit));
        }
        // Entities of a touched file land where the base's line first holds
        // them; one only the branch holds lands at the tip, as the merge would.
        let entities = entity_landings(root, base, &touched)?;
        Ok(Self { available: true, landed, entities, touched, tip })
    }

    /// The base the clone points its default branch at, if it has one.
    pub fn default_base(root: &Path) -> Option<String> {
        git_ok(root, &["rev-parse", "--verify", "-q", "refs/remotes/origin/HEAD"]).then(|| "origin/HEAD".to_string())
    }

    /// The landing index of a repo-relative path; uncommitted is the tip.
    pub fn index(&self, path: &str) -> usize {
        self.landed.get(path).map_or(self.tip, |(i, _)| *i)
    }

    /// A position for the entity keyed `key` ([`crate::landed::key`]) filed
    /// in `path` at `at`.
    pub fn position(&self, path: &str, key: &str, at: DateTime<Utc>) -> Position {
        Position { index: self.entity_index(path, key), at }
    }

    /// The landing index of one entity: its file's, unless the file was
    /// touched after it landed, then the first version holding the entity.
    pub fn entity_index(&self, path: &str, key: &str) -> usize {
        if !self.touched.contains(path) {
            return self.index(path);
        }
        self.entities.get(&(path.to_string(), key.to_string())).copied().unwrap_or(self.tip)
    }
}

/// The tracked paths git says changed or vanished after they were added:
/// modified or deleted on the first-parent line, or in the working tree.
/// The write-once checks read only these, not every landed file.
pub fn touched_after_landing(root: &Path) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    let history = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "--first-parent", "--diff-merges=first-parent", "--relative", "--name-only"])
        .args(["--diff-filter=DM", "--format=", "HEAD", "--"])
        .args(TRACKED)
        .output();
    let worktree = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["diff", "--relative", "--name-only", "--diff-filter=DM", "HEAD", "--"])
        .args(TRACKED)
        .output();
    for out_put in [history, worktree].into_iter().flatten().filter(|o| o.status.success()) {
        out.extend(String::from_utf8_lossy(&out_put.stdout).lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string));
    }
    out
}

/// A path's text at a commit, for the write-once checks.
pub fn content_at(root: &Path, commit: &str, path: &str) -> Option<String> {
    let out = Command::new("git").arg("-C").arg(root).args(["show", &format!("{commit}:{path}")]).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// A repo-relative path for a file under `root`.
pub fn relative(root: &Path, file: &Path) -> String {
    file.strip_prefix(root).unwrap_or(file).to_string_lossy().replace('\\', "/")
}

struct Adds {
    adds: BTreeMap<String, (usize, String)>,
    commits: usize,
}

/// Walk `rev`'s first-parent line oldest first, recording the first commit
/// that added each tracked path. Merges are diffed against their first
/// parent (`--first-parent` implies it), so a branch's files land at the
/// merge commit.
fn first_parent_adds(root: &Path, rev: &str) -> Result<Adds, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "--first-parent", "--diff-merges=first-parent", "--reverse", "--relative"])
        .args(["--name-only", "--diff-filter=A", "--format=%x01%H", rev, "--"])
        .args(TRACKED)
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if !out.status.success() {
        return Err(format!("git log: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    let order = first_parent_order(root, rev)?;
    let commits = order.len();
    let mut adds = BTreeMap::new();
    let mut current: Option<(usize, String)> = None;
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        if let Some(sha) = line.strip_prefix('\u{1}') {
            current = order.get(sha.trim()).map(|i| (*i, sha.trim().to_string()));
        } else if let (Some(at), false) = (&current, line.trim().is_empty()) {
            adds.entry(line.trim().to_string()).or_insert_with(|| at.clone());
        }
    }
    Ok(Adds { adds, commits })
}

/// For each touched log file, where each entity first appears on `rev`'s
/// first-parent line.
fn entity_landings(
    root: &Path,
    rev: &str,
    touched: &std::collections::BTreeSet<String>,
) -> Result<BTreeMap<(String, String), usize>, String> {
    let mut out = BTreeMap::new();
    let logs: Vec<&String> = touched.iter().filter(|p| p.starts_with(".decisions/log/")).collect();
    if logs.is_empty() {
        return Ok(out);
    }
    let order = first_parent_order(root, rev)?;
    for path in logs {
        for commit in file_versions(root, rev, path) {
            let Some(index) = order.get(&commit).copied() else { continue };
            let Some(text) = content_at(root, &commit, path) else { continue };
            for key in crate::landed::entities(&text).unwrap_or_default().into_keys() {
                out.entry((path.clone(), key)).or_insert(index);
            }
        }
    }
    Ok(out)
}

/// The first-parent commits of `rev` that changed `path`, oldest first.
pub fn file_versions(root: &Path, rev: &str, path: &str) -> Vec<String> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "--first-parent", "--diff-merges=first-parent", "--reverse", "--format=%H", rev, "--", path])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
        .unwrap_or_default()
}

/// Every first-parent commit of `rev`, oldest = 0.
fn first_parent_order(root: &Path, rev: &str) -> Result<BTreeMap<String, usize>, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-list", "--first-parent", "--reverse", rev])
        .output()
        .map_err(|e| format!("git: {e}"))?;
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .enumerate()
        .map(|(i, sha)| (sha.trim().to_string(), i))
        .collect())
}

fn git_ok(root: &Path, args: &[&str]) -> bool {
    Command::new("git").arg("-C").arg(root).args(args).output().is_ok_and(|o| o.status.success())
}

#[path = "landing_tests.rs"]
#[cfg(test)]
mod tests;
