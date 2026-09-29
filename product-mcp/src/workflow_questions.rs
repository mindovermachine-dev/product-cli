//! Open questions at the workflow's phase gates (advance, finalize, status).
//!
//! The judgement lives in `pf::question_gate`; this module loads the session's
//! What graph and shapes the transport's replies. A missing graph means no
//! questions — the gates never fail for want of a graph.

use product_core::author::domain::session_dir;
use product_core::pf::model::DomainGraph;
use product_core::pf::question_gate::{self as gate, summary};
use product_core::pf::session::DomainSession;
use product_core::pf::workflow::{Phase, WorkflowSession};
use serde_json::{json, Value};

use super::workflow::WorkflowCtx;

/// Tool families callable in every phase (questions arise in all of them).
const PHASE_FREE: [&str; 1] = ["product_question_"];

/// True if the tool is visible whatever the current phase.
pub fn is_phase_free(name: &str) -> bool {
    PHASE_FREE.iter().any(|p| name.starts_with(p))
}

/// Tools that record *which session* acted: the transport stamps `session_id`.
pub fn stamp_session(name: &str, args: &mut Value, session: &str) {
    let stamps = is_phase_free(name) || name == "product_how_add";
    if let (true, Value::Object(m)) = (stamps, args) {
        m.entry("session_id").or_insert_with(|| json!(session));
    }
}

fn graph(session: &WorkflowSession, ctx: &WorkflowCtx) -> DomainGraph {
    DomainSession::load(&session_dir(&ctx.canonical, &session.product)).map(|s| s.graph).unwrap_or_default()
}

/// The refusal for an advance that open blocking questions forbid, if any.
pub fn advance_refusal(session: &WorkflowSession, to: Option<Phase>, ctx: &WorkflowCtx) -> Option<Value> {
    let from = session.phase;
    let target = to.or_else(|| from.next()).filter(|t| *t > from)?;
    let g = graph(session, ctx);
    let blocking: Vec<Value> = gate::advance_blockers(&g, from, target).into_iter().map(summary).collect();
    if blocking.is_empty() {
        return None;
    }
    Some(json!({
        "ok": false, "from": from, "to": target,
        "reason": format!("{} open question(s) block leaving the {from} phase — answer, defer or wont-fix them with product_question_answer.", blocking.len()),
        "blockingQuestions": blocking,
    }))
}

/// The finalize verdict on open questions: `Err` refuses with the blockers,
/// `Ok` carries the warnings + the reviewer's report section.
pub fn finalize_questions(session: &WorkflowSession, ctx: &WorkflowCtx) -> Result<Value, Value> {
    let g = graph(session, ctx);
    let check = gate::finalize_check(&g);
    let warnings: Vec<Value> = check.warnings.into_iter().map(summary).collect();
    if !check.blocking.is_empty() {
        let blocking: Vec<Value> = check.blocking.into_iter().map(summary).collect();
        return Err(json!({
            "ok": false,
            "reason": format!("{} open question(s) block finalize.", blocking.len()),
            "blockingQuestions": blocking,
            "warnings": warnings,
        }));
    }
    Ok(json!({ "warnings": warnings, "report": gate::session_report(&g, &session.id) }))
}

/// The counts `product_workflow_status` shows.
pub fn status_counts(session: &WorkflowSession, ctx: &WorkflowCtx) -> Value {
    gate::status_counts(&graph(session, ctx), session.phase, &session.id)
}
