//! A key binding before its namespace's first policy (ruled 2026-10-05,
//! narrowing D5 (c)): never exempt as a pre-policy act. It is judged by D7
//! and by the signature requirement of that first policy. The pre-policy
//! exemption covers acceptances and revocations only.
//!
//! **How one arises.** `init` dates a namespace's genesis grant with its
//! first policy — every namespace, since each has its own genesis (ruling
//! 47) — so a binding dated before the policy is also before the grant, and
//! D7 refuses it. By hand a namespace can be founded with a genesis older
//! than its policy (`hand::found`), and a binding dated between them is
//! *before* the policy by D6 (same landing, earlier `at`) yet after the
//! grant: that is the store the ruling's cases reach.
//!
//! **What the ruling gives.**
//! - A signed binding before the policy is trusted.
//! - An unsigned one fails `L011` where the first policy requires a
//!   signature, and is never trusted.
//! - Under a `[none]` first policy, D7 alone decides it.
//! - A binding in a namespace no policy governs at all stays a schema fault.
//!
//! Since `init --namespace` binds the holder's key in the same act, dated
//! with the policy and the genesis, a skewed further key dated before them
//! is refused by D7: as of it the namespace has no genesis (asserted below).

mod common;

use chrono::Duration;
use common::{hand, Repo};
use ledger_core::author::{Author, KeyArgs};
use ledger_core::authority::{BindingAct, Policy, Scheme};
use ledger_core::mint::UlidMint;

const OWNER: &str = "owner@customer.example";
const NS: &str = "fixture.ledger";
const SECOND: &str = "second.ledger";

fn public(key: &str) -> (String, String) {
    let text = std::fs::read_to_string(format!("{key}.pub")).expect("pub");
    let mut parts = text.split_whitespace();
    (parts.next().expect("type").to_string(), parts.next().expect("blob").to_string())
}

fn policy_of(repo: &Repo, ns: &str) -> Policy {
    let store = ledger_core::store::load(repo.path());
    store.log.iter().flat_map(|l| l.file.policies.iter()).find(|p| p.namespace == ns).cloned().expect("policy")
}

/// Every namespace's derived `allowed_signers`, concatenated.
fn signers(repo: &Repo) -> String {
    ledger_core::layout::namespaces(repo.path()).into_iter().map(|ns| repo.signers(&ns)).collect()
}

/// The verb, with its clock `behind` the policy's.
fn bind_behind(repo: &Repo, ns: &str, behind: Duration, key_type: String, blob: String) -> Result<ledger_core::author::Applied, ledger_core::author::AuthorError> {
    let at = policy_of(repo, ns).at - behind;
    let mut author = Author::new(repo.path(), OWNER.parse().expect("id"), at, UlidMint::system());
    author.identity_add(KeyArgs { namespace: ns.into(), key_type, key: blob, principal: None })
}

/// A store governed in `NS` with the owner's key bound, committed.
fn governed() -> Repo {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117", "--without-key"]);
    repo.bind_own_key(NS, "owner");
    hand::commit(&repo, "governed");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    repo
}

/// An unsigned binding for the owner in `ns`, dated `behind` `at`.
fn unsigned_before(repo: &Repo, ns: &str, at: chrono::DateTime<chrono::Utc>) -> (String, String) {
    let forged = repo.keygen("forged");
    let mut b = hand::binding(BindingAct::Add, OWNER, OWNER, ns, Some(&format!("{forged}.pub")), None, None);
    b.at = at - Duration::seconds(1);
    b.hash = ledger_core::authority::payload::binding_hash(&b);
    let id = b.id.to_string();
    hand::file(repo, vec![b], Vec::new(), &[]);
    (id, public(&forged).1)
}

#[test]
fn in_a_fresh_store_d7_refuses_a_binding_dated_before_the_first_policy() {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117", "--without-key"]);
    let key = repo.keygen("owner");
    repo.use_key(&key);
    let (key_type, blob) = public(&key);
    let refused = bind_behind(&repo, NS, Duration::seconds(1), key_type, blob).expect_err("refused");
    assert!(format!("{refused:?}").contains("D7"), "{refused:?}");
}

#[test]
fn a_signed_binding_before_the_first_policy_is_trusted_and_signs() {
    let repo = governed();
    let owner = repo.path().join("keys/owner").display().to_string();
    // A namespace founded by hand with its genesis older than its policy;
    // then a key added on a clock behind the policy, signed by the owner's
    // key trusted in the first namespace (keys are store-wide until issue 6).
    hand::found(&repo, SECOND, OWNER, vec![Scheme::Ssh], Some(&owner));
    let next = repo.keygen("owner-second");
    let (key_type, blob) = public(&next);
    bind_behind(&repo, SECOND, Duration::seconds(1), key_type, blob.clone()).expect("D7 admits it, and it is signed");
    hand::commit(&repo, "second namespace, governed and bound in one commit");
    let store = ledger_core::store::load(repo.path());
    let binding = store.log.iter().flat_map(|l| l.file.key_bindings.iter()).find(|b| b.key.as_deref() == Some(blob.as_str())).cloned().expect("binding");
    assert!(binding.at < policy_of(&repo, SECOND).at, "before the policy: {binding:?}");
    let out = repo.ledger(&["verify", "--no-blame"]);
    assert_eq!(out.status.code(), Some(0), "{}", common::both(&out));
    assert!(signers(&repo).contains(&blob), "it reaches allowed_signers: {}", signers(&repo));
    // And it signs: an acceptance in the second namespace under that key.
    repo.declare_in(SECOND);
    repo.use_key(&next);
    let grant = hand::word(&repo.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &format!("ns:{SECOND}")]), "grant:");
    repo.ok(&["grant", "accept", &grant]);
    let out = repo.ok(&[
        "add", "--set", "ledger-design", "--namespace", SECOND, "--statement", "Signed with a key bound before the policy.",
        "--store", "constraint", "--discharge", "analyzer:DEC001",
    ]);
    repo.ok_tty(&["accept", &common::decision_id(&out)]);
}

/// `init` dates a later namespace's genesis with its policy (ruling 47), so
/// a further key dated before that policy is before the genesis too, and D7
/// refuses it: as of the binding nobody is the genesis holder there.
#[test]
fn a_skewed_add_dated_before_inits_genesis_is_refused_by_d7() {
    let repo = governed();
    repo.ok(&["init", "--namespace", SECOND, "--external-ref", "contract 2026/117"]);
    let next = repo.keygen("owner-second");
    let (key_type, blob) = public(&next);
    let refused = bind_behind(&repo, SECOND, Duration::seconds(1), key_type, blob).expect_err("refused");
    let text = format!("{refused:?}");
    assert!(text.contains("D7") && text.contains(&format!("no live key in `{SECOND}`")), "{text}");
}

#[test]
fn an_unsigned_binding_before_a_signed_first_policy_fails_l011_and_is_never_trusted() {
    let repo = governed();
    let owner = repo.path().join("keys/owner").display().to_string();
    // A namespace founded by hand, its genesis older than its signed first
    // policy; a binding filed with the policy, dated a second before it.
    let policy = hand::found(&repo, SECOND, OWNER, vec![Scheme::Ssh], Some(&owner));
    let (id, blob) = unsigned_before(&repo, SECOND, policy.at);
    hand::commit(&repo, "second namespace, with a hand-filed binding");
    let out = repo.ledger(&["verify", "--no-blame"]);
    let text = common::both(&out);
    assert_eq!(out.status.code(), Some(1), "{text}");
    assert!(text.contains("[L011]") && text.contains(&id) && text.contains("not trusted"), "{text}");
    repo.ok(&["identity", "sync"]);
    assert!(!signers(&repo).contains(&blob), "never trusted: {}", signers(&repo));
}

#[test]
fn a_binding_in_a_namespace_no_policy_governs_is_a_schema_fault_and_init_does_not_launder_it() {
    let repo = governed();
    let (id, _) = unsigned_before(&repo, SECOND, chrono::Utc::now());
    hand::commit(&repo, "a binding in a namespace nobody governs");
    let out = repo.ledger(&["verify", "--no-blame"]);
    let text = common::both(&out);
    assert_eq!(out.status.code(), Some(1), "{text}");
    assert!(text.contains("[SCHEMA]") && text.contains(&id) && text.contains("a namespace with no policy"), "{text}");
    // Opening the namespace dates its genesis with its policy, after the
    // binding (ruling 47): the binding is then before the genesis, and D7
    // refuses it — still a schema fault, now for that reason.
    repo.ok(&["init", "--namespace", SECOND, "--external-ref", "contract 2026/117"]);
    hand::commit(&repo, "opened after the fact");
    let out = repo.ledger(&["verify", "--no-blame"]);
    let text = common::both(&out);
    assert_eq!(out.status.code(), Some(1), "{text}");
    assert!(text.contains(&format!("[SCHEMA] {id}")) && text.contains("D7"), "{text}");
}

#[test]
fn under_a_none_first_policy_d7_alone_decides_a_binding_before_it() {
    let repo = governed();
    // A namespace founded by hand with a `[none]` first policy, its genesis
    // older; an unsigned binding dated before the policy, after the genesis.
    let policy = hand::found(&repo, SECOND, OWNER, vec![Scheme::None], None);
    let (id, blob) = unsigned_before(&repo, SECOND, policy.at);
    hand::commit(&repo, "a [none] first policy, and an unsigned binding before it");
    repo.ok(&["identity", "sync"]);
    let out = repo.ledger(&["verify", "--no-blame"]);
    let text = common::both(&out);
    assert_eq!(out.status.code(), Some(0), "{text}");
    assert!(!text.contains(&id), "no finding: {text}");
    assert!(signers(&repo).contains(&blob), "trusted unsigned, by D7 alone: {}", signers(&repo));
}
