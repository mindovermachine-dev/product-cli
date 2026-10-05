//! No key binding is left silently untrusted (ruled 2026-10-05): every
//! filed binding is either trusted — it reaches `allowed_signers` — or the
//! gate names it in a finding. Walks each way a binding can be filed and
//! asserts both the invariant and which side each case lands on.

mod common;

use chrono::{Duration, Utc};
use common::{hand, Repo};
use ledger_core::authority::{BindingAct, Policy, Scheme};
use ledger_core::mint::UlidMint;

const OWNER: &str = "owner@customer.example";
const OTHER: &str = "architect@customer.example";
const NS: &str = "fixture.ledger";
const SECOND: &str = "second.ledger";

/// Where a binding should land.
#[derive(Debug, PartialEq)]
enum Expect {
    Trusted,
    Finding(&'static str),
}

/// Every binding in the store: trusted, or named by a finding — never
/// neither — and the side each one is on, by id.
fn accounted(repo: &Repo) -> Vec<(String, Expect)> {
    let store = ledger_core::store::load(repo.path());
    let today = Utc::now().date_naive();
    let landing = ledger_core::landing::Landing::compute(repo.path(), None).expect("landing");
    let trusted: Vec<String> =
        ledger_core::signing::check::check(&store, &landing, today).trusted.iter().map(|b| b.id.to_string()).collect();
    let report = ledger_core::verify::verify(&store, &ledger_core::verify::Options::full(today));
    let mut out = Vec::new();
    for b in store.log.iter().flat_map(|l| l.file.key_bindings.iter()) {
        let id = b.id.to_string();
        let finding = report.findings.iter().find(|f| f.subject == id).map(|f| f.class.code());
        let side = match (trusted.contains(&id), finding) {
            (true, None) => Expect::Trusted,
            (false, Some(class)) => Expect::Finding(class),
            (t, f) => panic!("{id} is trusted={t} with finding {f:?} — every binding is exactly one of the two"),
        };
        out.push((id, side));
    }
    out
}

fn side_of(repo: &Repo, blob: &str) -> Expect {
    let store = ledger_core::store::load(repo.path());
    let id = store.log.iter().flat_map(|l| l.file.key_bindings.iter()).find(|b| b.key.as_deref() == Some(blob)).map(|b| b.id.to_string()).expect("binding");
    accounted(repo).into_iter().find(|(i, _)| *i == id).map(|(_, e)| e).expect("accounted")
}

fn blob(key: &str) -> String {
    std::fs::read_to_string(format!("{key}.pub")).expect("pub").split_whitespace().nth(1).expect("blob").to_string()
}

/// Governed in `NS`, the owner's key bound by `init` itself, committed.
fn governed() -> (Repo, String) {
    let repo = Repo::with_identity(OWNER);
    repo.declare();
    let key = repo.keygen("owner");
    repo.use_key(&key);
    repo.ok(&["init", "--namespace", NS, "--external-ref", "contract 2026/117"]);
    hand::commit(&repo, "governed");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    (repo, key)
}

/// A hand-filed binding for `principal` by `by` in `ns`, dated `at`,
/// signed by `signer` when given.
fn by_hand(repo: &Repo, by: &str, principal: &str, ns: &str, at: chrono::DateTime<Utc>, signer: Option<&str>) -> String {
    let id: ledger_core::id::KeyBindingId = UlidMint::system().mint_id("key").expect("id");
    let key = repo.keygen(&format!("k{}", id.ulid()));
    let mut b = hand::binding(BindingAct::Add, principal, by, ns, Some(&format!("{key}.pub")), None, None);
    b.at = at;
    b.hash = ledger_core::authority::payload::binding_hash(&b);
    let ulid = b.id.ulid().to_string();
    let signers: Vec<(&str, &str)> = signer.map(|s| vec![(ulid.as_str(), s)]).unwrap_or_default();
    hand::file(repo, vec![b], Vec::new(), &signers);
    blob(&key)
}

/// A first policy for `ns` filed by hand by the owner under the genesis
/// grant, signed by `signer` when given.
fn first_policy(repo: &Repo, ns: &str, schemes: Vec<Scheme>, signer: Option<&str>) -> chrono::DateTime<Utc> {
    let store = ledger_core::store::load(repo.path());
    let genesis = store.log.iter().flat_map(|l| l.file.grants.iter()).find(|g| g.genesis).cloned().expect("genesis");
    let mut p = Policy {
        id: UlidMint::system().mint_id("pol").expect("id"),
        namespace: ns.into(),
        schemes,
        require_sk: false,
        accept_role: "acceptor".into(),
        reaccept_within_days: None,
        replaces: None,
        by: OWNER.parse().expect("id"),
        under: Some(genesis.id.clone()),
        at: Utc::now(),
        hash: ledger_core::hash::VersionHash::zero(),
    };
    p.hash = ledger_core::authority::payload::policy_hash(&p);
    let ulid = p.id.ulid().to_string();
    let signers: Vec<(&str, &str)> = signer.map(|s| vec![(ulid.as_str(), s)]).unwrap_or_default();
    let at = p.at;
    hand::file(repo, Vec::new(), vec![p], &signers);
    at
}

#[test]
fn every_filed_binding_is_trusted_or_named_by_a_finding() {
    let (repo, owner) = governed();
    let at_init = Expect::Trusted;
    assert!(accounted(&repo).iter().all(|(_, e)| *e == at_init), "init's self-bound binding is trusted");

    // Governed: the owner's own further key, signed by their live key.
    let signed = by_hand(&repo, OWNER, OWNER, NS, Utc::now(), Some(&owner));
    // Governed: the same, unsigned.
    let unsigned = by_hand(&repo, OWNER, OWNER, NS, Utc::now(), None);
    // D7: someone who is not the genesis holder files another's first key.
    let refused = by_hand(&repo, OTHER, OTHER, NS, Utc::now(), None);
    // Ungoverned: a binding in a namespace no policy governs.
    let ungoverned = by_hand(&repo, OWNER, OWNER, "third.ledger", Utc::now(), Some(&owner));
    // Before a signed first policy: one signed, one unsigned.
    let at = first_policy(&repo, SECOND, vec![Scheme::Ssh], Some(&owner));
    let before_signed = by_hand(&repo, OWNER, OWNER, SECOND, at - Duration::seconds(1), Some(&owner));
    let before_unsigned = by_hand(&repo, OWNER, OWNER, SECOND, at - Duration::seconds(1), None);
    // Before a `[none]` first policy, unsigned.
    let at = first_policy(&repo, "fourth.ledger", vec![Scheme::None], None);
    let before_none = by_hand(&repo, OWNER, OWNER, "fourth.ledger", at - Duration::seconds(1), None);
    hand::commit(&repo, "every way a binding is filed");

    let cases = [
        ("governed, signed", &signed, Expect::Trusted),
        ("governed, unsigned", &unsigned, Expect::Finding("L011")),
        ("D7 refuses the filer", &refused, Expect::Finding("SCHEMA")),
        ("no policy governs the namespace", &ungoverned, Expect::Finding("SCHEMA")),
        ("before a signed first policy, signed", &before_signed, Expect::Trusted),
        ("before a signed first policy, unsigned", &before_unsigned, Expect::Finding("L011")),
        ("before a [none] first policy, unsigned", &before_none, Expect::Trusted),
    ];
    for (label, blob, expected) in cases {
        assert_eq!(side_of(&repo, blob), expected, "{label}");
    }
    // And the invariant over the whole store, `accounted` panicking on any
    // binding that is neither.
    assert_eq!(accounted(&repo).len(), 8);
}
