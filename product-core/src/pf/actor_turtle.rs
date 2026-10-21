//! Turtle emission + seed parsing for the §3.7 actor layer.
//!
//! Emit and parse live side by side so they stay symmetric. An actor kind is a
//! `pf:ActorKind` resource whose claims are `pf:hasField` blank nodes (the §3.2
//! payload encoding, plus `pf:claimMany` / `pf:valuesFrom`). The edges that
//! hang authorization off existing nodes — `pf:actsAs` on a trigger,
//! `pf:offeredTo` on a UI step, `pf:publicView` / `pf:scopedBy` (one
//! `pf:forActor` rule per actor kind) on a read model — are emitted here as separate statements and patched onto the parsed nodes,
//! so an actor-free graph emits exactly what it did before (byte-stable, same
//! graph content hash).

use std::collections::{BTreeMap, HashMap};

use oxigraph::store::Store;

use super::model::{ActorKind, Claim, DomainGraph, ScopeKey, ScopeRule};
use super::seed::{lit as term_lit, local, multi, opt, select};
use super::turtle::lit;
use crate::error::Result;

/// Emit every actor kind, then the actor edges on triggers, steps, and views.
pub(super) fn emit_actors(out: &mut String, g: &DomainGraph) {
    g.actor_kinds.iter().for_each(|a| emit_actor_kind(out, a));
    for t in &g.triggers {
        if let Some(a) = &t.acts_as {
            out.push_str(&format!("d:{} pf:actsAs d:{} .\n\n", t.id, a));
        }
    }
    for w in &g.wireframe_steps {
        for a in &w.offered_to {
            out.push_str(&format!("d:{} pf:offeredTo d:{} .\n\n", w.id, a));
        }
    }
    for rm in &g.read_models {
        if rm.public {
            out.push_str(&format!("d:{} pf:publicView true .\n\n", rm.id));
        }
        for r in &rm.scoped_by {
            out.push_str(&format!("d:{} pf:scopedBy [ pf:forActor d:{}", rm.id, r.actor));
            for k in &r.keys {
                out.push_str(&format!(" ; pf:scopeKey [ pf:viewField {} ; pf:claim {} ]", lit(&k.field), lit(&k.claim)));
            }
            out.push_str(" ] .\n\n");
        }
    }
}

fn emit_actor_kind(out: &mut String, a: &ActorKind) {
    out.push_str(&format!("d:{} a pf:ActorKind ;\n  rdfs:label {}", a.id, lit(&a.label)));
    if !a.nature.is_empty() {
        out.push_str(&format!(" ;\n  pf:actorNature {}", lit(&a.nature)));
    }
    for c in &a.claims {
        out.push_str(&format!(" ;\n  pf:hasField [ pf:attrName {}", lit(&c.name)));
        if let Some(ty) = &c.ty {
            out.push_str(&format!(" ; pf:attrType {}", lit(ty)));
        }
        if c.many {
            out.push_str(" ; pf:claimMany true");
        }
        if let Some(r) = &c.values_from {
            out.push_str(&format!(" ; pf:valuesFrom d:{}", r));
        }
        out.push_str(" ]");
    }
    out.push_str(" .\n\n");
}

/// Parse the actor layer. Runs after triggers, steps, and read models are
/// parsed, because it patches the actor edges onto them.
pub(super) fn parse_actors(store: &Store, g: &mut DomainGraph) -> Result<()> {
    let mut claims = parse_claims(store)?;
    for row in select(store, "?s ?label ?nature",
        "?s a pf:ActorKind . OPTIONAL { ?s rdfs:label ?label } OPTIONAL { ?s pf:actorNature ?nature }")? {
        let id = local(row.get("s"));
        g.actor_kinds.push(ActorKind {
            claims: claims.remove(&id).unwrap_or_default(),
            id, label: term_lit(row.get("label")), nature: term_lit(row.get("nature")),
        });
    }
    let acts_as = multi(store, "pf:Trigger", "pf:actsAs")?;
    for t in &mut g.triggers {
        t.acts_as = acts_as.get(&t.id).and_then(|v| v.first().cloned());
    }
    let offered = multi(store, "pf:WireframeStep", "pf:offeredTo")?;
    for w in &mut g.wireframe_steps {
        w.offered_to = offered.get(&w.id).cloned().unwrap_or_default();
    }
    parse_scopes(store, g)
}

fn parse_claims(store: &Store) -> Result<HashMap<String, Vec<Claim>>> {
    let mut out: HashMap<String, Vec<Claim>> = HashMap::new();
    for row in select(store, "?s ?name ?ty ?many ?from",
        "?s a pf:ActorKind ; pf:hasField ?b . ?b pf:attrName ?name . OPTIONAL { ?b pf:attrType ?ty } OPTIONAL { ?b pf:claimMany ?many } OPTIONAL { ?b pf:valuesFrom ?from }")? {
        out.entry(local(row.get("s"))).or_default().push(Claim {
            name: term_lit(row.get("name")), ty: opt(row.get("ty")),
            many: term_lit(row.get("many")) == "true",
            values_from: opt(row.get("from")).map(|_| local(row.get("from"))),
        });
    }
    Ok(out)
}

fn parse_scopes(store: &Store, g: &mut DomainGraph) -> Result<()> {
    let public: Vec<String> = select(store, "?s", "?s a pf:ReadModel ; pf:publicView true")?
        .iter().map(|r| local(r.get("s"))).collect();
    // Keyed by (view, actor): a rule is one blank node per actor kind.
    let mut rules: BTreeMap<(String, String), Vec<ScopeKey>> = BTreeMap::new();
    for row in select(store, "?s ?actor", "?s a pf:ReadModel ; pf:scopedBy ?b . ?b pf:forActor ?actor")? {
        rules.entry((local(row.get("s")), local(row.get("actor")))).or_default();
    }
    for row in select(store, "?s ?actor ?field ?claim",
        "?s a pf:ReadModel ; pf:scopedBy ?b . ?b pf:forActor ?actor ; pf:scopeKey ?k . ?k pf:viewField ?field ; pf:claim ?claim")? {
        rules.entry((local(row.get("s")), local(row.get("actor")))).or_default().push(ScopeKey {
            field: term_lit(row.get("field")), claim: term_lit(row.get("claim")),
        });
    }
    let mut by_view: HashMap<String, Vec<ScopeRule>> = HashMap::new();
    for ((view, actor), keys) in rules {
        by_view.entry(view).or_default().push(ScopeRule { actor, keys });
    }
    for rm in &mut g.read_models {
        rm.public = public.contains(&rm.id);
        rm.scoped_by = by_view.remove(&rm.id).unwrap_or_default();
    }
    Ok(())
}
