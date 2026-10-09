//! Shared fixtures for this crate's unit tests.

use chrono::{DateTime, NaiveDate, Utc};

use crate::acceptance::{Acceptance, AcceptanceScope};
use crate::allocation::AllocationKind;
use crate::changeset::{ChangeSet, DecisionRecord};
use crate::hash::{version_hash, VersionHash};
use crate::id::{AcceptanceId, ChangeSetId, DecisionId};
use crate::identity::Identity;
use crate::set::{DecisionSet, Ground};
use crate::store::{LoggedChangeSet, Store};
use crate::tier::Tier;
use crate::version::VersionRaw;

pub const DEC_ULID: &str = "01K2C4YQJ3F8M0PT5W7NZ9RDXV";
pub const CS_ULID: &str = "01K2C4YQJ3F8M0PT5W7NZ9RDXW";
pub const ACC_ULID: &str = "01K2C4YQJ3F8M0PT5W7NZ9RDXX";

pub fn decision_id() -> DecisionId {
    format!("dec:hafeok.ledger/{DEC_ULID}").parse().expect("decision id")
}

pub fn identity(s: &str) -> Identity {
    s.parse().expect("identity")
}

pub fn date(s: &str) -> NaiveDate {
    s.parse().expect("date")
}

pub fn stamp(s: &str) -> DateTime<Utc> {
    s.parse().expect("timestamp")
}

pub fn zero_hash() -> VersionHash {
    format!("sha256:{}", "0".repeat(64)).parse().expect("hash")
}

/// A minimal, well-formed version. Its stored hash is a placeholder; call
/// [`sealed`] to stamp the real one.
pub fn version() -> VersionRaw {
    VersionRaw {
        decision: decision_id(),
        parent: None,
        merged_from: None,
        hash: zero_hash(),
        set: "ledger-design".into(),
        statement: "Monetary amounts use decimal, never double.".into(),
        allocation: Some(AllocationKind::Constraint),
        discharge: vec!["analyzer:DEC001-no-float-money".parse().expect("ref")],
        discharge_stage: None,
        actor: None,
        expectation: None,
        exposure: None,
        accepted_by: None,
        review_by: None,
        tolerance_floor_at_creation: Tier::T1,
        tolerance_override: None,
        based_on: vec!["prd:decision-ledger-prd#4.2.1".parse().expect("basis")],
        revisit_if: Vec::new(),
        supersedes: None,
        key: None,
        exported: false,
    }
}

/// Stamp a version with the hash of its own content, as a writer would.
pub fn sealed(mut raw: VersionRaw) -> VersionRaw {
    raw.hash = version_hash(&raw);
    raw
}

pub fn set() -> DecisionSet {
    DecisionSet {
        format: 1,
        id: "ledger-design".into(),
        title: "Decision Ledger — L0 settled design".into(),
        tolerance_floor: Tier::T1,
        ground: Ground::Characterised,
        owner: identity("emk@delegate.dk"),
        created_at: date("2026-08-10"),
        notes: None,
    }
}

pub fn acceptance(version: &VersionRaw) -> Acceptance {
    Acceptance {
        id: format!("acc:{ACC_ULID}").parse().expect("acceptance id"),
        decision: version.decision.clone(),
        version: version.hash.clone(),
        actor: identity("fixture-human@example"),
        at: stamp("2026-08-10T09:20:00Z"),
        scope: AcceptanceScope::Version,
        expires_at: Some(date("2027-08-10")),
        under: None,
        signature: String::new(),
    }
}

pub fn changeset(versions: Vec<VersionRaw>, acceptances: Vec<Acceptance>) -> ChangeSet {
    // One identity object per decision, however many of its versions the
    // change-set carries: a decision is introduced once (LP-5.11).
    let mut introduced = std::collections::BTreeSet::new();
    let decisions = versions
        .iter()
        .filter(|v| introduced.insert(v.decision.to_string()))
        .map(|v| DecisionRecord {
            id: v.decision.clone(),
            created_at: stamp("2026-08-10T09:14:22Z"),
            created_by: identity("fixture-human@example"),
        })
        .collect();
    let mut cs = ChangeSet::empty(
        1,
        format!("cs:{CS_ULID}").parse::<ChangeSetId>().expect("change-set id"),
        stamp("2026-08-10T09:14:22Z"),
        identity("fixture-human@example"),
        Some("fixture".into()),
    );
    cs.decisions = decisions;
    cs.versions = versions;
    cs.acceptances = acceptances;
    cs
}

/// An in-memory store holding one change-set plus the default set.
pub fn store(changeset: ChangeSet) -> Store {
    Store {
        root: std::path::PathBuf::from("/fixture"),
        dir: std::path::PathBuf::from("/fixture/.decisions"),
        sets: vec![set()],
        roles: Vec::new(),
        sidecars: Vec::new(),
        log: vec![LoggedChangeSet {
            path: std::path::PathBuf::from(format!("/fixture/.decisions/log/{CS_ULID}.yml")),
            file: changeset,
        }],
        schema_findings: Vec::new(),
    }
}

/// A legacy-shape (formats 1–5) revocation of the fixture acceptance.
pub fn legacy_revocation(at: &str, reason: &str) -> crate::acceptance::Revocation {
    crate::acceptance::Revocation {
        id: None,
        revokes: None,
        acceptance: Some(acceptance_id()),
        at: stamp(at),
        actor: None,
        by: Some(identity("fixture-human@example")),
        reason: reason.into(),
        under: None,
        hash: None,
    }
}

/// The acceptance-id string, for tests that need to name it.
pub fn acceptance_id() -> AcceptanceId {
    format!("acc:{ACC_ULID}").parse().expect("acceptance id")
}

/// The git identity variables a caller's environment may export. Each one
/// overrides the `user.name` / `user.email` a fixture configures for its own
/// commits, so every git the tests spawn clears them (#138): the author of a
/// fixture commit is the repository's, never the container's.
pub(crate) const GIT_IDENTITY_VARS: [&str; 4] =
    ["GIT_AUTHOR_NAME", "GIT_AUTHOR_EMAIL", "GIT_COMMITTER_NAME", "GIT_COMMITTER_EMAIL"];

/// A `git -C dir` command that takes its identity from the repository's own
/// configuration alone.
pub(crate) fn git_command(dir: &std::path::Path) -> std::process::Command {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("-C").arg(dir);
    for var in GIT_IDENTITY_VARS {
        cmd.env_remove(var);
    }
    cmd
}

/// Run git in `dir`, failing the test on a non-zero exit; returns its output.
pub(crate) fn git(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    let out = git_command(dir).args(args).output().expect("git");
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    out
}

mod tests {
    use super::*;

    #[test]
    fn every_git_the_tests_run_has_the_callers_identity_variables_cleared() {
        let cmd = git_command(std::path::Path::new("."));
        let cleared: Vec<_> = cmd.get_envs().filter(|(_, v)| v.is_none()).map(|(k, _)| k.to_os_string()).collect();
        for var in GIT_IDENTITY_VARS {
            assert!(cleared.iter().any(|k| k == var), "{var} is not cleared");
        }
    }
}
