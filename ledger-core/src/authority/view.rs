//! The authority records read off a store — the questions every check asks.
//!
//! Built per run from the log and the role files, never cached: whether a
//! grant is live (filed, accepted by its holder, unrevoked, unsuperseded),
//! whether it is available at an instant, which policy is in force for a
//! namespace, which grant is the genesis. The role check
//! ([`super::check::authorize`]), the schema rules and the emitter all read
//! the same answers from here.
//!
//! **Authority is per namespace** (LP-6.31, ruling 47). A grant, grant
//! acceptance, interval, revocation or role belongs to the namespace whose
//! directory holds it, and nothing in one namespace's authority has effect
//! in another: [`Authority::of`] and [`Authority::as_of`] read one
//! namespace's records, and every role check, genesis lookup and policy
//! verdict goes through them. [`Authority::build`] is the store-wide union,
//! for readings that are namespace-independent (a policy by its own
//! `namespace`, every binding the store holds for a repository notice); its
//! `roles` map collides across namespaces and its `genesis` is undefined
//! with more than one, so neither is read off it. Key bindings are the
//! namespace's own too: a key is bound, closed and trusted per namespace
//! (LP-6.32, LP-4.37, LP-4.39).

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};

use crate::landing::{relative, Landing, Position};
use crate::store::Store;

use super::availability::{Availability, Unavailability};
use super::binding::KeyBinding;
use super::grant::{Grant, GrantAcceptance};
use super::policy::Policy;
use super::revocation::{Revocable, Revocation};
use super::role::Role;

/// Every authority record in a store, indexed.
#[derive(Default)]
pub struct Authority<'a> {
    pub roles: BTreeMap<String, &'a Role>,
    pub grants: BTreeMap<String, &'a Grant>,
    pub grant_acceptances: Vec<&'a GrantAcceptance>,
    pub unavailabilities: BTreeMap<String, &'a Unavailability>,
    pub availabilities: Vec<&'a Availability>,
    pub revocations: Vec<&'a Revocation>,
    pub bindings: Vec<&'a KeyBinding>,
    pub policies: Vec<&'a Policy>,
}

impl<'a> Authority<'a> {
    /// Every authority record the store holds, whichever namespace: the
    /// store-wide union (see the module note on what may be read off it).
    pub fn build(store: &'a Store) -> Self {
        Self::gather(store, None)
    }

    /// Every authority record of one namespace: the records its directory
    /// holds, and the roles declared under its `roles/` (ruling 47).
    pub fn of(store: &'a Store, namespace: &str) -> Self {
        Self::gather(store, Some(namespace))
    }

    fn gather(store: &'a Store, namespace: Option<&str>) -> Self {
        let mut a = Self::default();
        for role in store.roles.iter().filter(|r| namespace.is_none_or(|ns| r.namespace == ns)) {
            a.roles.insert(role.id.clone(), role);
        }
        for logged in &store.log {
            let cs = &logged.file;
            if namespace.is_some_and(|ns| logged.namespace != ns) {
                continue;
            }
            a.bindings.extend(&cs.key_bindings);
            a.grants.extend(cs.grants.iter().map(|g| (g.id.to_string(), g)));
            a.grant_acceptances.extend(&cs.grant_acceptances);
            a.unavailabilities.extend(cs.unavailabilities.iter().map(|u| (u.id.to_string(), u)));
            a.availabilities.extend(&cs.availabilities);
            a.revocations.extend(&cs.revocations);
            a.policies.extend(&cs.policies);
        }
        a
    }

    /// One namespace's records as they stood at `pos` (D6): role files
    /// landed no later (landing alone: a role file has no signed `at`);
    /// enabling entries (grants, grant acceptances, key bindings that open a
    /// key) that are not after it; terminating ones (revocations, key
    /// closes) and governing ones (policies) unless the act is before them;
    /// and the availability intervals landed no later (their clock
    /// decides). This is what `verify` judges a historic act against
    /// (`A006`, D7).
    pub fn as_of(store: &'a Store, landing: &Landing, pos: Position, namespace: &str) -> Self {
        let mut a = Self::default();
        for role in store.roles.iter().filter(|r| r.namespace == namespace && role_landing(landing, r) <= pos.index) {
            a.roles.insert(role.id.clone(), role);
        }
        for logged in store.log.iter().filter(|l| l.namespace == namespace) {
            let path = relative(&store.root, &logged.path);
            let cs = &logged.file;
            let at = |list: &str, id: String, t| landing.position(&path, &crate::landed::key(list, &id), t);
            // A key's close is terminating, like a revocation: it applies
            // unless the act is before it. An opening binding enables.
            a.bindings.extend(cs.key_bindings.iter().filter(|b| {
                let here = at("key_bindings", b.id.to_string(), b.at);
                if b.closes.is_some() { !pos.before(&here) } else { here.not_after(&pos) }
            }));
            let landed = |list: &str, id: String| landing.entity_index(&path, &crate::landed::key(list, &id)) <= pos.index;
            a.grants.extend(
                cs.grants.iter().filter(|g| at("grants", g.id.to_string(), g.at).not_after(&pos)).map(|g| (g.id.to_string(), g)),
            );
            a.grant_acceptances
                .extend(cs.grant_acceptances.iter().filter(|ga| at("grant_acceptances", ga.id.to_string(), ga.at).not_after(&pos)));
            a.unavailabilities.extend(
                cs.unavailabilities.iter().filter(|u| landed("unavailabilities", u.id.to_string())).map(|u| (u.id.to_string(), u)),
            );
            a.availabilities.extend(cs.availabilities.iter().filter(|v| landed("availabilities", v.id.to_string())));
            // A terminating entry applies unless the act is before it (D6):
            // landed earlier but backdated, or landed later and dated
            // earlier — either way the act is not before it, so it holds.
            a.revocations.extend(cs.revocations.iter().filter(|r| {
                !pos.before(&landing.position(&path, &crate::landed::revocation_key(r), r.at))
            }));
            // A policy governs every act not before it (D5 (c), D6): an act
            // that landed after a policy is under it, however it is dated.
            a.policies.extend(cs.policies.iter().filter(|p| !pos.before(&at("policies", p.id.to_string(), p.at))));
        }
        a
    }

    /// Whether some revocation names this record.
    pub fn is_revoked(&self, target: &Revocable) -> bool {
        self.revocations.iter().any(|r| r.target().as_ref() == Some(target))
    }

    /// Whether a later grant supersedes this one.
    pub fn is_superseded(&self, grant: &Grant) -> bool {
        self.grants.values().any(|g| g.supersedes.as_ref() == Some(&grant.id))
    }

    /// Whether the holder has accepted this exact grant.
    pub fn is_accepted(&self, grant: &Grant) -> bool {
        self.grant_acceptances
            .iter()
            .any(|ga| ga.grant == grant.id && ga.actor == grant.holder && ga.signs == grant.hash)
    }

    /// Unrevoked and unsuperseded — standing, whether or not accepted yet.
    pub fn is_standing(&self, grant: &Grant) -> bool {
        !self.is_revoked(&Revocable::Grant(grant.id.clone())) && !self.is_superseded(grant)
    }

    /// Standing and accepted: the grant confers authority.
    pub fn is_live(&self, grant: &Grant) -> bool {
        self.is_standing(grant) && self.is_accepted(grant)
    }

    /// Whether no unavailability covers the grant at `at`. An interval
    /// covers `[from, until)`, unless an availability ended it at or before
    /// `at`.
    pub fn is_available(&self, grant: &Grant, at: DateTime<Utc>) -> bool {
        !self.unavailabilities.values().filter(|u| u.grant == grant.id).any(|u| {
            let ended = self
                .availabilities
                .iter()
                .any(|av| av.ends == u.id && av.available_at <= at);
            u.from <= at && u.until.is_none_or(|until| at < until) && !ended
        })
    }

    /// The namespace's live genesis grant, when it has one (LP-6.5). Read
    /// off a per-namespace view only: each namespace has its own.
    pub fn genesis(&self) -> Option<&'a Grant> {
        self.grants.values().copied().find(|g| g.genesis && self.is_live(g))
    }

    /// Every policy of a namespace, in filing order.
    pub fn policies_of(&self, namespace: &str) -> Vec<&'a Policy> {
        self.policies.iter().copied().filter(|p| p.namespace == namespace).collect()
    }

    /// The policy in force: the tip of the namespace's `replaces` chain.
    /// `None` for an ungoverned (pre-v2) namespace. A forked chain is a
    /// schema fault; the smallest-hash tip stands in so readings stay total.
    pub fn policy(&self, namespace: &str) -> Option<&'a Policy> {
        let all = self.policies_of(namespace);
        let replaced: BTreeSet<String> =
            all.iter().filter_map(|p| p.replaces.as_ref()).map(ToString::to_string).collect();
        all.into_iter()
            .filter(|p| !replaced.contains(&p.hash.to_string()))
            .min_by_key(|p| p.hash.to_string())
    }

    /// The tips of a namespace's policy chain (more than one is a fork).
    pub fn policy_tips(&self, namespace: &str) -> usize {
        let all = self.policies_of(namespace);
        let replaced: BTreeSet<String> =
            all.iter().filter_map(|p| p.replaces.as_ref()).map(ToString::to_string).collect();
        all.iter().filter(|p| !replaced.contains(&p.hash.to_string())).count()
    }

    /// Whether the store carries any authority record at all.
    pub fn is_empty(&self) -> bool {
        self.roles.is_empty() && self.grants.is_empty() && self.policies.is_empty()
    }
}

/// A role file's landing index (D6). A role file carries no signed `at` —
/// `created_at` is a date its author typed, in no payload — so landing alone
/// places it: a role counts for an act that landed no earlier than it.
fn role_landing(landing: &Landing, role: &Role) -> usize {
    landing.index(&format!("{}/{}", crate::layout::relative_dir(&role.namespace, crate::layout::Kind::Role), role.file_name()))
}
