//! A `rotate` is signed by the key it closes (LP-4.12, ruling 53).
//!
//! The verification report's section 5: the owner held K1 and K2, rotated
//! K1 to K3 signing with K2, and both the writer and `verify` accepted it.
//! The rotate then proved nothing about K1. Now the writer refuses the
//! wrong key, and a hand-filed rotate signed by any other live key of the
//! same principal fails `L011`.

mod common;

use common::{hand, Repo};
use ledger_core::authority::BindingAct;

const OWNER: &str = "owner@customer.example";
const NS: &str = "fixture.ledger";

/// A governed namespace; the owner holds K1 (self-bound) and K2. Returns
/// the two private keys and K1's binding id.
fn two_keys() -> (Repo, String, String, String) {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117", "--without-key"]);
    let k1 = repo.bind_own_key(NS, "k1");
    let store = ledger_core::store::load(repo.path());
    let k1_id = store.log.iter().flat_map(|l| l.file.key_bindings.iter()).next().expect("K1").id.to_string();
    let k2 = repo.keygen("k2");
    repo.ok(&["identity", "add", "--namespace", NS, "--key-file", &format!("{k2}.pub")]);
    hand::commit(&repo, "two keys");
    (repo, k1, k2, k1_id)
}

fn verify(repo: &Repo) -> (i32, String) {
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

#[test]
fn the_writer_refuses_a_rotate_signed_by_another_live_key() {
    let (repo, _k1, k2, k1_id) = two_keys();
    repo.use_key(&k2);
    let k3 = repo.keygen("k3");
    let before = repo.log_files();
    let refused = repo.refused(&["identity", "rotate", "--key-file", &format!("{k3}.pub"), &k1_id]);
    assert!(refused.contains("a `rotate` by the key it closes"), "{refused}");
    assert_eq!(repo.log_files(), before, "a refused rotate files nothing");
}

#[test]
fn a_rotate_signed_by_the_key_it_closes_is_filed_and_verifies() {
    let (repo, k1, _k2, k1_id) = two_keys();
    repo.use_key(&k1);
    let k3 = repo.keygen("k3");
    repo.ok(&["identity", "rotate", "--key-file", &format!("{k3}.pub"), &k1_id]);
    hand::commit(&repo, "rotated with K1");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
}

/// The test the prompt asks for: a rotate signed by another live key of the
/// same principal, filed by hand, fails `L011` and is never trusted.
#[test]
fn a_hand_filed_rotate_signed_by_another_live_key_fails_l011() {
    let (repo, _k1, k2, k1_id) = two_keys();
    let k3 = repo.keygen("k3");
    let rotate = hand::binding(BindingAct::Rotate, OWNER, OWNER, NS, Some(&format!("{k3}.pub")), Some(&k1_id), None);
    let ulid = rotate.id.ulid().to_string();
    let id = rotate.id.to_string();
    hand::file(&repo, vec![rotate], Vec::new(), &[(ulid.as_str(), k2.as_str())]);
    hand::commit(&repo, "rotated with K2");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[L011] {id}")), "{text}");
    assert!(text.contains("a `rotate` is signed by the key it closes"), "{text}");
}
