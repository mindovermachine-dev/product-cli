//! Every entity under `ns/<ns>/` belongs to `<ns>` (LP-3.33, ruling 45; AC-45).
//!
//! A file's namespace is its directory. A hand-written file under `ns/A/`
//! holding an entity of B — a decision, an acceptance, a binding, a policy,
//! or a revocation of B's acceptance — is a `SCHEMA` fault naming the
//! entity. The verbs never write one: a change-set is homed by the one
//! namespace its entities name.

mod common;

use common::{hand, Repo};

const A: &str = "fixture.ledger";
const B: &str = "other.ledger";

fn verify(repo: &Repo) -> (i32, String) {
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

/// A store with a decision of A and a decision of B, each in its own
/// directory and set.
fn two_namespaces() -> (Repo, String, String) {
    let repo = Repo::human();
    repo.declare_in(A);
    repo.declare_in(B);
    let a = repo.add("A's decision.", &[]);
    let out = repo.ok(&["add", "--set", "ledger-design", "--namespace", B, "--statement", "B's decision.", "--store", "constraint", "--discharge", "analyzer:DEC001"]);
    let b = common::decision_id(&out);
    (repo, a, b)
}

/// The log file holding `id`, moved from its directory into `into`'s.
fn misfile(repo: &Repo, id: &str, into: &str) {
    let from = hand::file_holding(repo, id);
    let target = repo.log_dir(into).join(from.file_name().expect("name"));
    std::fs::create_dir_all(repo.log_dir(into)).expect("dir");
    std::fs::rename(&from, &target).expect("move");
}

#[test]
fn a_decision_of_b_filed_under_a_is_a_schema_fault_naming_it() {
    let (repo, _, b) = two_namespaces();
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "each in its own directory: {text}");
    misfile(&repo, &b, A);
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[SCHEMA] {b}: is a decision of `{B}`, but the file is under `{A}`'s directory")), "{text}");
    assert!(text.contains("ruling 45"), "{text}");
}

#[test]
fn an_acceptance_of_b_filed_under_a_is_a_schema_fault() {
    let (repo, _, b) = two_namespaces();
    let acc = hand::accept(&repo, &hand::HandAccept { decision: &b, actor: "fixture-human@example", at: chrono::Utc::now(), under: None, key: None });
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
    misfile(&repo, &acc, A);
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[SCHEMA] {acc}: accepts a decision of `{B}`")), "{text}");
}

#[test]
fn a_revocation_under_a_of_an_acceptance_of_b_is_a_schema_fault() {
    let (repo, _, b) = two_namespaces();
    let acc = hand::accept(&repo, &hand::HandAccept { decision: &b, actor: "fixture-human@example", at: chrono::Utc::now(), under: None, key: None });
    // A legacy-shape revocation of B's acceptance, hand-filed under A.
    let text = format!(
        "format: 1\nid: cs:01K2C4YQJ3F8M0PT5W7NZ9RDZ5\ncreated_at: 2026-08-10T09:30:00Z\ncreated_by: fixture-human@example\nrevocations:\n- acceptance: {acc}\n  at: 2026-08-10T09:30:00Z\n  by: fixture-human@example\n  reason: filed in the wrong directory\n"
    );
    std::fs::write(repo.log_dir(A).join("01K2C4YQJ3F8M0PT5W7NZ9RDZ5.yml"), text).expect("write");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[SCHEMA]") && text.contains(&format!("revokes an acceptance of `{B}`, but the file is under `{A}`'s directory")), "{text}");
    // The same revocation under B is in its place.
    std::fs::rename(repo.log_dir(A).join("01K2C4YQJ3F8M0PT5W7NZ9RDZ5.yml"), repo.log_dir(B).join("01K2C4YQJ3F8M0PT5W7NZ9RDZ5.yml")).expect("move");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
}

#[test]
fn a_binding_and_a_policy_of_b_filed_under_a_are_schema_faults() {
    let owner = "owner@customer.example";
    let repo = Repo::with_identity(owner);
    repo.declare_in(A);
    repo.declare_in(B);
    let key = repo.keygen("owner");
    repo.use_key(&key);
    repo.ok(&["init", "--namespace", A, "--external-ref", "contract 2026/117"]);
    repo.ok(&["init", "--namespace", B, "--external-ref", "contract 2026/117"]);
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
    // B's opening change-set, holding its policy and binding, moved under A;
    // its sidecars stay under B, so they are named too.
    let store = ledger_core::store::load(repo.path());
    let policy = store.log.iter().flat_map(|l| l.file.policies.iter()).find(|p| p.namespace == B).expect("policy").id.to_string();
    misfile(&repo, &policy, A);
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[SCHEMA] {policy}: is the policy of `{B}`")), "{text}");
    assert!(text.contains("binds a key in `") || text.contains("signs an entity of `"), "{text}");
}
