//! Landed entities are immutable, and each lands on its own (D6, #70).
//!
//! An entity's landing is the first first-parent commit whose version of
//! its file holds it: appending to a landed file does not backdate the new
//! entry to the file's landing. Removing or editing a landed entity — a key
//! close, a revocation — fails `L007`.

mod common;

use chrono::{Duration, DurationRound};
use common::hand::{self, HandAccept};
use common::Repo;

const OWNER: &str = "owner@customer.example";
const NS: &str = "fixture.ledger";

fn governed() -> (Repo, String) {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117", "--without-key"]);
    let key = repo.bind_own_key(NS, "owner");
    let grant = hand::word(&repo.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &format!("ns:{NS}")]), "grant:");
    repo.ok(&["grant", "accept", &grant]);
    hand::commit(&repo, "governed");
    (repo, key)
}

fn verify(repo: &Repo) -> (i32, String) {
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

fn acceptor_grant(repo: &Repo) -> String {
    let store = ledger_core::store::load(repo.path());
    store.log.iter().flat_map(|l| l.file.grants.iter()).find(|g| g.role == "acceptor").map(|g| g.id.to_string()).expect("grant")
}

fn first_binding(repo: &Repo) -> String {
    let store = ledger_core::store::load(repo.path());
    store.log.iter().flat_map(|l| l.file.key_bindings.iter()).map(|b| b.id.to_string()).next().expect("binding")
}

/// The owner's first key closed, by a second key; committed. Returns the
/// closed binding's id, the close's id, and the second key's binding time —
/// after every enabling entry and two seconds before the close.
fn closed_first_key(repo: &Repo) -> (String, String, chrono::DateTime<chrono::Utc>) {
    let binding = first_binding(repo);
    let spare = repo.keygen("owner-next");
    repo.ok(&["identity", "add", "--namespace", NS, "--key-file", &format!("{spare}.pub")]);
    let bound = hand::last_binding_at(repo);
    std::thread::sleep(std::time::Duration::from_millis(2100));
    repo.use_key(&spare);
    let out = repo.ok(&["identity", "revoke", &binding]);
    hand::commit(repo, "closed");
    (binding, hand::word(&out, "key:"), bound)
}

#[test]
fn an_acceptance_appended_to_a_landed_file_after_the_close_fails_l011() {
    let (repo, key) = governed();
    let id = repo.add("Money is decimal.", &[]);
    hand::commit(&repo, "filed");
    let landed = hand::file_holding(&repo, &first_binding(&repo));
    let (_, _, bound) = closed_first_key(&repo);
    // Signed with the closed key, dated inside its window, appended to the
    // file that landed before the close.
    let at = bound.duration_trunc(Duration::seconds(1)).unwrap_or(bound) + Duration::seconds(1);
    let acc = hand::accept(&repo, &HandAccept { decision: &id, actor: OWNER, at, under: Some(&acceptor_grant(&repo)), key: Some(&key) });
    hand::append_into(&repo, &acc, &landed);
    hand::commit(&repo, "appended to a landed file");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[L011]") && text.contains("not dated and landed before the close"), "{text}");
    assert!(!text.contains("need re-acceptance"), "it lands where it was appended, not with its file: {text}");
    assert!(!text.contains("has changed") && !text.contains("is gone"), "appending changes no landed entity: {text}");
    // The append needs format 7 and raised the declaration in the same step,
    // which is not a correction (LP-3.16, ruling 58): `L007` on the
    // declaration, beside the `L011` the appended act earns.
    assert!(text.contains("[L007]") && text.contains("its format declaration was 6 at"), "{text}");
}

#[test]
fn a_deleted_key_close_fails_l007() {
    let (repo, _) = governed();
    let (_, close, _) = closed_first_key(&repo);
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
    std::fs::remove_file(hand::file_holding(&repo, &close)).expect("remove");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "uncommitted: {text}");
    assert!(text.contains(&format!("[L007] {close}")) && text.contains("is gone"), "{text}");
    hand::commit(&repo, "deleted the close");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "committed: {text}");
    assert!(text.contains(&format!("[L007] {close}")) && text.contains("reopen the key"), "{text}");
}

#[test]
fn a_deleted_revocation_fails_l007() {
    let (repo, _) = governed();
    let id = repo.add("Money is decimal.", &[]);
    repo.ok_tty(&["accept", &id]);
    hand::commit(&repo, "accepted");
    let store = ledger_core::store::load(repo.path());
    let acc = store.log.iter().flat_map(|l| l.file.acceptances.iter()).map(|a| a.id.to_string()).next().expect("acc");
    let out = repo.ok_tty(&["revoke", &acc, "--reason", "premature"]);
    assert!(out.contains(&format!("under {} (`acceptor`)", acceptor_grant(&repo))), "revoke shows its grant: {out}");
    hand::commit(&repo, "revoked");
    let store = ledger_core::store::load(repo.path());
    let rev = store.log.iter().flat_map(|l| l.file.revocations.iter()).find_map(|r| r.id.as_ref()).map(|r| r.to_string()).expect("rev");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
    std::fs::remove_file(hand::file_holding(&repo, &rev)).expect("remove");
    hand::commit(&repo, "deleted the revocation");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[L007] {rev}")) && text.contains("revive what it ended"), "{text}");
}
