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

/// D9 (e): a `fallback-1` over a set, and a primary over `*` in its role.
fn set_fallback_beside_a_star_primary(primary_role: &str, away_now: bool) -> Result<Authorized, Denial> {
    let primary = grant("1", primary_role, ARCHITECT, "*", 0);
    let fb = grant("2", "architect", BACKUP, "set:ledger-design", 1);
    let mut cs = fixture::changeset(vec![primary.clone(), fb.clone()], vec![accepted("3", &primary), accepted("4", &fb)]);
    if away_now {
        cs.unavailabilities.push(away("5", &primary, "2026-10-01T00:00:00Z", None));
    }
    let roles = vec![
        role("architect", &[Capability::AcceptDecision, Capability::GrantRole]),
        role("reviewer", &[Capability::AcceptDecision]),
    ];
    check(&fixture::store(roles, cs), BACKUP, Act::Accept, decision())
}

#[test]
fn a_set_fallback_waits_on_an_available_star_primary_of_its_role() {
    let refused = set_fallback_beside_a_star_primary("architect", false);
    assert!(matches!(refused, Err(Denial::Outranked(..))), "the primary covers the target: {refused:?}");
    let acts = set_fallback_beside_a_star_primary("architect", true);
    assert!(acts.is_ok(), "the primary is unavailable, so the fallback acts: {acts:?}");
}

#[test]
fn a_primary_in_another_role_never_outranks_a_fallback() {
    assert!(set_fallback_beside_a_star_primary("reviewer", false).is_ok());
    assert!(set_fallback_beside_a_star_primary("reviewer", true).is_ok());
}

#[test]
fn grants_of_equal_rank_act_concurrently() {
    let one = grant("1", "architect", ARCHITECT, "set:ledger-design", 1);
    let two = grant("2", "architect", BACKUP, "*", 1);
    let cs = fixture::changeset(vec![one.clone(), two.clone()], vec![accepted("3", &one), accepted("4", &two)]);
    let store = fixture::store(may_accept(), cs);
    assert_eq!(check(&store, ARCHITECT, Act::Accept, decision()).map(|a| a.grant), Ok(one.id.to_string()));
    assert_eq!(check(&store, BACKUP, Act::Accept, decision()).map(|a| a.grant), Ok(two.id.to_string()));
}

#[test]
fn a_grantor_below_star_grants_over_its_own_scope_only() {
    let g = grant("1", "architect", ARCHITECT, "ns:hafeok.ledger", 0);
    let store = fixture::store(may_accept(), fixture::changeset(vec![g.clone()], vec![accepted("2", &g)]));
    let own = GrantScope::Namespace("hafeok.ledger".into());
    assert!(check(&store, ARCHITECT, Act::Grant, Target::Scope(&own)).is_ok());
    for other in ["*", "ns:hafeok.other", "set:ledger-design", "pattern:money"] {
        let scope: GrantScope = other.parse().expect("scope");
        assert_eq!(
            check(&store, ARCHITECT, Act::Grant, Target::Scope(&scope)),
            Err(Denial::WrongScope(g.id.to_string())),
            "a grant over ns:hafeok.ledger may not grant over {other}"
        );
    }
}
