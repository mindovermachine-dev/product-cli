//! D9 (b), (c): which grant an act is made under.
//!
//! `accept` is authorised by the policy's `accept_role` alone
//! (`Author::decision_authority` passes it as `role`), so its candidates are
//! all in one role and the choice is narrowest covering scope, then lowest
//! rank. Candidates in different roles arise on the grant verbs, which
//! filter by capability only; those cases are tested there.

use crate::authority::fixture::{self, accepted, grant, role};
use crate::authority::grant::GrantScope;
use crate::authority::role::{Act, Capability};
use crate::authority::view::Authority;
use crate::authority::{authorize, Denial, Target};
use crate::testkit;

const WHO: &str = "architect@customer.example";

fn roles() -> Vec<crate::authority::Role> {
    vec![
        role("acceptor", &[Capability::AcceptDecision]),
        role("delegate", &[Capability::GrantRole]),
        role("lead", &[Capability::GrantRole, Capability::AcceptDecision]),
    ]
}

fn store_of(grants: Vec<crate::authority::Grant>) -> crate::store::Store {
    let acc = grants.iter().enumerate().map(|(i, g)| accepted(&format!("9{i}"), g)).collect();
    fixture::store(roles(), fixture::changeset(grants, acc))
}

fn at() -> chrono::DateTime<chrono::Utc> {
    testkit::stamp("2026-10-15T12:00:00Z")
}

/// `accept`'s path: the policy's accept role is the only role that counts.
fn accept(store: &crate::store::Store, as_role: Option<&str>) -> Result<String, Denial> {
    let target = Target::Decision { namespace: "hafeok.ledger", set: "ledger-design" };
    authorize(&Authority::build(store), &testkit::identity(WHO), Act::Accept, target, at(), Some("acceptor"), as_role)
        .map(|a| a.grant)
}

/// The grant verb's path: any role that may `grant-role` counts.
fn grant_over_ns(store: &crate::store::Store, as_role: Option<&str>) -> Result<String, Denial> {
    let scope = GrantScope::Namespace("hafeok.ledger".into());
    authorize(&Authority::build(store), &testkit::identity(WHO), Act::Grant, Target::Scope(&scope), at(), None, as_role)
        .map(|a| a.grant)
}

#[test]
fn one_qualifying_grant_is_used_without_as() {
    let g = grant("1", "acceptor", WHO, "ns:hafeok.ledger", 0);
    assert_eq!(accept(&store_of(vec![g.clone()]), None), Ok(g.id.to_string()));
}

#[test]
fn accept_counts_only_the_accept_role() {
    // `lead` may accept-decision, but the policy names `acceptor`.
    let lead = grant("1", "lead", WHO, "ns:hafeok.ledger", 0);
    assert_eq!(accept(&store_of(vec![lead]), None), Err(Denial::NoGrant));
}

#[test]
fn on_accept_two_grants_need_as_and_the_narrowest_scope_then_lowest_rank_wins() {
    let star = grant("1", "acceptor", WHO, "*", 0);
    let set = grant("2", "acceptor", WHO, "set:ledger-design", 0);
    let ns = grant("3", "acceptor", WHO, "ns:hafeok.ledger", 0);
    let store = store_of(vec![star.clone(), set.clone(), ns]);
    assert_eq!(accept(&store, None), Err(Denial::Ambiguous(vec!["acceptor".into()])));
    assert_eq!(accept(&store, Some("acceptor")), Ok(set.id.to_string()));
    let ns0 = grant("4", "acceptor", WHO, "ns:hafeok.ledger", 0);
    let ns2 = grant("5", "acceptor", WHO, "ns:hafeok.ledger", 2);
    assert_eq!(accept(&store_of(vec![star, ns2, ns0.clone()]), Some("acceptor")), Ok(ns0.id.to_string()));
}

#[test]
fn on_a_grant_verb_two_grants_in_different_roles_need_as() {
    let narrow = grant("1", "delegate", WHO, "ns:hafeok.ledger", 0);
    let broad = grant("2", "lead", WHO, "ns:hafeok.ledger", 0);
    let store = store_of(vec![narrow.clone(), broad]);
    assert_eq!(grant_over_ns(&store, None), Err(Denial::Ambiguous(vec!["delegate".into(), "lead".into()])));
    assert_eq!(grant_over_ns(&store, Some("delegate")), Ok(narrow.id.to_string()));
}

#[test]
fn on_a_grant_verb_the_broader_role_is_refused_when_a_narrower_one_qualifies() {
    let narrow = grant("1", "delegate", WHO, "ns:hafeok.ledger", 0);
    let broad = grant("2", "lead", WHO, "ns:hafeok.ledger", 0);
    assert_eq!(
        grant_over_ns(&store_of(vec![narrow, broad.clone()]), Some("lead")),
        Err(Denial::Broader { grant: broad.id.to_string(), role: "lead".into(), narrower: "delegate".into() })
    );
}

#[test]
fn as_naming_a_role_with_no_qualifying_grant_is_refused() {
    let g = grant("1", "delegate", WHO, "ns:hafeok.ledger", 0);
    assert_eq!(grant_over_ns(&store_of(vec![g]), Some("lead")), Err(Denial::NotInRole("lead".into())));
}

fn store_with(roles: Vec<crate::authority::Role>, grants: Vec<crate::authority::Grant>) -> crate::store::Store {
    let acc = grants.iter().enumerate().map(|(i, g)| accepted(&format!("8{i}"), g)).collect();
    fixture::store(roles, fixture::changeset(grants, acc))
}

#[test]
fn roles_whose_may_sets_are_not_nested_are_not_ordered() {
    // Equal counts, different sets: neither claims a subset of the other.
    let a = role("granter-a", &[Capability::GrantRole, Capability::AcceptDecision]);
    let b = role("granter-b", &[Capability::GrantRole, Capability::RevokeGrant]);
    // Fewer capabilities, but not a subset: a count would order it.
    let c = role("granter-c", &[Capability::GrantRole, Capability::SignOffPattern, Capability::WaiveInvalidation]);
    let ga = grant("1", "granter-a", WHO, "ns:hafeok.ledger", 0);
    let gb = grant("2", "granter-b", WHO, "ns:hafeok.ledger", 0);
    let gc = grant("3", "granter-c", WHO, "ns:hafeok.ledger", 0);
    let store = store_with(vec![a, b, c], vec![ga.clone(), gb.clone(), gc.clone()]);
    assert_eq!(grant_over_ns(&store, Some("granter-a")), Ok(ga.id.to_string()));
    assert_eq!(grant_over_ns(&store, Some("granter-b")), Ok(gb.id.to_string()));
    assert_eq!(grant_over_ns(&store, Some("granter-c")), Ok(gc.id.to_string()), "two claims fewer is no subset");
}

#[test]
fn the_genesis_holder_with_a_one_capability_granting_role_can_grant_another_role() {
    let steward = role(
        "steward",
        &[Capability::GrantRole, Capability::RevokeGrant, Capability::DeclareUnavailability, Capability::RotateGenesis],
    );
    let delegate = role("delegate", &[Capability::GrantRole]);
    let acceptor = role("acceptor", &[Capability::AcceptDecision]);
    let holder = fixture::GENESIS_HOLDER;
    let genesis = fixture::genesis("1", "steward");
    let narrow = grant("2", "delegate", holder, "ns:hafeok.ledger", 0);
    let store = store_with(vec![steward, delegate, acceptor], vec![genesis.clone(), narrow.clone()]);
    let auth = Authority::build(&store);
    let scope = GrantScope::Namespace("hafeok.ledger".into());
    let who = testkit::identity(holder);
    // `delegate` grants only `delegate`, so for `acceptor` the genesis is
    // the one candidate — not refused for claiming more than `delegate`.
    let by = super::grantor(&auth, &who, &scope, "acceptor", at(), None).map(|a| a.grant);
    assert_eq!(by, Ok(genesis.id.to_string()));
    // Granting `delegate` itself, both are candidates: `--as` is required,
    // and the genesis role, a superset, is refused for the narrower one.
    assert_eq!(
        super::grantor(&auth, &who, &scope, "delegate", at(), None).map(|a| a.grant),
        Err(Denial::Ambiguous(vec!["delegate".into(), "steward".into()]))
    );
    assert_eq!(super::grantor(&auth, &who, &scope, "delegate", at(), Some("delegate")).map(|a| a.grant), Ok(narrow.id.to_string()));
    assert!(matches!(super::grantor(&auth, &who, &scope, "delegate", at(), Some("steward")), Err(Denial::Broader { .. })));
}

#[test]
fn below_the_genesis_a_grantor_holding_no_grant_in_the_role_is_refused() {
    let delegate = role("delegate", &[Capability::GrantRole]);
    let acceptor = role("acceptor", &[Capability::AcceptDecision]);
    let g = grant("1", "delegate", WHO, "ns:hafeok.ledger", 0);
    let store = store_with(vec![delegate, acceptor], vec![g]);
    let scope = GrantScope::Namespace("hafeok.ledger".into());
    let out = super::grantor(&Authority::build(&store), &testkit::identity(WHO), &scope, "acceptor", at(), None);
    assert!(matches!(out, Err(Denial::OwnRoleOnly(_))), "{out:?}");
}
