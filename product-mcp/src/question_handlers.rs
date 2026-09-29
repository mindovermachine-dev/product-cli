//! MCP handlers for authored open questions — parity with `product question`.
//!
//! Questions are What-graph nodes, so writes go through `pf::edit` (shape-checked
//! in-loop) exactly like `product_domain_new`. The workflow transport injects
//! `session_id` so a question raised or answered in a session is stamped with it.

use std::path::{Path, PathBuf};

use product_core::author::domain::session_dir;
use product_core::pf::edit::{create, remove};
use product_core::pf::ids::NodeKind;
use product_core::pf::ops::OpResult;
use product_core::pf::question::{self, Answer, Filter};
use product_core::pf::question_gate;
use product_core::pf::session::DomainSession;
use serde_json::{json, Map, Value};

use crate::pf_mcp::product_of;

fn opt(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

fn req(args: &Value, key: &str) -> Result<String, String> {
    opt(args, key).ok_or_else(|| format!("missing required argument '{key}'"))
}

fn strs(args: &Value, key: &str) -> Vec<String> {
    match args.get(key) {
        Some(Value::Array(a)) => a.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        Some(Value::String(s)) => s.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect(),
        _ => vec![],
    }
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn open(args: &Value, repo_root: &Path) -> Result<(PathBuf, DomainSession), String> {
    let p = product_of(args, repo_root)?;
    let dir = session_dir(repo_root, &p);
    let s = DomainSession::load(&dir).map_err(|_| format!("no What graph for '{p}' yet — author one first"))?;
    Ok((dir, s))
}

fn op(r: OpResult) -> Result<Value, String> {
    serde_json::to_value(r).map_err(|e| format!("{e}"))
}

/// `product_question_new` — raise a question (or promote a derived one via `from_derived`).
pub fn handle_new(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let (dir, mut s) = open(args, repo_root)?;
    let (statement, mut concerns) = match opt(args, "from_derived") {
        Some(text) => question::derived_seed(&s.graph, &text)
            .ok_or_else(|| format!("no current derived question reads {text:?} — list them with include_derived=true"))?,
        None => (req(args, "statement")?, vec![]),
    };
    let explicit = strs(args, "concerns");
    if !explicit.is_empty() {
        concerns = explicit;
    }
    let id = opt(args, "id").unwrap_or_else(|| question::next_id(&s.graph));
    let mut m = Map::new();
    m.insert("statement".into(), json!(statement));
    m.insert("concerns".into(), json!(concerns));
    m.insert("raised_at".into(), json!(now()));
    for (field, key) in [("context", "context"), ("raised_by", "raised_by"), ("raised_in_session", "session_id")] {
        if let Some(v) = opt(args, key) {
            m.insert(field.into(), json!(v));
        }
    }
    if let Some(b) = args.get("blocking").filter(|v| !v.is_null()) {
        m.insert("blocking".into(), b.clone());
    }
    let r = create(&mut s, NodeKind::OpenQuestion, &id, &m);
    s.save(&dir).map_err(|e| format!("{e}"))?;
    op(r)
}

/// `product_question_list` — authored questions, plus derived gaps on request.
pub fn handle_list(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let (_, s) = open(args, repo_root)?;
    let filter = Filter {
        status: opt(args, "status"),
        concerns: opt(args, "concerns"),
        context: opt(args, "context"),
        blocking: opt(args, "blocking"),
    };
    let derived = args.get("include_derived").and_then(Value::as_bool).unwrap_or(false);
    let rows = question::list(&s.graph, &filter, derived);
    Ok(json!({ "questions": rows, "count": rows.len() }))
}

/// `product_question_show` — one question with each concerned node's current state.
pub fn handle_show(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let (_, s) = open(args, repo_root)?;
    let id = req(args, "id")?;
    question::show(&s.graph, &id).ok_or_else(|| format!("no open question with id {id:?}"))
}

/// `product_question_answer` — answer / defer / wont-fix, with a resolution.
pub fn handle_answer(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let (dir, mut s) = open(args, repo_root)?;
    let id = req(args, "id")?;
    let a = Answer {
        status: opt(args, "status").unwrap_or_else(|| "answered".into()),
        resolution: opt(args, "resolution"),
        resolved_by: strs(args, "resolved_by"),
        session: opt(args, "session_id"),
        at: now(),
    };
    let r = question::answer(&mut s, &id, &a);
    s.save(&dir).map_err(|e| format!("{e}"))?;
    op(r)
}

/// `product_question_rm` — delete a question raised by mistake.
pub fn handle_rm(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let (dir, mut s) = open(args, repo_root)?;
    let id = req(args, "id")?;
    if !s.graph.is_kind(&id, NodeKind::OpenQuestion) {
        return Err(format!("no open question with id {id:?}"));
    }
    let r = remove(&mut s, &id);
    s.save(&dir).map_err(|e| format!("{e}"))?;
    op(r)
}

/// `product_question_export` — the questions as a Markdown document.
pub fn handle_export(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let format = opt(args, "format").unwrap_or_else(|| "md".into());
    if format != "md" {
        return Err(format!("unsupported format {format:?} — use md"));
    }
    let p = product_of(args, repo_root)?;
    let (_, s) = open(args, repo_root)?;
    Ok(json!({ "format": "md", "document": question_gate::to_markdown(&s.graph, &p) }))
}

/// Refuse a How write whose `answers` names something that is not a question,
/// before the How contract is touched.
pub fn check_answers(args: &Value, repo_root: &Path) -> Result<Vec<String>, String> {
    let answers = strs(args, "answers");
    if answers.is_empty() {
        return Ok(answers);
    }
    let (_, s) = open(args, repo_root)?;
    match answers.iter().find(|q| !s.graph.is_kind(q, NodeKind::OpenQuestion)) {
        Some(bad) => Err(format!("`answers` names {bad:?}, which is not an open question in the What graph")),
        None => Ok(answers),
    }
}

/// Mark `answers` answered by the How element just authored.
pub fn link_how_answers(args: &Value, repo_root: &Path, answers: &[String], element: &str, text: &str) -> Result<Value, String> {
    let (dir, mut s) = open(args, repo_root)?;
    let session = opt(args, "session_id");
    let results = question_gate::answer_from_how(&mut s, answers, element, text, session.as_deref(), &now());
    s.save(&dir).map_err(|e| format!("{e}"))?;
    serde_json::to_value(results).map_err(|e| format!("{e}"))
}
