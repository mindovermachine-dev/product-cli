//! A role takes effect only from its own position (D6).
//!
//! `Authority::as_of` admits a role file only when it is not after the act
//! being judged: landed no later, and created no later. An act that landed
//! before the role its grant names was made under a role that did not yet
//! exist, and fails `A006`.

mod common;

use common::{hand, Repo};

const OWNER: &str = "owner@customer.example";
const NS: &str = "fixture.ledger";

fn verify(repo: &Repo) -> (i32, String) {
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

#[test]
fn an_act_that_landed_before_the_role_its_grant_names_fails_a006() {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117"]);
    repo.bind_own_key(NS, "owner");
    let grant = hand::word(&repo.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &format!("ns:{NS}")]), "grant:");
    repo.ok(&["grant", "accept", &grant]);
    let id = repo.add("Money is decimal.", &[]);
    repo.ok_tty(&["accept", &id]);
    // Land everything but the `acceptor` role file, then the role file.
    repo.git(&["add", "-A"]);
    repo.git(&["reset", "-q", "--", ".decisions/roles/acceptor.yml"]);
    repo.git(&["commit", "-q", "-m", "the acceptance, before its role"]);
    hand::commit(&repo, "the role, after");
    let store = ledger_core::store::load(repo.path());
    let acc = store.log.iter().flat_map(|l| l.file.acceptances.iter()).map(|a| a.id.to_string()).next().expect("acc");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[A006] {acc}")), "{text}");
}
