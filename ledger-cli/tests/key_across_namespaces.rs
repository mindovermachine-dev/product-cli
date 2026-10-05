//! Key lifecycle across namespaces, as it stands — pinned for #104.
//! A close names one binding and bindings are per namespace, while
//! `init` binds the same key in every namespace it opens. So a key closed in
//! one namespace stays live in another. `init` in a third namespace takes
//! the key as live, but the signature check resolves it to the closed
//! binding and refuses the write.

mod common;

use common::{hand, Repo};

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

#[test]
fn a_key_closed_in_one_namespace_and_bound_in_another_does_not_vouch_for_the_first_binding_in_a_third() {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    let key = repo.keygen("owner");
    repo.use_key(&key);
    repo.ok(&["init", "--namespace", A, "--external-ref", MANDATE]);
    hand::commit(&repo, "A");
    pause();
    repo.ok(&["init", "--namespace", B, "--external-ref", MANDATE]);
    hand::commit(&repo, "B, the same key bound");
    pause();
    let in_a = bindings(&repo, A).first().map(|b| b.id.to_string()).expect("bound in A");
    repo.ok(&["identity", "revoke", &in_a]);
    hand::commit(&repo, "closed in A");
    pause();
    // Today: `init` reads the key as live (bound in B) and binds it in C,
    // but the signature check resolves the key to its binding in A, which is
    // closed — so the write would fail `L011` twice and is refused. The key
    // does not vouch for the first binding in C.
    let refused = repo.refused(&["init", "--namespace", C, "--external-ref", MANDATE]);
    assert!(refused.contains("[L011]") && refused.contains("was closed by"), "{refused}");
    assert!(refused.contains(&in_a), "the close in A is named: {refused}");
    assert!(bindings(&repo, C).is_empty(), "nothing written");
    let store = ledger_core::store::load(repo.path());
    assert!(store.log.iter().flat_map(|l| l.file.policies.iter()).all(|p| p.namespace != C), "no policy in C");
    // And `verify` says nothing about the key being closed in A, live in B.
    let out = repo.ledger(&["verify", "--no-blame"]);
    let text = common::both(&out);
    assert_eq!(out.status.code(), Some(0), "{text}");
    assert!(!text.contains(&in_a), "no notice names the split: {text}");
}
