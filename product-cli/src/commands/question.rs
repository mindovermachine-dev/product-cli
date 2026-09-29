//! Open questions — raise, list, show, answer, remove, export.
//!
//! `product question …` is a thin adapter over `pf::question` /
//! `pf::question_gate`: authored questions are What-graph nodes
//! (`kind=open-question`), so every write goes through the same in-loop shape
//! check as `product domain new`. `--root` points at another repo root (a
//! session workspace, CI checkout) instead of the discovered one.

use clap::Subcommand;
use product_core::author::domain::session_dir;
use product_core::pf::edit::{create, remove};
use product_core::pf::ids::NodeKind;
use product_core::pf::question::{self, Answer, Filter};
use product_core::pf::question_gate;
use product_core::pf::session::DomainSession;
use serde_json::{json, Map, Value};
use std::path::PathBuf;

use super::BoxResult;

/// Where the question's What graph lives: `--root` + `--product`.
#[derive(clap::Args, Clone, Default)]
pub struct Target {
    /// The repo root to operate on (default: the discovered repo)
    #[arg(long)]
    root: Option<PathBuf>,
    #[arg(long)]
    product: Option<String>,
}

#[derive(Subcommand)]
pub enum QuestionCommands {
    /// Answer a question: --status answered|deferred|wont-fix --resolution "…"
    Answer {
        id: String,
        #[arg(long, default_value = "answered")]
        status: String,
        #[arg(long)]
        resolution: Option<String>,
        /// Ids of what answered it (What nodes or How decisions)
        #[arg(long = "resolved-by", value_delimiter = ',')]
        resolved_by: Vec<String>,
        /// The workflow session answering it
        #[arg(long)]
        session: Option<String>,
        #[command(flatten)]
        target: Target,
    },
    /// Render the questions as Markdown, grouped by context and status
    Export {
        /// Output format (md)
        #[arg(long, default_value = "md")]
        format: String,
        #[command(flatten)]
        target: Target,
    },
    /// List questions (authored; --derived adds the recomputed gaps)
    List {
        #[arg(long)]
        status: Option<String>,
        /// Only questions concerning this node id
        #[arg(long)]
        concerns: Option<String>,
        #[arg(long)]
        context: Option<String>,
        /// Only questions blocking this gate (what|how|build|finalize)
        #[arg(long)]
        blocking: Option<String>,
        /// Include derived (validation/completeness) questions
        #[arg(long)]
        derived: bool,
        /// Output format: text | json
        #[arg(long, default_value = "text")]
        format: String,
        #[command(flatten)]
        target: Target,
    },
    /// Raise a question about one or more nodes
    New {
        #[arg(long)]
        statement: String,
        /// The node ids it concerns (comma-separated)
        #[arg(long, value_delimiter = ',', required = true)]
        concerns: Vec<String>,
        /// An explicit id (default: the next free q-NNN)
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        context: Option<String>,
        /// The gate it blocks: what | how | build | finalize
        #[arg(long)]
        blocking: Option<String>,
        #[arg(long = "raised-by")]
        raised_by: Option<String>,
        /// The workflow session raising it
        #[arg(long)]
        session: Option<String>,
        #[command(flatten)]
        target: Target,
    },
    /// Delete a question (e.g. raised by mistake)
    Rm {
        id: String,
        #[command(flatten)]
        target: Target,
    },
    /// Show one question with the current state of each node it concerns
    Show {
        id: String,
        #[command(flatten)]
        target: Target,
    },
}

pub(crate) fn handle_question(cmd: QuestionCommands) -> BoxResult {
    match cmd {
        QuestionCommands::Answer { id, status, resolution, resolved_by, session, target } => {
            let (_, dir, mut s) = open(&target)?;
            let a = Answer { status, resolution, resolved_by, session, at: now() };
            let r = question::answer(&mut s, &id, &a);
            s.save(&dir)?;
            report("Answered", &id, r)
        }
        QuestionCommands::Export { format, target } => export(&format, &target),
        QuestionCommands::List { status, concerns, context, blocking, derived, format, target } => {
            list(Filter { status, concerns, context, blocking }, derived, &format, &target)
        }
        QuestionCommands::New { statement, concerns, id, context, blocking, raised_by, session, target } => {
            let (_, dir, mut s) = open(&target)?;
            let id = id.unwrap_or_else(|| question::next_id(&s.graph));
            let mut m = Map::new();
            m.insert("statement".into(), json!(statement));
            m.insert("concerns".into(), json!(concerns));
            m.insert("raised_at".into(), json!(now()));
            let opts = [("context", context), ("blocking", blocking), ("raised_by", raised_by), ("raised_in_session", session)];
            for (k, v) in opts {
                if let Some(v) = v {
                    m.insert(k.into(), json!(v));
                }
            }
            let r = create(&mut s, NodeKind::OpenQuestion, &id, &m);
            s.save(&dir)?;
            report("Raised", &id, r)
        }
        QuestionCommands::Rm { id, target } => {
            let (_, dir, mut s) = open(&target)?;
            if !s.graph.is_kind(&id, NodeKind::OpenQuestion) {
                return Err(format!("no open question with id {id:?}").into());
            }
            let r = remove(&mut s, &id);
            s.save(&dir)?;
            report("Removed", &id, r)
        }
        QuestionCommands::Show { id, target } => {
            let (_, _, s) = open(&target)?;
            let v = question::show(&s.graph, &id).ok_or_else(|| format!("no open question with id {id:?}"))?;
            println!("{}", serde_json::to_string_pretty(&v)?);
            Ok(())
        }
    }
}

/// Refuse `--answers` naming something that is not a question, before the How is written.
pub(super) fn check_answers(product: Option<String>, questions: &[String]) -> BoxResult {
    if questions.is_empty() {
        return Ok(());
    }
    let (_, _, s) = open(&Target { root: None, product })?;
    match questions.iter().find(|q| !s.graph.is_kind(q, NodeKind::OpenQuestion)) {
        Some(bad) => Err(format!("--answers names {bad:?}, which is not an open question in the What graph").into()),
        None => Ok(()),
    }
}

/// Mark the questions a How element answers (called by `product how add --answers`).
pub(super) fn answer_from_how(product: Option<String>, questions: &[String], element: &str, text: &str) -> BoxResult {
    if questions.is_empty() {
        return Ok(());
    }
    let (_, dir, mut s) = open(&Target { root: None, product })?;
    let results = question_gate::answer_from_how(&mut s, questions, element, text, None, &now());
    s.save(&dir)?;
    for (qid, r) in questions.iter().zip(results) {
        report("Answered", qid, r)?;
    }
    Ok(())
}

fn open(t: &Target) -> Result<(String, PathBuf, DomainSession), Box<dyn std::error::Error>> {
    let root = t.root.clone().unwrap_or_else(super::shared::domain_root);
    let p = match &t.product {
        Some(p) => p.clone(),
        None => product_core::config::ProductConfig::load_from_root(&root).ok()
            .map(|c| c.name.trim().to_string()).filter(|n| !n.is_empty())
            .or_else(super::shared::default_product_name)
            .ok_or("no product — pass --product or set `name` in product.toml")?,
    };
    product_core::pf::ids::validate_id(&p)?;
    let dir = session_dir(&root, &p);
    let s = DomainSession::load(&dir).map_err(|_| format!("no What graph for product {p:?} under {}", root.display()))?;
    Ok((p, dir, s))
}

fn list(filter: Filter, derived: bool, format: &str, target: &Target) -> BoxResult {
    let (_, _, s) = open(target)?;
    let rows = question::list(&s.graph, &filter, derived);
    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&json!({ "questions": rows, "count": rows.len() }))?);
        return Ok(());
    }
    if rows.is_empty() {
        println!("(no questions)");
    }
    for r in &rows {
        println!("{}", row_text(r.source, &r.body));
    }
    Ok(())
}

fn row_text(source: &str, b: &Value) -> String {
    let s = |k: &str| b.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    if source == "derived" {
        return format!("[derived] {} — {}", s("focus"), s("question"));
    }
    let gate = s("blocking");
    let gate = if gate.is_empty() { String::new() } else { format!(" (blocks {gate})") };
    format!("{} [{}]{} — {}", s("id"), s("status"), gate, s("statement"))
}

fn export(format: &str, target: &Target) -> BoxResult {
    if format != "md" {
        return Err(format!("unsupported format {format:?} — use md").into());
    }
    let (p, _, s) = open(target)?;
    print!("{}", question_gate::to_markdown(&s.graph, &p));
    Ok(())
}

fn report(verb: &str, id: &str, r: product_core::pf::ops::OpResult) -> BoxResult {
    if r.ok {
        println!("{verb} {id}");
        return Ok(());
    }
    let lines: Vec<String> = r.violations.iter().map(|v| format!("  - [{}] {}", v.path, v.message)).collect();
    Err(format!("Rejected {id} — no change made:\n{}", lines.join("\n")).into())
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}
