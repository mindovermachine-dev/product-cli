//! Open questions at the phase gates: blockers, status counts, the session report.
//!
//! A question's `blocking` names the gate it holds shut — leaving the What,
//! How or Build phase, or finalizing the session. The workflow transport asks
//! this module which open questions stand in the way; it never decides by
//! itself. The Markdown export (for readers outside the tool) lives here too
//! because it is the same "what is still open" reading of the graph.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use super::model::{DomainGraph, OpenQuestion};
use super::ops::OpResult;
use super::question::{answer, Answer};
use super::session::DomainSession;
use super::workflow::Phase;

/// The compact form of a question used in refusals, warnings and reports.
pub fn summary(q: &OpenQuestion) -> Value {
    json!({
        "id": q.id,
        "statement": q.statement,
        "status": q.status,
        "blocking": q.blocking,
        "concerns": q.concerns,
    })
}

/// The gate a phase closes when the session leaves it.
pub fn gate_of(phase: Phase) -> &'static str {
    phase.as_str()
}

/// The open questions blocking any of `gates`.
pub fn blockers<'a>(graph: &'a DomainGraph, gates: &[&str]) -> Vec<&'a OpenQuestion> {
    graph.open_questions.iter()
        .filter(|q| q.is_open() && gates.iter().any(|g| q.blocks(g)))
        .collect()
}

/// The open questions that forbid moving from `from` to `to`: every phase left
/// on the way (a jump What → Build leaves both What and How).
pub fn advance_blockers(graph: &DomainGraph, from: Phase, to: Phase) -> Vec<&OpenQuestion> {
    let left: Vec<&str> = Phase::all().into_iter().filter(|p| *p >= from && *p < to).map(gate_of).collect();
    blockers(graph, &left)
}

/// The finalize check: questions that refuse it, and the open ones it only warns about.
#[derive(Debug, Default)]
pub struct FinalizeCheck<'a> {
    pub blocking: Vec<&'a OpenQuestion>,
    pub warnings: Vec<&'a OpenQuestion>,
}

/// Split the open questions into finalize-blocking and merely-open.
pub fn finalize_check(graph: &DomainGraph) -> FinalizeCheck<'_> {
    let (blocking, warnings) = graph.open_questions.iter()
        .filter(|q| q.is_open())
        .partition(|q| q.blocks("finalize"));
    FinalizeCheck { blocking, warnings }
}

/// The counts `product_workflow_status` shows.
pub fn status_counts(graph: &DomainGraph, phase: Phase, session: &str) -> Value {
    let open = graph.open_questions.iter().filter(|q| q.is_open()).count();
    let blocking_here = blockers(graph, &[gate_of(phase)]).len();
    let answered = graph.open_questions.iter()
        .filter(|q| !q.is_open() && q.answered_in_session.as_deref() == Some(session))
        .count();
    json!({ "open": open, "blockingThisPhase": blocking_here, "answeredThisSession": answered })
}

/// The reviewer's section of the finalize report: raised / answered in this
/// session, and everything still open.
pub fn session_report(graph: &DomainGraph, session: &str) -> Value {
    let pick = |f: &dyn Fn(&OpenQuestion) -> bool| -> Vec<Value> {
        graph.open_questions.iter().filter(|q| f(q)).map(summary).collect()
    };
    json!({
        "raised": pick(&|q| q.raised_in_session.as_deref() == Some(session)),
        "answered": pick(&|q| !q.is_open() && q.answered_in_session.as_deref() == Some(session)),
        "stillOpen": pick(&|q| q.is_open()),
    })
}

/// Mark each of `questions` answered by a How element (decision / principle /
/// pattern), recording it in `resolved_by`. The resolution defaults to naming
/// the element when the question carries none yet.
pub fn answer_from_how(
    session: &mut DomainSession,
    questions: &[String],
    element: &str,
    summary_text: &str,
    workflow_session: Option<&str>,
    at: &str,
) -> Vec<OpResult> {
    questions.iter().map(|qid| {
        let has_resolution = session.graph.open_questions.iter()
            .any(|q| &q.id == qid && q.resolution.as_deref().is_some_and(|r| !r.trim().is_empty()));
        let a = Answer {
            status: "answered".into(),
            resolution: (!has_resolution).then(|| format!("Answered by How element {element}: {summary_text}")),
            resolved_by: vec![element.to_string()],
            session: workflow_session.map(str::to_string),
            at: at.to_string(),
        };
        answer(session, qid, &a)
    }).collect()
}

/// Render the questions as a Markdown document grouped by context, then status.
pub fn to_markdown(graph: &DomainGraph, product: &str) -> String {
    let mut groups: BTreeMap<String, BTreeMap<String, Vec<&OpenQuestion>>> = BTreeMap::new();
    for q in &graph.open_questions {
        let ctx = q.context.clone().unwrap_or_else(|| "(no context)".to_string());
        groups.entry(ctx).or_default().entry(q.status.clone()).or_default().push(q);
    }
    let mut out = format!("# Open questions — {product}\n\n");
    if groups.is_empty() {
        out.push_str("_No open questions recorded._\n");
        return out;
    }
    for (ctx, by_status) in &groups {
        out.push_str(&format!("## {ctx}\n\n"));
        for (status, qs) in by_status {
            out.push_str(&format!("### {status}\n\n"));
            for q in qs {
                out.push_str(&question_md(q));
            }
        }
    }
    out
}

fn question_md(q: &OpenQuestion) -> String {
    let mut s = format!("- **{}** — {}\n", q.id, q.statement);
    if !q.concerns.is_empty() {
        s.push_str(&format!("  - concerns: {}\n", q.concerns.join(", ")));
    }
    if !q.blocking.is_empty() {
        s.push_str(&format!("  - blocks: {}\n", q.blocking));
    }
    if let Some(r) = &q.resolution {
        s.push_str(&format!("  - resolution: {r}\n"));
    }
    if !q.resolved_by.is_empty() {
        s.push_str(&format!("  - resolved by: {}\n", q.resolved_by.join(", ")));
    }
    s
}
