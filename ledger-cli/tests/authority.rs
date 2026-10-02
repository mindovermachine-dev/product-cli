//! The authority model end to end: genesis, grants, keys, policy, availability.
//!
//! One namespace is put under policy and every verb that changes who may
//! act is driven through the binary, each with its refusal: an identity
//! without a live, accepted, available grant of the accept role cannot
//! accept; a grant is live only once its holder accepts it; the trust root
//! (`allowed_signers`) is derived from key bindings and held by `verify`.

mod common;

use common::Repo;

const OWNER: &str = "owner@customer.example";
const ARCHITECT: &str = "architect@customer.example";
const NS: &str = "fixture.ledger";

fn governed() -> Repo {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    let out = repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117"]);
    assert!(out.contains("genesis grant:"), "{out}");
    repo
}

fn grant_id(out: &str) -> String {
    out.split_whitespace()
        .map(|w| w.trim_matches(|c: char| c == '(' || c == ')' || c == '`'))
        .find(|w| w.starts_with("grant:"))
        .expect("a grant id")
        .to_string()
}

fn key_file(repo: &Repo, name: &str, key_type: &str) -> String {
    let path = repo.path().join(name);
    std::fs::write(&path, format!("{key_type} AAAAC3NzaC1lZDI1NTE5AAAAI{name} {name}@laptop\n")).expect("key");
    path.display().to_string()
}

#[test]
fn init_namespace_bootstraps_the_genesis_and_the_gate_stays_green() {
    let repo = governed();
    assert!(repo.path().join(".decisions/roles/steward.yml").is_file());
    let shown = repo.ok(&["policy", "show", "--namespace", NS]);
    assert!(shown.contains("accept-decision role: steward"), "{shown}");
    repo.ok(&["verify", "--no-blame"]);
    let again = repo.refused(&["init", "--namespace", NS, "--external-ref", "x"]);
    assert!(again.contains("already under policy"), "{again}");
}

#[test]
fn only_a_holder_of_the_accept_role_accepts_in_a_governed_namespace() {
    let repo = governed();
    let id = repo.add("Money is decimal.", &[]);
    repo.act_as(ARCHITECT);
    let before = repo.log_files();
    let refused = repo.refused(&["accept", &id]);
    assert!(refused.contains("holds no grant"), "{refused}");
    assert_eq!(repo.log_files(), before, "a refused accept writes nothing");
    repo.act_as(OWNER);
    let ok = repo.ok_tty(&["accept", &id]);
    assert!(ok.contains("under grant:"), "{ok}");
    repo.ok(&["verify", "--no-blame"]);
}

#[test]
fn a_grant_is_live_only_once_its_holder_accepts_it() {
    let repo = governed();
    let id = repo.add("Money is decimal.", &[]);
    let out = repo.ok(&["grant", "new", "steward", "--to", ARCHITECT, "--scope", &format!("ns:{NS}")]);
    let grant = grant_id(&out);
    repo.act_as(ARCHITECT);
    let refused = repo.refused(&["accept", &id]);
    assert!(refused.contains("has not accepted it"), "{refused}");
    repo.ok(&["grant", "accept", &grant]);
    repo.ok_tty(&["accept", &id]);
    repo.ok(&["verify", "--no-blame"]);
}

#[test]
fn an_unavailable_holder_cannot_accept_until_available_again() {
    let repo = governed();
    let id = repo.add("Money is decimal.", &[]);
    let genesis = repo
        .log_files()
        .iter()
        .map(|f| std::fs::read_to_string(repo.path().join(".decisions/log").join(f)).expect("log"))
        .find(|text| text.contains("genesis: true"))
        .map(|text| grant_id(&text))
        .expect("the genesis grant");
    let out = repo.ok(&["unavailable", &genesis, "--from", "2020-01-01T00:00:00Z", "--reason", "leave"]);
    let refused = repo.refused(&["accept", &id]);
    assert!(refused.contains("unavailable now"), "{refused}");
    let interval = out.split_whitespace().find(|w| w.starts_with("unav:")).expect("interval id");
    repo.ok(&["available", interval]);
    repo.ok_tty(&["accept", &id]);
}

#[test]
fn grant_revocation_needs_a_terminal_and_files_its_own_entity() {
    let repo = governed();
    let out = repo.ok(&["grant", "new", "steward", "--to", ARCHITECT, "--scope", &format!("ns:{NS}")]);
    let grant = grant_id(&out);
    let piped = repo.piped(&["grant", "revoke", &grant, "--reason", "moved"]);
    assert_ne!(piped.status.code(), Some(0));
    let revoked = repo.ok_tty(&["grant", "revoke", &grant, "--reason", "moved to the platform team"]);
    assert!(revoked.contains("revoked grant:"), "{revoked}");
    let log = repo.log_files().pop().expect("log");
    let text = std::fs::read_to_string(repo.path().join(".decisions/log").join(log)).expect("read");
    assert!(text.contains("format: 6") && text.contains("id: rev:") && text.contains("revokes: grant:"), "{text}");
    repo.ok(&["verify", "--no-blame"]);
}

#[test]
fn key_bindings_derive_allowed_signers_and_verify_holds_it() {
    let repo = governed();
    let first = repo.ok(&["identity", "add", "--namespace", NS, "--key-file", &key_file(&repo, "owner", "ssh-ed25519")]);
    let binding = first.split_whitespace().find(|w| w.starts_with("key:")).expect("binding id").to_string();
    let log = repo.log_files().pop().expect("log");
    let text = std::fs::read_to_string(repo.path().join(".decisions/log").join(log)).expect("read");
    assert!(text.contains("self_bound: true") && text.contains("mandate: contract 2026/117"), "{text}");
    let signers = repo.path().join(".decisions/allowed_signers");
    let derived = std::fs::read_to_string(&signers).expect("allowed_signers");
    assert!(derived.contains(&format!("{OWNER} namespaces=\"ledger-accept@{NS}\" valid-after=")), "{derived}");
    repo.ok(&["verify", "--no-blame"]);

    repo.ok(&["identity", "rotate", &binding, "--key-file", &key_file(&repo, "owner2", "ssh-ed25519")]);
    let rotated = std::fs::read_to_string(&signers).expect("allowed_signers");
    assert_eq!(rotated.matches("valid-before=").count(), 1, "the old window closed: {rotated}");
    assert_eq!(rotated.lines().filter(|l| !l.starts_with('#')).count(), 2, "{rotated}");

    std::fs::write(&signers, format!("{rotated}{ARCHITECT} ssh-ed25519 AAAAforged\n")).expect("tamper");
    let out = repo.ledger(&["verify", "--no-blame"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(common::both(&out).contains("[SIGNERS]"), "{}", common::both(&out));
}

#[test]
fn a_software_key_is_refused_under_an_sk_policy() {
    let repo = governed();
    repo.ok(&["policy", "set", "--namespace", NS, "--require-sk", "true"]);
    let refused = repo.refused(&["identity", "add", "--namespace", NS, "--key-file", &key_file(&repo, "soft", "ssh-ed25519")]);
    assert!(refused.contains("hardware-backed"), "{refused}");
    repo.ok(&["identity", "add", "--namespace", NS, "--key-file", &key_file(&repo, "hard", "sk-ssh-ed25519@openssh.com")]);
    repo.ok(&["verify", "--no-blame"]);
}

#[test]
fn policy_changes_and_role_declarations_are_the_genesis_holders() {
    let repo = governed();
    repo.act_as(ARCHITECT);
    let refused = repo.refused(&["policy", "set", "--namespace", NS, "--scheme", "dsse"]);
    assert!(refused.contains("genesis holder"), "{refused}");
    let role = repo.refused(&["role", "declare", "reviewer", "--may", "accept-decision"]);
    assert!(role.contains("may not declare a role"), "{role}");
    repo.act_as(OWNER);
    repo.ok(&["role", "declare", "reviewer", "--may", "accept-decision"]);
    let set = repo.ok(&["policy", "set", "--namespace", NS, "--accept-role", "reviewer"]);
    assert!(set.contains("replaces pol:"), "{set}");
    let shown = repo.ok(&["policy", "show", "--namespace", NS]);
    assert!(shown.contains("accept-decision role: reviewer"), "{shown}");
    repo.ok(&["verify", "--no-blame"]);
}

#[test]
fn a_namespace_without_policy_is_not_role_checked() {
    let repo = Repo::with_identity(ARCHITECT);
    repo.declare();
    let id = repo.add("Money is decimal.", &[]);
    repo.ok_tty(&["accept", &id]);
    repo.ok(&["verify", "--no-blame"]);
}
