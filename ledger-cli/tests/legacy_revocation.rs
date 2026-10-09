//! An old-style revocation (`acceptance`, `by`) is valid only as a pre-policy act.
//!
//! Formats 1–5 carry the legacy revocation shape: no id, no `under`, no
//! sidecar. Before its namespace's first policy nothing is role-checked or
//! signature-checked (D5 (c)), so the shape stands there. An old-style
//! revocation that is not before that policy (D6: landed after it, however
//! it is dated) is judged like a `rev:` revocation, and since it can name no
//! grant it fails `A006`. Before this rule it passed `verify` and silently
//! revoked another holder's acceptance.

mod common;

use common::{hand, Repo};

const OWNER: &str = "owner@customer.example";
const INTRUDER: &str = "intruder@customer.example";
const NS: &str = "fixture.ledger";
const CS: &str = "01KZZZZZZZZZZZZZZZZZZZZZZZ";

fn verify(repo: &Repo) -> (i32, String) {
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

/// Put the namespace under policy, the owner bound and holding the acceptor
/// role, then file and accept a decision. Returns the acceptance id.
fn governed_and_accepted(repo: &Repo) -> String {
    repo.declare();
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117", "--without-key"]);
    repo.bind_own_key(NS, "owner");
    let grant = hand::word(&repo.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &format!("ns:{NS}")]), "grant:");
    repo.ok(&["grant", "accept", &grant]);
    let id = repo.add("Money is decimal.", &[]);
    repo.ok_tty(&["accept", &id]);
    hand::commit(repo, "governed, accepted");
    only_acceptance(repo)
}

fn only_acceptance(repo: &Repo) -> String {
    let store = ledger_core::store::load(repo.path());
    store.log.iter().flat_map(|l| l.file.acceptances.iter()).map(|a| a.id.to_string()).next().expect("acceptance")
}

/// Hand-file a format-1 change-set holding one unsigned old-style revocation.
fn legacy_revocation(repo: &Repo, acceptance: &str, by: &str, at: &str) {
    let text = format!(
        "format: 1\nid: cs:{CS}\ncreated_at: {at}\ncreated_by: {by}\nrevocations:\n- acceptance: {acceptance}\n  at: {at}\n  by: {by}\n  reason: not mine to revoke\n"
    );
    std::fs::write(repo.log_dir(NS).join(format!("{CS}.yml")), text).expect("write");
}

fn export(repo: &Repo) -> String {
    let out = repo.ledger(&["export", "--format", "ntriples", "--namespace", NS, "--out", "-"]);
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// The case as reported: an unsigned old-style revocation of the owner's
/// acceptance, by someone holding no grant, landed after the first policy.
#[test]
fn an_old_style_revocation_landed_after_the_first_policy_fails_a006() {
    let repo = Repo::with_identity(OWNER);
    let acc = governed_and_accepted(&repo);
    legacy_revocation(&repo, &acc, INTRUDER, "2026-10-05T10:00:00Z");
    hand::commit(&repo, "an old-style revocation after the policy");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[A006] {acc}")), "the finding names the revoked acceptance: {text}");
    assert!(text.contains("an old-style revocation"), "{text}");
    assert!(text.contains(INTRUDER), "{text}");
    // The export is the log's projection and carries the revocation as it
    // does an unauthorised `rev:` one; the gate is what refuses it.
    let ulid = acc.trim_start_matches("acc:");
    assert!(export(&repo).contains(&format!("<urn:rev:legacy-{ulid}> <urn:ledger:ns#revokes> <urn:{acc}>")));
}

/// D6: an act is before the policy only if it is dated *and* landed before
/// it. Backdating does not move a revocation that landed after.
#[test]
fn an_old_style_revocation_backdated_before_the_policy_but_landed_after_fails_a006() {
    let repo = Repo::with_identity(OWNER);
    let acc = governed_and_accepted(&repo);
    legacy_revocation(&repo, &acc, OWNER, "2020-01-01T00:00:00Z");
    hand::commit(&repo, "backdated old-style revocation");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[A006]") && text.contains("an old-style revocation"), "{text}");
}

/// Before the namespace's first policy the old shape stands: nothing there
/// is role-checked, and the revocation keeps revoking.
#[test]
fn an_old_style_revocation_before_the_first_policy_stands() {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    let id = repo.add("Money is decimal.", &[]);
    repo.ok_tty(&["accept", &id]);
    let acc = only_acceptance(&repo);
    // Dated now: after the acceptance, before the policy filed next — so
    // both dated and landed before it.
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    legacy_revocation(&repo, &acc, OWNER, &now);
    hand::commit(&repo, "accepted and revoked, ungoverned");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117", "--without-key"]);
    hand::commit(&repo, "governed afterwards");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains(&id), "the acceptance stays revoked, so the decision awaits acceptance: {text}");
}

/// Ruling 49, the verification report's case 3c. A hand-written change-set
/// with an earlier ULID files a decision in an ungoverned namespace, an
/// acceptance of it that reuses the governed acceptance's id, and an
/// old-style revocation of that id by someone holding nothing. Resolved to
/// the first filing, the revocation was judged in the ungoverned namespace,
/// needed no grant and no signature, and unsaid the signed acceptance: the
/// store was conformant. The duplicate id is now a schema fault.
#[test]
fn an_acceptance_id_reused_in_an_ungoverned_namespace_is_a_schema_fault() {
    const EARLY: &str = "01K2C4YQJ3F8M0PT5W7NZ9RDZ0";
    const MALLORY: &str = "mallory@example";
    let repo = Repo::with_identity(OWNER);
    let acc = governed_and_accepted(&repo);
    let dec = "dec:open.ns/01K2C4YQJ3F8M0PT5W7NZ9RDZ1";
    let version = format!(
        "decision: {dec}\nhash: sha256:{zero}\nset: ledger-design\nstatement: Anything at all.\nallocation: constraint\ndischarge: [analyzer:X]\ntolerance_floor_at_creation: T1\n",
        zero = "0".repeat(64)
    );
    let raw: ledger_core::version::VersionRaw = serde_yaml::from_str(&version).expect("version");
    let hash = ledger_core::hash::version_hash(&raw).to_string();
    let text = format!(
        "format: 1\nid: cs:{EARLY}\ncreated_at: 2026-08-10T09:14:22Z\ncreated_by: {MALLORY}\n\
decisions:\n- id: {dec}\n  created_at: 2026-08-10T09:14:22Z\n  created_by: {MALLORY}\n\
versions:\n- decision: {dec}\n  hash: {hash}\n  set: ledger-design\n  statement: Anything at all.\n  allocation: constraint\n  discharge: [analyzer:X]\n  tolerance_floor_at_creation: T1\n\
acceptances:\n- id: {acc}\n  decision: {dec}\n  version: {hash}\n  actor: {MALLORY}\n  at: 2026-08-10T09:20:00Z\n\
revocations:\n- acceptance: {acc}\n  at: 2026-08-10T09:30:00Z\n  by: {MALLORY}\n  reason: withdrawn\n"
    );
    // The file belongs to `open.ns`, whose set it names (LP-3.34, LP-5.22).
    repo.declare_in("open.ns");
    std::fs::create_dir_all(repo.log_dir("open.ns")).expect("log dir");
    std::fs::write(repo.log_dir("open.ns").join(format!("{EARLY}.yml")), text).expect("write");
    repo.act_as(MALLORY);
    hand::commit(&repo, "a reused acceptance id in an ungoverned namespace");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[SCHEMA] {acc}: this id is filed twice")), "{text}");
    assert!(!text.contains(&format!("[SCHEMA] {dec}")), "the decision itself is introduced once: {text}");
}
