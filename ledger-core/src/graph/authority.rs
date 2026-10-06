//! Emitting the authority records as triples — the vocabulary's read model.
//!
//! The node shapes follow `ledger/spec/authority/ledger-authority.ttl` and
//! its SHACL shapes: roles at `<urn:ledger-role:{id}>`, every other record
//! at `<urn:{id}>`, identities as `mailto:` IRIs. Every node is complete
//! when its record is filed: nothing here adds a triple to a node another
//! record created (ruling 3) — a revocation, an availability and a grant
//! acceptance are nodes of their own that *name* what they concern.

use crate::authority::{Grant, KeyBinding, Policy, Revocation, Role};
use crate::changeset::ChangeSet;

use super::turtle::{literal, mailto, stamp, Triples};

fn role_iri(id: &str) -> String {
    format!("<urn:ledger-role:{id}>")
}

fn iri(id: &impl std::fmt::Display) -> String {
    format!("<urn:{id}>")
}

/// A role node: closed to title, owner, note and its capabilities.
pub(super) fn emit_role(t: &mut Triples, role: &Role) {
    let iri = role_iri(&role.id);
    t.add(&iri, "a", "ledger:Role".into());
    t.add(&iri, "a", "prov:Entity".into());
    t.add(&iri, "ledger:owner", mailto(&role.owner));
    if let Some(title) = &role.title {
        t.add(&iri, "ledger:title", literal(title));
    }
    if let Some(notes) = &role.notes {
        t.add(&iri, "ledger:note", literal(notes));
    }
    for cap in &role.may {
        t.add(&iri, "ledger:may", literal(cap.as_str()));
    }
}

/// Every authority record one change-set filed.
pub(super) fn emit_changeset(t: &mut Triples, cs: &ChangeSet, cs_iri: &str) {
    for g in &cs.grants {
        emit_grant(t, g, cs_iri);
    }
    for ga in &cs.grant_acceptances {
        let n = iri(&ga.id);
        typed(t, &n, "ledger:GrantAcceptance", &ga.id.to_string(), cs_iri);
        t.add(&n, "ledger:grant", iri(&ga.grant));
        t.add(&n, "ledger:signsHash", literal(&ga.signs.to_string()));
        t.add(&n, "prov:wasAttributedTo", mailto(&ga.actor));
        t.add(&n, "prov:generatedAtTime", stamp(&ga.at));
    }
    for u in &cs.unavailabilities {
        let n = iri(&u.id);
        typed(t, &n, "ledger:Unavailability", &u.id.to_string(), cs_iri);
        t.add(&n, "ledger:grant", iri(&u.grant));
        t.add(&n, "ledger:from", stamp(&u.from));
        if let Some(until) = &u.until {
            t.add(&n, "ledger:until", stamp(until));
        }
        t.add(&n, "ledger:basis", literal(u.basis.as_str()));
        if let Some(reason) = &u.reason {
            t.add(&n, "ledger:reason", literal(reason));
        }
        t.add(&n, "prov:wasAttributedTo", mailto(&u.by));
        t.add(&n, "prov:generatedAtTime", stamp(&u.at));
    }
    for av in &cs.availabilities {
        let n = iri(&av.id);
        typed(t, &n, "ledger:Availability", &av.id.to_string(), cs_iri);
        t.add(&n, "ledger:ends", iri(&av.ends));
        t.add(&n, "ledger:availableAt", stamp(&av.available_at));
        t.add(&n, "prov:wasAttributedTo", mailto(&av.by));
        t.add(&n, "prov:generatedAtTime", stamp(&av.at));
    }
    for b in &cs.key_bindings {
        emit_binding(t, b, cs_iri);
    }
    for p in &cs.policies {
        emit_policy(t, p, cs_iri);
    }
}

/// The type pair, id and generating activity every record node carries.
fn typed(t: &mut Triples, node: &str, class: &str, id: &str, cs_iri: &str) {
    t.add(node, "a", class.into());
    t.add(node, "a", "prov:Entity".into());
    t.add(node, "ledger:id", literal(id));
    t.add(node, "prov:wasGeneratedBy", cs_iri.to_string());
}

fn emit_grant(t: &mut Triples, g: &Grant, cs_iri: &str) {
    let n = iri(&g.id);
    typed(t, &n, "ledger:Grant", &g.id.to_string(), cs_iri);
    t.add(&n, "ledger:hash", literal(&g.hash.to_string()));
    t.add(&n, "ledger:role", role_iri(&g.role));
    t.add(&n, "ledger:scope", literal(&g.scope.to_string()));
    t.add(&n, "ledger:holder", mailto(&g.holder));
    t.add(&n, "ledger:grantedBy", mailto(&g.granted_by));
    t.add(&n, "ledger:order", literal(&g.order.to_string()));
    t.add(&n, "ledger:rank", format!("\"{}\"^^xsd:integer", g.order.rank()));
    for limit in &g.limits {
        t.add(&n, "ledger:limit", literal(limit.as_str()));
    }
    if g.genesis {
        t.add(&n, "ledger:genesis", literal("true"));
    }
    if let Some(r) = &g.external_ref {
        t.add(&n, "ledger:externalRef", literal(r));
    }
    if let Some(old) = &g.supersedes {
        t.add(&n, "ledger:supersedesGrant", iri(old));
    }
    under(t, &n, g.under.as_ref());
    t.add(&n, "prov:wasAttributedTo", mailto(&g.granted_by));
    t.add(&n, "prov:generatedAtTime", stamp(&g.at));
}

/// A `rev:` revocation entity: its own node naming what it revokes. Every
/// payload field is a triple, so a consumer can rebuild the signed bytes.
pub(super) fn emit_revocation(t: &mut Triples, r: &Revocation, cs_iri: &str) {
    let (Some(target), Some(actor)) = (r.target(), r.actor()) else { return };
    let n = match (&r.id, r.revoked_acceptance()) {
        (Some(id), _) => {
            let n = iri(id);
            typed(t, &n, "ledger:Revocation", &id.to_string(), cs_iri);
            n
        }
        (None, Some(acc)) => {
            let n = format!("<urn:rev:legacy-{}>", acc.ulid());
            t.add(&n, "a", "ledger:Revocation".into());
            t.add(&n, "a", "prov:Entity".into());
            t.add(&n, "prov:wasGeneratedBy", cs_iri.to_string());
            n
        }
        (None, None) => return,
    };
    let hash = r.hash.clone().unwrap_or_else(|| crate::authority::payload::revocation_hash(r));
    t.add(&n, "ledger:hash", literal(&hash.to_string()));
    t.add(&n, "ledger:revokes", iri(&target));
    t.add(&n, "ledger:revocationReason", literal(&r.reason));
    under(t, &n, r.under.as_ref());
    t.add(&n, "prov:wasAttributedTo", mailto(actor));
    t.add(&n, "prov:generatedAtTime", stamp(&r.at));
}

fn emit_binding(t: &mut Triples, b: &KeyBinding, cs_iri: &str) {
    let n = iri(&b.id);
    typed(t, &n, "ledger:KeyBinding", &b.id.to_string(), cs_iri);
    t.add(&n, "ledger:hash", literal(&b.hash.to_string()));
    t.add(&n, "ledger:bindingAct", literal(b.act.as_str()));
    t.add(&n, "ledger:principal", mailto(&b.principal));
    t.add(&n, "ledger:namespace", literal(&b.namespace));
    if let Some(kt) = &b.key_type {
        t.add(&n, "ledger:keyType", literal(kt));
    }
    if let Some(key) = &b.key {
        t.add(&n, "ledger:publicKey", literal(key));
    }
    if let Some(closed) = &b.closes {
        t.add(&n, "ledger:closes", iri(closed));
    }
    if b.self_bound {
        t.add(&n, "ledger:selfBound", literal("true"));
    }
    if let Some(m) = &b.mandate {
        t.add(&n, "ledger:mandate", literal(m));
    }
    under(t, &n, b.under.as_ref());
    t.add(&n, "prov:wasAttributedTo", mailto(&b.by));
    t.add(&n, "prov:generatedAtTime", stamp(&b.at));
}

fn emit_policy(t: &mut Triples, p: &Policy, cs_iri: &str) {
    let n = iri(&p.id);
    typed(t, &n, "ledger:NamespacePolicy", &p.id.to_string(), cs_iri);
    t.add(&n, "ledger:hash", literal(&p.hash.to_string()));
    t.add(&n, "ledger:namespace", literal(&p.namespace));
    for scheme in &p.schemes {
        t.add(&n, "ledger:requiresScheme", literal(scheme.as_str()));
    }
    if p.require_sk {
        t.add(&n, "ledger:requiresSecurityKey", literal("true"));
    }
    t.add(&n, "ledger:acceptRole", role_iri(&p.accept_role));
    if let Some(days) = p.reaccept_within_days {
        t.add(&n, "ledger:reacceptWithinDays", format!("\"{days}\"^^xsd:integer"));
    }
    if let Some(prior) = &p.replaces {
        t.add(&n, "ledger:replacesPolicy", literal(&prior.to_string()));
    }
    under(t, &n, p.under.as_ref());
    t.add(&n, "prov:wasAttributedTo", mailto(&p.by));
    t.add(&n, "prov:generatedAtTime", stamp(&p.at));
}

/// `ledger:under` — the grant an act is made under (D9 (a)).
fn under(t: &mut Triples, node: &str, grant: Option<&crate::id::GrantId>) {
    if let Some(g) = grant {
        t.add(node, "ledger:under", iri(g));
    }
}

/// A node per signature sidecar of an entity the store emits, and an edge
/// from the entity to it (spec v1.8): `ledger:signature <urn:sig:<file>>`,
/// with the scheme and the sidecar's path under `.decisions/`. With every
/// payload field already a triple, an export-only verifier rebuilds the
/// signed bytes and checks them against the named sidecar.
pub(super) fn emit_signatures(t: &mut Triples, store: &crate::store::Store) {
    let mut entities: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    for cs in store.log.iter().map(|l| &l.file) {
        entities.extend(cs.acceptances.iter().map(|a| (a.id.ulid().to_string(), iri(&a.id))));
        entities.extend(cs.revocations.iter().filter_map(|r| r.id.as_ref().map(|i| (i.ulid().to_string(), iri(i)))));
        entities.extend(cs.key_bindings.iter().map(|b| (b.id.ulid().to_string(), iri(&b.id))));
        entities.extend(cs.policies.iter().map(|p| (p.id.ulid().to_string(), iri(&p.id))));
    }
    for sidecar in &store.sidecars {
        let Some(entity) = entities.get(&sidecar.ulid) else { continue };
        let node = format!("<urn:sig:{}>", sidecar.file);
        t.add(entity, "ledger:signature", node.clone());
        t.add(&node, "a", "ledger:Signature".into());
        t.add(&node, "ledger:signatureScheme", literal(sidecar.scheme.as_str()));
        t.add(&node, "ledger:signatureFile", literal(&format!("{}/{}", crate::signing::SIG_DIR, sidecar.file)));
    }
}
