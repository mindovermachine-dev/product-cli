//! A close ends the key in its own namespace (ruling 47, LP-6.32; ruling 69).
//!
//! The same key bound in two namespaces is two keys: closing it in `A`
//! leaves `B`'s bindings, acceptances, review items and `allowed_signers`
//! untouched (AC-47, second bullet), and `verify` reports the split as a
//! notice, never a finding. Within a namespace the rules of 2026-10-06
//! stand: a key is bound once there, a closed key is never bound again
//! there, and every check asks over all bindings of the key there. The
//! writer's close in every namespace it holds is issue 8's.

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

/// The key self-bound in A at `init`, then self-bound again in B by `init`
/// B (ruling 47: every namespace is opened as the first is).
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
    hand::commit(&repo, "B, the same key self-bound there");
    pause();
    (repo, key)
}

/// The key self-bound in B first, then in A by `identity add` — the genesis
/// holder's first key in A, so self-bound there too.
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
    assert!(bindings(&repo, A)[0].self_bound, "the genesis holder's first key in A is self-bound there");
    (repo, key)
}

/// Revoke the key's binding in A, then show that B is untouched: the key
/// still signs there, the split is a notice, and a third namespace opens
/// with the same key self-bound.
fn closed_in_a_stays_closed_in_a(repo: &Repo, key: &str) {
    let in_a = bindings(repo, A)[0].id.to_string();
    let close = hand::word(&repo.ok(&["identity", "revoke", &in_a]), "key:");
    hand::commit(repo, "closed in A");
    pause();
    // A's line ends at the close; B's does not.
    assert!(repo.signers(A).lines().filter(|l| !l.starts_with('#')).all(|l| l.contains("valid-before=")), "{}", repo.signers(A));
    assert!(repo.signers(B).lines().filter(|l| !l.starts_with('#')).all(|l| !l.contains("valid-before=")), "{}", repo.signers(B));
    // The key still signs in B.
    let id = decision(repo, B, "Signed in B after the key was closed in A.");
    repo.ok_tty(&["accept", &id]);
    hand::commit(repo, "accepted in B with the key closed in A");
    let (code, text) = verify(repo, &[]);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains(&format!("of {OWNER} is closed in `{A}` and open in `{B}`")), "the notice of ruling 69: {text}");
    // In A it signs nothing: the verb names the close, the gate fails a
    // hand-filed acceptance.
    repo.declare_in(A);
    acceptor(repo, A);
    let in_a_only = decision(repo, A, "Signed in A after the close.");
    hand::commit(repo, "a decision in A");
    let refused = refusal(repo, &["accept", &in_a_only]);
    assert!(refused.contains(&format!("live key in `{A}`: it was closed in `{A}` by {close}")), "{refused}");
    let text = hand_signed_finding(repo, A, &in_a_only, key);
    assert!(text.contains(&format!("closed in `{A}` by {close}")), "{text}");
    // And a third namespace opens with the same key, self-bound there.
    repo.ok(&["init", "--namespace", C, "--external-ref", MANDATE]);
    assert!(bindings(repo, C)[0].self_bound, "self-bound in C");
}

#[test]
fn bound_in_a_then_b_a_close_in_a_leaves_b_open_and_is_a_notice() {
    let (repo, key) = a_then_b();
    closed_in_a_stays_closed_in_a(&repo, &key);
}

#[test]
fn bound_in_b_then_a_a_close_in_a_leaves_b_open_and_is_a_notice() {
    let (repo, key) = b_then_a();
    closed_in_a_stays_closed_in_a(&repo, &key);
}

/// AC-47, second bullet: a rotate in A leaves B's acceptances, review items
/// and `allowed_signers` unchanged.
#[test]
fn a_rotate_in_a_leaves_bs_acceptances_review_items_and_allowed_signers_unchanged() {
    let (repo, key) = a_then_b();
    let id = decision(&repo, B, "Accepted in B before the rotate in A.");
    repo.ok_tty(&["accept", &id]);
    repo.ok(&["policy", "set", "--namespace", B, "--reaccept-within-days", "30"]);
    hand::commit(&repo, "accepted in B");
    pause();
    let signers_b = repo.signers(B);
    let (code, before) = verify(&repo, &["--today", "2099-01-01"]);
    assert_eq!(code, 0, "{before}");
    let in_a = bindings(&repo, A)[0].id.to_string();
    let next = repo.keygen("owner-next");
    repo.ok(&["identity", "rotate", &in_a, "--key-file", &format!("{next}.pub")]);
    hand::commit(&repo, "rotated in A");
    let (code, text) = verify(&repo, &["--today", "2099-01-01"]);
    assert_eq!(code, 0, "no L012, however late: {text}");
    assert!(!text.contains("need re-acceptance") && !text.contains("[L012]"), "no review item in B: {text}");
    assert_eq!(repo.signers(B), signers_b, "B's allowed_signers is byte-identical");
    assert!(text.contains(&format!("of {OWNER} is closed in `{A}` and open in `{B}`")), "{text}");
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

/// Carried: no agent identity and no non-interactive session produces a
/// key close, in either namespace.
#[test]
fn no_agent_identity_and_no_non_interactive_session_produces_a_key_close() {
    let (repo, _) = a_then_b();
    let before = repo.log_files();
    for ns in [A, B] {
        let id = bindings(&repo, ns)[0].id.to_string();
        let piped = repo.piped(&["identity", "revoke", &id]);
        assert_ne!(piped.status.code(), Some(0), "{}", common::both(&piped));
        for agent in ["claude@anthropic.com", "noreply@anthropic.com", "github-actions@github.com"] {
            repo.act_as(agent);
            let out = repo.tty(&["identity", "revoke", &id]);
            assert_ne!(out.status.code(), Some(0), "{agent} in `{ns}`: {}", common::both(&out));
        }
        repo.act_as(OWNER);
    }
    assert_eq!(repo.log_files(), before, "nothing filed");
    let in_a = bindings(&repo, A)[0].id.to_string();
    repo.ok(&["identity", "revoke", &in_a]);
    assert_eq!(repo.log_files().len(), before.len() + 1, "the holder, at a terminal, closes");
}
