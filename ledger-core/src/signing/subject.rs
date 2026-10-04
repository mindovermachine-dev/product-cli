//! The signable entities a store holds, each with the bytes it signs.
//!
//! - an **acceptance**: signed by its actor in its decision's namespace;
//! - a **revocation of an acceptance**: by its actor, in the revoked
//!   acceptance's namespace (a grant's revocation is signed with the grant,
//!   D5 (b), #82 — not yet);
//! - a **key binding**: by its filer (`by`) in its namespace — the
//!   principal for its own keys, the genesis holder for a first key (D7)
//!   or a revoke it files; a self-bound binding by the key it binds;
//! - a **policy change** (format 7): by its author, under the policy it
//!   replaces. A namespace's first policy replaces nothing and is unsigned.

use chrono::{DateTime, Utc};

use crate::authority::payload::{acceptance_bytes, binding_bytes, policy_bytes, revocation_bytes};
use crate::authority::{KeyBinding, Policy, Revocable};
use crate::identity::Identity;
use crate::landing::{relative, Landing, Position};
use crate::store::Store;

/// What kind of entity a subject is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Acceptance,
    Revocation,
    Binding,
    Policy,
}

/// One signable entity.
#[derive(Debug, Clone)]
pub struct Subject<'a> {
    pub kind: Kind,
    /// The full id (`acc:…`, `rev:…`, `key:…`, `pol:…`).
    pub id: String,
    /// The id's ULID — the sidecar's name.
    pub ulid: String,
    /// The ledger namespace whose policy governs it and whose signature
    /// namespace (`ledger-accept@<ns>`) it is signed in.
    pub namespace: String,
    /// Whose key signs.
    pub signer: Identity,
    pub at: DateTime<Utc>,
    pub position: Position,
    /// The exact signed bytes.
    pub bytes: Vec<u8>,
    pub binding: Option<&'a KeyBinding>,
    pub policy: Option<&'a Policy>,
}

fn ulid_of(id: &str) -> String {
    id.split_once(':').map(|(_, u)| u.to_string()).unwrap_or_default()
}

/// Every signable entity in `store`, positioned by `landing`.
pub fn subjects<'a>(store: &'a Store, landing: &Landing) -> Vec<Subject<'a>> {
    let mut out = Vec::new();
    for logged in &store.log {
        let path = relative(&store.root, &logged.path);
        let cs = &logged.file;
        for a in &cs.acceptances {
            out.push(Subject {
                kind: Kind::Acceptance,
                id: a.id.to_string(),
                ulid: ulid_of(&a.id.to_string()),
                namespace: a.decision.namespace().to_string(),
                signer: a.actor.clone(),
                at: a.at,
                position: landing.position(&path, a.at),
                bytes: acceptance_bytes(a),
                binding: None,
                policy: None,
            });
        }
        for r in cs.revocations.iter().filter(|r| r.is_entity()) {
            let (Some(Revocable::Acceptance(acc)), Some(actor)) = (r.target(), r.actor()) else { continue };
            let Some(namespace) = acceptance_namespace(store, &acc.to_string()) else { continue };
            out.push(Subject {
                kind: Kind::Revocation,
                id: r.subject(),
                ulid: ulid_of(&r.subject()),
                namespace,
                signer: actor.clone(),
                at: r.at,
                position: landing.position(&path, r.at),
                bytes: revocation_bytes(r),
                binding: None,
                policy: None,
            });
        }
        for b in &cs.key_bindings {
            out.push(Subject {
                kind: Kind::Binding,
                id: b.id.to_string(),
                ulid: ulid_of(&b.id.to_string()),
                namespace: b.namespace.clone(),
                signer: b.by.clone(),
                at: b.at,
                position: landing.position(&path, b.at),
                bytes: binding_bytes(b),
                binding: Some(b),
                policy: None,
            });
        }
        for p in cs.policies.iter().filter(|p| p.at_hashed && p.replaces.is_some()) {
            out.push(Subject {
                kind: Kind::Policy,
                id: p.id.to_string(),
                ulid: ulid_of(&p.id.to_string()),
                namespace: p.namespace.clone(),
                signer: p.by.clone(),
                at: p.at,
                position: landing.position(&path, p.at),
                bytes: policy_bytes(p),
                binding: None,
                policy: Some(p),
            });
        }
    }
    out
}

/// The namespace of the acceptance `id`, wherever it is filed.
pub fn acceptance_namespace(store: &Store, id: &str) -> Option<String> {
    store
        .log
        .iter()
        .flat_map(|l| l.file.acceptances.iter())
        .find(|a| a.id.to_string() == id)
        .map(|a| a.decision.namespace().to_string())
}

/// Every entity ULID a sidecar may name: the signable entities' and
/// nothing else, so an orphan sidecar is a `SCHEMA` fault.
pub fn signable_ulids(store: &Store) -> std::collections::BTreeSet<String> {
    subjects(store, &Landing::unknown()).into_iter().map(|s| s.ulid).chain(
        // A first policy is never signed, but it is an entity a hand-written
        // sidecar could name; it is listed so the fault reads "not required".
        store.log.iter().flat_map(|l| l.file.policies.iter().map(|p| ulid_of(&p.id.to_string()))),
    ).collect()
}
