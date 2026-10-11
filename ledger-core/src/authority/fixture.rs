//! Builders for authority-record tests — roles, grants, acceptances, intervals.

use chrono::NaiveDate;

use crate::changeset::ChangeSet;
use crate::store::Store;
use crate::testkit;

use super::availability::{Basis, Unavailability};
use super::grant::{Grant, GrantAcceptance, GrantScope, Limit, Order};
use super::payload::grant_hash;
use super::role::{Capability, Role};

pub const GENESIS_HOLDER: &str = "owner@customer.example";
/// The namespace the fixture roles and policies belong to.
pub const NS: &str = "hafeok.ledger";

/// A ULID ending in `tail`, distinct per record in one test.
pub fn ulid(tail: &str) -> String {
    format!("01K5M{}{}", "0".repeat(21 - tail.len()), tail)
}

pub fn role(id: &str, may: &[Capability]) -> Role {
    Role {
        namespace: NS.to_string(),
        format: 6,
        id: id.into(),
        title: None,
        owner: testkit::identity(GENESIS_HOLDER),
        may: may.to_vec(),
        created_at: NaiveDate::from_ymd_opt(2026, 10, 1).unwrap_or_default(),
        notes: None,
    }
}

/// A sealed grant. `order` 0 is primary.
pub fn grant(tail: &str, role: &str, holder: &str, scope: &str, order: u32) -> Grant {
    let mut g = Grant {
        id: format!("grant:{}", ulid(tail)).parse().expect("grant id"),
        role: role.into(),
        scope: scope.parse::<GrantScope>().expect("scope"),
        holder: testkit::identity(holder),
        granted_by: testkit::identity(GENESIS_HOLDER),
        order: Order::fallback(order),
        limits: Vec::new(),
        genesis: false,
        external_ref: None,
        supersedes: None,
        under: None,
        at: testkit::stamp("2026-10-01T09:00:00Z"),
        hash: crate::hash::VersionHash::zero(),
    };
    g.hash = grant_hash(&g);
    g
}

/// The genesis grant: self-granted, `*`, primary, with its mandate.
pub fn genesis(tail: &str, role: &str) -> Grant {
    let mut g = grant(tail, role, GENESIS_HOLDER, "*", 0);
    g.genesis = true;
    g.external_ref = Some("contract 2026/117".into());
    reseal(g)
}

pub fn limited(mut g: Grant, limits: &[Limit]) -> Grant {
    g.limits = limits.to_vec();
    reseal(g)
}

pub fn reseal(mut g: Grant) -> Grant {
    g.hash = grant_hash(&g);
    g
}

/// The holder's acceptance of exactly this grant.
pub fn accepted(tail: &str, g: &Grant) -> GrantAcceptance {
    GrantAcceptance {
        id: format!("gacc:{}", ulid(tail)).parse().expect("gacc id"),
        grant: g.id.clone(),
        signs: g.hash.clone(),
        actor: g.holder.clone(),
        at: testkit::stamp("2026-10-01T09:05:00Z"),
    }
}

/// A self-declared interval on `g`, open-ended when `until` is `None`.
pub fn away(tail: &str, g: &Grant, from: &str, until: Option<&str>) -> Unavailability {
    Unavailability {
        id: format!("unav:{}", ulid(tail)).parse().expect("unav id"),
        grant: g.id.clone(),
        from: testkit::stamp(from),
        until: until.map(testkit::stamp),
        basis: Basis::SelfDeclared,
        reason: Some("leave".into()),
        by: g.holder.clone(),
        at: testkit::stamp("2026-10-01T10:00:00Z"),
    }
}

/// A change-set (format 6) holding the given grants and acceptances.
pub fn changeset(grants: Vec<Grant>, accepted: Vec<GrantAcceptance>) -> ChangeSet {
    let mut cs = testkit::changeset(Vec::new(), Vec::new());
    // Format 7: tests add policies to it, and a policy is a format-7 entry.
    cs.format = crate::format::SIGNING_FORMAT;
    cs.grants = grants;
    cs.grant_acceptances = accepted;
    cs
}

/// An in-memory store holding the roles and the one change-set.
pub fn store(roles: Vec<Role>, cs: ChangeSet) -> Store {
    let mut store = testkit::store(cs);
    store.roles = roles;
    store
}
