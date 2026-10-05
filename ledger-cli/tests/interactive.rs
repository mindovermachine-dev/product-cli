//! Signing verbs and authority-record verbs refuse a non-interactive caller
//! and write nothing.
//!
//! Signing is a person's act (PRD §5, §10), and so is every record of who
//! may sign (#85): run without a terminal on stdin — piped, scripted, from
//! an agent harness — those verbs exit non-zero before the store is opened.
//! There is no override to test: no flag and no environment variable turns
//! the check off. The same invocations under a pseudo-terminal get past it,
//! so the refusal is the terminal check and nothing else.

mod common;

use std::collections::BTreeMap;

use common::Repo;

/// Every file under `.decisions/`, with its bytes: "writes nothing" is
/// checked against the whole store, not one directory.
fn snapshot(repo: &Repo) -> BTreeMap<String, Vec<u8>> {
    fn walk(dir: &std::path::Path, base: &std::path::Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, base, out);
            } else {
                let rel = path.strip_prefix(base).unwrap_or(&path).display().to_string();
                out.insert(rel, std::fs::read(&path).unwrap_or_default());
            }
        }
    }
    let mut out = BTreeMap::new();
    let store = repo.path().join(".decisions");
    walk(&store, &store, &mut out);
    out
}

fn refused_without_a_terminal(repo: &Repo, args: &[&str]) {
    let before = snapshot(repo);
    let out = repo.piped(args);
    let text = common::both(&out);
    assert_ne!(out.status.code(), Some(0), "ledger {args:?} must refuse piped stdin:\n{text}");
    assert!(text.contains("runs only at a terminal"), "{text}");
    assert!(text.contains("Nothing was written"), "{text}");
    assert_eq!(snapshot(repo), before, "ledger {args:?} wrote under a refusal");
}

fn filed() -> (Repo, String) {
    let repo = Repo::human();
    repo.declare();
    let id = repo.add("Monetary amounts use decimal, never double.", &[]);
    (repo, id)
}

#[test]
fn accept_naming_a_decision_refuses_piped_stdin() {
    let (repo, id) = filed();
    refused_without_a_terminal(&repo, &["accept", &id]);
    // The same invocation at a terminal signs: the refusal was the check.
    let signed = repo.ok_tty(&["accept", &id]);
    assert!(signed.contains("accepted"), "{signed}");
}

#[test]
fn a_selection_dry_run_stays_scriptable_but_its_confirm_refuses() {
    let (repo, _) = filed();
    let read = repo.ok(&["accept", "--set", "ledger-design"]);
    let manifest = read
        .split_whitespace()
        .find(|w| w.starts_with("sha256:"))
        .expect("the dry run prints its manifest")
        .to_string();
    refused_without_a_terminal(&repo, &["accept", "--set", "ledger-design", "--confirm", &manifest]);
    repo.ok_tty(&["accept", "--set", "ledger-design", "--confirm", &manifest]);
}

#[test]
fn revoke_refuses_piped_stdin() {
    let (repo, id) = filed();
    repo.ok_tty(&["accept", &id]);
    let acc = snapshot(&repo)
        .values()
        .flat_map(|bytes| String::from_utf8_lossy(bytes).into_owned().lines().map(str::to_string).collect::<Vec<_>>())
        .find_map(|line| line.trim().strip_prefix("- id: acc:").map(|u| format!("acc:{u}")))
        .expect("the acceptance just filed");
    refused_without_a_terminal(&repo, &["revoke", &acc, "--reason", "filed in error"]);
}

#[test]
fn piped_stdin_is_refused_even_with_a_closed_or_empty_stdin() {
    let (repo, id) = filed();
    let before = snapshot(&repo);
    let mut cmd = assert_cmd::Command::cargo_bin("ledger").expect("binary");
    let out = cmd
        .arg("--root")
        .arg(repo.path())
        .args(["accept", &id])
        .write_stdin("y\nyes\n")
        .output()
        .expect("run");
    assert_ne!(out.status.code(), Some(0), "typing `yes` into a pipe is not a terminal");
    assert_eq!(snapshot(&repo), before);
}

// ---- #85: every verb that writes an authority record ----------------------

const OWNER: &str = "owner@customer.example";
const NS: &str = "fixture.ledger";

/// A governed store with a live, accepted grant, an open unavailability on
/// it and a key binding, so every refusing verb below names real records.
struct Governed {
    repo: Repo,
    grant: String,
    interval: String,
    binding: String,
    key_file: String,
}

fn word(out: &str, prefix: &str) -> String {
    out.split_whitespace()
        .map(|w| w.trim_matches(|c: char| c == '(' || c == ')' || c == '`'))
        .find(|w| w.starts_with(prefix))
        .unwrap_or_else(|| panic!("no {prefix} id in {out}"))
        .to_string()
}

fn governed() -> Governed {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117", "--without-key"]);
    let grant = word(&repo.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &format!("ns:{NS}")]), "grant:");
    repo.ok(&["grant", "accept", &grant]);
    let owner = repo.keygen("owner");
    repo.use_key(&owner);
    let binding = word(&repo.ok(&["identity", "add", "--namespace", NS, "--key-file", &format!("{owner}.pub")]), "key:");
    let key_file = format!("{}.pub", repo.keygen("spare"));
    let interval = word(&repo.ok(&["unavailable", &grant, "--from", "2030-01-01T00:00:00Z", "--reason", "leave"]), "unav:");
    Governed { repo, grant, interval, binding, key_file }
}

/// One test per refusing authority verb: piped stdin exits non-zero, says
/// why, and leaves `.decisions/` byte-identical.
macro_rules! refuses_piped {
    ($($name:ident => |$g:ident| $args:expr;)*) => {$(
        #[test]
        fn $name() {
            let $g = governed();
            let args: Vec<String> = $args;
            let args: Vec<&str> = args.iter().map(String::as_str).collect();
            refused_without_a_terminal(&$g.repo, &args);
            // At a terminal the same invocation gets past the check: the
            // refusal was the terminal check and nothing else.
            let at_tty = common::both(&$g.repo.tty(&args));
            assert!(!at_tty.contains("runs only at a terminal"), "{at_tty}");
        }
    )*};
}

fn v(args: &[&str]) -> Vec<String> {
    args.iter().map(|a| a.to_string()).collect()
}

refuses_piped! {
    init_namespace_refuses_piped_stdin => |g| v(&["init", "--namespace", "fixture.other", "--external-ref", "m"]);
    role_declare_refuses_piped_stdin => |g| v(&["role", "declare", "reviewer", "--may", "accept-decision"]);
    grant_new_refuses_piped_stdin => |g| v(&["grant", "new", "acceptor", "--to", "a@customer.example", "--scope", &format!("ns:{NS}")]);
    grant_accept_refuses_piped_stdin => |g| v(&["grant", "accept", &g.grant]);
    grant_revoke_refuses_piped_stdin => |g| v(&["grant", "revoke", &g.grant, "--reason", "moved"]);
    unavailable_refuses_piped_stdin => |g| v(&["unavailable", &g.grant, "--reason", "leave"]);
    available_refuses_piped_stdin => |g| v(&["available", &g.interval]);
    identity_add_refuses_piped_stdin => |g| v(&["identity", "add", "--namespace", NS, "--key-file", &g.key_file]);
    identity_rotate_refuses_piped_stdin => |g| v(&["identity", "rotate", &g.binding, "--key-file", &g.key_file]);
    identity_revoke_refuses_piped_stdin => |g| v(&["identity", "revoke", &g.binding]);
    policy_set_refuses_piped_stdin => |g| v(&["policy", "set", "--namespace", NS, "--require-sk", "true"]);
}

#[test]
fn the_decision_and_derived_file_verbs_stay_scriptable() {
    let g = governed();
    let repo = &g.repo;
    let id = repo.add("Money is decimal.", &[]);
    for args in [
        vec!["revise", id.as_str(), "--statement", "Money is decimal, never double."],
        vec!["allocate", id.as_str(), "--store", "judgment", "--actor", OWNER],
        vec!["identity", "sync"],
        vec!["reindex"],
        vec!["merge", "--install"],
        vec!["init"],
    ] {
        let out = repo.piped(&args);
        assert_eq!(out.status.code(), Some(0), "ledger {args:?} must run piped:\n{}", common::both(&out));
    }
}
