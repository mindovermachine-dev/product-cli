//! Unit tests for authored open questions (shape, answer, gates, export).

use serde_json::{json, Map, Value};

use super::*;
use crate::pf::edit::{create, remove};
use crate::pf::ids::NodeKind;
use crate::pf::model::*;
use crate::pf::question_gate::{advance_blockers, answer_from_how, finalize_check, session_report, status_counts, to_markdown};
use crate::pf::seed::from_turtle;
use crate::pf::session::DomainSession;
use crate::pf::turtle::to_turtle;
use crate::pf::validate::validate_graph;
use crate::pf::workflow::Phase;

fn session() -> DomainSession {
    let mut s = DomainSession::start("demo", None, vec![], None, "2026-01-01T00:00:00Z".into()).expect("start");
    s.graph.contexts.push(BoundedContext { id: "Billing".into(), label: "Billing".into(), ..Default::default() });
    s.graph.entities.push(Entity {
        id: "Invoice".into(), label: "Invoice".into(), context: "Billing".into(),
        definition: "a bill".into(), ..Default::default()
    });
    s
}

fn fields(v: Value) -> Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

fn raise(s: &mut DomainSession, id: &str, blocking: &str) -> OpResult {
    create(s, NodeKind::OpenQuestion, id, &fields(json!({
        "statement": "Which status set is the domain status?", "concerns": ["Invoice"],
        "context": "Billing", "blocking": blocking,
    })))
}

#[test]
fn a_question_persists_and_round_trips() {
    let mut s = session();
    assert!(raise(&mut s, "q-001", "how").ok);
    let q = &s.graph.open_questions[0];
    assert_eq!(q.status, "open");
    assert_eq!(q.blocking, "how");
    let parsed = from_turtle(&to_turtle(&s.graph, "demo")).expect("parse");
    assert_eq!(parsed.open_questions, s.graph.open_questions);
}

#[test]
fn blocking_false_is_informational() {
    let mut s = session();
    let r = create(&mut s, NodeKind::OpenQuestion, "q-001", &fields(json!({
        "statement": "keep it?", "concerns": ["Invoice"], "blocking": false,
    })));
    assert!(r.ok, "{:?}", r.violations);
    assert_eq!(s.graph.open_questions[0].blocking, "");
}

#[test]
fn a_dangling_concern_is_rejected() {
    let mut s = session();
    let r = create(&mut s, NodeKind::OpenQuestion, "q-001", &fields(json!({
        "statement": "?", "concerns": ["Ghost"],
    })));
    assert!(!r.ok);
    assert!(r.violations.iter().any(|v| v.path == "concerns"));
    let none = create(&mut s, NodeKind::OpenQuestion, "q-002", &fields(json!({ "statement": "?" })));
    assert!(!none.ok, "a question must concern ≥ 1 node");
}

#[test]
fn a_settled_question_needs_a_resolution() {
    let mut s = session();
    assert!(raise(&mut s, "q-001", "").ok);
    let bare = answer(&mut s, "q-001", &Answer { status: "wont-fix".into(), ..Default::default() });
    assert!(!bare.ok);
    assert!(bare.violations.iter().any(|v| v.path == "resolution"));
    let ok = answer(&mut s, "q-001", &Answer {
        status: "answered".into(), resolution: Some("the ledger one".into()),
        resolved_by: vec!["Invoice".into()], session: Some("s1".into()), at: "t".into(),
    });
    assert!(ok.ok, "{:?}", ok.violations);
    assert_eq!(s.graph.open_questions[0].answered_in_session.as_deref(), Some("s1"));
}

#[test]
fn removing_a_concerned_node_leaves_the_open_question_dangling() {
    let mut s = session();
    assert!(raise(&mut s, "q-001", "").ok);
    assert!(remove(&mut s, "Invoice").ok);
    let dangling = validate_graph(&s.graph);
    assert!(dangling.iter().any(|v| v.focus == "q-001" && v.path == "concerns"), "{dangling:?}");
}

#[test]
fn next_id_counts_up() {
    let mut s = session();
    assert_eq!(next_id(&s.graph), "q-001");
    assert!(raise(&mut s, "q-007", "").ok);
    assert_eq!(next_id(&s.graph), "q-008");
}

#[test]
fn list_unifies_authored_and_derived() {
    let mut s = session();
    assert!(raise(&mut s, "q-001", "").ok);
    let all = list(&s.graph, &Filter::default(), true);
    assert!(all.iter().any(|l| l.source == "authored"));
    assert!(all.iter().any(|l| l.source == "derived"));
    let only = list(&s.graph, &Filter { concerns: Some("Invoice".into()), ..Default::default() }, false);
    assert_eq!(only.len(), 1);
    assert_eq!(concerning(&s.graph, "Invoice").len(), 1);
}

#[test]
fn a_derived_gap_can_be_promoted() {
    let s = session();
    let gap = list(&s.graph, &Filter::default(), true).into_iter().find(|l| l.source == "derived").expect("gap");
    let text = gap.body["question"].as_str().expect("text");
    let (stmt, concerns) = derived_seed(&s.graph, text).expect("seed");
    assert_eq!(stmt, text);
    assert!(!concerns.is_empty());
}

#[test]
fn gates_block_advance_and_finalize() {
    let mut s = session();
    assert!(raise(&mut s, "q-001", "what").ok);
    assert!(raise(&mut s, "q-002", "finalize").ok);
    assert!(raise(&mut s, "q-003", "").ok);
    assert_eq!(advance_blockers(&s.graph, Phase::What, Phase::How).len(), 1);
    assert!(advance_blockers(&s.graph, Phase::How, Phase::Build).is_empty());
    let fc = finalize_check(&s.graph);
    assert_eq!(fc.blocking.len(), 1);
    assert_eq!(fc.warnings.len(), 2);
    assert_eq!(status_counts(&s.graph, Phase::What, "s1")["blockingThisPhase"], json!(1));
}

#[test]
fn a_how_decision_answers_a_question() {
    let mut s = session();
    assert!(raise(&mut s, "q-001", "how").ok);
    let r = answer_from_how(&mut s, &["q-001".into()], "dec-persist", "event-sourced", Some("s1"), "t");
    assert!(r.iter().all(|r| r.ok));
    let q = &s.graph.open_questions[0];
    assert_eq!(q.status, "answered");
    assert_eq!(q.resolved_by, vec!["dec-persist".to_string()]);
    assert!(q.resolution.as_deref().is_some_and(|r| r.contains("dec-persist")));
    assert_eq!(session_report(&s.graph, "s1")["answered"].as_array().map(Vec::len), Some(1));
}

#[test]
fn markdown_groups_by_context_and_status() {
    let mut s = session();
    assert!(raise(&mut s, "q-001", "").ok);
    let md = to_markdown(&s.graph, "demo");
    assert!(md.contains("## Billing") && md.contains("### open") && md.contains("q-001"), "{md}");
}
