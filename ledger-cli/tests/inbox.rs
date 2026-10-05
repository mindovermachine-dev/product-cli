//! `ledger inbox` (#79): three repositories, twelve agent branches with
//! proposed decisions, one holder, one sitting.
//!
//! Each repository is a bare remote with two clones: the holder's, which
//! opts the namespace in with its own genesis (`init --namespace`), binds a
//! key and grants itself the accept role; and an agent's, which proposes
//! decisions on branches and pushes them with their exports.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::Repo;

const OWNER: &str = "owner@customer.example";
const AGENT: &str = "claude-agent@anthropic.invalid";

struct Fixture {
    root: tempfile::TempDir,
    holders: Vec<Repo>,
    config: PathBuf,
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git").arg("-C").arg(dir).args(args).output().expect("git");
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn ledger_ok(dir: &Path, args: &[&str]) -> String {
    let out = common::invoke(dir, args);
    assert_eq!(out.status.code(), Some(0), "ledger {args:?}: {}", common::both(&out));
    common::both(&out)
}

/// One repository: a governed holder clone pushed to a bare remote.
fn repository(root: &Path, i: usize) -> Repo {
    let ns = format!("repo{i}.ledger");
    let bare = root.join(format!("remote{i}.git"));
    git(root, &["init", "-q", "--bare", "--initial-branch=main", &bare.display().to_string()]);
    let holder = Repo::with_identity(OWNER);
    holder.git(&["remote", "add", "origin", &bare.display().to_string()]);
    holder.declare();
    holder.ok(&["init", "--namespace", &ns, "--external-ref", &format!("contract {i}")]);
    holder.bind_own_key(&ns, "owner");
    let grant = common::hand::word(&holder.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &format!("ns:{ns}")]), "grant:");
    holder.ok(&["grant", "accept", &grant]);
    common::hand::commit(&holder, "governed");
    holder.git(&["push", "-q", "-u", "origin", "main"]);
    holder
}

/// An agent proposes `decisions` decisions on each of `branches` branches.
fn propose(root: &Path, i: usize, per_branch: &[usize]) {
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
fn fixture() -> Fixture {
    fixture_with(|_| {})
}

/// The fixture, with `before` run on the first holder's clone (pushed to
/// main) before the agents cut their branches.
fn fixture_with(before: impl Fn(&Repo)) -> Fixture {
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

fn inbox(f: &Fixture, args: &[&str]) -> std::process::Output {
    let mut full = vec!["inbox"];
    full.extend_from_slice(args);
    full.extend_from_slice(&["--config", f.config.to_str().expect("utf-8")]);
    common::invoke(f.holders[0].path(), &full)
}

fn manifest(out: &str) -> String {
    out.lines().find_map(|l| l.strip_prefix("manifest")).map(|m| m.trim().to_string()).unwrap_or_else(|| panic!("no manifest: {out}"))
}

#[test]
fn three_repositories_twelve_branches_one_sitting_every_branch_green() {
    let f = fixture();
    let list = inbox(&f, &["list", "--json"]);
    let text = common::both(&list);
    assert_eq!(list.status.code(), Some(0), "{text}");
    let json: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&list.stdout)).expect("json");
    let items = json["items"].as_array().expect("items");
    assert_eq!(items.len(), 20, "every proposed decision in one list: {text}");
    assert!(items.iter().all(|i| i["identity_class"] == "agent" && i["grant"].as_str().is_some_and(|g| g.starts_with("grant:"))), "{text}");
    assert!(items.iter().all(|i| i["rule"] == "rule:DD042" && i["citing"][0] == "symbol:Billing.Invoice.Total"), "{text}");
    let branches: std::collections::BTreeSet<(String, String)> =
        items.iter().map(|i| (i["repository"].to_string(), i["branch"].to_string())).collect();
    assert_eq!(branches.len(), 12, "twelve branches: {text}");

    let dry = common::both(&inbox(&f, &["accept", "--all"]));
    let m = manifest(&dry);
    let started = std::time::Instant::now();
    let signed = inbox(&f, &["accept", "--all", "--confirm", &m]);
    let elapsed = started.elapsed();
    let out = common::both(&signed);
    assert_eq!(signed.status.code(), Some(0), "{out}");
    assert_eq!(out.matches("green").count(), 12, "every branch's verify --base green: {out}");
    assert!(!out.contains("NOT green"), "{out}");
    println!("TIMING: 20 acceptances, 12 branches, 1 confirmation: {:.2}s", elapsed.as_secs_f64());

    // Independently: fetch, and verify every branch against its base.
    let mut sidecars = 0;
    for (i, h) in f.holders.iter().enumerate() {
        h.git(&["fetch", "-q", "origin"]);
        let on_main = git(h.path(), &["ls-tree", "--name-only", "origin/main", ".decisions/sig/"]).lines().count();
        let branches = git(h.path(), &["for-each-ref", "--format=%(refname:strip=3)", "refs/remotes/origin/agent/"]);
        for b in branches.lines() {
            let wt = f.root.path().join(format!("check-{i}-{}", b.replace('/', "-")));
            h.git(&["worktree", "add", "-q", "--detach", &wt.display().to_string(), &format!("origin/{b}")]);
            let v = common::invoke(&wt, &["verify", "--base", "origin/main", "--export"]);
            assert_eq!(v.status.code(), Some(0), "repo{i} {b}: {}", common::both(&v));
            sidecars += std::fs::read_dir(wt.join(".decisions/sig")).map(|d| d.count()).unwrap_or(0) - on_main;
        }
    }
    assert_eq!(sidecars, 20, "one signature per acceptance, beyond the sidecars main already holds");
}

#[test]
fn a_namespace_with_no_policy_is_named_unchecked_not_omitted() {
    let f = fixture();
    let h = &f.holders[0];
    // A second namespace, never opted in, proposed on main.
    let out = h.ok(&["add", "--set", "ledger-design", "--namespace", "loose.ledger", "--statement", "Unchecked."]);
    assert!(out.contains("dec:loose.ledger/"), "{out}");
    h.ok(&["export", "--format", "ntriples"]);
    common::hand::commit(h, "a loose decision");
    h.git(&["push", "-q", "origin", "main"]);
    h.git(&["fetch", "-q", "origin"]);
    let list = common::both(&inbox(&f, &["list"]));
    assert!(list.contains("unchecked: `loose.ledger` in repo0 @ main has no policy"), "{list}");
    assert!(!list.contains("Unchecked."), "its proposed decisions are in nobody's list: {list}");
}

#[test]
fn the_inbox_refuses_an_agent_a_pipe_and_another_principals_clone() {
    let f = fixture();
    // Piped stdin: the confirming run needs a terminal.
    let dry = common::both(&inbox(&f, &["accept", "--all"]));
    let m = manifest(&dry);
    let mut args = vec!["inbox", "accept", "--all", "--confirm", &m, "--config"];
    let cfg = f.config.display().to_string();
    args.push(&cfg);
    let piped = common::piped(f.holders[0].path(), &args);
    assert_ne!(piped.status.code(), Some(0), "{}", common::both(&piped));
    assert!(common::both(&piped).contains("terminal"), "{}", common::both(&piped));
    // A clone configured as another person: a sitting is one principal's.
    f.holders[2].git(&["config", "user.email", "someone-else@customer.example"]);
    let mixed = common::both(&inbox(&f, &["list"]));
    assert!(mixed.contains("a sitting is one principal's"), "{mixed}");
    // An agent identity on every clone: refused before anything is written.
    for h in &f.holders {
        h.git(&["config", "user.email", AGENT]);
    }
    let agent = inbox(&f, &["accept", "--all"]);
    assert_eq!(agent.status.code(), Some(1), "{}", common::both(&agent));
    assert!(common::both(&agent).contains("is not a principal who can accept"), "{}", common::both(&agent));
}

#[test]
fn a_branch_whose_remote_cannot_be_reached_for_a_push_is_listed_so() {
    let f = fixture();
    f.holders[1].git(&["config", "remote.origin.pushurl", "/nonexistent/remote.git"]);
    let list = common::both(&inbox(&f, &["list"]));
    assert!(list.contains("repo1 — agent/0") && list.contains("remote unreachable for a push:"), "{list}");
    let refused = inbox(&f, &["accept", "--all"]);
    assert_eq!(refused.status.code(), Some(1));
    assert!(common::both(&refused).contains("repo1 @ agent/0: its remote cannot be reached for a push"), "{}", common::both(&refused));
}

#[test]
fn a_stale_item_is_re_read_before_signing_and_refused_when_its_hash_moved() {
    let f = fixture();
    let dry = common::both(&inbox(&f, &["accept", "--all"]));
    let m = manifest(&dry);
    // The agent revises a decision on repo0 agent/0 after the holder read it.
    let agent = f.root.path().join("agent0");
    git(&agent, &["checkout", "-q", "agent/0"]);
    let store = ledger_core::store::load(&agent);
    let decision = store.log.iter().flat_map(|l| l.file.decisions.iter()).map(|d| d.id.to_string()).next_back().expect("decision");
    ledger_ok(&agent, &["revise", &decision, "--statement", "Revised after it was read."]);
    ledger_ok(&agent, &["export", "--format", "ntriples"]);
    git(&agent, &["add", "-A"]);
    git(&agent, &["commit", "-q", "-m", "revised"]);
    git(&agent, &["push", "-q", "origin", "agent/0"]);
    let out = common::both(&inbox(&f, &["accept", "--all", "--confirm", &m]));
    assert!(out.contains("repo0 @ agent/0: not signed"), "{out}");
    assert!(out.contains("moved"), "the drift is named: {out}");
    assert!(out.contains("11 branch(es) signed and pushed, 1 not"), "{out}");
}

#[test]
fn a_remote_branch_that_advances_between_fetch_and_push_refuses_the_push_and_is_reported_stale() {
    let f = fixture();
    let bare = f.root.path().join("remote1.git");
    let agent = f.root.path().join("agent1");
    let marker = f.root.path().join("raced");
    // The race, made deterministic: the holder clone's pre-push hook runs
    // after the inbox fetched and signed agent/1 and before its push lands,
    // and the agent pushes a new commit to agent/1 there.
    hook(&f.holders[1], "pre-push", &format!(
        "grep -q ' refs/heads/agent/1 ' || exit 0\n[ -e '{m}' ] && exit 0\ntouch '{m}'\n\
         cd '{a}' && git checkout -q agent/1 && echo raced > raced.txt && git add raced.txt && git commit -q -m raced && git push -q origin agent/1\n",
        m = marker.display(), a = agent.display(),
    ));
    let dry = common::both(&inbox(&f, &["accept", "--all"]));
    let out = inbox(&f, &["accept", "--all", "--confirm", &manifest(&dry)]);
    let text = common::both(&out);
    assert!(marker.exists(), "the hook ran: {text}");
    assert_eq!(out.status.code(), Some(1), "{text}");
    assert!(text.contains("repo1 @ agent/1: signed, not pushed — stale"), "{text}");
    assert!(text.contains("nothing on the remote was overwritten"), "{text}");
    assert!(text.contains("11 branch(es) signed and pushed, 1 not"), "the others proceed: {text}");
    // The remote holds the agent's commit, not the holder's: no push was forced.
    let raced = git(&agent, &["rev-parse", "HEAD"]);
    assert_eq!(git(&bare, &["rev-parse", "agent/1"]), raced, "the remote branch was not overwritten");
    assert!(!git(&bare, &["log", "--format=%ae", "main..agent/1"]).contains(OWNER), "no holder commit landed on agent/1");
    // The signed commit went with its worktree, and the item is listed again.
    let left = f.holders[1].path().join(".git/ledger-inbox/agent_1");
    assert!(!left.exists(), "the worktree of the refused push is gone");
    let list = common::both(&inbox(&f, &["list"]));
    assert!(list.contains("repo1 — agent/1"), "the item stays in the list: {list}");
}

/// A hook, executable, in a holder clone's git directory.
fn hook(h: &Repo, name: &str, body: &str) {
    let path = h.path().join(".git/hooks").join(name);
    std::fs::write(&path, format!("#!/bin/sh\nunset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE\n{body}")).expect("hook");
    std::fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("chmod");
}

#[test]
fn a_branch_whose_verify_fails_after_signing_is_not_pushed_and_the_others_go_through() {
    let f = fixture();
    // repo2 agent/broken already carries a finding the accept gate does not
    // refuse (it refuses only what a write introduces): an unallocated
    // decision, `L001`. Signing the allocated one beside it succeeds; the
    // branch's `verify --base` does not.
    let agent = f.root.path().join("agent2");
    git(&agent, &["checkout", "-q", "-b", "agent/broken", "origin/main"]);
    let ns = "repo2.ledger";
    ledger_ok(&agent, &["add", "--set", "ledger-design", "--namespace", ns, "--statement", "Proposed beside a defect.", "--store", "constraint", "--discharge", "analyzer:DEC001"]);
    ledger_ok(&agent, &["add", "--set", "ledger-design", "--namespace", ns, "--statement", "Left unallocated."]);
    ledger_ok(&agent, &["export", "--format", "ntriples"]);
    git(&agent, &["add", "-A"]);
    git(&agent, &["commit", "-q", "-m", "proposed, one unallocated"]);
    git(&agent, &["push", "-q", "origin", "agent/broken"]);
    let pushed = git(&agent, &["rev-parse", "HEAD"]);
    f.holders[2].git(&["fetch", "-q", "origin"]);
    let list = inbox(&f, &["list", "--json"]);
    let json: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&list.stdout)).expect("json");
    let mut args = vec!["accept".to_string()];
    for i in json["items"].as_array().expect("items").iter().filter(|i| i["statement"] != "Left unallocated.") {
        args.extend(["--decision".to_string(), i["decision"].as_str().expect("decision").to_string()]);
    }
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    let dry = common::both(&inbox(&f, &argv));
    let m = manifest(&dry);
    let mut confirm = argv.clone();
    confirm.extend(["--confirm", &m]);
    let out = inbox(&f, &confirm);
    let text = common::both(&out);
    assert_eq!(out.status.code(), Some(1), "{text}");
    assert!(text.contains("repo2 @ agent/broken: signed, not pushed — verify --base origin/main is NOT green, so nothing was pushed"), "{text}");
    assert!(text.contains("L001"), "the finding is named: {text}");
    assert!(text.contains("12 branch(es) signed and pushed, 1 not"), "the others go through: {text}");
    let bare = f.root.path().join("remote2.git");
    assert_eq!(git(&bare, &["rev-parse", "agent/broken"]), pushed, "nothing reached the remote");
    assert!(!f.holders[2].path().join(".git/ledger-inbox/agent_broken").exists(), "its worktree is discarded");
}

#[test]
fn a_killed_sitting_leaves_a_worktree_the_next_run_sweeps() {
    let f = fixture();
    // Kill the inbox itself (the push's parent) between verify and push.
    hook(&f.holders[0], "pre-push", "kill -9 $(awk '{print $4}' /proc/$PPID/stat)\nexit 1\n");
    let dry = common::both(&inbox(&f, &["accept", "--all"]));
    let killed = inbox(&f, &["accept", "--all", "--confirm", &manifest(&dry)]);
    assert_ne!(killed.status.code(), Some(0), "{}", common::both(&killed));
    let left = f.holders[0].path().join(".git/ledger-inbox/agent_0");
    assert!(left.join(".decisions/sig").is_dir(), "the kill left the signed worktree behind");
    let signed = git(&left, &["rev-parse", "HEAD"]);
    assert!(!git(f.holders[0].path(), &["worktree", "list"]).is_empty());
    let bare = f.root.path().join("remote0.git");
    assert!(!git(&bare, &["log", "--format=%ae", "main..agent/0"]).contains(OWNER), "nothing was pushed");
    std::fs::remove_file(f.holders[0].path().join(".git/hooks/pre-push")).expect("unhook");
    let list = common::both(&inbox(&f, &["list"]));
    let inbox_dir = f.holders[0].path().join(".git/ledger-inbox");
    assert!(!left.exists(), "the next run swept it: {list}");
    assert_eq!(std::fs::read_dir(&inbox_dir).map(|d| d.count()).unwrap_or(0), 0, "and every worktree it made since is discarded");
    assert!(!git(f.holders[0].path(), &["worktree", "list"]).contains("ledger-inbox"), "and git forgot it");
    assert!(!git(f.holders[0].path(), &["for-each-ref", "--contains", signed.trim()]).contains("refs/"), "the signed commit is on no ref");
    assert!(list.contains("repo0 — agent/0"), "the item is listed again: {list}");
}

#[test]
fn an_acceptance_under_a_since_closed_key_is_affirmed_in_the_same_batch() {
    let accepted = std::cell::RefCell::new(String::new());
    let f = fixture_with(|h| {
        let ns = "repo0.ledger";
        let id = common::decision_id(&h.ok(&[
            "add", "--set", "ledger-design", "--namespace", ns, "--statement", "Accepted before the close.",
            "--store", "constraint", "--discharge", "analyzer:DEC001",
        ]));
        h.ok_tty(&["accept", &id]);
        let store = ledger_core::store::load(h.path());
        let first = store.log.iter().flat_map(|l| l.file.key_bindings.iter()).map(|b| b.id.to_string()).next().expect("binding");
        let next = h.keygen("owner-next");
        h.ok(&["identity", "add", "--namespace", ns, "--key-file", &format!("{next}.pub")]);
        std::thread::sleep(std::time::Duration::from_millis(1100));
        h.use_key(&next);
        h.ok(&["identity", "revoke", &first]);
        h.ok(&["export", "--format", "ntriples"]);
        common::hand::commit(h, "accepted, then the key closed");
        h.git(&["push", "-q", "origin", "main"]);
        *accepted.borrow_mut() = id;
    });
    let id = accepted.borrow().clone();
    let list = common::both(&inbox(&f, &["list"]));
    assert_eq!(list.matches("needs re-acceptance (key-closed)").count(), 1, "listed once, on main: {list}");
    assert!(list.contains(&id), "{list}");
    let dry = common::both(&inbox(&f, &["accept", "--all"]));
    assert!(dry.contains(&format!("repo0 @ main  {id}")), "the affirmation rides the same batch: {dry}");
    let out = common::both(&inbox(&f, &["accept", "--all", "--confirm", &manifest(&dry)]));
    assert!(out.contains("13 branch(es) signed and pushed, 0 not"), "{out}");
    assert!(!out.contains("NOT green"), "{out}");
}
