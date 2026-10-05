//! The inbox fixture: three repositories, each a bare remote with a
//! holder's clone (its own genesis) and an agent's, the agents proposing
//! twenty decisions on twelve branches.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::common::{self, Repo};

pub const OWNER: &str = "owner@customer.example";
pub const AGENT: &str = "claude-agent@anthropic.invalid";

pub struct Fixture {
    pub root: tempfile::TempDir,
    pub holders: Vec<Repo>,
    pub config: PathBuf,
}

pub fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git").arg("-C").arg(dir).args(args).output().expect("git");
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).into_owned()
}

pub fn ledger_ok(dir: &Path, args: &[&str]) -> String {
    let out = common::invoke(dir, args);
    assert_eq!(out.status.code(), Some(0), "ledger {args:?}: {}", common::both(&out));
    common::both(&out)
}

/// One repository: a governed holder clone pushed to a bare remote.
pub fn repository(root: &Path, i: usize) -> Repo {
    let ns = format!("repo{i}.ledger");
    let bare = root.join(format!("remote{i}.git"));
    git(root, &["init", "-q", "--bare", "--initial-branch=main", &bare.display().to_string()]);
    let holder = Repo::with_identity(OWNER);
    holder.git(&["remote", "add", "origin", &bare.display().to_string()]);
    holder.declare();
    holder.ok(&["init", "--namespace", &ns, "--external-ref", &format!("contract {i}"), "--without-key"]);
    holder.bind_own_key(&ns, "owner");
    let grant = common::hand::word(&holder.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &format!("ns:{ns}")]), "grant:");
    holder.ok(&["grant", "accept", &grant]);
    common::hand::commit(&holder, "governed");
    holder.git(&["push", "-q", "-u", "origin", "main"]);
    holder
}

/// An agent proposes `decisions` decisions on each of `branches` branches.
pub fn propose(root: &Path, i: usize, per_branch: &[usize]) {
    let ns = format!("repo{i}.ledger");
    let agent = root.join(format!("agent{i}"));
    git(root, &["clone", "-q", &root.join(format!("remote{i}.git")).display().to_string(), &agent.display().to_string()]);
    git(&agent, &["config", "user.email", AGENT]);
    git(&agent, &["config", "user.name", "Agent"]);
    for (b, n) in per_branch.iter().enumerate() {
        git(&agent, &["checkout", "-q", "-b", &format!("agent/{b}"), "origin/main"]);
        for d in 0..*n {
            let statement = format!("Repository {i}, branch {b}, decision {d}.");
            ledger_ok(&agent, &[
                "add", "--set", "ledger-design", "--namespace", &ns, "--statement", &statement,
                "--store", "constraint", "--discharge", "analyzer:DEC001",
                "--based-on", "rule:DD042", "--based-on", "symbol:Billing.Invoice.Total",
            ]);
        }
        ledger_ok(&agent, &["export", "--format", "ntriples"]);
        git(&agent, &["add", "-A"]);
        git(&agent, &["commit", "-q", "-m", "proposed"]);
        git(&agent, &["push", "-q", "origin", &format!("agent/{b}")]);
    }
}

/// Three repositories; twelve branches; twenty proposed decisions.
pub fn fixture() -> Fixture {
    fixture_with(|_| {})
}

/// The fixture, with `before` run on the first holder's clone (pushed to
/// main) before the agents cut their branches.
pub fn fixture_with(before: impl Fn(&Repo)) -> Fixture {
    let root = tempfile::tempdir().expect("tempdir");
    let mut holders = Vec::new();
    let shape: [&[usize]; 3] = [&[2, 2, 2, 1], &[2, 2, 1, 1], &[2, 2, 2, 1]];
    for (i, per_branch) in shape.iter().enumerate() {
        holders.push(repository(root.path(), i));
        if i == 0 {
            before(&holders[0]);
        }
        propose(root.path(), i, per_branch);
    }
    for h in &holders {
        h.git(&["fetch", "-q", "origin"]);
    }
    let config = root.path().join("inbox.yml");
    let mut text = String::from("repositories:\n");
    for (i, h) in holders.iter().enumerate() {
        text.push_str(&format!("  - name: repo{i}\n    path: {}\n    default_branch: main\n", h.path().display()));
    }
    std::fs::write(&config, text).expect("config");
    Fixture { root, holders, config }
}

pub fn inbox(f: &Fixture, args: &[&str]) -> std::process::Output {
    let mut full = vec!["inbox"];
    full.extend_from_slice(args);
    full.extend_from_slice(&["--config", f.config.to_str().expect("utf-8")]);
    common::invoke(f.holders[0].path(), &full)
}

pub fn manifest(out: &str) -> String {
    out.lines().find_map(|l| l.strip_prefix("manifest")).map(|m| m.trim().to_string()).unwrap_or_else(|| panic!("no manifest: {out}"))
}


/// A hook, executable, in a holder clone's git directory.
pub fn hook(h: &Repo, name: &str, body: &str) {
    let path = h.path().join(".git/hooks").join(name);
    std::fs::write(&path, format!("#!/bin/sh\nunset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE\n{body}")).expect("hook");
    std::fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("chmod");
}

