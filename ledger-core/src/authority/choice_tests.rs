//! D9 (b), (c): which grant an act is made under.

use crate::authority::fixture::{self, accepted, grant, role};
use crate::authority::role::{Act, Capability};
use crate::authority::view::Authority;
use crate::authority::{authorize, Denial, Target};
use crate::testkit;

const WHO: &str = "architect@customer.example";

fn decision() -> Target<'static> {
    Target::Decision { namespace: "hafeok.ledger", set: "ledger-design" }
}

fn roles() -> Vec<crate::authority::Role> {
    vec![
        role("acceptor", &[Capability::AcceptDecision]),
        role("architect", &[Capability::AcceptDecision, Capability::GrantRole]),
    ]
}

fn store_of(grants: Vec<crate::authority::Grant>) -> crate::store::Store {
    let acc = grants.iter().enumerate().map(|(i, g)| accepted(&format!("9{i}"), g)).collect();
    fixture::store(roles(), fixture::changeset(grants, acc))
}

fn pick(store: &crate::store::Store, as_role: Option<&str>) -> Result<String, Denial> {
    let auth = Authority::build(store);
    authorize(&auth, &testkit::identity(WHO), Act::Accept, decision(), testkit::stamp("2026-10-15T12:00:00Z"), None, as_role)
        .map(|a| a.grant)
}

#[test]
fn one_qualifying_grant_is_used_without_as() {
    let g = grant("1", "acceptor", WHO, "ns:hafeok.ledger", 0);
    assert_eq!(pick(&store_of(vec![g.clone()]), None), Ok(g.id.to_string()));
}

#[test]
fn two_qualifying_grants_in_different_roles_need_as() {
    let narrow = grant("1", "acceptor", WHO, "ns:hafeok.ledger", 0);
    let broad = grant("2", "architect", WHO, "*", 0);
    let store = store_of(vec![narrow.clone(), broad]);
    assert_eq!(pick(&store, None), Err(Denial::Ambiguous(vec!["acceptor".into(), "architect".into()])));
    assert_eq!(pick(&store, Some("acceptor")), Ok(narrow.id.to_string()));
}

#[test]
fn the_broader_role_is_refused_when_a_narrower_one_qualifies() {
    let narrow = grant("1", "acceptor", WHO, "ns:hafeok.ledger", 0);
    let broad = grant("2", "architect", WHO, "*", 0);
    let refused = pick(&store_of(vec![narrow, broad.clone()]), Some("architect"));
    assert_eq!(
        refused,
        Err(Denial::Broader { grant: broad.id.to_string(), role: "architect".into(), narrower: "acceptor".into() })
    );
}

#[test]
fn within_the_named_role_the_narrowest_scope_wins_then_the_lowest_rank() {
    let star = grant("1", "acceptor", WHO, "*", 0);
    let set = grant("2", "acceptor", WHO, "set:ledger-design", 0);
    let ns = grant("3", "acceptor", WHO, "ns:hafeok.ledger", 0);
    assert_eq!(pick(&store_of(vec![star.clone(), set.clone(), ns]), Some("acceptor")), Ok(set.id.to_string()));
    let ns0 = grant("4", "acceptor", WHO, "ns:hafeok.ledger", 0);
    let ns2 = grant("5", "acceptor", WHO, "ns:hafeok.ledger", 2);
    assert_eq!(pick(&store_of(vec![star, ns2, ns0.clone()]), Some("acceptor")), Ok(ns0.id.to_string()));
}

#[test]
fn as_naming_a_role_with_no_qualifying_grant_is_refused() {
    let g = grant("1", "acceptor", WHO, "ns:hafeok.ledger", 0);
    assert_eq!(pick(&store_of(vec![g]), Some("architect")), Err(Denial::NotInRole("architect".into())));
}
