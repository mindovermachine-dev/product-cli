//! Turtle emission + seed parsing for the authored open-question node.
//!
//! Emit and parse live side by side so they stay symmetric. `concerns` is an
//! edge (`pf:concerns d:<id>`); `resolved_by` is a literal because it may name
//! a How element, which is not a What-graph instance.

use oxigraph::store::Store;

use super::model::{DomainGraph, OpenQuestion};
use super::seed::{lit as term_lit, local, multi, opt, select};
use super::turtle::lit;
use crate::error::Result;

/// Emit one open question as a `pf:OpenQuestion` resource.
pub(super) fn emit_open_question(out: &mut String, q: &OpenQuestion) {
    out.push_str(&format!("d:{} a pf:OpenQuestion ;\n  pf:questionStatement {}", q.id, lit(&q.statement)));
    out.push_str(&format!(" ;\n  pf:questionStatus {}", lit(&q.status)));
    for c in &q.concerns {
        out.push_str(&format!(" ;\n  pf:concerns d:{}", c));
    }
    if let Some(c) = &q.context {
        out.push_str(&format!(" ;\n  pf:inContext d:{}", c));
    }
    if !q.blocking.is_empty() {
        out.push_str(&format!(" ;\n  pf:blocksGate {}", lit(&q.blocking)));
    }
    for r in &q.resolved_by {
        out.push_str(&format!(" ;\n  pf:resolvedBy {}", lit(r)));
    }
    let literals = [
        ("raisedBy", &q.raised_by),
        ("raisedAt", &q.raised_at),
        ("resolution", &q.resolution),
        ("answeredAt", &q.answered_at),
        ("raisedInSession", &q.raised_in_session),
        ("answeredInSession", &q.answered_in_session),
    ];
    for (pred, v) in literals {
        if let Some(v) = v {
            out.push_str(&format!(" ;\n  pf:{pred} {}", lit(v)));
        }
    }
    out.push_str(" .\n\n");
}

/// Parse every `pf:OpenQuestion` into `g.open_questions`.
pub(super) fn parse_open_questions(store: &Store, g: &mut DomainGraph) -> Result<()> {
    let concerns = multi(store, "pf:OpenQuestion", "pf:concerns")?;
    let resolved = multi(store, "pf:OpenQuestion", "pf:resolvedBy")?;
    for row in select(store, "?s ?stmt ?status ?ctx ?gate ?by ?at ?res ?ansAt ?rin ?ain",
        "?s a pf:OpenQuestion . OPTIONAL { ?s pf:questionStatement ?stmt } OPTIONAL { ?s pf:questionStatus ?status } \
         OPTIONAL { ?s pf:inContext ?ctx } OPTIONAL { ?s pf:blocksGate ?gate } OPTIONAL { ?s pf:raisedBy ?by } \
         OPTIONAL { ?s pf:raisedAt ?at } OPTIONAL { ?s pf:resolution ?res } OPTIONAL { ?s pf:answeredAt ?ansAt } \
         OPTIONAL { ?s pf:raisedInSession ?rin } OPTIONAL { ?s pf:answeredInSession ?ain }")? {
        let id = local(row.get("s"));
        g.open_questions.push(OpenQuestion {
            concerns: concerns.get(&id).cloned().unwrap_or_default(),
            resolved_by: resolved.get(&id).cloned().unwrap_or_default(),
            statement: term_lit(row.get("stmt")),
            status: opt(row.get("status")).unwrap_or_else(|| "open".to_string()),
            context: opt(row.get("ctx")).map(|_| local(row.get("ctx"))),
            blocking: term_lit(row.get("gate")),
            raised_by: opt(row.get("by")),
            raised_at: opt(row.get("at")),
            resolution: opt(row.get("res")),
            answered_at: opt(row.get("ansAt")),
            raised_in_session: opt(row.get("rin")),
            answered_in_session: opt(row.get("ain")),
            id,
        });
    }
    Ok(())
}
