//! `ledger inbox accept` when a branch goes wrong: a stale item, a push
//! race, a finding already on the branch, a write that introduces one, a
//! killed sitting, and a remote that cannot be reached for a push. Each
//! branch fails alone; nothing that was not verified reaches a remote.

mod common;
#[path = "inbox_fixture/mod.rs"]
mod fixture;

use fixture::*;

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

#[test]
fn a_finding_already_on_a_branch_does_not_hold_back_its_acceptances() {
    let f = fixture();
    // repo2 agent/broken already carries an unallocated decision (`L001`).
    // Accepting the allocated one beside it introduces nothing, so it is
    // pushed, and the `L001` is reported as remaining.
    let agent = f.root.path().join("agent2");
    git(&agent, &["checkout", "-q", "-b", "agent/broken", "origin/main"]);
    let ns = "repo2.ledger";
    ledger_ok(&agent, &["add", "--set", "ledger-design", "--namespace", ns, "--statement", "Proposed beside a defect.", "--store", "constraint", "--discharge", "analyzer:DEC001"]);
    ledger_ok(&agent, &["add", "--set", "ledger-design", "--namespace", ns, "--statement", "Left unallocated."]);
    ledger_ok(&agent, &["export", "--format", "ntriples"]);
    git(&agent, &["add", "-A"]);
    git(&agent, &["commit", "-q", "-m", "proposed, one unallocated"]);
    git(&agent, &["push", "-q", "origin", "agent/broken"]);
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
    assert_eq!(out.status.code(), Some(0), "a finding already there does not fail the sitting: {text}");
    assert!(text.contains("repo2 @ agent/broken: 1 accepted"), "{text}");
    assert!(text.contains("pushed — no finding introduced; 1 already on the branch remain:"), "{text}");
    assert!(text.contains("[L001]"), "the remaining finding is reported: {text}");
    assert!(text.contains("13 branch(es) signed and pushed, 0 not"), "{text}");
    let bare = f.root.path().join("remote2.git");
    assert!(git(&bare, &["log", "--format=%ae", "main..agent/broken"]).contains(OWNER), "the acceptance reached the remote");
    assert!(!f.holders[2].path().join(".git/ledger-inbox/agent_broken").exists(), "its worktree is discarded");
}

#[test]
fn a_write_that_introduces_a_finding_pushes_nothing_and_the_others_go_through() {
    let f = fixture();
    // repo1's clone signs as the holder but commits authored by someone else
    // (`author.email` overrides `user.email` for the author only). The
    // accept gate cannot see it — it runs before the commit — but the
    // acceptance's introducing commit is then not its actor's: `L009`, new.
    f.holders[1].git(&["config", "author.email", "someone-else@customer.example"]);
    let bare = f.root.path().join("remote1.git");
    let tips = git(&bare, &["for-each-ref", "refs/heads/"]);
    let dry = common::both(&inbox(&f, &["accept", "--all"]));
    let out = inbox(&f, &["accept", "--all", "--confirm", &manifest(&dry)]);
    let text = common::both(&out);
    assert_eq!(out.status.code(), Some(1), "{text}");
    assert_eq!(text.matches("signing introduced finding(s) on verify --base origin/main, so nothing was pushed").count(), 4, "repo1's four branches: {text}");
    assert!(text.contains("[L009]"), "the introduced finding is named: {text}");
    assert!(text.contains("8 branch(es) signed and pushed, 4 not"), "the others go through: {text}");
    assert_eq!(git(&bare, &["for-each-ref", "refs/heads/"]), tips, "nothing reached repo1's remote");
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
    assert!(ledger_core::layout::namespaces(&left).iter().any(|ns| ledger_core::layout::sig_dir(&left, ns).is_dir()), "the kill left the signed worktree behind");
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

