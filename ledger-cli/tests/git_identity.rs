//! The suites' git runs with the caller's identity variables cleared (#138).
//!
//! `GIT_AUTHOR_*` and `GIT_COMMITTER_*` in the environment override the
//! `user.name` / `user.email` a fixture configures, so a container that
//! exports them made every author-dependent test fail. The shared helper
//! clears them on every git it spawns and on every invocation of the binary;
//! these tests hold that in place.

mod common;

use std::ffi::OsString;

fn cleared(envs: std::process::CommandEnvs<'_>) -> Vec<OsString> {
    envs.filter(|(_, v)| v.is_none()).map(|(k, _)| k.to_os_string()).collect()
}

#[test]
fn every_git_the_suites_run_has_the_callers_identity_variables_cleared() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cmd = common::git_command(dir.path());
    let cleared = cleared(cmd.get_envs());
    for var in common::GIT_IDENTITY_VARS {
        assert!(cleared.iter().any(|k| k == var), "git: {var} is not cleared");
    }
}

#[test]
fn every_invocation_of_the_binary_has_the_callers_identity_variables_cleared() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cmd = common::ledger_command(dir.path());
    let cleared = cleared(cmd.get_envs());
    for var in common::GIT_IDENTITY_VARS {
        assert!(cleared.iter().any(|k| k == var), "ledger: {var} is not cleared");
    }
}

#[test]
fn a_fixture_commit_is_authored_by_the_configured_identity_whatever_the_environment_exports() {
    // The intruding identity is set on the parent shell, as a container
    // exports it; the helper's own git runs underneath and must not see it.
    let repo = common::Repo::human();
    std::fs::write(repo.path().join("note"), "x").expect("write");
    repo.git(&["add", "note"]);
    repo.git(&["commit", "-q", "-m", "note"]);
    let out = common::git(repo.path(), &["log", "-1", "--format=%an <%ae> %cn <%ce>"]);
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "Fixture <fixture-human@example> Fixture <fixture-human@example>");
}
