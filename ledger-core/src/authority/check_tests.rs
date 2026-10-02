//! Every arm of the role check, one store each.

use crate::authority::fixture::{self, accepted, away, genesis, grant, limited, role};
use crate::authority::grant::{GrantScope, Limit};
use crate::authority::role::{Act, Capability};
use crate::authority::view::Authority;
use crate::testkit;

use super::*;

const NOW: &str = "2026-10-15T12:00:00Z";
const ARCHITECT: &str = "architect@customer.example";
const BACKUP: &str = "backup@customer.example";

fn decision() -> Target<'static> {
    Target::Decision { namespace: "hafeok.ledger", set: "ledger-design" }
}

fn may_accept() -> Vec<crate::authority::Role> {
    vec![role("architect", &[Capability::AcceptDecision, Capability::GrantRole])]
}

fn check(store: &crate::store::Store, who: &str, act: Act, target: Target<'_>) -> Result<Authorized, Denial> {
    authorize(&Authority::build(store), &testkit::identity(who), act, target, testkit::stamp(NOW), None)
}

#[test]
fn a_live_accepted_available_primary_grant_authorizes() {
    let g = grant("1", "architect", ARCHITECT, "ns:hafeok.ledger", 0);
    let store = fixture::store(may_accept(), fixture::changeset(vec![g.clone()], vec![accepted("2", &g)]));
    let ok = check(&store, ARCHITECT, Act::Accept, decision()).expect("authorized");
    assert_eq!(ok.grant, g.id.to_string());
}

#[test]
fn no_grant_is_refused() {
    let store = fixture::store(may_accept(), fixture::changeset(Vec::new(), Vec::new()));
    assert_eq!(check(&store, ARCHITECT, Act::Accept, decision()), Err(Denial::NoGrant));
}

#[test]
fn a_role_without_the_capability_is_no_grant() {
    let g = grant("1", "reader", ARCHITECT, "*", 0);
    let roles = vec![role("reader", &[Capability::SignOffPattern])];
    let store = fixture::store(roles, fixture::changeset(vec![g.clone()], vec![accepted("2", &g)]));
    assert_eq!(check(&store, ARCHITECT, Act::Accept, decision()), Err(Denial::NoGrant));
}

#[test]
fn a_grant_not_accepted_by_its_holder_is_refused() {
    let g = grant("1", "architect", ARCHITECT, "ns:hafeok.ledger", 0);
    let store = fixture::store(may_accept(), fixture::changeset(vec![g.clone()], Vec::new()));
    assert_eq!(check(&store, ARCHITECT, Act::Accept, decision()), Err(Denial::NotAccepted(g.id.to_string())));
}

#[test]
fn an_unavailable_holder_is_refused_until_the_interval_ends() {
    let g = grant("1", "architect", ARCHITECT, "ns:hafeok.ledger", 0);
    let mut cs = fixture::changeset(vec![g.clone()], vec![accepted("2", &g)]);
    cs.unavailabilities.push(away("3", &g, "2026-10-10T00:00:00Z", Some("2026-10-20T00:00:00Z")));
    let store = fixture::store(may_accept(), cs);
    assert_eq!(check(&store, ARCHITECT, Act::Accept, decision()), Err(Denial::Unavailable(g.id.to_string())));
    let later = authorize(
        &Authority::build(&store),
        &testkit::identity(ARCHITECT),
        Act::Accept,
        decision(),
        testkit::stamp("2026-10-20T00:00:00Z"),
        None,
    );
    assert!(later.is_ok(), "the interval is [from, until): {later:?}");
}

#[test]
fn an_availability_ends_an_open_interval_early() {
    let g = grant("1", "architect", ARCHITECT, "ns:hafeok.ledger", 0);
    let mut cs = fixture::changeset(vec![g.clone()], vec![accepted("2", &g)]);
    let interval = away("3", &g, "2026-10-10T00:00:00Z", None);
    cs.availabilities.push(crate::authority::Availability {
        id: format!("avail:{}", fixture::ulid("4")).parse().expect("id"),
        ends: interval.id.clone(),
        available_at: testkit::stamp("2026-10-14T00:00:00Z"),
        by: g.holder.clone(),
        at: testkit::stamp("2026-10-14T00:00:00Z"),
    });
    cs.unavailabilities.push(interval);
    let store = fixture::store(may_accept(), cs);
    assert!(check(&store, ARCHITECT, Act::Accept, decision()).is_ok());
}

#[test]
fn a_grant_over_another_scope_is_refused_as_wrong_scope() {
    let g = grant("1", "architect", ARCHITECT, "ns:other.ns", 0);
    let store = fixture::store(may_accept(), fixture::changeset(vec![g.clone()], vec![accepted("2", &g)]));
    assert_eq!(check(&store, ARCHITECT, Act::Accept, decision()), Err(Denial::WrongScope(g.id.to_string())));
    let by_set = grant("3", "architect", ARCHITECT, "set:ledger-design", 0);
    let store = fixture::store(may_accept(), fixture::changeset(vec![by_set.clone()], vec![accepted("4", &by_set)]));
    assert!(check(&store, ARCHITECT, Act::Accept, decision()).is_ok(), "a set scope covers its decisions");
}

#[test]
fn a_fallback_limit_withholds_its_act() {
    let primary = grant("1", "architect", ARCHITECT, "*", 0);
    let fb = limited(grant("2", "architect", BACKUP, "*", 1), &[Limit::NoGrants]);
    let mut cs = fixture::changeset(vec![primary.clone(), fb.clone()], vec![accepted("3", &primary), accepted("4", &fb)]);
    cs.unavailabilities.push(away("5", &primary, "2026-10-01T00:00:00Z", None));
    let store = fixture::store(may_accept(), cs);
    let scope = GrantScope::Namespace("hafeok.ledger".into());
    assert_eq!(
        check(&store, BACKUP, Act::Grant, Target::Scope(&scope)),
        Err(Denial::Limited(fb.id.to_string(), "no-grants"))
    );
    assert!(check(&store, BACKUP, Act::Accept, decision()).is_ok(), "the limit withholds granting, not accepting");
}

#[test]
fn a_fallback_acts_only_while_those_before_it_cannot() {
    let primary = grant("1", "architect", ARCHITECT, "*", 0);
    let fb = grant("2", "architect", BACKUP, "*", 1);
    let cs = fixture::changeset(vec![primary.clone(), fb.clone()], vec![accepted("3", &primary), accepted("4", &fb)]);
    let store = fixture::store(may_accept(), cs);
    assert_eq!(
        check(&store, BACKUP, Act::Accept, decision()),
        Err(Denial::Outranked(fb.id.to_string(), primary.id.to_string()))
    );
}

#[test]
fn a_revoked_grant_confers_nothing() {
    let g = grant("1", "architect", ARCHITECT, "*", 0);
    let mut cs = fixture::changeset(vec![g.clone()], vec![accepted("2", &g)]);
    cs.revocations.push(crate::authority::Revocation {
        id: Some(format!("rev:{}", fixture::ulid("3")).parse().expect("id")),
        revokes: Some(crate::authority::Revocable::Grant(g.id.clone())),
        acceptance: None,
        at: testkit::stamp("2026-10-02T00:00:00Z"),
        actor: Some(testkit::identity(fixture::GENESIS_HOLDER)),
        by: None,
        reason: "moved to the platform team".into(),
        hash: None,
    });
    let store = fixture::store(may_accept(), cs);
    assert_eq!(check(&store, ARCHITECT, Act::Accept, decision()), Err(Denial::NotStanding(g.id.to_string())));
}

#[test]
fn a_policy_mapping_counts_only_the_mapped_role() {
    let g = genesis("1", "steward");
    let roles = vec![role("steward", Capability::ALL), role("architect", &[Capability::AcceptDecision])];
    let store = fixture::store(roles, fixture::changeset(vec![g.clone()], vec![accepted("2", &g)]));
    let auth = Authority::build(&store);
    let who = testkit::identity(fixture::GENESIS_HOLDER);
    let at = testkit::stamp(NOW);
    assert!(authorize(&auth, &who, Act::Accept, decision(), at, Some("steward")).is_ok());
    assert_eq!(authorize(&auth, &who, Act::Accept, decision(), at, Some("architect")), Err(Denial::NoGrant));
}
