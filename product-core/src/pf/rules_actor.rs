//! The §3.7 actor checks — per-node shapes plus the graph-only strict half.
//!
//! Per-node: an actor kind declares a closed nature and well-typed claims; a
//! trigger's `acts_as`, a UI step's `offered_to`, and a View's scope resolve and
//! agree with each other. Strict (`domain validate --strict`): the surface half
//! of actor coverage and scope coverage (§6.3). Both strict checks are inert
//! until the graph declares an actor kind — an actor-free What is a pre-§3.7
//! What and judged as before. The Decider half of actor coverage, claim
//! justification over Decider reads, and provider coverage (§4.2) need the
//! Decider's actor input and the infrastructure contract; they are not here.

use std::collections::BTreeSet;

use super::ids::NodeKind;
use super::model::{ActorKind, DomainGraph, ReadModel, Trigger, WireframeStep, ACTOR_NATURES};
use super::validate::Violation;

/// The nature a trigger's `source` requires of the actor kind it acts as.
fn nature_for(source: &str) -> Option<&'static str> {
    match source {
        "user" => Some("human"),
        "automated" => Some("process"),
        "external" => Some("external"),
        _ => None,
    }
}

fn nature_of<'a>(graph: &'a DomainGraph, id: &str) -> Option<&'a str> {
    graph.actor_kinds.iter().find(|a| a.id == id).map(|a| a.nature.as_str())
}

/// §3.7 — an actor kind: a closed nature, uniquely named claims typed from the
/// §3.2 datatype vocabulary, value spaces drawn from declared reference data.
pub(super) fn check_actor_kind(a: &ActorKind, graph: &DomainGraph, v: &mut Vec<Violation>) {
    if !ACTOR_NATURES.contains(&a.nature.as_str()) {
        v.push(Violation::new(&a.id, "nature",
            "§3.7 An actor kind must declare its nature: human, process, or external."));
    }
    let mut seen = BTreeSet::new();
    for c in &a.claims {
        if c.name.trim().is_empty() || !seen.insert(c.name.as_str()) {
            v.push(Violation::new(&a.id, "claims",
                "§3.7 Every claim must carry a name, unique within its actor kind."));
        }
        if let Some(ty) = &c.ty {
            if !super::rules_data::DATATYPES.contains(&ty.as_str()) {
                v.push(Violation::new(&a.id, "claims",
                    "§3.7 A claim's type must come from the §3.2 datatype vocabulary (string, integer, number, boolean, date)."));
            }
        }
        if let Some(r) = &c.values_from {
            if !graph.is_kind(r, NodeKind::ReferenceSet) {
                v.push(Violation::new(&a.id, "claims",
                    "§3.7 A claim's values_from must resolve to declared reference data (§3.1)."));
            }
        }
    }
}

/// §3.7 — `acts_as` names an actor kind whose nature the trigger's source admits.
pub(super) fn check_trigger_actor(t: &Trigger, graph: &DomainGraph, v: &mut Vec<Violation>) {
    let Some(a) = &t.acts_as else { return };
    match (nature_of(graph, a), nature_for(&t.source)) {
        (None, _) => v.push(Violation::new(&t.id, "acts_as",
            "§3.7 A trigger's acts_as must resolve to a declared ActorKind.")),
        (Some(have), Some(want)) if have != want => v.push(Violation::new(&t.id, "acts_as",
            &format!("§3.7 A {} trigger acts as an actor of nature {want}, not {have}.", t.source))),
        _ => {}
    }
}

/// §3.7 — a UI step is offered to declared actor kinds, each a human (a step is
/// where a person sees something and acts, §3.2.1).
pub(super) fn check_step_actor(w: &WireframeStep, graph: &DomainGraph, v: &mut Vec<Violation>) {
    for a in &w.offered_to {
        match nature_of(graph, a) {
            None => v.push(Violation::new(&w.id, "offered_to",
                "§3.7 A UI step's offered_to must resolve to a declared ActorKind.")),
            Some(n) if n != "human" => v.push(Violation::new(&w.id, "offered_to",
                "§3.7 A UI step is offered only to human actor kinds — a process or external system acts through a trigger.")),
            _ => {}
        }
    }
}

/// §3.7 — a View is public or scoped, never both; a scope has one rule per
/// declared actor kind, and every key compares a named view field with a claim
/// that rule's actor kind carries.
pub(super) fn check_view_scope(rm: &ReadModel, graph: &DomainGraph, v: &mut Vec<Violation>) {
    if rm.public && !rm.scoped_by.is_empty() {
        v.push(Violation::new(&rm.id, "scoped_by",
            "§3.7 A View is either public or scoped by the actor's claims, not both."));
    }
    let mut seen = BTreeSet::new();
    for r in &rm.scoped_by {
        let Some(a) = graph.actor_kinds.iter().find(|a| a.id == r.actor) else {
            v.push(Violation::new(&rm.id, "scoped_by", &format!(
                "§3.7 A scope rule's actor '{}' must resolve to a declared ActorKind.", r.actor)));
            continue;
        };
        if !seen.insert(r.actor.as_str()) {
            v.push(Violation::new(&rm.id, "scoped_by", &format!(
                "§3.7 A View carries one scope rule per actor kind; '{}' has more than one.", r.actor)));
        }
        for k in &r.keys {
            if k.field.trim().is_empty() || k.claim.trim().is_empty() {
                v.push(Violation::new(&rm.id, "scoped_by",
                    "§3.7 A scope key must name both the view field and the claim it is compared with."));
            } else if !a.claims.iter().any(|c| c.name == k.claim) {
                v.push(Violation::new(&rm.id, "scoped_by", &format!(
                    "§3.7 Scope rule for '{}' reads claim '{}', which that actor kind does not carry.", a.id, k.claim)));
            }
        }
    }
}

/// Run the actor shape that targets the node with this id (the in-loop path).
pub(super) fn check_local(graph: &DomainGraph, id: &str, v: &mut Vec<Violation>) {
    match graph.kind_of(id) {
        Some(NodeKind::ActorKind) => graph.actor_kinds.iter().filter(|n| n.id == id).for_each(|a| check_actor_kind(a, graph, v)),
        Some(NodeKind::Trigger) => graph.triggers.iter().filter(|n| n.id == id).for_each(|t| check_trigger_actor(t, graph, v)),
        Some(NodeKind::WireframeStep) => graph.wireframe_steps.iter().filter(|n| n.id == id).for_each(|w| check_step_actor(w, graph, v)),
        Some(NodeKind::ReadModel) => graph.read_models.iter().filter(|n| n.id == id).for_each(|rm| check_view_scope(rm, graph, v)),
        _ => {}
    }
}

/// Run every per-node actor shape over the whole graph.
pub(super) fn check_all(graph: &DomainGraph, v: &mut Vec<Violation>) {
    graph.actor_kinds.iter().for_each(|a| check_actor_kind(a, graph, v));
    graph.triggers.iter().for_each(|t| check_trigger_actor(t, graph, v));
    graph.wireframe_steps.iter().for_each(|w| check_step_actor(w, graph, v));
    graph.read_models.iter().for_each(|rm| check_view_scope(rm, graph, v));
}

/// The strict actor checks (§6.3). Inert when the graph declares no actor kind.
pub fn actor_conformance(graph: &DomainGraph) -> Vec<Violation> {
    let mut v = Vec::new();
    if graph.actor_kinds.is_empty() {
        return v;
    }
    surface_coverage(graph, &mut v);
    scope_coverage(graph, &mut v);
    v
}

/// The surface half of actor coverage: every step that offers or surfaces
/// something names who it is offered to, and every automated or external
/// trigger names the actor kind it acts as. (A user trigger's actor is its
/// step's.) The Decider half — each offered (command, actor kind) accepted or
/// reachably rejected — needs the Decider's actor input.
fn surface_coverage(graph: &DomainGraph, v: &mut Vec<Violation>) {
    for w in &graph.wireframe_steps {
        if (!w.offers.is_empty() || !w.surfaces.is_empty()) && w.offered_to.is_empty() {
            v.push(Violation::new(&w.id, "offered_to",
                "§6.3 Actor coverage: this UI step is offered to no actor kind — its surface authorization is unstated."));
        }
    }
    for t in &graph.triggers {
        if matches!(t.source.as_str(), "automated" | "external") && t.acts_as.is_none() {
            v.push(Violation::new(&t.id, "acts_as",
                "§6.3 Actor coverage: this trigger acts as no actor kind — who issues its command is unstated."));
        }
    }
}

/// Scope coverage, at the seam: every View a step surfaces is declared public,
/// or carries a scope rule for every actor kind the step is offered to. (That
/// each rule's claims are carried by its actor kind is the per-node shape.)
fn scope_coverage(graph: &DomainGraph, v: &mut Vec<Violation>) {
    for w in &graph.wireframe_steps {
        for s in &w.surfaces {
            let Some(rm) = graph.read_models.iter().find(|r| r.id == s.projection) else { continue };
            if rm.public {
                continue;
            }
            if rm.scoped_by.is_empty() {
                v.push(Violation::new(&rm.id, "scoped_by", &format!(
                    "§6.3 Scope coverage: step '{}' surfaces this View, which is neither scoped nor declared public.", w.id)));
                continue;
            }
            for a in w.offered_to.iter().filter(|a| !rm.scoped_by.iter().any(|r| &r.actor == *a)) {
                v.push(Violation::new(&w.id, "offered_to", &format!(
                    "§6.3 Scope coverage: View '{}' has no scope rule for actor kind '{a}', to whom this step is offered.", rm.id)));
            }
        }
    }
}

#[cfg(test)]
#[path = "rules_actor_tests.rs"]
mod tests;
