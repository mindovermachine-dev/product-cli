//! A policy is an act of the genesis holder, judged at `verify` (`A006`).
//!
//! Every policy, first or change, names the grant it is made under
//! (`under`). That grant must be the genesis grant, held by the policy's
//! `by`, live and available as of the policy (D6), through the same
//! `authorize_named` every other act goes through. A signature says only
//! that `by` filed it; it does not say `by` was the one who may.

mod common;

use common::{hand, Repo};
use ledger_core::authority::{Authority, Policy};

const OWNER: &str = "owner@customer.example";
const ARCHITECT: &str = "architect@customer.example";
const NS: &str = "fixture.ledger";
const SECOND: &str = "second.ledger";

fn verify(repo: &Repo) -> (i32, String) {
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

/// `NS` under policy, the owner (genesis holder) bound, and the architect
/// bound by the owner's vouching (D7) and holding nothing. Returns the
/// architect's private key.
fn governed_with_a_bound_architect() -> (Repo, String) {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117"]);
    repo.bind_own_key(NS, "owner");
    let arch = repo.vouch_for(NS, ARCHITECT, "architect");
    hand::commit(&repo, "governed");
    (repo, arch)
}

fn genesis(repo: &Repo) -> String {
    let store = ledger_core::store::load(repo.path());
    store.log.iter().flat_map(|l| l.file.grants.iter()).find(|g| g.genesis).map(|g| g.id.to_string()).expect("genesis")
}

/// The next policy of `NS`, as a hand would write it, filed by `by` under
/// the genesis grant.
fn next_policy(repo: &Repo, by: &str) -> Policy {
    let store = ledger_core::store::load(repo.path());
    let current = Authority::build(&store).policy(NS).cloned().expect("policy");
    let mut next = current.clone();
    next.id = ledger_core::mint::UlidMint::system().mint_id("pol").expect("id");
    next.replaces = Some(current.hash.clone());
    next.reaccept_within_days = Some(30);
    next.by = by.parse().expect("identity");
    next.under = Some(genesis(repo).parse().expect("grant id"));
    next.at = chrono::Utc::now();
    next.hash = ledger_core::authority::payload::policy_hash(&next);
    next
}

#[test]
fn a_policy_change_signed_by_a_bound_principal_who_is_not_the_genesis_holder_fails_a006() {
    let (repo, arch) = governed_with_a_bound_architect();
    let next = next_policy(&repo, ARCHITECT);
    let (id, ulid) = (next.id.to_string(), next.id.ulid().to_string());
    hand::file(&repo, Vec::new(), vec![next], &[(&ulid, &arch)]);
    hand::commit(&repo, "a policy change by the architect, signed");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[A006] {id}")) && text.contains(ARCHITECT), "{text}");
    assert!(!text.contains("[L011]"), "the signature is good — the act is not theirs: {text}");
}

#[test]
fn under_none_an_unsigned_policy_change_by_a_non_genesis_principal_fails_a006() {
    let (repo, _) = governed_with_a_bound_architect();
    repo.ok(&["policy", "set", "--namespace", NS, "--scheme", "none"]);
    hand::commit(&repo, "governed, unsigned");
    let next = next_policy(&repo, ARCHITECT);
    let id = next.id.to_string();
    hand::file(&repo, Vec::new(), vec![next], &[]);
    hand::commit(&repo, "a policy change by the architect, unsigned");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[A006] {id}")), "{text}");
    assert!(!text.contains("[L011]"), "`none` requires no signature: {text}");
}

#[test]
fn a_first_policy_for_an_ungoverned_namespace_by_a_non_genesis_author_fails_a006() {
    let (repo, _) = governed_with_a_bound_architect();
    let mut first = next_policy(&repo, ARCHITECT);
    first.namespace = SECOND.to_string();
    first.replaces = None;
    first.hash = ledger_core::authority::payload::policy_hash(&first);
    let id = first.id.to_string();
    hand::file(&repo, Vec::new(), vec![first], &[]);
    hand::commit(&repo, "the architect puts a second namespace under policy");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[A006] {id}")) && text.contains(ARCHITECT), "{text}");
}

#[test]
fn the_genesis_holders_own_policy_change_still_passes() {
    let (repo, _) = governed_with_a_bound_architect();
    repo.ok(&["policy", "set", "--namespace", NS, "--reaccept-within-days", "30"]);
    repo.ok(&["init", "--namespace", SECOND, "--external-ref", "contract 2026/117"]);
    hand::commit(&repo, "the genesis holder's policy acts");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
}
