//! Signatures end to end (#70): `L011`, `L012`, the landing order (D6), the
//! re-judged role (`A006`, D5/D9), the first key (D7), and the refusals.
//!
//! Every governed store here signs for real: ed25519 keys made with
//! `ssh-keygen`, configured as `git config user.signingkey`, and verified
//! by `ssh-keygen -Y verify` against the derived `allowed_signers`.

mod common;

use chrono::{Duration, DurationRound};
use common::hand::{self, HandAccept};
use common::Repo;

const OWNER: &str = "owner@customer.example";
const ARCHITECT: &str = "architect@customer.example";
const NS: &str = "fixture.ledger";

/// A namespace under policy; the genesis holder's key bound; the genesis
/// holder granted the accept role. Committed.
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

fn verify(repo: &Repo, extra: &[&str]) -> (i32, String) {
    let mut args = vec!["verify", "--no-blame"];
    args.extend_from_slice(extra);
    let out = repo.ledger(&args);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

fn green(repo: &Repo) {
    let (code, text) = verify(repo, &[]);
    assert_eq!(code, 0, "{text}");
}

/// A second, whole-second instant after `t`, for hand acts dated between.
fn second_after(t: chrono::DateTime<chrono::Utc>) -> chrono::DateTime<chrono::Utc> {
    t.duration_trunc(Duration::seconds(1)).unwrap_or(t) + Duration::seconds(1)
}

#[test]
fn an_accept_is_signed_into_a_sidecar_and_verifies() {
    let (repo, _) = governed();
    let id = repo.add("Money is decimal.", &[]);
    let out = repo.ok_tty(&["accept", &id]);
    let sig = hand::word(&out, "sig/");
    assert!(sig.ends_with(".ssh.sig"), "{out}");
    assert!(repo.ns_dir(NS).join(&sig).is_file());
    green(&repo);
}

#[test]
fn a_missing_or_tampered_signature_fails_l011() {
    let (repo, _) = governed();
    let id = repo.add("Money is decimal.", &[]);
    let out = repo.ok_tty(&["accept", &id]);
    let sig = repo.ns_dir(NS).join(hand::word(&out, "sig/"));
    let original = std::fs::read(&sig).expect("sig");
    let mut tampered = original.clone();
    let mid = tampered.len() / 2;
    tampered[mid] = if tampered[mid] == b'A' { b'B' } else { b'A' };
    std::fs::write(&sig, &tampered).expect("tamper");
    let (code, text) = verify(&repo, &[]);
    assert_eq!(code, 1);
    assert!(text.contains("[L011]"), "{text}");
    std::fs::remove_file(&sig).expect("remove");
    let (code, text) = verify(&repo, &[]);
    assert_eq!(code, 1);
    assert!(text.contains("[L011]") && text.contains("requires a `ssh` signature"), "{text}");
}

#[test]
fn none_is_valid_only_where_policy_lists_it() {
    let (repo, _) = governed();
    let id = repo.add("Money is decimal.", &[]);
    let at = chrono::Utc::now();
    let grant = acceptor_grant(&repo, OWNER);
    hand::accept(&repo, &HandAccept { decision: &id, actor: OWNER, at, under: Some(&grant), key: None });
    let (code, text) = verify(&repo, &[]);
    assert_eq!(code, 1, "an unsigned acceptance where policy is `ssh`: {text}");
    assert!(text.contains("[L011]"), "{text}");
    repo.ok(&["policy", "set", "--namespace", NS, "--scheme", "none"]);
    // Under the new policy an unsigned acceptance is a `none` acceptance;
    // the one filed before the change is still judged under `ssh`.
    let later = repo.add("Time is UTC.", &[]);
    hand::accept(&repo, &HandAccept { decision: &later, actor: OWNER, at: chrono::Utc::now() + Duration::seconds(2), under: Some(&grant), key: None });
    let (_, text) = verify(&repo, &[]);
    assert_eq!(text.matches("[L011]").count(), 1, "only the pre-change one: {text}");
}

#[test]
fn none_is_exclusive() {
    let (repo, _) = governed();
    let refused = repo.refused(&["policy", "set", "--namespace", NS, "--scheme", "ssh", "--scheme", "none"]);
    assert!(refused.contains("`none` is exclusive"), "{refused}");
    // The same policy written by hand is a schema fault at `verify`.
    let store = ledger_core::store::load(repo.path());
    let current = ledger_core::authority::Authority::build(&store).policy(NS).cloned().expect("policy");
    let mut mixed = current.clone();
    mixed.id = ledger_core::mint::UlidMint::system().mint_id("pol").expect("id");
    mixed.replaces = Some(current.hash.clone());
    mixed.schemes = vec![ledger_core::authority::Scheme::Ssh, ledger_core::authority::Scheme::None];
    mixed.at = chrono::Utc::now();
    mixed.hash = ledger_core::authority::payload::policy_hash(&mixed);
    let id = mixed.id.to_string();
    hand::file(&repo, Vec::new(), vec![mixed], &[]);
    let (code, text) = verify(&repo, &[]);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[SCHEMA] {id}")) && text.contains("`none` is exclusive"), "{text}");
}

/// The id of `who`'s live acceptor grant.
fn acceptor_grant(repo: &Repo, who: &str) -> String {
    let store = ledger_core::store::load(repo.path());
    store
        .log
        .iter()
        .flat_map(|l| l.file.grants.iter())
        .find(|g| g.role == "acceptor" && g.holder.as_str() == who)
        .map(|g| g.id.to_string())
        .expect("an acceptor grant")
}

#[test]
fn dsse_is_verified_never_signed() {
    let (repo, _) = governed();
    repo.ok(&["policy", "set", "--namespace", NS, "--scheme", "dsse"]);
    let id = repo.add("Money is decimal.", &[]);
    let refused = repo.refused_tty(&["accept", &id]);
    assert!(refused.contains("hosted ledger service") && refused.contains("never signs"), "{refused}");
}

#[test]
fn an_agent_held_software_key_and_a_software_key_under_sk_are_refused() {
    let (repo, key) = governed();
    let id = repo.add("Money is decimal.", &[]);
    repo.use_key(&format!("{key}.pub"));
    let refused = repo.refused_tty(&["accept", &id]);
    assert!(refused.contains("an agent holds") && refused.contains("nothing confirms"), "{refused}");
    repo.use_key(&key);
    repo.ok(&["policy", "set", "--namespace", NS, "--require-sk", "true"]);
    let refused = repo.refused_tty(&["accept", &id]);
    assert!(refused.contains("hardware-backed") && refused.contains("software key"), "{refused}");
}

#[test]
fn a_key_closed_after_the_acceptance_is_a_review_item_then_l012_past_the_deadline() {
    let (repo, key) = governed();
    let id = repo.add("Money is decimal.", &[]);
    repo.ok_tty(&["accept", &id]);
    let acc = last_acceptance(&repo);
    hand::commit(&repo, "accepted");
    let binding = first_binding(&repo);
    let spare = repo.keygen("owner-next");
    repo.ok(&["identity", "add", "--namespace", NS, "--key-file", &format!("{spare}.pub")]);
    std::thread::sleep(std::time::Duration::from_millis(1100));
    repo.ok(&["policy", "set", "--namespace", NS, "--reaccept-within-days", "30"]);
    repo.ok(&["identity", "revoke", &binding]);
    hand::commit(&repo, "closed the key");
    let (code, text) = verify(&repo, &[]);
    assert_eq!(code, 0, "a review item, not a failure: {text}");
    assert!(text.contains("need re-acceptance") && text.contains(&acc), "{text}");
    let (code, text) = verify(&repo, &["--today", "2099-01-01"]);
    assert_eq!(code, 1);
    assert!(text.contains("[L012]") && text.contains(&acc), "{text}");
    // Affirmation: a new acceptance by the holder with a live key.
    repo.use_key(&spare);
    repo.ok_tty(&["accept", &id]);
    let (code, text) = verify(&repo, &["--today", "2099-01-01"]);
    assert_eq!(code, 0, "affirmed: {text}");
    let _ = key;
}

fn last_acceptance(repo: &Repo) -> String {
    let store = ledger_core::store::load(repo.path());
    store.log.iter().flat_map(|l| l.file.acceptances.iter()).map(|a| a.id.to_string()).next_back().expect("acceptance")
}

fn first_binding(repo: &Repo) -> String {
    let store = ledger_core::store::load(repo.path());
    store.log.iter().flat_map(|l| l.file.key_bindings.iter()).map(|b| b.id.to_string()).next().expect("binding")
}

#[test]
fn a_backdated_acceptance_landed_after_the_close_fails_l011_not_l012() {
    let (repo, key) = governed();
    let id = repo.add("Money is decimal.", &[]);
    hand::commit(&repo, "filed");
    let binding = first_binding(&repo);
    let spare = repo.keygen("owner-next");
    repo.ok(&["identity", "add", "--namespace", NS, "--key-file", &format!("{spare}.pub")]);
    // After every enabling entry, before the close two seconds later.
    let bound = hand::last_binding_at(&repo);
    std::thread::sleep(std::time::Duration::from_millis(2100));
    repo.use_key(&spare);
    repo.ok(&["identity", "revoke", &binding]);
    hand::commit(&repo, "closed");
    // Signed with the closed key, dated inside its window, landed after.
    let grant = acceptor_grant(&repo, OWNER);
    let at = second_after(bound);
    hand::accept(&repo, &HandAccept { decision: &id, actor: OWNER, at, under: Some(&grant), key: Some(&key) });
    hand::commit(&repo, "backdated");
    let (code, text) = verify(&repo, &[]);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[L011]") && text.contains("not dated and landed before the close"), "{text}");
    assert!(!text.contains("need re-acceptance"), "{text}");
}

#[test]
fn a_branch_verified_against_its_base_agrees_with_the_merge_ref() {
    let (repo, key) = governed();
    let id = repo.add("Money is decimal.", &[]);
    let binding = first_binding(&repo);
    let spare = repo.keygen("owner-next");
    repo.ok(&["identity", "add", "--namespace", NS, "--key-file", &format!("{spare}.pub")]);
    // Dated after every enabling entry (the grant's acceptance included) and
    // still before the close, which comes two seconds later.
    let bound = hand::last_binding_at(&repo);
    hand::commit(&repo, "base");
    repo.git(&["checkout", "-q", "-b", "topic"]);
    let grant = acceptor_grant(&repo, OWNER);
    hand::accept(&repo, &HandAccept { decision: &id, actor: OWNER, at: second_after(bound), under: Some(&grant), key: Some(&key) });
    hand::commit(&repo, "accepted on the branch");
    repo.git(&["checkout", "-q", "main"]);
    std::thread::sleep(std::time::Duration::from_millis(2100));
    repo.use_key(&spare);
    repo.ok(&["identity", "revoke", &binding]);
    hand::commit(&repo, "closed on main");
    repo.git(&["checkout", "-q", "topic"]);
    let (naive, naive_text) = verify(&repo, &["--base", "topic"]);
    assert_eq!(naive, 0, "without the base the close is not even in the store: {naive_text}");
    let (local, local_text) = verify(&repo, &["--base", "main"]);
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["merge", "-q", "--no-ff", "-m", "merge topic", "topic"]);
    let (merged, merged_text) = verify(&repo, &[]);
    assert_eq!((local, merged), (1, 1), "local:\n{local_text}\nmerged:\n{merged_text}");
    let class = |t: &str| t.lines().filter(|l| l.contains("[L011]")).count();
    assert_eq!(class(&local_text), class(&merged_text));
}

#[test]
fn a_signed_acceptance_by_a_bound_principal_with_no_grant_fails_a006() {
    let (repo, _) = governed();
    let id = repo.add("Money is decimal.", &[]);
    let arch = repo.vouch_for(NS, ARCHITECT, "architect");
    let at = chrono::Utc::now() + Duration::seconds(1);
    hand::accept(&repo, &HandAccept { decision: &id, actor: ARCHITECT, at, under: None, key: Some(&arch) });
    let (code, text) = verify(&repo, &[]);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[A006]") && text.contains("names no grant"), "{text}");
    assert!(!text.contains("[L011]"), "the signature itself holds: {text}");
    let owners = acceptor_grant(&repo, OWNER);
    let later = repo.add("Time is UTC.", &[]);
    hand::accept(&repo, &HandAccept { decision: &later, actor: ARCHITECT, at, under: Some(&owners), key: Some(&arch) });
    let (_, text) = verify(&repo, &[]);
    assert!(text.contains("not a filed grant of theirs"), "naming another's grant: {text}");
}

#[test]
fn a_revocation_ends_the_grant_for_acts_not_before_it() {
    let (repo, _) = governed();
    let first = repo.add("Money is decimal.", &[]);
    let second = repo.add("Time is UTC.", &[]);
    let grant = hand::word(&repo.ok(&["grant", "new", "acceptor", "--to", ARCHITECT, "--scope", &format!("ns:{NS}")]), "grant:");
    let arch = repo.vouch_for(NS, ARCHITECT, "architect");
    repo.act_as(ARCHITECT);
    repo.use_key(&arch);
    repo.ok(&["grant", "accept", &grant]);
    repo.ok_tty(&["accept", &first]);
    hand::commit(&repo, "accepted before the revocation");
    let before_revocation = hand::last_binding_at(&repo) + Duration::seconds(1);
    std::thread::sleep(std::time::Duration::from_millis(2100));
    repo.act_as(OWNER);
    repo.use_key(&repo.path().join("keys/owner").display().to_string());
    repo.ok_tty(&["grant", "revoke", &grant, "--reason", "moved teams"]);
    hand::commit(&repo, "revoked");
    green(&repo);
    // Made after the revocation landed, dated before it.
    hand::accept(&repo, &HandAccept { decision: &second, actor: ARCHITECT, at: before_revocation, under: Some(&grant), key: Some(&arch) });
    hand::commit(&repo, "backdated");
    let (code, text) = verify(&repo, &[]);
    assert_eq!(code, 1, "{text}");
    assert_eq!(text.matches("[A006]").count(), 1, "the earlier acceptance stands: {text}");
    assert!(text.contains("revoked"), "{text}");
}

#[test]
fn a_missing_ssh_keygen_is_a_named_finding_never_a_skipped_check() {
    let (repo, _) = governed();
    let id = repo.add("Money is decimal.", &[]);
    repo.ok_tty(&["accept", &id]);
    green(&repo);
    // A PATH with git and nothing else: no `ssh-keygen` to verify with.
    let bin = repo.path().join("bin-without-ssh");
    std::fs::create_dir_all(&bin).expect("bin");
    let git = std::process::Command::new("sh").args(["-c", "command -v git"]).output().expect("which git");
    let git = String::from_utf8_lossy(&git.stdout).trim().to_string();
    std::os::unix::fs::symlink(&git, bin.join("git")).expect("link git");
    let out = common::ledger_command(repo.path())
        .env("PATH", &bin)
        .args(["verify", "--no-blame"])
        .output()
        .expect("run");
    let text = common::both(&out);
    assert_eq!(out.status.code(), Some(1), "{text}");
    assert!(text.contains("[L011] ssh-keygen:") && text.contains("not on the PATH"), "the named finding: {text}");
    assert!(text.contains("could not be checked"), "each signature it needed fails too: {text}");
}
