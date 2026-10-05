//! A role takes effect only from its own position (D6).
//!
//! `Authority::as_of` admits a role file only when it landed no later than
//! the act being judged. A role file carries no signed `at`, so landing
//! alone decides: its `created_at` date plays no part. An act that landed
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
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117", "--without-key"]);
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

/// Landing alone decides: a role and the act that uses it, landed in one
/// commit, stand together whatever date the role file states.
#[test]
fn a_role_and_an_act_landed_in_the_same_commit_pass_whatever_the_role_is_dated() {
    for created_at in ["2099-12-31", "2000-01-01"] {
        let repo = Repo::with_identity(OWNER);
        repo.declare();
        repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117", "--without-key"]);
        let role = repo.path().join(".decisions/roles/acceptor.yml");
        let text = std::fs::read_to_string(&role).expect("role");
        let dated: String = text
            .lines()
            .map(|l| if l.starts_with("created_at:") { format!("created_at: {created_at}") } else { l.to_string() })
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&role, dated + "\n").expect("date the role");
        repo.bind_own_key(NS, "owner");
        let grant = hand::word(&repo.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &format!("ns:{NS}")]), "grant:");
        repo.ok(&["grant", "accept", &grant]);
        let id = repo.add("Money is decimal.", &[]);
        repo.ok_tty(&["accept", &id]);
        hand::commit(&repo, "the role and the act, one commit");
        let (code, text) = verify(&repo);
        assert_eq!(code, 0, "created_at {created_at}: {text}");
    }
}
