//! `allowed_signers` is one derived file per namespace (LP-4.10, ruling 47; PRD §3.4; issue 7).
//!
//! Each namespace's file holds that namespace's lines, with `valid-before`
//! from closes in that namespace only; `[SIGNERS]` compares each committed
//! file with that namespace's trusted bindings; a namespace that binds
//! nothing has no file, and a stale one is removed when the files are
//! regenerated.

mod common;

use common::{hand, Repo};

const OWNER: &str = "owner@customer.example";
const MANDATE: &str = "contract 2026/117";
const A: &str = "a.ledger";
const B: &str = "b.ledger";

fn verify(repo: &Repo) -> (i32, String) {
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

/// Two namespaces, the owner's key self-bound in each by `init`.
fn two_bound() -> (Repo, String) {
    let repo = Repo::with_identity(OWNER);
    let key = repo.keygen("owner");
    repo.use_key(&key);
    repo.ok(&["init", "--namespace", A, "--external-ref", MANDATE]);
    repo.ok(&["init", "--namespace", B, "--external-ref", MANDATE]);
    hand::commit(&repo, "two namespaces, one key bound in each");
    (repo, key)
}

fn lines(text: &str) -> Vec<&str> {
    text.lines().filter(|l| !l.starts_with('#')).collect()
}

#[test]
fn each_namespace_has_its_own_file_holding_its_own_lines() {
    let (repo, _) = two_bound();
    for ns in [A, B] {
        assert!(repo.signers_path(ns).is_file(), "`{ns}` has its file");
        let text = repo.signers(ns);
        let own = lines(&text);
        assert_eq!(own.len(), 1, "{ns}: {own:?}");
        assert!(own[0].contains(&format!("namespaces=\"ledger-accept@{ns}\"")), "{ns}: {}", own[0]);
    }
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
}

#[test]
fn a_close_in_another_namespace_does_not_reach_the_file() {
    let (repo, _) = two_bound();
    let in_a = {
        let store = ledger_core::store::load(repo.path());
        store.log.iter().flat_map(|l| l.file.key_bindings.iter()).find(|b| b.namespace == A).map(|b| b.id.to_string()).expect("binding in A")
    };
    let before = repo.signers(B);
    repo.ok(&["identity", "revoke", &in_a]);
    hand::commit(&repo, "closed in A");
    assert!(lines(&repo.signers(A)).iter().all(|l| l.contains("valid-before=")), "{}", repo.signers(A));
    assert_eq!(repo.signers(B), before, "B's file is byte-identical: the close in A is A's");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
}

#[test]
fn a_file_in_a_namespace_that_binds_nothing_fails_signers_and_sync_removes_it() {
    let repo = Repo::with_identity(OWNER);
    let key = repo.keygen("owner");
    repo.use_key(&key);
    repo.ok(&["init", "--namespace", A, "--external-ref", MANDATE]);
    repo.declare_in(B);
    // A file nobody derived, in a namespace with no binding.
    let stale = repo.signers_path(B);
    std::fs::create_dir_all(stale.parent().expect("dir")).expect("mkdir");
    std::fs::write(&stale, format!("{}{OWNER} namespaces=\"ledger-accept@{B}\",valid-after=\"20261001000000Z\" ssh-ed25519 AAAA\n", ledger_core::authority::signers::HEADER)).expect("write");
    hand::commit(&repo, "a stale allowed_signers in B");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[SIGNERS]") && text.contains(&format!("committed in `{B}` but the log binds no key there")), "{text}");
    let out = repo.ok(&["identity", "sync"]);
    assert!(out.contains(&format!("removed ns/{B}/allowed_signers")) && out.contains(&format!("regenerated ns/{A}/allowed_signers")), "{out}");
    assert!(!stale.exists(), "the stale file is gone");
    assert!(repo.signers_path(A).is_file(), "A's file stands");
    hand::commit(&repo, "synced");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
}

#[test]
fn a_namespace_whose_only_binding_is_untrusted_has_no_file_after_sync() {
    let repo = Repo::with_identity(OWNER);
    let key = repo.keygen("owner");
    repo.use_key(&key);
    repo.ok(&["init", "--namespace", A, "--external-ref", MANDATE]);
    repo.declare_in(B);
    // A forged binding in B, hand-filed with no policy to govern it: a
    // schema fault, never trusted — so B derives no file, and one written
    // beside it is stale.
    let forged = repo.keygen("forged");
    let b = hand::binding(ledger_core::authority::BindingAct::Add, OWNER, OWNER, B, Some(&format!("{forged}.pub")), None, None);
    hand::file(&repo, vec![b], Vec::new(), &[]);
    std::fs::write(repo.signers_path(B), ledger_core::authority::signers::HEADER).expect("write");
    let out = repo.ok(&["identity", "sync"]);
    assert!(out.contains(&format!("removed ns/{B}/allowed_signers")), "{out}");
    assert!(!repo.signers_path(B).exists());
}
