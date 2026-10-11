//! A key belongs to one principal, and a closed key is never bound again
//! (ruled 2026-10-06, extending ruling 1 of #104).
//!
//! Binding a key that is, or was, bound to a different principal anywhere in
//! the store, or a key closed for its principal in any namespace, is refused
//! at filing and is a schema fault at `verify`.

mod common;

use common::{hand, Repo};
use ledger_core::authority::BindingAct;

const OWNER: &str = "owner@customer.example";
const OTHER: &str = "architect@customer.example";
const MANDATE: &str = "contract 2026/117";
const NS: &str = "fixture.ledger";

fn pause() {
    std::thread::sleep(std::time::Duration::from_millis(1100));
}

fn verify(repo: &Repo) -> (i32, String) {
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

fn bindings(repo: &Repo) -> Vec<ledger_core::authority::KeyBinding> {
    let store = ledger_core::store::load(repo.path());
    store.log.iter().flat_map(|l| l.file.key_bindings.iter()).cloned().collect()
}

/// The owner (genesis holder, principal A) with key `owner` bound, and the
/// architect (principal B) with their own first key, vouched for by the
/// owner, and an accepted acceptor grant. Returns (repo, A's key, B's key).
fn two_principals() -> (Repo, String, String) {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    let a = repo.keygen("owner");
    repo.use_key(&a);
    repo.ok(&["init", "--namespace", NS, "--external-ref", MANDATE]);
    let b = repo.vouch_for(NS, OTHER, "architect");
    let grant = hand::word(&repo.ok(&["grant", "new", "acceptor", "--to", OTHER, "--scope", &format!("ns:{NS}")]), "grant:");
    hand::commit(&repo, "two principals");
    pause();
    repo.act_as(OTHER);
    repo.use_key(&b);
    repo.ok(&["grant", "accept", &grant]);
    hand::commit(&repo, "the architect's grant");
    pause();
    (repo, a, b)
}

#[test]
fn a_principal_adding_a_key_another_has_open_is_refused_at_filing() {
    let (repo, a, _) = two_principals();
    let before = bindings(&repo).len();
    let refused = repo.refused(&["identity", "add", "--namespace", NS, "--key-file", &format!("{a}.pub")]);
    assert!(refused.contains("[SCHEMA]") && refused.contains("a key belongs to one principal") && refused.contains(OWNER), "{refused}");
    assert_eq!(bindings(&repo).len(), before, "nothing written");
}

/// The architect's own `add` of the owner's key, hand-filed past the verb
/// and signed with the architect's live key.
fn hand_file_the_owners_key_as_the_architects(repo: &Repo, a: &str, b: &str) -> String {
    let stolen = hand::binding(BindingAct::Add, OTHER, OTHER, NS, Some(&format!("{a}.pub")), None, None);
    let id = stolen.id.to_string();
    let ulid = stolen.id.ulid().to_string();
    hand::file(repo, vec![stolen], Vec::new(), &[(&ulid, b)]);
    hand::commit(repo, "the owner's key, filed as the architect's");
    pause();
    id
}

#[test]
fn a_hand_filed_binding_of_another_principals_key_is_a_schema_fault() {
    let (repo, a, b) = two_principals();
    let id = hand_file_the_owners_key_as_the_architects(&repo, &a, &b);
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[SCHEMA]") && text.contains(&id) && text.contains("a key belongs to one principal"), "{text}");
    repo.ok(&["identity", "sync"]);
    let signers = std::fs::read_to_string(repo.signers_path(NS)).unwrap_or_default();
    let blob = std::fs::read_to_string(format!("{a}.pub")).expect("pub").split_whitespace().nth(1).map(str::to_string).expect("blob");
    assert!(signers.lines().filter(|l| l.contains(&blob)).all(|l| l.starts_with(OWNER)), "never trusted as the architect's: {signers}");
}

/// The case ruling 4 prevents: the owner closes the key, and the architect,
/// holding a binding of it, signs with it anyway.
#[test]
fn after_the_owner_closes_the_key_an_acceptance_signed_with_it_as_another_principal_fails() {
    let (repo, a, b) = two_principals();
    hand_file_the_owners_key_as_the_architects(&repo, &a, &b);
    repo.act_as(OWNER);
    repo.use_key(&a);
    let theirs = bindings(&repo).into_iter().find(|k| k.principal.as_str() == OWNER).map(|k| k.id.to_string()).expect("owner's binding");
    repo.ok(&["identity", "revoke", &theirs]);
    let decision = common::decision_id(&repo.ok(&[
        "add", "--set", "ledger-design", "--namespace", NS, "--statement", "Signed with the owner's closed key, as the architect.",
        "--store", "constraint", "--discharge", "analyzer:DEC001",
    ]));
    hand::commit(&repo, "the owner closes their key");
    pause();
    let grant = {
        let store = ledger_core::store::load(repo.path());
        store.log.iter().flat_map(|l| l.file.grants.iter()).find(|g| g.holder.as_str() == OTHER).map(|g| g.id.to_string()).expect("grant")
    };
    let acc = hand::accept(&repo, &hand::HandAccept { decision: &decision, actor: OTHER, at: chrono::Utc::now(), under: Some(&grant), key: Some(&a) });
    hand::commit(&repo, "accepted as the architect with the owner's key");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[L011]") && text.contains(&acc), "the acceptance fails: {text}");
}

#[test]
fn the_genesis_holder_vouching_for_a_closed_key_as_a_first_key_is_refused() {
    let (repo, a, b) = two_principals();
    let architects = bindings(&repo).into_iter().find(|k| k.principal.as_str() == OTHER).map(|k| k.id.to_string()).expect("binding");
    repo.ok(&["identity", "revoke", &architects]);
    hand::commit(&repo, "the architect revokes their key");
    pause();
    repo.act_as(OWNER);
    repo.use_key(&a);
    let refused = repo.refused(&["identity", "add", "--namespace", NS, "--for", OTHER, "--key-file", &format!("{b}.pub")]);
    assert!(refused.contains("[SCHEMA]") && refused.contains("a closed key is never bound again"), "{refused}");
}
