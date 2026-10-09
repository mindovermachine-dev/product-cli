//! Each way an entity can sit outside its namespace's directory (ruling 45).

use crate::finding::VerifyClass;
use crate::store::LoggedChangeSet;
use crate::testkit;

use super::findings;

const OTHER: &str = "hafeok.other";

fn subjects(store: &crate::store::Store) -> Vec<String> {
    let found = findings(store);
    assert!(found.iter().all(|f| f.class == VerifyClass::Schema && f.message.contains("ruling 45")), "{found:?}");
    found.into_iter().map(|f| f.subject).collect()
}

#[test]
fn a_store_whose_files_hold_their_own_namespace_has_no_finding() {
    let sealed = testkit::sealed(testkit::version());
    let acceptance = testkit::acceptance(&sealed);
    let store = testkit::store(testkit::changeset(vec![sealed], vec![acceptance]));
    assert!(subjects(&store).is_empty());
}

#[test]
fn a_decision_its_version_and_its_acceptance_of_another_namespace_are_each_named() {
    let mut other = testkit::version();
    other.decision = format!("dec:{OTHER}/01K2C4YQJ3F8M0PT5W7NZ9RDY0").parse().expect("id");
    let other = testkit::sealed(other);
    let mut acc = testkit::acceptance(&other);
    acc.id = "acc:01K2C4YQJ3F8M0PT5W7NZ9RDY1".parse().expect("id");
    let cs = testkit::changeset(vec![other], vec![acc]);
    let mut store = testkit::store(cs);
    store.log[0].namespace = testkit::NS.to_string();
    let found = subjects(&store);
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(found.iter().all(|s| s.contains(OTHER) || s.starts_with("acc:")), "{found:?}");
}

/// A sealed `add` binding of the genesis holder in `ns`.
fn binding_in(ns: &str) -> crate::authority::KeyBinding {
    use crate::authority::fixture;
    let mut binding = crate::authority::KeyBinding {
        id: "key:01K2C4YQJ3F8M0PT5W7NZ9RDY3".parse().expect("id"),
        act: crate::authority::BindingAct::Add,
        principal: testkit::identity(fixture::GENESIS_HOLDER),
        namespace: ns.into(),
        key_type: Some("ssh-ed25519".into()),
        key: Some("AAAA".into()),
        closes: None,
        self_bound: false,
        mandate: None,
        by: testkit::identity(fixture::GENESIS_HOLDER),
        under: None,
        at: testkit::stamp("2026-08-11T09:00:00Z"),
        hash: crate::hash::VersionHash::zero(),
    };
    binding.hash = crate::authority::payload::binding_hash(&binding);
    binding
}

/// A sealed `[none]` policy of `ns`.
fn policy_in(ns: &str) -> crate::authority::Policy {
    use crate::authority::fixture;
    let mut policy = crate::authority::Policy {
        id: "pol:01K2C4YQJ3F8M0PT5W7NZ9RDY4".parse().expect("id"),
        namespace: ns.into(),
        schemes: vec![crate::authority::Scheme::None],
        require_sk: false,
        accept_role: "acceptor".into(),
        reaccept_within_days: None,
        replaces: None,
        by: testkit::identity(fixture::GENESIS_HOLDER),
        under: None,
        at: testkit::stamp("2026-08-11T09:00:00Z"),
        hash: crate::hash::VersionHash::zero(),
    };
    policy.hash = crate::authority::payload::policy_hash(&policy);
    policy
}

#[test]
fn a_binding_a_policy_and_a_revocation_target_of_another_namespace_are_named() {
    use crate::authority::fixture;
    let sealed = testkit::sealed(testkit::version());
    let acceptance = testkit::acceptance(&sealed);
    let acc_id = acceptance.id.clone();
    let mut store = testkit::store(testkit::changeset(vec![sealed], vec![acceptance]));
    // A second file, under `hafeok.other`, revoking the acceptance filed
    // under `hafeok.ledger`, with a binding and a policy of `hafeok.ledger`.
    let mut cs = fixture::changeset(Vec::new(), Vec::new());
    cs.id = "cs:01K2C4YQJ3F8M0PT5W7NZ9RDY2".parse().expect("id");
    let mut revocation = testkit::legacy_revocation("2026-08-11T09:00:00Z", "wrong file");
    revocation.acceptance = Some(acc_id);
    cs.revocations.push(revocation);
    cs.key_bindings.push(binding_in(testkit::NS));
    cs.policies.push(policy_in(testkit::NS));
    store.log.push(LoggedChangeSet {
        namespace: OTHER.into(),
        path: format!("/fixture/.decisions/ns/{OTHER}/log/01K2C4YQJ3F8M0PT5W7NZ9RDY2.yml").into(),
        file: cs,
    });
    let found = subjects(&store);
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(found.iter().any(|s| s.starts_with("rev:") || s.starts_with("acc:")), "the revocation, by its subject: {found:?}");
    assert!(found.iter().any(|s| s.starts_with("key:")) && found.iter().any(|s| s.starts_with("pol:")), "{found:?}");
}

#[test]
fn a_candidate_homed_nowhere_is_not_judged_here() {
    let sealed = testkit::sealed(testkit::version());
    let mut store = testkit::store(testkit::changeset(vec![sealed], Vec::new()));
    store.log[0].namespace = String::new();
    assert!(subjects(&store).is_empty(), "an unhomed candidate is refused by the finding that says why");
}
