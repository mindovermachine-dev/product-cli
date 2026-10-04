//! The authority records read off a store — the questions every check asks.
//!
//! Built per run from the log and the role files, never cached: whether a
//! grant is live (filed, accepted by its holder, unrevoked, unsuperseded),
//! whether it is available at an instant, which policy is in force for a
//! namespace, which grant is the genesis. The role check
//! ([`super::check::authorize`]), the schema rules and the emitter all read
//! the same answers from here.

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
    /// Read every authority record the store holds.
    pub fn build(store: &'a Store) -> Self {
        let mut a = Self::default();
        for role in &store.roles {
            a.roles.insert(role.id.clone(), role);
        }
        for cs in store.log.iter().map(|l| &l.file) {
            a.grants.extend(cs.grants.iter().map(|g| (g.id.to_string(), g)));
            a.grant_acceptances.extend(&cs.grant_acceptances);
            a.unavailabilities.extend(cs.unavailabilities.iter().map(|u| (u.id.to_string(), u)));
            a.availabilities.extend(&cs.availabilities);
            a.revocations.extend(&cs.revocations);
            a.bindings.extend(&cs.key_bindings);
            a.policies.extend(&cs.policies);
        }
        a
    }

    /// The records as they stood at `pos` (D6): enabling entries (grants,
    /// grant acceptances, key bindings, policies) that are not after it,
    /// terminating ones (revocations) that are before it, and the
    /// availability intervals landed no later (their clock decides). This
    /// is what `verify` judges a historic act against (`A006`, D7).
    pub fn as_of(store: &'a Store, landing: &Landing, pos: Position) -> Self {
        let mut a = Self::default();
        for role in &store.roles {
            a.roles.insert(role.id.clone(), role);
        }
        for logged in &store.log {
            let path = relative(&store.root, &logged.path);
            let at = |t| landing.position(&path, t);
            let landed = landing.index(&path) <= pos.index;
            let cs = &logged.file;
            a.grants.extend(cs.grants.iter().filter(|g| at(g.at).not_after(&pos)).map(|g| (g.id.to_string(), g)));
            a.grant_acceptances.extend(cs.grant_acceptances.iter().filter(|ga| at(ga.at).not_after(&pos)));
            if landed {
                a.unavailabilities.extend(cs.unavailabilities.iter().map(|u| (u.id.to_string(), u)));
                a.availabilities.extend(&cs.availabilities);
            }
            a.revocations.extend(cs.revocations.iter().filter(|r| at(r.at).before(&pos)));
            a.bindings.extend(cs.key_bindings.iter().filter(|b| at(b.at).not_after(&pos)));
            a.policies.extend(cs.policies.iter().filter(|p| at(p.at).not_after(&pos)));
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

    /// The live genesis grant, when the store has one.
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
