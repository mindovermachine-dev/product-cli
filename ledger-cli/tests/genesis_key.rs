//! #96: `init --namespace` binds the genesis holder's key in the same act;
//! a namespace's first policy is signed when its author holds a trusted
//! key; `verify` names a `[none]` namespace.

mod common;

use common::hand;
use common::Repo;
use ledger_core::authority::BindingAct;

const OWNER: &str = "owner@customer.example";
const NS: &str = "fixture.ledger";
const SECOND: &str = "second.ledger";
const MANDATE: &str = "contract 2026/117";

fn verify(repo: &Repo) -> (i32, String) {
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

/// A store whose holder's `user.signingkey` names a key before `init`.
fn keyed() -> (Repo, String, String) {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    let key = repo.keygen("owner");
    repo.use_key(&key);
    let out = repo.ok(&["init", "--namespace", NS, "--external-ref", MANDATE]);
    (repo, key, out)
}

/// A store with no usable key configured (the path names no key, so a
/// global `user.signingkey` cannot leak in), initialised with
/// `--without-key`.
fn unkeyed() -> (Repo, String) {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.use_key("/nonexistent/owner-key");
    let out = repo.ok(&["init", "--namespace", NS, "--external-ref", MANDATE, "--without-key"]);
    (repo, out)
}

#[test]
fn with_no_usable_key_init_refuses_and_names_what_is_missing() {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.use_key("/nonexistent/owner-key");
    let out = repo.ledger(&["init", "--namespace", NS, "--external-ref", MANDATE]);
    let refused = common::both(&out);
    assert_eq!(out.status.code(), Some(2), "a usage refusal: {refused}");
    assert!(refused.contains("no usable key") && refused.contains("/nonexistent/owner-key"), "{refused}");
    assert!(refused.contains("--without-key"), "the way to proceed is named: {refused}");
    let store = ledger_core::store::load(repo.path());
    assert!(store.log.iter().all(|l| l.file.policies.is_empty() && l.file.grants.is_empty()), "nothing written");
}

fn policy_of(repo: &Repo, ns: &str) -> ledger_core::authority::Policy {
    let store = ledger_core::store::load(repo.path());
    store.log.iter().flat_map(|l| l.file.policies.iter()).find(|p| p.namespace == ns).cloned().expect("policy")
}

fn sidecar(repo: &Repo, ulid: &str) -> std::path::PathBuf {
    repo.path().join(".decisions/sig").join(format!("{ulid}.ssh.sig"))
}

/// A self-bound binding for the holder's address, filed by hand with a key
/// nobody vouched for — what anyone with commit access could write.
fn forge_self_bound(repo: &Repo, ns: &str) -> String {
    let forged_key = repo.keygen("forged");
    let mut forged = hand::binding(BindingAct::Add, OWNER, OWNER, ns, Some(&format!("{forged_key}.pub")), None, None);
    forged.self_bound = true;
    forged.mandate = Some(MANDATE.into());
    forged.hash = ledger_core::authority::payload::binding_hash(&forged);
    let id = forged.id.to_string();
    let ulid = forged.id.ulid().to_string();
    hand::file(repo, vec![forged], Vec::new(), &[(&ulid, &forged_key)]);
    id
}

#[test]
fn init_binds_the_configured_key_self_bound_and_it_signs_the_first_policy() {
    let (repo, key, out) = keyed();
    assert!(out.contains("bound") && out.contains("self-bound"), "{out}");
    let store = ledger_core::store::load(repo.path());
    let bindings: Vec<_> = store.log.iter().flat_map(|l| l.file.key_bindings.iter()).collect();
    assert_eq!(bindings.len(), 1, "one binding, in the init change-set");
    let b = bindings[0];
    assert!(b.self_bound && b.mandate.as_deref() == Some(MANDATE), "{b:?}");
    let public = std::fs::read_to_string(format!("{key}.pub")).expect("pub");
    assert!(public.contains(b.key.as_deref().unwrap_or("?")), "the configured key is the one bound");
    let policy = policy_of(&repo, NS);
    assert_eq!(b.at, policy.at, "dated with the policy, so the policy governs it");
    assert!(sidecar(&repo, b.id.ulid()).exists(), "the binding is signed by the key it binds");
    assert!(sidecar(&repo, policy.id.ulid()).exists(), "the first policy is signed");
    let signers = std::fs::read_to_string(repo.path().join(".decisions/allowed_signers")).expect("allowed_signers");
    assert!(signers.contains(b.key.as_deref().unwrap_or("?")), "{signers}");
    hand::commit(&repo, "governed");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
    assert!(!text.contains("has no trusted key"), "bound at init, the window is closed: {text}");
}

#[test]
fn bound_at_init_a_forged_self_bound_binding_is_never_trusted() {
    let (repo, _, _) = keyed();
    hand::commit(&repo, "governed");
    let id = forge_self_bound(&repo, NS);
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&id) && text.contains("D7") && text.contains("first in the store"), "{text}");
}

#[test]
fn unbound_at_init_the_window_stays_open_and_init_says_so() {
    let (repo, out) = unkeyed();
    assert!(out.contains("warning: no key bound") && out.contains("--without-key"), "{out}");
    hand::commit(&repo, "governed");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
    assert!(
        text.contains(&format!("notice: the genesis holder {OWNER} has no trusted key")) && text.contains(&format!("`{NS}`")),
        "verify says the window is open: {text}"
    );
    // The window #96 closes: the first self-bound binding to land for the
    // address — here a forged one — is the one trusted.
    let id = forge_self_bound(&repo, NS);
    repo.ok(&["identity", "sync"]);
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "unbound, the forged first binding is trusted: {text}");
    let signers = std::fs::read_to_string(repo.path().join(".decisions/allowed_signers")).expect("allowed_signers");
    let store = ledger_core::store::load(repo.path());
    let forged = store.log.iter().flat_map(|l| l.file.key_bindings.iter()).find(|b| b.id.to_string() == id).expect("forged");
    assert!(signers.contains(forged.key.as_deref().unwrap_or("?")), "and it reaches allowed_signers: {signers}");
    assert!(!text.contains("has no trusted key"), "the notice goes once a key is trusted — here the forged one: {text}");
}

#[test]
fn a_first_policy_filed_before_any_key_stays_valid_unsigned() {
    let (repo, _) = unkeyed();
    let policy = policy_of(&repo, NS);
    assert!(!sidecar(&repo, policy.id.ulid()).exists(), "nothing to sign with");
    hand::commit(&repo, "governed");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
}

#[test]
fn a_later_namespaces_first_policy_is_signed_and_unsigned_it_is_l011() {
    let (repo, _, _) = keyed();
    let out = repo.ok(&["init", "--namespace", SECOND, "--external-ref", MANDATE]);
    assert!(out.contains("signed"), "{out}");
    let policy = policy_of(&repo, SECOND);
    let sig = sidecar(&repo, policy.id.ulid());
    assert!(sig.exists(), "signed with the key trusted in the first namespace");
    hand::commit(&repo, "second namespace");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
    std::fs::remove_file(&sig).expect("remove");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[L011]") && text.contains(&policy.id.to_string()), "{text}");
}

#[test]
fn verify_names_a_none_namespace_in_a_notice_and_passes() {
    let (repo, _, _) = keyed();
    repo.ok(&["policy", "set", "--namespace", NS, "--scheme", "none"]);
    hand::commit(&repo, "unsigned");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains(&format!("notice: namespace `{NS}` is under policy `[none]`")), "{text}");
    assert!(text.contains("an absent signature is no finding"), "{text}");
    let json = repo.ledger(&["verify", "--no-blame", "--json"]);
    let report: serde_json::Value = serde_json::from_slice(&json.stdout).expect("json");
    assert_eq!(report["unsigned"][0], NS);
}

#[test]
fn init_in_a_later_namespace_binds_the_holders_key_there_and_they_sign_with_no_identity_add() {
    let (repo, key, _) = keyed();
    let out = repo.ok(&["init", "--namespace", SECOND, "--external-ref", MANDATE]);
    assert!(out.contains(&format!("in `{SECOND}`, signed by their key trusted elsewhere")), "{out}");
    let policy = policy_of(&repo, SECOND);
    let store = ledger_core::store::load(repo.path());
    let here: Vec<_> = store.log.iter().flat_map(|l| l.file.key_bindings.iter()).filter(|b| b.namespace == SECOND).collect();
    assert_eq!(here.len(), 1, "one binding, in the init change-set");
    let b = here[0];
    assert!(!b.self_bound && b.mandate.is_none() && b.by.as_str() == OWNER, "their own add: {b:?}");
    assert_eq!(b.at, policy.at, "dated with the policy");
    assert!(std::fs::read_to_string(format!("{key}.pub")).expect("pub").contains(b.key.as_deref().unwrap_or("?")), "the existing key");
    assert!(sidecar(&repo, b.id.ulid()).exists(), "signed by their key trusted in the first namespace");
    // They sign in the new namespace straight away.
    let grant = hand::word(&repo.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &format!("ns:{SECOND}")]), "grant:");
    repo.ok(&["grant", "accept", &grant]);
    let out = repo.ok(&[
        "add", "--set", "ledger-design", "--namespace", SECOND, "--statement", "Signed with no separate identity add.",
        "--store", "constraint", "--discharge", "analyzer:DEC001",
    ]);
    repo.ok_tty(&["accept", &common::decision_id(&out)]);
    hand::commit(&repo, "second namespace, accepted in");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
}

/// The genesis holder with bindings in the store, every one closed.
fn every_key_closed() -> (Repo, String) {
    let (repo, _, _) = keyed();
    hand::commit(&repo, "governed");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let id = ledger_core::store::load(repo.path()).log.iter().flat_map(|l| l.file.key_bindings.iter()).map(|b| b.id.to_string()).next().expect("binding");
    repo.ok(&["identity", "revoke", &id]);
    hand::commit(&repo, "closed the holder's only key");
    (repo, id)
}

#[test]
fn with_every_key_closed_init_in_a_later_namespace_refuses_and_names_them() {
    let (repo, closed) = every_key_closed();
    let out = repo.ledger(&["init", "--namespace", SECOND, "--external-ref", MANDATE]);
    let refused = common::both(&out);
    assert_eq!(out.status.code(), Some(2), "a usage refusal: {refused}");
    assert!(refused.contains("no live key") && refused.contains(&closed), "{refused}");
    assert!(refused.contains("--without-key"), "the way to proceed is named: {refused}");
    let store = ledger_core::store::load(repo.path());
    assert!(store.log.iter().all(|l| l.file.policies.iter().all(|p| p.namespace != SECOND)), "nothing written");
}

#[test]
fn with_every_key_closed_and_without_key_init_warns_and_initialises_unbound() {
    let (repo, closed) = every_key_closed();
    let out = repo.ok(&["init", "--namespace", SECOND, "--external-ref", MANDATE, "--without-key"]);
    assert!(out.contains("warning: no live key") && out.contains(&closed), "{out}");
    let store = ledger_core::store::load(repo.path());
    assert!(store.log.iter().flat_map(|l| l.file.key_bindings.iter()).all(|b| b.namespace != SECOND), "no binding in {SECOND}");
    hand::commit(&repo, "second namespace, unbound");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
}
