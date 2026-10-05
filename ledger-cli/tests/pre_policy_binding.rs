//! How a key binding can land before its namespace's first policy today,
//! and what fails when it does (the question raised on #99).
//!
//! **Not through the verbs in a fresh store.** `identity add` refuses without
//! a policy in its checkout, and in a fresh store the genesis grant carries
//! the first policy's `at`. So a binding dated before that policy is also
//! before the grant, and D7 refuses it (asserted below). `init --namespace`
//! (#96) binds dated with the policy.
//!
//! **Through the verbs in a later namespace.** The genesis grant is older than
//! the new namespace's first policy, and no verb compares a binding's `at`
//! with that policy's. A binding filed by a clock that runs behind the
//! policy's, and committed with it, is *before* the policy by D6 (same
//! landing, earlier `at`). D7 admits it, but `trust_bindings` skips a binding
//! with no policy in force, so it is never trusted. Nothing names it: the
//! verb's gate passes and `verify` passes. The first act signed with the key
//! fails `L011`, and the verb refuses to write it. By hand, any writer can
//! make the same entry.

mod common;

use chrono::Duration;
use common::{hand, Repo};
use ledger_core::author::{Author, KeyArgs};
use ledger_core::mint::UlidMint;

const OWNER: &str = "owner@customer.example";
const NS: &str = "fixture.ledger";

const SECOND: &str = "second.ledger";

fn public(key: &str) -> (String, String) {
    let text = std::fs::read_to_string(format!("{key}.pub")).expect("pub");
    let mut parts = text.split_whitespace();
    (parts.next().expect("type").to_string(), parts.next().expect("blob").to_string())
}

fn policy_of(repo: &Repo, ns: &str) -> ledger_core::authority::Policy {
    let store = ledger_core::store::load(repo.path());
    store.log.iter().flat_map(|l| l.file.policies.iter()).find(|p| p.namespace == ns).cloned().expect("policy")
}

/// The verb, with its clock `behind` the policy's.
fn bind_behind(repo: &Repo, ns: &str, behind: Duration, key_type: String, blob: String) -> Result<ledger_core::author::Applied, ledger_core::author::AuthorError> {
    let at = policy_of(repo, ns).at - behind;
    let mut author = Author::new(repo.path(), OWNER.parse().expect("id"), at, UlidMint::system());
    author.identity_add(KeyArgs { namespace: ns.into(), key_type, key: blob, principal: None })
}

#[test]
fn in_a_fresh_store_d7_refuses_a_binding_dated_before_the_first_policy() {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117"]);
    let key = repo.keygen("owner");
    repo.use_key(&key);
    let (key_type, blob) = public(&key);
    let refused = bind_behind(&repo, NS, Duration::seconds(1), key_type, blob).expect_err("refused");
    assert!(format!("{refused:?}").contains("D7"), "{refused:?}");
}

#[test]
fn in_a_later_namespace_a_binding_dated_before_its_first_policy_is_never_trusted_and_nothing_names_it() {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117"]);
    repo.bind_own_key(NS, "owner");
    hand::commit(&repo, "governed");
    // The clock behind is still after everything else the store holds.
    std::thread::sleep(std::time::Duration::from_millis(2100));
    repo.ok(&["init", "--namespace", SECOND, "--external-ref", "contract 2026/117"]);
    // The genesis holder's first key in the second namespace — their own
    // add, signed by the key trusted in the first — filed by a clock behind
    // the second policy's.
    let next = repo.keygen("owner-second");
    let (key_type, blob) = public(&next);
    bind_behind(&repo, SECOND, Duration::seconds(1), key_type, blob.clone()).expect("D7 admits it and the verb's gate finds nothing to refuse");
    hand::commit(&repo, "second namespace, governed and bound in one commit");
    let store = ledger_core::store::load(repo.path());
    let binding = store.log.iter().flat_map(|l| l.file.key_bindings.iter()).find(|b| b.namespace == SECOND).cloned().expect("binding");
    assert!(binding.at < policy_of(&repo, SECOND).at, "{binding:?}");

    // Nothing names it.
    let out = repo.ledger(&["verify", "--no-blame"]);
    let text = common::both(&out);
    assert_eq!(out.status.code(), Some(0), "{text}");
    assert!(!text.contains(&binding.id.to_string()), "no finding names the binding: {text}");
    let signers = std::fs::read_to_string(repo.path().join(".decisions/allowed_signers")).unwrap_or_default();
    assert!(!signers.contains(&blob), "it never reaches allowed_signers: {signers}");

    // What fails: the first act signed with the key.
    repo.use_key(&next);
    let grant = hand::word(&repo.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &format!("ns:{SECOND}")]), "grant:");
    repo.ok(&["grant", "accept", &grant]);
    let out = repo.ok(&[
        "add", "--set", "ledger-design", "--namespace", SECOND, "--statement", "Signed with a key bound before the policy.",
        "--store", "constraint", "--discharge", "analyzer:DEC001",
    ]);
    let id = common::decision_id(&out);
    let refused = repo.refused_tty(&["accept", &id]);
    assert!(refused.contains("L011"), "the acceptance it signs fails L011, so `accept` refuses: {refused}");
}

/// What skipping protects today, and the proposed rule ("judged by D7
/// alone") would give up: an unsigned binding, filed by hand for the genesis
/// holder in a namespace with no policy yet, with a key nobody vouched for.
/// D7 admits it (their own add, with a live key elsewhere); only its
/// position keeps it out of the trust root.
#[test]
fn a_hand_filed_unsigned_binding_before_the_first_policy_is_not_trusted_today() {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117"]);
    repo.bind_own_key(NS, "owner");
    hand::commit(&repo, "governed");
    let forged = repo.keygen("forged");
    let b = hand::binding(ledger_core::authority::BindingAct::Add, OWNER, OWNER, SECOND, Some(&format!("{forged}.pub")), None, None);
    hand::file(&repo, vec![b], Vec::new(), &[]);
    hand::commit(&repo, "a binding in a namespace nobody governs yet");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    repo.ok(&["init", "--namespace", SECOND, "--external-ref", "contract 2026/117"]);
    hand::commit(&repo, "second namespace governed");
    let out = repo.ledger(&["verify", "--no-blame"]);
    assert_eq!(out.status.code(), Some(0), "{}", common::both(&out));
    let signers = std::fs::read_to_string(repo.path().join(".decisions/allowed_signers")).unwrap_or_default();
    assert!(!signers.contains(&public(&forged).1), "the forged key is not trusted: {signers}");
}
