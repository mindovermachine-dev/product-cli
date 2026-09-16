//! Reading the observation store: what ran, and how it behaved across runs.
//!
//! A read, over a store this binary never writes. `check` does not look here
//! and nothing printed below can fail a build: a run record is measurement,
//! and a reading over several is still measurement.

use std::path::Path;

use clap::Args;
use eval_core::behaviour::{self, Reading};
use eval_core::escape;
use eval_core::{Backend, EvalStore, RunRecord};
use product_core::error::Result;
use serde_json::json;

use crate::exit;
use crate::render::Report;

#[derive(Args)]
pub struct RunsArgs {
    /// Read behaviour across runs instead of listing them.
    #[arg(long)]
    pub readings: bool,
    /// Look for resolutions nobody can be shown to have held.
    #[arg(long)]
    pub escapes: bool,
    #[arg(long)]
    pub json: bool,
}

/// List observed runs, or read them against each other.
pub fn runs(root: &Path, args: &RunsArgs) -> Result<Report> {
    let store = EvalStore::new(Backend::from_env(root.join(".spec"))?.open()?);
    let observed = store.read_runs();

    if args.escapes {
        return Ok(escapes(&observed, args.json));
    }
    if args.readings {
        return Ok(read(&observed, args.json));
    }
    Ok(list(&observed, args.json))
}

/// What ran.
fn list(observed: &[RunRecord], as_json: bool) -> Report {
    let text = observed
        .iter()
        .map(|r| {
            format!(
                "{}  {}  {} → {}\n  {} proposed, {} kept{}",
                r.ran_at.format("%Y-%m-%d %H:%M"),
                r.model.as_deref().unwrap_or("(no model)"),
                r.task,
                r.subject,
                r.proposed.len(),
                r.kept.len(),
                r.address.as_ref().map_or(String::new(), |a| format!("  at {}", a.short())),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let body = as_json.then(|| {
        json!(observed
            .iter()
            .map(|r| json!({
                "run": r.id,
                "tool": r.tool,
                "task": r.task,
                "subject": r.subject,
                "model": r.model,
                "proposed": r.proposed,
                "kept": r.kept,
                "address": r.address_digest(),
                "arrangement": r.arrangement_digest(),
            }))
            .collect::<Vec<_>>())
    });

    Report::text(exit::CONFORMANT, empty_or(text, "no runs observed")).with_json(body)
}

/// How behaviour read across runs.
fn read(observed: &[RunRecord], as_json: bool) -> Report {
    let readings = behaviour::read(observed);
    let collapsed = behaviour::collapsed(observed);
    let uncomparable = observed.iter().filter(|r| r.address.is_none()).count();

    let mut lines: Vec<String> = readings.iter().map(render).collect();
    lines.extend(collapsed.iter().map(|(one, other)| {
        format!(
            "collapsed  {} and {}\n  \
             declared distinct, answered the same — either the declaration is decorative, \
             or the difference never reached the worker",
            short(one),
            short(other)
        )
    }));
    if uncomparable > 0 {
        lines.push(format!(
            "{uncomparable} run(s) declared no address, so they stand alone rather than \
             being compared to unlike things"
        ));
    }
    if !lines.is_empty() {
        lines.push(NOT_A_VERDICT.to_string());
    }

    let body = as_json.then(|| {
        json!({
            "readings": readings.iter().map(as_value).collect::<Vec<_>>(),
            "collapsed": collapsed,
            "without_address": uncomparable,
            "note": NOT_A_VERDICT,
        })
    });

    Report::text(exit::CONFORMANT, empty_or(lines.join("\n\n"), "nothing to read"))
        .with_json(body)
}

/// Resolutions nobody can be shown to have held.
///
/// Findings, never a gate. Whether a particular escape is tolerable is a
/// judgement, and a judgement needs an owner; what this gives that owner is a
/// list they did not have to assemble by reading transcripts.
fn escapes(observed: &[RunRecord], as_json: bool) -> Report {
    let found = escape::check_all(observed);

    let text = found
        .iter()
        .map(|(run, finding)| {
            format!("{}  {}\n  {} — {}", short_run(run), finding.kind.as_str(), finding.subject, finding.message)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let body = as_json.then(|| {
        json!({
            "escapes": found.iter().map(|(run, f)| json!({
                "run": run,
                "kind": f.kind.as_str(),
                "subject": f.subject,
                "message": f.message,
            })).collect::<Vec<_>>(),
            "note": NOT_A_GATE,
        })
    });

    let rendered = if found.is_empty() {
        "no escape candidates".to_string()
    } else {
        format!("{text}\n\n{NOT_A_GATE}")
    };
    Report::text(exit::CONFORMANT, rendered).with_json(body)
}

/// What the escape list is for, and what it is not.
const NOT_A_GATE: &str =
    "Findings, not a gate. Whether an escape is tolerable is a judgement, and a judgement \
     needs an owner; nothing here decides for them.";

fn short_run(id: &str) -> String {
    id.chars().rev().take(6).collect::<Vec<_>>().into_iter().rev().collect()
}

/// What every reading here is, and is not.
const NOT_A_VERDICT: &str =
    "Readings, not verdicts. A shift at fixed coordinates says the ground moved or something \
     undeclared was resolved; it never says one act was wrong. Drift is drift in the \
     arrangement unless an independent measure of the world moved the same way.";

fn render(reading: &Reading) -> String {
    let mut out = format!(
        "address {}  {} run(s) across {} arrangement(s)",
        short(&reading.address),
        reading.runs,
        reading.arrangements
    );
    out.push_str(&measure("agreement", reading.agreement, "the same arrangement, asked again"));
    out.push_str(&measure(
        "across arrangements",
        reading.across_arrangements,
        "a different arrangement, the same question",
    ));
    out.push_str(&measure("drift", reading.drift(), "how far behaviour moved with the worker"));
    out
}

/// One measured line, or the honest absence of one.
///
/// A measure with nothing to average is printed as `—`, never as zero: an
/// absent reading and a reading of zero say opposite things.
fn measure(name: &str, value: Option<f64>, gloss: &str) -> String {
    match value {
        Some(v) => format!("\n  {name:<20} {v:.2}   {gloss}"),
        None => format!("\n  {name:<20} —      not enough runs to say"),
    }
}

fn as_value(reading: &Reading) -> serde_json::Value {
    json!({
        "address": reading.address,
        "runs": reading.runs,
        "arrangements": reading.arrangements,
        "agreement": reading.agreement,
        "across_arrangements": reading.across_arrangements,
        "drift": reading.drift(),
    })
}

fn short(digest: &str) -> String {
    digest.strip_prefix("sha256:").unwrap_or(digest).chars().take(12).collect()
}

fn empty_or(text: String, fallback: &str) -> String {
    if text.is_empty() { fallback.to_string() } else { text }
}
