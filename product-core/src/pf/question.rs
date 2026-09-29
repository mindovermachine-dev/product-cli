//! Authored open questions: the shape check plus the raise/answer/list operations.
//!
//! The node itself is [`super::model::OpenQuestion`]. Every write goes through
//! [`super::edit`] so a question is shape-checked in-loop like any other node;
//! this module only builds the field maps and reads the results. Gates, the
//! session report and the Markdown export live in [`super::question_gate`].

use serde::Serialize;
use serde_json::{json, Map, Value};

use super::ids::NodeKind;
use super::model::{DomainGraph, OpenQuestion, QUESTION_GATES, QUESTION_STATUSES};
use super::ops::OpResult;
use super::questions::{open_questions, Focus};
use super::session::DomainSession;
use super::validate::Violation;

/// Run the open-question shape over every question (the `validate_graph` path).
pub(super) fn check_all(graph: &DomainGraph, v: &mut Vec<Violation>) {
    for q in &graph.open_questions {
        check_open_question(q, graph, v);
    }
}

/// The shape rules for an open question.
pub(super) fn check_open_question(q: &OpenQuestion, graph: &DomainGraph, v: &mut Vec<Violation>) {
    if q.statement.trim().is_empty() {
        v.push(Violation::new(&q.id, "statement", "An open question must state the question."));
    }
    if !QUESTION_STATUSES.contains(&q.status.as_str()) {
        v.push(Violation::new(&q.id, "status",
            "An open question's status must be one of open, answered, deferred, wont-fix."));
    }
    if !q.blocking.is_empty() && !QUESTION_GATES.contains(&q.blocking.as_str()) {
        v.push(Violation::new(&q.id, "blocking",
            "An open question blocks one of the gates what, how, build, finalize — or none (false/empty)."));
    }
    if q.concerns.is_empty() {
        v.push(Violation::new(&q.id, "concerns", "An open question must concern at least one node."));
    }
    // A settled question keeps its history: a concerned node may since have been
    // removed (often *because* of the answer). Only an open one must resolve.
    if q.is_open() {
        for c in q.concerns.iter().filter(|c| !graph.contains(c)) {
            v.push(Violation::new(&q.id, "concerns",
                &format!("An open question's concerned node '{c}' does not exist in the graph (dangling).")));
        }
    }
    if let Some(ctx) = &q.context {
        if !graph.is_kind(ctx, NodeKind::BoundedContext) {
            v.push(Violation::new(&q.id, "context",
                "An open question's context must resolve to a declared bounded context."));
        }
    }
    if !q.is_open() && q.resolution.as_deref().map(str::trim).unwrap_or("").is_empty() {
        v.push(Violation::new(&q.id, "resolution",
            "A question that is answered, deferred or wont-fix must carry a resolution saying why."));
    }
}

/// The next free `q-NNN` id.
pub fn next_id(graph: &DomainGraph) -> String {
    let max = graph.open_questions.iter()
        .filter_map(|q| q.id.strip_prefix("q-").and_then(|n| n.parse::<u32>().ok()))
        .max()
        .unwrap_or(0);
    let mut n = max + 1;
    while graph.contains(&format!("q-{n:03}")) {
        n += 1;
    }
    format!("q-{n:03}")
}

/// What to record when answering a question.
#[derive(Debug, Clone, Default)]
pub struct Answer {
    pub status: String,
    pub resolution: Option<String>,
    pub resolved_by: Vec<String>,
    pub session: Option<String>,
    pub at: String,
}

/// Answer (or defer / wont-fix) a question. `resolved_by` is appended to any
/// ids already recorded, so several decisions can answer one question.
pub fn answer(session: &mut DomainSession, id: &str, a: &Answer) -> OpResult {
    if !session.graph.is_kind(id, NodeKind::OpenQuestion) {
        return OpResult { ok: false, node: Some(id.to_string()), violations: vec![
            Violation::new(id, "id", &format!("no open question with id {id:?} in the graph.")),
        ] };
    }
    let mut resolved: Vec<String> = session.graph.open_questions.iter()
        .find(|q| q.id == id).map(|q| q.resolved_by.clone()).unwrap_or_default();
    for r in &a.resolved_by {
        if !resolved.contains(r) {
            resolved.push(r.clone());
        }
    }
    let mut m = Map::new();
    m.insert("status".into(), json!(a.status));
    m.insert("resolved_by".into(), json!(resolved));
    m.insert("answered_at".into(), json!(a.at));
    if let Some(r) = &a.resolution {
        m.insert("resolution".into(), json!(r));
    }
    if let Some(s) = &a.session {
        m.insert("answered_in_session".into(), json!(s));
    }
    super::edit::edit(session, id, &m)
}

/// A filter over questions (every set field must match).
#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub status: Option<String>,
    pub concerns: Option<String>,
    pub context: Option<String>,
    pub blocking: Option<String>,
}

impl Filter {
    pub fn matches(&self, q: &OpenQuestion) -> bool {
        self.status.as_ref().is_none_or(|s| &q.status == s)
            && self.concerns.as_ref().is_none_or(|c| q.concerns.contains(c))
            && self.context.as_ref().is_none_or(|c| q.context.as_ref() == Some(c))
            && self.blocking.as_ref().is_none_or(|b| &q.blocking == b)
    }
}

/// One row of the unified list: an authored question or a derived gap.
#[derive(Debug, Clone, Serialize)]
pub struct Listed {
    /// `authored` (a persistent node) or `derived` (a recomputed gap).
    pub source: &'static str,
    #[serde(flatten)]
    pub body: Value,
}

/// The authored questions matching `filter`, plus — when asked — the derived
/// facilitation gaps (filtered by `concerns` on their focus node only).
pub fn list(graph: &DomainGraph, filter: &Filter, include_derived: bool) -> Vec<Listed> {
    let mut out: Vec<Listed> = graph.open_questions.iter()
        .filter(|q| filter.matches(q))
        .map(|q| Listed { source: "authored", body: serde_json::to_value(q).unwrap_or(Value::Null) })
        .collect();
    let derived_applies = filter.status.as_deref().is_none_or(|s| s == "open")
        && filter.context.is_none() && filter.blocking.is_none();
    if include_derived && derived_applies {
        out.extend(open_questions(graph, Focus::All).into_iter()
            .filter(|d| filter.concerns.as_ref().is_none_or(|c| d.focus.as_ref() == Some(c)))
            .map(|d| Listed { source: "derived", body: serde_json::to_value(d).unwrap_or(Value::Null) }));
    }
    out
}

/// Promote a derived gap into an authored question's seed: find the derived
/// question whose text is `text`, returning its statement and the node it is
/// about (as the default `concerns`). `None` if no current gap has that text.
pub fn derived_seed(graph: &DomainGraph, text: &str) -> Option<(String, Vec<String>)> {
    open_questions(graph, Focus::All).into_iter()
        .find(|d| d.question.trim() == text.trim())
        .map(|d| (d.question, d.focus.into_iter().collect()))
}

/// The open authored questions that concern `node` (for `domain show`).
pub fn concerning<'a>(graph: &'a DomainGraph, node: &str) -> Vec<&'a OpenQuestion> {
    graph.open_questions.iter().filter(|q| q.is_open() && q.concerns.iter().any(|c| c == node)).collect()
}

/// One question plus the current state of each node it concerns (`None` once
/// the node is gone).
pub fn show(graph: &DomainGraph, id: &str) -> Option<Value> {
    let q = graph.open_questions.iter().find(|q| q.id == id)?;
    let concerns: Vec<Value> = q.concerns.iter().map(|c| json!({
        "id": c,
        "kind": graph.kind_of(c).map(|k| k.cli_name()),
        "node": super::query::node_value(graph, c),
    })).collect();
    Some(json!({ "question": q, "concerns": concerns }))
}

#[cfg(test)]
#[path = "question_tests.rs"]
mod tests;
