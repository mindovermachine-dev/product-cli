//! A key binding before its namespace's first policy (ruled 2026-10-05,
//! narrowing D5 (c)): never exempt as a pre-policy act. It is judged by D7
//! and by the signature requirement of that first policy. The pre-policy
//! exemption covers acceptances and revocations only.
//!
//! **How one arises.** In a fresh store the genesis grant carries the first
//! policy's `at`, so a binding dated before the policy is also before the
//! grant, and D7 refuses it. In a later namespace the grant is older than
//! the policy, and no verb compares a binding's `at` with the policy's. A
//! binding filed on a clock behind the policy's, and committed with it, is
//! *before* the policy by D6 (same landing, earlier `at`). By hand, any
//! writer can file the same.
//!
//! **What the ruling gives.**
//! - A signed binding before the policy is trusted.
//! - An unsigned one fails `L011` where the first policy requires a
//!   signature, and is never trusted.
//! - Under a `[none]` first policy, D7 alone decides it.
//! - A binding in a namespace no policy governs at all stays a schema fault.
//!
//! Since `init --namespace` binds the holder's key in the same act, even in
//! a later namespace, a skewed further key dated before that binding is
//! refused for what it would do to `init`'s own binding (asserted below).

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

fn signers(repo: &Repo) -> String {
    std::fs::read_to_string(repo.path().join(".decisions/allowed_signers")).unwrap_or_default()
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

/// The second namespace's first policy as an earlier CLI filed it — no key
/// binding in the same act — signed by the owner's key trusted in `NS`.
fn legacy_second_policy(repo: &Repo, key: &str) -> Policy {
    let store = ledger_core::store::load(repo.path());
    let genesis = store.log.iter().flat_map(|l| l.file.grants.iter()).find(|g| g.genesis).cloned().expect("genesis");
    let mut policy = Policy {
        id: UlidMint::system().mint_id("pol").expect("id"),
        namespace: SECOND.into(),
        schemes: vec![Scheme::Ssh],
        require_sk: false,
        accept_role: "acceptor".into(),
        reaccept_within_days: None,
        replaces: None,
        by: OWNER.parse().expect("id"),
        under: Some(genesis.id.clone()),
        at: chrono::Utc::now(),
        hash: ledger_core::hash::VersionHash::zero(),
    };
    policy.hash = ledger_core::authority::payload::policy_hash(&policy);
    let ulid = policy.id.ulid().to_string();
    hand::file(repo, Vec::new(), vec![policy.clone()], &[(&ulid, key)]);
    policy
}

#[test]
fn a_signed_binding_before_the_first_policy_is_trusted_and_signs() {
    let repo = governed();
    let owner = repo.path().join("keys/owner").display().to_string();
    // A namespace opened before `init` bound the holder's key in the same
    // act; then a key added on a clock behind its policy, signed by the
    // owner's key trusted in the first namespace.
    legacy_second_policy(&repo, &owner);
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
    repo.use_key(&next);
    let grant = hand::word(&repo.ok(&["grant", "new", "acceptor", "--to", OWNER, "--scope", &format!("ns:{SECOND}")]), "grant:");
    repo.ok(&["grant", "accept", &grant]);
    let out = repo.ok(&[
        "add", "--set", "ledger-design", "--namespace", SECOND, "--statement", "Signed with a key bound before the policy.",
        "--store", "constraint", "--discharge", "analyzer:DEC001",
    ]);
    repo.ok_tty(&["accept", &common::decision_id(&out)]);
}

/// Since `init` binds the holder's key in the same act, a further key dated
/// before that binding takes the namespace's first-key place. `init`'s own
/// binding — signed by the holder's key from another namespace — is then a
/// further `add`, which must be signed by a key of theirs in this namespace,
/// and it would fail `L011`. The verb refuses to introduce that.
#[test]
fn a_skewed_add_dated_before_inits_binding_is_refused_for_what_it_would_do_to_it() {
    let repo = governed();
    repo.ok(&["init", "--namespace", SECOND, "--external-ref", "contract 2026/117"]);
    let next = repo.keygen("owner-second");
    let (key_type, blob) = public(&next);
    let refused = bind_behind(&repo, SECOND, Duration::seconds(1), key_type, blob).expect_err("refused");
    let text = format!("{refused:?}");
    assert!(text.contains("L011") && text.contains("no key bound to"), "{text}");
}

#[test]
fn an_unsigned_binding_before_a_signed_first_policy_fails_l011_and_is_never_trusted() {
    let repo = governed();
    // Filed with the policy, dated a second before it.
    repo.ok(&["init", "--namespace", SECOND, "--external-ref", "contract 2026/117"]);
    let (id, blob) = unsigned_before(&repo, SECOND, policy_of(&repo, SECOND).at);
    hand::commit(&repo, "second namespace, with a hand-filed binding");
    let out = repo.ledger(&["verify", "--no-blame"]);
    let text = common::both(&out);
    assert_eq!(out.status.code(), Some(1), "{text}");
    assert!(text.contains("[L011]") && text.contains(&id) && text.contains("not trusted"), "{text}");
    repo.ok(&["identity", "sync"]);
    assert!(!signers(&repo).contains(&blob), "never trusted: {}", signers(&repo));
}

#[test]
fn a_binding_in_a_namespace_no_policy_governs_is_a_schema_fault_and_init_will_not_govern_it() {
    let repo = governed();
    let (id, _) = unsigned_before(&repo, SECOND, chrono::Utc::now());
    hand::commit(&repo, "a binding in a namespace nobody governs");
    let out = repo.ledger(&["verify", "--no-blame"]);
    let text = common::both(&out);
    assert_eq!(out.status.code(), Some(1), "{text}");
    assert!(text.contains("[SCHEMA]") && text.contains(&id) && text.contains("a namespace with no policy"), "{text}");
    // Opting the namespace in would turn it into L011 — judged under the new
    // first policy, unsigned — which init's gate refuses to introduce.
    let refused = repo.refused(&["init", "--namespace", SECOND, "--external-ref", "contract 2026/117"]);
    assert!(refused.contains("L011") && refused.contains(&id), "{refused}");
}

#[test]
fn under_a_none_first_policy_d7_alone_decides_a_binding_before_it() {
    let repo = governed();
    let store = ledger_core::store::load(repo.path());
    let genesis = store.log.iter().flat_map(|l| l.file.grants.iter()).find(|g| g.genesis).cloned().expect("genesis");
    let mut mint = UlidMint::system();
    let mut policy = Policy {
        id: mint.mint_id("pol").expect("id"),
        namespace: SECOND.into(),
        schemes: vec![Scheme::None],
        require_sk: false,
        accept_role: "acceptor".into(),
        reaccept_within_days: None,
        replaces: None,
        by: OWNER.parse().expect("id"),
        under: Some(genesis.id.clone()),
        at: chrono::Utc::now(),
        hash: ledger_core::hash::VersionHash::zero(),
    };
    policy.hash = ledger_core::authority::payload::policy_hash(&policy);
    let at = policy.at;
    hand::file(&repo, Vec::new(), vec![policy], &[]);
    let (id, blob) = unsigned_before(&repo, SECOND, at);
    hand::commit(&repo, "a [none] first policy, and an unsigned binding before it");
    repo.ok(&["identity", "sync"]);
    let out = repo.ledger(&["verify", "--no-blame"]);
    let text = common::both(&out);
    assert_eq!(out.status.code(), Some(0), "{text}");
    assert!(!text.contains(&id), "no finding: {text}");
    assert!(signers(&repo).contains(&blob), "trusted unsigned, by D7 alone: {}", signers(&repo));
}
