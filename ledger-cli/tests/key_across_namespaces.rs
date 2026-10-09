//! A close ends the key, not the binding (ruled 2026-10-06, #104).
//!
//! Closing any binding of a principal's key closes that key in every
//! namespace of the store, from the close's position (D6), and every
//! signature check considers all bindings of the matched key, not the first
//! one it matched. A key already open in a namespace is never bound there
//! twice. Whichever order the bindings were filed in, a closed key signs
//! nothing and vouches for nothing.

mod common;

use common::{hand, Repo};
use ledger_core::authority::BindingAct;

const OWNER: &str = "owner@customer.example";
const MANDATE: &str = "contract 2026/117";
const A: &str = "a.ledger";
const B: &str = "b.ledger";
const C: &str = "c.ledger";

fn bindings(repo: &Repo, ns: &str) -> Vec<ledger_core::authority::KeyBinding> {
    let store = ledger_core::store::load(repo.path());
    store.log.iter().flat_map(|l| l.file.key_bindings.iter()).filter(|b| b.namespace == ns).cloned().collect()
}

fn pause() {
    std::thread::sleep(std::time::Duration::from_millis(1100));
}

/// Every namespace's derived `allowed_signers`, concatenated.
fn signers(repo: &Repo) -> String {
    ledger_core::layout::namespaces(repo.path()).into_iter().map(|ns| repo.signers(&ns)).collect()
}

fn verify(repo: &Repo, extra: &[&str]) -> (i32, String) {
    let mut args = vec!["verify", "--no-blame"];
    args.extend_from_slice(extra);
    let out = repo.ledger(&args);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

/// An acceptor grant for the owner in `ns`, accepted.
fn acceptor(repo: &Repo, ns: &str) {
    let grant = hand::word(&repo.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &format!("ns:{ns}")]), "grant:");
    repo.ok(&["grant", "accept", &grant]);
}

/// A signing verb under a terminal that must be refused (exit 1).
fn refusal(repo: &Repo, args: &[&str]) -> String {
    let out = repo.tty(args);
    let text = common::both(&out);
    assert_eq!(out.status.code(), Some(1), "{args:?} is refused: {text}");
    text
}

/// What `verify` says of an acceptance hand-filed in `ns` now, signed with
/// `key` — past the verb's refusal, the gate's own finding.
fn hand_signed_finding(repo: &Repo, ns: &str, decision: &str, key: &str) -> String {
    let store = ledger_core::store::load(repo.path());
    let grant = store.log.iter().flat_map(|l| l.file.grants.iter()).find(|g| g.role == "acceptor" && g.scope.to_string() == format!("ns:{ns}")).map(|g| g.id.to_string()).expect("acceptor grant");
    let acc = hand::accept(repo, &hand::HandAccept { decision, actor: OWNER, at: chrono::Utc::now(), under: Some(&grant), key: Some(key) });
    hand::commit(repo, "hand-signed with the closed key");
    let (code, text) = verify(repo, &[]);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[L011]") && text.contains(&acc), "{text}");
    text
}

fn decision(repo: &Repo, ns: &str, statement: &str) -> String {
    let out = repo.ok(&["add", "--set", "ledger-design", "--namespace", ns, "--statement", statement, "--store", "constraint", "--discharge", "analyzer:DEC001"]);
    common::decision_id(&out)
}

/// The key bound in A at `init`, then in B by `init` B: A's binding first.
fn a_then_b() -> (Repo, String) {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    let key = repo.keygen("owner");
    repo.use_key(&key);
    repo.ok(&["init", "--namespace", A, "--external-ref", MANDATE]);
    hand::commit(&repo, "A");
    pause();
    repo.ok(&["init", "--namespace", B, "--external-ref", MANDATE]);
    repo.declare_in(B);
    acceptor(&repo, B);
    hand::commit(&repo, "B, the same key bound");
    pause();
    (repo, key)
}

/// The key bound in B first (self-bound), then in A by `identity add`.
fn b_then_a() -> (Repo, String) {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    let key = repo.keygen("owner");
    repo.use_key("/nonexistent/owner-key");
    repo.ok(&["init", "--namespace", A, "--external-ref", MANDATE, "--without-key"]);
    hand::commit(&repo, "A, unbound");
    pause();
    repo.use_key(&key);
    repo.ok(&["init", "--namespace", B, "--external-ref", MANDATE]);
    repo.declare_in(B);
    acceptor(&repo, B);
    hand::commit(&repo, "B");
    pause();
    repo.ok(&["identity", "add", "--namespace", A, "--key-file", &format!("{key}.pub")]);
    hand::commit(&repo, "A, the same key bound second");
    pause();
    assert!(bindings(&repo, B)[0].id < bindings(&repo, A)[0].id, "B's binding is filed first");
    (repo, key)
}

/// Revoke the key's binding in A, then show it is closed in B as well and
/// vouches for no first binding in C.
fn closed_in_a_is_closed_everywhere(repo: &Repo, key: &str) {
    let in_a = bindings(repo, A)[0].id.to_string();
    let close = hand::word(&repo.ok(&["identity", "revoke", &in_a]), "key:");
    hand::commit(repo, "closed in A");
    pause();
    // Every allowed_signers line of the key ends at the close — B's too.
    for line in signers(repo).lines().filter(|l| !l.starts_with('#')) {
        assert!(line.contains("valid-before="), "every line of the closed key ends: {line}");
    }
    // An acceptance in B signed with the key is refused by the verb, naming
    // the close and its namespace.
    let id = decision(repo, B, "Signed in B after the key was closed in A.");
    hand::commit(repo, "a decision in B");
    let refused = refusal(repo, &["accept", &id]);
    assert!(refused.contains(&format!("live key in `{B}`: it was closed in `{A}` by {close}")), "{refused}");
    // And it vouches for no first binding in a new namespace.
    let out = repo.ledger(&["init", "--namespace", C, "--external-ref", MANDATE]);
    let text = common::both(&out);
    assert_eq!(out.status.code(), Some(2), "{text}");
    assert!(text.contains("no live key"), "{text}");
    assert!(bindings(repo, C).is_empty(), "nothing bound in C");
    let (code, text) = verify(repo, &[]);
    assert_eq!(code, 0, "{text}");
    // Filed by hand past the verb, the gate fails it: the finding names the
    // namespace it is refused in, the namespace of the close, and the close.
    let text = hand_signed_finding(repo, B, &id, key);
    assert!(text.contains(&format!("closed in `{A}` by {close}")) && text.contains(&format!("signs nothing in `{B}`")), "{text}");
}

#[test]
fn bound_in_a_then_b_and_closed_in_a_the_key_is_closed_in_b_and_vouches_in_no_third() {
    let (repo, key) = a_then_b();
    closed_in_a_is_closed_everywhere(&repo, &key);
}

#[test]
fn bound_in_b_then_a_and_closed_in_a_the_key_is_closed_in_b_and_vouches_in_no_third() {
    let (repo, key) = b_then_a();
    closed_in_a_is_closed_everywhere(&repo, &key);
}

/// A store governed in A with the owner's key bound, an acceptor grant, and
/// a second binding of the same key hand-filed beside the first — signed by
/// it, as the holder's own further `add`.
fn bound_twice() -> (Repo, String, String, String) {
    let repo = Repo::with_identity(OWNER);
    repo.declare_in(A);
    let key = repo.keygen("owner");
    repo.use_key(&key);
    repo.ok(&["init", "--namespace", A, "--external-ref", MANDATE]);
    acceptor(&repo, A);
    hand::commit(&repo, "A");
    pause();
    let first = bindings(&repo, A)[0].id.to_string();
    let twice = hand::binding(BindingAct::Add, OWNER, OWNER, A, Some(&format!("{key}.pub")), None, None);
    let second = twice.id.to_string();
    let ulid = twice.id.ulid().to_string();
    hand::file(&repo, vec![twice], Vec::new(), &[(&ulid, &key)]);
    hand::commit(&repo, "the same key, hand-filed again");
    pause();
    (repo, key, first, second)
}

#[test]
fn binding_a_key_already_open_in_the_namespace_is_refused_at_filing() {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    let key = repo.keygen("owner");
    repo.use_key(&key);
    repo.ok(&["init", "--namespace", A, "--external-ref", MANDATE]);
    let refused = repo.refused(&["identity", "add", "--namespace", A, "--key-file", &format!("{key}.pub")]);
    assert!(refused.contains("[SCHEMA]") && refused.contains("already has this key open"), "{refused}");
    assert_eq!(bindings(&repo, A).len(), 1, "nothing written");
}

#[test]
fn a_hand_filed_second_binding_of_an_open_key_is_a_schema_fault_and_never_trusted() {
    let (repo, _, first, second) = bound_twice();
    let (code, text) = verify(&repo, &[]);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[SCHEMA]") && text.contains(&second) && text.contains(&format!("already has this key open in `{A}` ({first})")), "{text}");
}

/// The attack: a second binding of a live key is hand-filed, the holder
/// revokes the first, and the key would still sign through the second.
#[test]
fn revoking_the_first_of_two_bindings_closes_the_key_and_a_later_acceptance_fails() {
    let (repo, key, first, _) = bound_twice();
    let close = hand::word(&repo.ok(&["identity", "revoke", &first]), "key:");
    hand::commit(&repo, "the holder revokes their key");
    pause();
    let id = decision(&repo, A, "Signed with the key after its revoke.");
    hand::commit(&repo, "a decision");
    let refused = refusal(&repo, &["accept", &id]);
    assert!(refused.contains(&format!("it was closed in `{A}` by {close}")), "{refused}");
    let text = hand_signed_finding(&repo, A, &id, &key);
    assert!(text.contains(&format!("closed in `{A}` by {close}")), "{text}");
}

#[test]
fn revoking_the_second_of_two_bindings_closes_the_key_as_well() {
    let (repo, key, _, second) = bound_twice();
    let close = hand::word(&repo.ok(&["identity", "revoke", &second]), "key:");
    hand::commit(&repo, "the duplicate revoked");
    pause();
    let id = decision(&repo, A, "Signed with the key after a revoke of its duplicate.");
    hand::commit(&repo, "a decision");
    let refused = refusal(&repo, &["accept", &id]);
    assert!(refused.contains(&format!("it was closed in `{A}` by {close}")), "{refused}");
    let text = hand_signed_finding(&repo, A, &id, &key);
    assert!(text.contains(&format!("closed in `{A}` by {close}")), "{text}");
}

#[test]
fn an_acceptance_before_a_close_in_another_namespace_is_a_review_item_then_l012() {
    let (repo, _) = a_then_b();
    let id = decision(&repo, B, "Accepted in B before the key closes in A.");
    repo.ok_tty(&["accept", &id]);
    let acc = {
        let store = ledger_core::store::load(repo.path());
        store.log.iter().flat_map(|l| l.file.acceptances.iter()).map(|a| a.id.to_string()).next_back().expect("acceptance")
    };
    repo.ok(&["policy", "set", "--namespace", B, "--reaccept-within-days", "30"]);
    hand::commit(&repo, "accepted in B");
    pause();
    let in_a = bindings(&repo, A)[0].id.to_string();
    repo.ok(&["identity", "revoke", &in_a]);
    hand::commit(&repo, "closed in A");
    let (code, text) = verify(&repo, &[]);
    assert_eq!(code, 0, "a review item, not a failure: {text}");
    assert!(text.contains("need re-acceptance") && text.contains(&acc), "{text}");
    let (code, text) = verify(&repo, &["--today", "2099-01-01"]);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[L012]") && text.contains(&acc), "{text}");
}
