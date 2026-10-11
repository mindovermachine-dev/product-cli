//! `ledger inbox` (#79): three repositories, twelve agent branches with
//! proposed decisions, one holder, one sitting.
//!
//! Each repository is a bare remote with two clones: the holder's, which
//! opts the namespace in with its own genesis (`init --namespace`), binds a
//! key and grants itself the accept role; and an agent's, which proposes
//! decisions on branches and pushes them with their exports.

mod common;
#[path = "inbox_fixture/mod.rs"]
mod fixture;

use fixture::*;

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
        let on_main = git(h.path(), &["ls-tree", "-r", "--name-only", "origin/main", ".decisions/"]).lines().filter(|p| p.contains("/sig/")).count();
        let branches = git(h.path(), &["for-each-ref", "--format=%(refname:strip=3)", "refs/remotes/origin/agent/"]);
        for b in branches.lines() {
            let wt = f.root.path().join(format!("check-{i}-{}", b.replace('/', "-")));
            h.git(&["worktree", "add", "-q", "--detach", &wt.display().to_string(), &format!("origin/{b}")]);
            let v = common::invoke(&wt, &["verify", "--base", "origin/main", "--export"]);
            assert_eq!(v.status.code(), Some(0), "repo{i} {b}: {}", common::both(&v));
            let in_tree: usize = ledger_core::layout::namespaces(&wt).iter().map(|ns| std::fs::read_dir(ledger_core::layout::sig_dir(&wt, ns)).map(|d| d.count()).unwrap_or(0)).sum();
            sidecars += in_tree - on_main;
        }
    }
    assert_eq!(sidecars, 20, "one signature per acceptance, beyond the sidecars main already holds");
}

#[test]
fn a_namespace_with_no_policy_is_named_unchecked_not_omitted() {
    let f = fixture();
    let h = &f.holders[0];
    // A second namespace, never opted in, proposed on main.
    h.declare_in("loose.ledger");
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
fn accepting_one_of_a_branchs_two_proposed_decisions_pushes_it_and_the_other_stays_listed() {
    let f = fixture();
    let list = inbox(&f, &["list", "--json"]);
    let json: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&list.stdout)).expect("json");
    let on_branch: Vec<String> = json["items"].as_array().expect("items").iter()
        .filter(|i| i["repository"] == "repo0" && i["branch"] == "agent/0")
        .map(|i| i["decision"].as_str().expect("decision").to_string())
        .collect();
    assert_eq!(on_branch.len(), 2, "{json}");
    let (chosen, left) = (&on_branch[0], &on_branch[1]);
    let dry = common::both(&inbox(&f, &["accept", "--decision", chosen]));
    let out = inbox(&f, &["accept", "--decision", chosen, "--confirm", &manifest(&dry)]);
    let text = common::both(&out);
    assert_eq!(out.status.code(), Some(0), "{text}");
    assert!(text.contains("repo0 @ agent/0: 1 accepted"), "{text}");
    assert!(text.contains("pushed"), "{text}");
    assert!(git(&f.root.path().join("remote0.git"), &["log", "--format=%ae", "main..agent/0"]).contains(OWNER), "the acceptance reached the remote");
    f.holders[0].git(&["fetch", "-q", "origin"]);
    let again = common::both(&inbox(&f, &["list"]));
    assert!(again.contains(left.as_str()) && !again.contains(chosen.as_str()), "the other is still listed, the accepted one is not: {again}");
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
