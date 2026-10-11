//! A sidecar on an act no policy governs is a schema fault (ruling 52).
//!
//! The verification report's section 1: a sidecar that is not a signature,
//! on an acceptance in a namespace with no policy (1a) and on one filed
//! before its namespace's first policy (1b). Neither act needs a signature
//! (D5 (c)), so the sidecar was never opened, the store was conformant, and
//! the export asserted a signature that is not there.

mod common;

use common::{hand, Repo};

const OWNER: &str = "owner@customer.example";
const NS: &str = "fixture.ledger";

/// Accept a fresh decision in an ungoverned namespace, then give the
/// acceptance a sidecar that is not a signature. Returns the sidecar's name.
fn accepted_with_a_false_sidecar(repo: &Repo) -> String {
    repo.declare();
    let id = repo.add("Money is decimal.", &[]);
    repo.ok_tty(&["accept", &id]);
    let store = ledger_core::store::load(repo.path());
    let acc = store.log.iter().flat_map(|l| l.file.acceptances.iter()).next().expect("acceptance").id.ulid().to_string();
    let dir = repo.sig_dir(NS);
    std::fs::create_dir_all(&dir).expect("sig dir");
    let name = format!("{acc}.ssh.sig");
    std::fs::write(dir.join(&name), "not a signature\n").expect("sidecar");
    name
}

fn verify(repo: &Repo) -> (i32, String) {
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

/// 1a: the namespace has no policy.
#[test]
fn a_sidecar_in_a_namespace_with_no_policy_is_a_schema_fault() {
    let repo = Repo::with_identity(OWNER);
    let sidecar = accepted_with_a_false_sidecar(&repo);
    hand::commit(&repo, "an acceptance with a sidecar, ungoverned");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[SCHEMA] sig/{sidecar}: signs acc:")), "{text}");
    assert!(text.contains("an act no policy governs"), "{text}");
}

/// 1b: the acceptance lands before its namespace's first policy (D5 (c)).
#[test]
fn a_sidecar_on_an_act_before_the_first_policy_is_a_schema_fault() {
    let repo = Repo::with_identity(OWNER);
    let sidecar = accepted_with_a_false_sidecar(&repo);
    hand::commit(&repo, "pre-policy");
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117", "--without-key"]);
    repo.bind_own_key(NS, "owner");
    hand::commit(&repo, "policy");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[SCHEMA] sig/{sidecar}")), "{text}");
}

/// The control: the same acceptance with no sidecar stays conformant.
#[test]
fn an_ungoverned_acceptance_with_no_sidecar_stays_conformant() {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    let id = repo.add("Money is decimal.", &[]);
    repo.ok_tty(&["accept", &id]);
    hand::commit(&repo, "ungoverned");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
}
