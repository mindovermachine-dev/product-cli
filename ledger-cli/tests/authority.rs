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

/// A namespace under policy, before any key is bound.
fn initialised() -> Repo {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    let out = repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117", "--without-key"]);
    assert!(out.contains("genesis grant:"), "{out}");
    repo
}

/// [`initialised`], with the genesis holder's key bound and configured:
/// the policy requires `ssh`, so every later act is signed.
fn governed() -> Repo {
    let repo = initialised();
    repo.bind_own_key(NS, "owner");
    repo
}

/// The architect's first key, vouched for by the genesis holder (D7); the
/// repo then acts and signs as the architect.
fn as_architect(repo: &Repo) {
    let key = repo.vouch_for(NS, ARCHITECT, "architect");
    repo.act_as(ARCHITECT);
    repo.use_key(&key);
}

/// [`governed`], with the genesis holder also granted the accept role: the
/// genesis role carries no decision capability (D9 (f)), so a genesis
/// holder who accepts holds a second grant.
fn governed_accepting() -> (Repo, String) {
    let repo = governed();
    let out = repo.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &format!("ns:{NS}")]);
    let grant = grant_id(&out);
    repo.ok(&["grant", "accept", &grant]);
    (repo, grant)
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
    let root = std::fs::read_to_string(repo.path().join(".decisions/roles/steward.yml")).expect("root role");
    for cap in ["grant-role", "revoke-grant", "declare-unavailability", "rotate-genesis"] {
        assert!(root.contains(cap), "the root role carries {cap}: {root}");
    }
    for cap in ["accept-decision", "sign-off-pattern", "waive-invalidation"] {
        assert!(!root.contains(cap), "the root role carries no decision capability: {root}");
    }
    let accept = std::fs::read_to_string(repo.path().join(".decisions/roles/acceptor.yml")).expect("accept role");
    assert!(accept.contains("accept-decision"), "{accept}");
    let shown = repo.ok(&["policy", "show", "--namespace", NS]);
    assert!(shown.contains("accept-decision role: acceptor"), "{shown}");
    repo.ok(&["verify", "--no-blame"]);
    let again = repo.refused(&["init", "--namespace", NS, "--external-ref", "x"]);
    assert!(again.contains("already under policy"), "{again}");
}

#[test]
fn only_a_holder_of_the_accept_role_accepts_in_a_governed_namespace() {
    let repo = governed();
    let id = repo.add("Money is decimal.", &[]);
    let genesis_only = repo.refused(&["accept", &id]);
    assert!(genesis_only.contains("holds no grant"), "the genesis role does not accept: {genesis_only}");
    let out = repo.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &format!("ns:{NS}")]);
    repo.ok(&["grant", "accept", &grant_id(&out)]);
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
    let out = repo.ok(&["grant", "new", "acceptor", "--to", ARCHITECT, "--scope", &format!("ns:{NS}")]);
    let grant = grant_id(&out);
    as_architect(&repo);
    let refused = repo.refused(&["accept", &id]);
    assert!(refused.contains("has not accepted it"), "{refused}");
    repo.ok(&["grant", "accept", &grant]);
    repo.ok_tty(&["accept", &id]);
    repo.ok(&["verify", "--no-blame"]);
}

#[test]
fn an_unavailable_holder_cannot_accept_until_available_again() {
    let (repo, acceptor) = governed_accepting();
    let id = repo.add("Money is decimal.", &[]);
    let out = repo.ok(&["unavailable", &acceptor, "--from", "2020-01-01T00:00:00Z", "--reason", "leave"]);
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
    assert!(text.contains("format: 7") && text.contains("under: grant:") && text.contains("id: rev:") && text.contains("revokes: grant:"), "{text}");
    repo.ok(&["verify", "--no-blame"]);
}

#[test]
fn key_bindings_derive_allowed_signers_and_verify_holds_it() {
    let repo = initialised();
    let owner = repo.keygen("owner");
    repo.use_key(&owner);
    let first = repo.ok(&["identity", "add", "--namespace", NS, "--key-file", &format!("{owner}.pub")]);
    let binding = first.split_whitespace().find(|w| w.starts_with("key:")).expect("binding id").to_string();
    let log = repo.log_files().pop().expect("log");
    let text = std::fs::read_to_string(repo.path().join(".decisions/log").join(log)).expect("read");
    assert!(text.contains("self_bound: true") && text.contains("mandate: contract 2026/117"), "{text}");
    let ulid = binding.trim_start_matches("key:");
    assert!(repo.path().join(format!(".decisions/sig/{ulid}.ssh.sig")).is_file(), "self-signed by the key it binds");
    let signers = repo.path().join(".decisions/allowed_signers");
    let derived = std::fs::read_to_string(&signers).expect("allowed_signers");
    assert!(derived.contains(&format!("{OWNER} namespaces=\"ledger-accept@{NS}\",valid-after=")), "{derived}");
    repo.ok(&["verify", "--no-blame"]);

    let next = repo.keygen("owner2");
    repo.ok(&["identity", "rotate", &binding, "--key-file", &format!("{next}.pub")]);
    let rotated = std::fs::read_to_string(&signers).expect("allowed_signers");
    assert_eq!(rotated.matches("valid-before=").count(), 1, "the old window closed: {rotated}");
    assert_eq!(rotated.lines().filter(|l| !l.starts_with('#')).count(), 2, "{rotated}");
    repo.ok(&["verify", "--no-blame"]);

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
    // The signer's own key is a software key too: every signed act now
    // waits on a hardware-backed key, which this suite cannot touch.
    let hard = key_file(&repo, "hard", "sk-ssh-ed25519@openssh.com");
    let refused = repo.refused(&["identity", "add", "--namespace", NS, "--key-file", &hard]);
    assert!(refused.contains("hardware-backed") && refused.contains("software key"), "{refused}");
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

fn acceptance_of(repo: &Repo) -> String {
    let store = ledger_core::store::load(repo.path());
    store.log.iter().flat_map(|c| c.file.acceptances.iter()).map(|a| a.id.to_string()).next_back().expect("acceptance")
}

#[test]
fn revoke_files_a_revocation_entity_and_leaves_the_acceptance_untouched() {
    let (repo, _) = governed_accepting();
    let id = repo.add("Money is decimal.", &[]);
    repo.ok_tty(&["accept", &id]);
    let acc = acceptance_of(&repo);
    let files_before: Vec<(String, String)> = repo
        .log_files()
        .into_iter()
        .map(|f| (f.clone(), std::fs::read_to_string(repo.path().join(".decisions/log").join(&f)).expect("read")))
        .collect();
    let out = repo.ok_tty(&["revoke", &acc, "--reason", "filed against the wrong version"]);
    assert!(out.contains("by rev:"), "{out}");
    for (file, text) in &files_before {
        let now = std::fs::read_to_string(repo.path().join(".decisions/log").join(file)).expect("read");
        assert_eq!(&now, text, "{file} was edited by a revocation");
    }
    let log = repo.log_files().pop().expect("log");
    let text = std::fs::read_to_string(repo.path().join(".decisions/log").join(log)).expect("read");
    assert!(text.contains("format: 7") && text.contains("under: grant:") && text.contains(&format!("revokes: {acc}")) && text.contains("hash: sha256:"), "{text}");
    repo.ok(&["verify", "--no-blame"]);
}

#[test]
fn revoking_in_a_governed_namespace_needs_the_accept_role() {
    let (repo, _) = governed_accepting();
    let id = repo.add("Money is decimal.", &[]);
    repo.ok_tty(&["accept", &id]);
    let acc = acceptance_of(&repo);
    repo.act_as(ARCHITECT);
    let refused = repo.refused(&["revoke", &acc, "--reason", "not mine to unsay"]);
    assert!(refused.contains("may not revoke an acceptance"), "{refused}");
}

#[test]
fn a_model_identity_cannot_revoke_a_persons_acceptance() {
    let repo = Repo::with_identity(ARCHITECT);
    repo.declare();
    let id = repo.add("Money is decimal.", &[]);
    repo.ok_tty(&["accept", &id]);
    let acc = acceptance_of(&repo);
    repo.act_as("noreply@anthropic.com");
    let refused = repo.refused(&["revoke", &acc, "--reason", "an agent tidying up"]);
    assert!(refused.contains("[L006]") && refused.contains("revoker"), "{refused}");
}

#[test]
fn the_accept_role_is_never_the_genesis_role() {
    let repo = Repo::with_identity(OWNER);
    let refused = repo.ledger(&["init", "--namespace", NS, "--external-ref", "m", "--accept-role", "steward"]);
    assert_eq!(refused.status.code(), Some(2), "{}", common::both(&refused));
    assert!(common::both(&refused).contains("must differ from the genesis role"), "{}", common::both(&refused));
    assert!(repo.log_files().is_empty(), "a refused init files nothing");
    let repo = governed();
    let out = repo.ledger(&["policy", "set", "--namespace", NS, "--accept-role", "steward"]);
    assert_eq!(out.status.code(), Some(2), "{}", common::both(&out));
    assert!(common::both(&out).contains("must differ from the genesis role"), "{}", common::both(&out));
}

#[test]
fn bootstrap_refuses_an_existing_root_role_without_the_root_capabilities() {
    let repo = Repo::with_identity(OWNER);
    let role = "format: 6\nid: steward\nowner: owner@customer.example\nmay: [accept-decision, grant-role]\ncreated_at: 2026-10-02\n";
    std::fs::create_dir_all(repo.path().join(".decisions/roles")).expect("roles dir");
    std::fs::write(repo.path().join(".decisions/roles/steward.yml"), role).expect("role");
    let refused = repo.refused(&["init", "--namespace", NS, "--external-ref", "m"]);
    assert!(refused.contains("cannot be the genesis role") && refused.contains("revoke-grant"), "{refused}");
    assert!(repo.log_files().is_empty(), "a refused init files nothing");
}

#[test]
fn a_grantor_below_star_is_refused_a_grant_over_another_scope() {
    let repo = governed();
    let out = repo.ok(&["grant", "new", "acceptor", "--to", ARCHITECT, "--scope", &format!("ns:{NS}")]);
    let grant = grant_id(&out);
    repo.ok(&["role", "declare", "delegate", "--may", "grant-role"]);
    let out = repo.ok(&["grant", "new", "delegate", "--to", ARCHITECT, "--scope", &format!("ns:{NS}")]);
    let delegate = grant_id(&out);
    repo.act_as(ARCHITECT);
    repo.ok(&["grant", "accept", &grant]);
    repo.ok(&["grant", "accept", &delegate]);
    for scope in ["*", "ns:fixture.other", "set:ledger-design"] {
        let refused = repo.refused(&["grant", "new", "delegate", "--to", "third@customer.example", "--scope", scope]);
        assert!(refused.contains("does not cover this"), "{scope}: {refused}");
    }
    repo.ok(&["grant", "new", "delegate", "--to", "third@customer.example", "--scope", &format!("ns:{NS}")]);
    repo.ok(&["verify", "--no-blame"]);
}

/// Ruling 50, the verification report's section 2: a hand edit made the
/// genesis role carry `accept-decision` and named it as the policy's accept
/// role. `verify` was conformant, and the genesis grant alone accepted. Both
/// halves are now schema faults at verification.
#[test]
fn a_policy_naming_the_genesis_role_and_a_widened_genesis_role_fail_verify() {
    use ledger_core::authority::payload::policy_hash;
    use ledger_core::changeset::ChangeSet;
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117", "--without-key"]);
    let log = repo.path().join(".decisions/log");
    let file = std::fs::read_dir(&log).expect("log").flatten().map(|e| e.path()).next().expect("the init change-set");
    let mut cs: ChangeSet = serde_yaml::from_str(&std::fs::read_to_string(&file).expect("read")).expect("parse");
    for p in &mut cs.policies {
        p.accept_role = "steward".into();
        p.hash = policy_hash(p);
    }
    std::fs::write(&file, serde_yaml::to_string(&cs).expect("yaml")).expect("write");
    let roles = repo.path().join(".decisions/roles");
    let steward = std::fs::read_to_string(roles.join("steward.yml")).expect("steward");
    std::fs::write(roles.join("steward.yml"), steward.replace("- rotate-genesis\n", "- rotate-genesis\n- accept-decision\n")).expect("widen");
    std::fs::remove_file(roles.join("acceptor.yml")).expect("acceptor");
    common::hand::commit(&repo, "the genesis role is the accept role");
    let out = repo.ledger(&["verify", "--no-blame"]);
    let text = common::both(&out);
    assert_eq!(out.status.code(), Some(1), "{text}");
    assert!(text.contains("maps accept-decision to `steward`, the genesis role as of the policy"), "{text}");
    assert!(text.contains("[SCHEMA] steward: is the genesis role and carries accept-decision"), "{text}");
}

/// Ruling 50 extends D9 (f) to the writer, the report's section 13.2: an
/// existing root role that also carries a decision capability is refused as
/// the genesis role.
#[test]
fn bootstrap_refuses_an_existing_root_role_that_carries_a_decision_capability() {
    let repo = Repo::with_identity(OWNER);
    let role = "format: 6\nid: steward\nowner: owner@customer.example\nmay: [grant-role, revoke-grant, declare-unavailability, rotate-genesis, accept-decision]\ncreated_at: 2026-10-07\n";
    std::fs::create_dir_all(repo.path().join(".decisions/roles")).expect("roles dir");
    std::fs::write(repo.path().join(".decisions/roles/steward.yml"), role).expect("role");
    let refused = repo.refused(&["init", "--namespace", NS, "--external-ref", "m", "--without-key"]);
    assert!(refused.contains("cannot be the genesis role") && refused.contains("carries accept-decision"), "{refused}");
    assert!(repo.log_files().is_empty(), "a refused init files nothing");
}

/// No agent identity and no non-interactive session produces an
/// acceptance, in a namespace whose accept role the genesis holder holds.
#[test]
fn no_agent_identity_and_no_non_interactive_session_produces_an_acceptance() {
    let (repo, _grant) = governed_accepting();
    let id = repo.add("Money is decimal.", &[]);
    let before = repo.log_files();
    let piped = repo.piped(&["accept", &id]);
    assert_ne!(piped.status.code(), Some(0), "{}", common::both(&piped));
    assert_eq!(repo.log_files(), before, "a piped accept files nothing");
    for agent in ["claude@anthropic.com", "noreply@anthropic.com", "github-actions@github.com"] {
        repo.act_as(agent);
        let out = repo.tty(&["accept", &id]);
        assert_ne!(out.status.code(), Some(0), "{agent}: {}", common::both(&out));
        assert_eq!(repo.log_files(), before, "{agent}: nothing filed");
    }
    repo.act_as(OWNER);
    repo.ok_tty(&["accept", &id]);
    assert_eq!(repo.log_files().len(), before.len() + 1, "the holder, at a terminal, accepts");
}
