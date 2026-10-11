//! Acts signed with a key whose close has already landed, dated back inside
//! the key's window, committed after the close (D6, #70). Each fails, and no
//! key such an act would introduce reaches `allowed_signers`.

mod common;

use chrono::{DateTime, Duration, DurationRound, Utc};
use common::hand;
use common::Repo;
use ledger_core::authority::payload::{binding_bytes, binding_hash, policy_bytes, policy_hash, revocation_bytes, revocation_hash};
use ledger_core::authority::{Authority, BindingAct, Revocable, Revocation};
use ledger_core::changeset::ChangeSet;
use ledger_core::mint::UlidMint;

const OWNER: &str = "owner@customer.example";
const NS: &str = "fixture.ledger";
const SECOND: &str = "second.ledger";

/// A governed namespace, the owner's first key closed by a second; the close
/// committed. Returns the closed key's private path and an instant inside
/// its window (after every enabling entry, before the close).
fn closed() -> (Repo, String, DateTime<Utc>) {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117", "--without-key"]);
    let key = repo.bind_own_key(NS, "owner");
    let grant = hand::word(&repo.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &format!("ns:{NS}")]), "grant:");
    repo.ok(&["grant", "accept", &grant]);
    hand::commit(&repo, "governed");
    let first = store(&repo).log.iter().flat_map(|l| l.file.key_bindings.iter()).map(|b| b.id.to_string()).next().expect("binding");
    let spare = repo.keygen("owner-next");
    repo.ok(&["identity", "add", "--namespace", NS, "--key-file", &format!("{spare}.pub")]);
    let bound = hand::last_binding_at(&repo);
    let inside = bound.duration_trunc(Duration::seconds(1)).unwrap_or(bound) + Duration::seconds(1);
    std::thread::sleep(std::time::Duration::from_millis(2100));
    repo.use_key(&spare);
    repo.ok(&["identity", "revoke", &first]);
    hand::commit(&repo, "closed the first key");
    (repo, key, inside)
}

fn store(repo: &Repo) -> ledger_core::store::Store {
    ledger_core::store::load(repo.path())
}

fn verify(repo: &Repo) -> (i32, String) {
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

/// File `cs` by hand, with one `ssh` sidecar per `(ulid, namespace, bytes)`
/// signed by `key`; commit it.
fn file_signed(repo: &Repo, cs: ChangeSet, signed: &[(String, String, Vec<u8>)], key: &str) {
    let dir = repo.sig_dir(NS);
    std::fs::create_dir_all(&dir).expect("sig dir");
    for (ulid, ns, bytes) in signed {
        std::fs::write(dir.join(format!("{ulid}.ssh.sig")), hand::ssh_sign(key, ns, bytes)).expect("sig");
    }
    std::fs::write(repo.log_dir(NS).join(cs.file_name()), serde_yaml::to_string(&cs).expect("yaml")).expect("log");
    hand::commit(repo, "signed with the closed key, backdated");
}

fn change_set(at: DateTime<Utc>) -> ChangeSet {
    ChangeSet::empty(7, UlidMint::system().mint_id("cs").expect("cs"), at, OWNER.parse().expect("id"), None)
}

fn public(key: &str) -> String {
    std::fs::read_to_string(format!("{key}.pub")).expect("pub").split_whitespace().nth(1).expect("blob").to_string()
}

fn derived_signers(repo: &Repo) -> String {
    ledger_core::authority::signers::derive(&store(repo)).into_values().collect()
}

fn fails_with(repo: &Repo, id: &str, class: &str) -> String {
    let (code, text) = verify(repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.lines().any(|l| l.contains(&format!("[{class}] {id}"))), "{id} fails {class}: {text}");
    text
}

#[test]
fn a_revocation_of_an_acceptance() {
    let (repo, key, inside) = closed();
    let id = repo.add("Money is decimal.", &[]);
    repo.ok_tty(&["accept", &id]);
    hand::commit(&repo, "accepted with the second key");
    let s = store(&repo);
    let acc = s.log.iter().flat_map(|l| l.file.acceptances.iter()).next().expect("acc").id.clone();
    let grant = s.log.iter().flat_map(|l| l.file.grants.iter()).find(|g| g.role == "acceptor").expect("grant").id.clone();
    let mut r = Revocation {
        id: Some(UlidMint::system().mint_id("rev").expect("rev")),
        revokes: Some(Revocable::Acceptance(acc)),
        acceptance: None,
        at: inside,
        actor: Some(OWNER.parse().expect("id")),
        by: None,
        reason: "backdated".into(),
        under: Some(grant),
        hash: None,
    };
    r.hash = Some(revocation_hash(&r));
    let rid = r.subject();
    let ulid = rid.split_once(':').map(|(_, u)| u.to_string()).expect("ulid");
    let mut cs = change_set(inside);
    let bytes = revocation_bytes(&r);
    cs.revocations.push(r);
    file_signed(&repo, cs, &[(ulid, NS.into(), bytes)], &key);
    fails_with(&repo, &rid, "L011");
}

#[test]
fn an_identity_rotate_and_an_identity_add_for_the_same_principal() {
    let (repo, key, inside) = closed();
    let live = store(&repo)
        .log
        .iter()
        .flat_map(|l| l.file.key_bindings.iter())
        .rfind(|b| b.act.opens())
        .map(|b| b.id.to_string())
        .expect("the second key");
    let rotated = repo.keygen("owner-rotated");
    let added = repo.keygen("owner-added");
    let mut rotate = hand::binding(BindingAct::Rotate, OWNER, OWNER, NS, Some(&format!("{rotated}.pub")), Some(&live), None);
    rotate.at = inside;
    rotate.hash = binding_hash(&rotate);
    let mut add = hand::binding(BindingAct::Add, OWNER, OWNER, NS, Some(&format!("{added}.pub")), None, None);
    add.at = inside;
    add.hash = binding_hash(&add);
    let (rid, aid) = (rotate.id.to_string(), add.id.to_string());
    let signed = vec![
        (rotate.id.ulid().to_string(), NS.to_string(), binding_bytes(&rotate)),
        (add.id.ulid().to_string(), NS.to_string(), binding_bytes(&add)),
    ];
    let mut cs = change_set(inside);
    cs.key_bindings = vec![rotate, add];
    file_signed(&repo, cs, &signed, &key);
    fails_with(&repo, &rid, "L011");
    fails_with(&repo, &aid, "L011");
    let signers = derived_signers(&repo);
    assert!(!signers.contains(&public(&rotated)) && !signers.contains(&public(&added)), "{signers}");
}

#[test]
fn a_policy_change_by_the_genesis_holder() {
    let (repo, key, inside) = closed();
    let current = Authority::build(&store(&repo)).policy(NS).cloned().expect("policy");
    let mut next = current.clone();
    next.id = UlidMint::system().mint_id("pol").expect("id");
    next.replaces = Some(current.hash.clone());
    next.require_sk = !current.require_sk;
    next.at = inside;
    next.hash = policy_hash(&next);
    let id = next.id.to_string();
    let signed = vec![(next.id.ulid().to_string(), NS.to_string(), policy_bytes(&next))];
    let mut cs = change_set(inside);
    cs.policies.push(next);
    file_signed(&repo, cs, &signed, &key);
    fails_with(&repo, &id, "L011");
}

#[test]
fn the_genesis_holders_first_binding_in_a_second_namespace_vouched_by_the_closed_key() {
    let (repo, key, inside) = closed();
    repo.ok(&["init", "--namespace", SECOND, "--external-ref", "contract 2026/117"]);
    hand::commit(&repo, "second namespace");
    let fresh = repo.keygen("owner-second");
    let mut b = hand::binding(BindingAct::Add, OWNER, OWNER, SECOND, Some(&format!("{fresh}.pub")), None, None);
    b.at = inside;
    b.hash = binding_hash(&b);
    let id = b.id.to_string();
    let signed = vec![(b.id.ulid().to_string(), SECOND.to_string(), binding_bytes(&b))];
    let mut cs = change_set(inside);
    cs.key_bindings.push(b);
    file_signed(&repo, cs, &signed, &key);
    // Dated before the second namespace's first policy but landed after it,
    // so the policy governs it (D5 (c), D6) — it is no pre-policy act.
    fails_with(&repo, &id, "L011");
    let signers = derived_signers(&repo);
    assert!(!signers.contains(&public(&fresh)), "{signers}");
}

const ARCHITECT: &str = "architect@customer.example";

/// A governed namespace with the owner's key bound; committed.
fn governed() -> (Repo, String) {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117", "--without-key"]);
    let key = repo.bind_own_key(NS, "owner");
    hand::commit(&repo, "governed");
    (repo, key)
}

#[test]
fn an_act_dated_before_the_first_policy_but_landed_after_it_is_checked() {
    let (repo, _) = governed();
    let id = repo.add("Money is decimal.", &[]);
    hand::commit(&repo, "filed");
    let policy_at = Authority::build(&store(&repo)).policy(NS).expect("policy").at;
    // Unsigned, by a principal holding no grant, dated before the policy.
    let acc = hand::accept(
        &repo,
        &hand::HandAccept { decision: &id, actor: ARCHITECT, at: policy_at - Duration::hours(1), under: None, key: None },
    );
    hand::commit(&repo, "landed after the policy");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[L011] {acc}")), "unsigned under `ssh`: {text}");
    assert!(text.contains(&format!("[A006] {acc}")), "no grant: {text}");
}

#[test]
fn a_principals_own_add_dated_before_the_close_of_their_only_key_is_a_d7_fault() {
    let (repo, _) = governed();
    let arch = repo.vouch_for(NS, ARCHITECT, "architect");
    let first = store(&repo)
        .log
        .iter()
        .flat_map(|l| l.file.key_bindings.iter())
        .find(|b| b.principal.as_str() == ARCHITECT)
        .map(|b| (b.id.to_string(), b.at))
        .expect("architect's key");
    let inside = first.1.duration_trunc(Duration::seconds(1)).unwrap_or(first.1) + Duration::seconds(1);
    std::thread::sleep(std::time::Duration::from_millis(2100));
    // The genesis holder closes the architect's only key.
    repo.ok(&["identity", "revoke", &first.0]);
    hand::commit(&repo, "closed the architect's key");
    // The architect's own `add`, dated inside the closed window, signed by
    // the closed key, landed after the close.
    let next = repo.keygen("architect-next");
    let mut b = hand::binding(BindingAct::Add, ARCHITECT, ARCHITECT, NS, Some(&format!("{next}.pub")), None, None);
    b.at = inside;
    b.hash = binding_hash(&b);
    let id = b.id.to_string();
    let signed = vec![(b.id.ulid().to_string(), NS.to_string(), binding_bytes(&b))];
    let mut cs = change_set(inside);
    cs.created_by = ARCHITECT.parse().expect("id");
    cs.key_bindings.push(b);
    file_signed(&repo, cs, &signed, &arch);
    // The close is terminating: as of the add, the architect has no live key,
    // so D7 refuses the filer before any signature is read.
    let text = fails_with(&repo, &id, "SCHEMA");
    assert!(text.contains("D7") && text.contains("has no live key"), "{text}");
    assert!(!derived_signers(&repo).contains(&public(&next)));
}

#[test]
fn under_none_a_principals_own_add_dated_before_their_keys_close_is_refused_and_never_trusted() {
    let (repo, _) = governed();
    repo.vouch_for(NS, ARCHITECT, "architect");
    let (first, bound) = store(&repo)
        .log
        .iter()
        .flat_map(|l| l.file.key_bindings.iter())
        .find(|b| b.principal.as_str() == ARCHITECT)
        .map(|b| (b.id.to_string(), b.at))
        .expect("architect's key");
    let inside = bound.duration_trunc(Duration::seconds(1)).unwrap_or(bound) + Duration::seconds(1);
    std::thread::sleep(std::time::Duration::from_millis(2100));
    repo.ok(&["identity", "revoke", &first]);
    repo.ok(&["policy", "set", "--namespace", NS, "--scheme", "none"]);
    hand::commit(&repo, "closed the architect's key; governed and unsigned");
    // No signature is required, so only D7 stands between this binding and
    // the trust root — and D7 holds only if the close applies to it.
    let next = repo.keygen("architect-next");
    let mut b = hand::binding(BindingAct::Add, ARCHITECT, ARCHITECT, NS, Some(&format!("{next}.pub")), None, None);
    b.at = inside;
    b.hash = binding_hash(&b);
    let id = b.id.to_string();
    let mut cs = change_set(inside);
    cs.created_by = ARCHITECT.parse().expect("id");
    cs.key_bindings.push(b);
    file_signed(&repo, cs, &[], "");
    // The forger regenerates the derived file too (verify reads the tree).
    repo.ok(&["identity", "sync"]);
    let text = fails_with(&repo, &id, "SCHEMA");
    assert!(text.contains("D7") && text.contains("has no live key"), "{text}");
    assert!(!derived_signers(&repo).contains(&public(&next)), "never trusted");
}
