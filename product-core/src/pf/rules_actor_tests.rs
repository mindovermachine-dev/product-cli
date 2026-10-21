//! Unit tests for the §3.7 actor checks (per-node shapes, strict coverage).

use super::*;
use crate::pf::model::*;
use crate::pf::validate::{validate_graph, validate_node};

fn customer() -> ActorKind {
    ActorKind {
        id: "customer".into(), label: "Customer".into(), nature: "human".into(),
        claims: vec![Claim { name: "customer_id".into(), ty: Some("string".into()), ..Default::default() }],
    }
}

/// A shop step surfacing `orders` and offering `cancel`, plus a restocking robot.
fn base() -> DomainGraph {
    let mut g = DomainGraph::default();
    g.read_models.push(ReadModel { id: "orders".into(), label: "Orders".into(), projects: vec!["Order".into()], ..Default::default() });
    g.wireframe_steps.push(WireframeStep {
        id: "my-orders".into(), label: "My orders".into(),
        surfaces: vec![Surface { projection: "orders".into(), aio: "display-collection".into() }],
        offers: vec![Offer { command: "cancel".into(), aio: "trigger-action".into() }],
        ..Default::default()
    });
    g.triggers.push(Trigger { id: "restock".into(), label: "Restock".into(), source: "automated".into(), issues: "cancel".into(), watches: Some("orders".into()), ..Default::default() });
    g
}

fn paths(v: &[Violation], focus: &str) -> Vec<String> {
    v.iter().filter(|x| x.focus == focus).map(|x| x.path.clone()).collect()
}

#[test]
fn an_actor_free_graph_is_judged_as_before() {
    assert!(actor_conformance(&base()).is_empty());
}

#[test]
fn surface_coverage_wants_offered_to_and_acts_as() {
    let mut g = base();
    g.actor_kinds.push(customer());
    let v = actor_conformance(&g);
    assert!(paths(&v, "my-orders").contains(&"offered_to".to_string()), "{v:?}");
    assert!(paths(&v, "restock").contains(&"acts_as".to_string()), "{v:?}");

    g.actor_kinds.push(ActorKind { id: "robot".into(), label: "Robot".into(), nature: "process".into(), claims: vec![] });
    g.wireframe_steps[0].offered_to = vec!["customer".into()];
    g.triggers[0].acts_as = Some("robot".into());
    let v = actor_conformance(&g);
    assert!(paths(&v, "my-orders").is_empty() && paths(&v, "restock").is_empty(), "{v:?}");
}

#[test]
fn scope_coverage_wants_a_rule_for_every_offered_actor_kind() {
    let mut g = base();
    g.actor_kinds.push(customer());
    g.actor_kinds.push(ActorKind { id: "staff".into(), label: "Staff".into(), nature: "human".into(), claims: vec![] });
    g.wireframe_steps[0].offered_to = vec!["customer".into(), "staff".into()];
    assert!(paths(&actor_conformance(&g), "orders").contains(&"scoped_by".to_string()), "unscoped View is a finding");

    // Customers see their own orders; staff are not covered yet.
    g.read_models[0].scoped_by = vec![ScopeRule { actor: "customer".into(), keys: vec![ScopeKey { field: "customer_id".into(), claim: "customer_id".into() }] }];
    let v = actor_conformance(&g);
    let missing: Vec<&Violation> = v.iter().filter(|x| x.focus == "my-orders" && x.message.contains("'staff'")).collect();
    assert_eq!(missing.len(), 1, "{v:?}");
    assert!(validate_node(&g, "orders").is_empty(), "staff need not carry customer_id: the rule is per actor kind");

    // Staff see every order — declared, not defaulted.
    g.read_models[0].scoped_by.push(ScopeRule { actor: "staff".into(), keys: vec![] });
    assert!(actor_conformance(&g).iter().all(|x| x.focus != "my-orders" && x.focus != "orders"));

    g.read_models[0].scoped_by.clear();
    g.read_models[0].public = true;
    assert!(actor_conformance(&g).iter().all(|x| x.path != "scoped_by"), "a public View needs no scope");
}

#[test]
fn per_node_shapes_hold_the_actor_layer_together() {
    let mut g = base();
    g.reference_sets.push(ReferenceSet { id: "tenants".into(), concept: "Order".into(), values: vec!["t1".into()], ..Default::default() });
    g.actor_kinds.push(ActorKind {
        id: "bad".into(), label: "Bad".into(), nature: "admin".into(),
        claims: vec![
            Claim { name: "x".into(), ty: Some("uuid".into()), ..Default::default() },
            Claim { name: "x".into(), values_from: Some("ghost".into()), ..Default::default() },
        ],
    });
    assert_eq!(validate_node(&g, "bad").len(), 4, "nature, type, duplicate, values_from");

    g.actor_kinds.push(customer());
    g.triggers[0].acts_as = Some("customer".into());
    assert!(paths(&validate_node(&g, "restock"), "restock").contains(&"acts_as".to_string()), "an automated trigger acts as a process");

    g.actor_kinds.push(ActorKind { id: "robot".into(), label: "Robot".into(), nature: "process".into(), claims: vec![] });
    g.wireframe_steps[0].offered_to = vec!["robot".into()];
    assert!(paths(&validate_node(&g, "my-orders"), "my-orders").contains(&"offered_to".to_string()), "a step is offered to humans");

    g.read_models[0].public = true;
    g.read_models[0].scoped_by = vec![
        ScopeRule { actor: "customer".into(), keys: vec![ScopeKey { field: "customer_id".into(), claim: String::new() }] },
        ScopeRule { actor: "customer".into(), keys: vec![ScopeKey { field: "tenant".into(), claim: "tenants".into() }] },
        ScopeRule { actor: "ghost".into(), keys: vec![] },
    ];
    // public+scoped, unnamed claim, duplicate rule, uncarried claim, unknown actor.
    assert_eq!(validate_node(&g, "orders").iter().filter(|x| x.path == "scoped_by").count(), 5);
    assert!(validate_graph(&g).iter().any(|x| x.focus == "bad"), "the whole-graph pass runs the actor shapes");
}
