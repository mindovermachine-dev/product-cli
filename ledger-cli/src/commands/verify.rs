//! `ledger verify` — the gate, wired to an exit code.

use std::path::PathBuf;

use chrono::{NaiveDate, Utc};
use ledger_core::finding::Gate;
use ledger_core::render::render;
use ledger_core::store;
use ledger_core::verify::{verify, Options};

use super::{resolve_root, EXIT_OK, EXIT_VIOLATIONS};

/// The flags, already separated from clap so the handler stays testable.
pub struct Args {
    pub gate: Option<String>,
    pub json: bool,
    pub today: Option<String>,
    pub blame: bool,
    /// Run the export stage: committed exports against the log.
    pub export: bool,
    /// The base landing is computed against (D6); default `origin/HEAD`
    /// when the clone has it.
    pub base: Option<String>,
    /// Whether this run has the legacy capability of ruling 82 (LP-3.35):
    /// without it, a history holding the flat layout is refused with exit 2.
    pub legacy_layout: bool,
}

pub fn run(root: Option<PathBuf>, args: Args) -> Result<i32, String> {
    let repo_root = resolve_root(root)?;
    let base = args.base.or_else(|| ledger_core::landing::Landing::default_base(&repo_root));
    if !args.legacy_layout {
        refuse_flat_history(&repo_root, base.as_deref())?;
    }
    let options = Options {
        gate: args.gate.as_deref().map(parse_gate).transpose()?,
        today: parse_today(args.today.as_deref())?,
        blame: args.blame,
        history: true,
        base: base.clone(),
        legacy_layout: args.legacy_layout,
    };
    let mut loaded = store::load(&repo_root);
    match &base {
        Some(base) => {
            // The merge's store: the base's log files (and signatures) a
            // branch checkout lacks, so a local verify judges what the merge
            // would.
            let added = ledger_core::revision::overlay_base(&mut loaded, base)?;
            if !args.json {
                println!("landing computed against base `{base}` — {added} change-set(s) read from the base");
            }
        }
        None if !args.json => {
            println!("landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)");
        }
        None => {}
    }
    let mut report = verify(&loaded, &options);
    if args.export {
        report.export = Some(ledger_core::graph::export::check(&repo_root, &loaded));
    }
    if args.json {
        let text = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
        println!("{text}");
    } else if report.is_conformant() {
        println!("{}", render(&report));
    } else {
        eprintln!("{}", render(&report));
    }
    Ok(if report.is_conformant() { EXIT_OK } else { EXIT_VIOLATIONS })
}

/// A verifier without the legacy capability cannot verify a repository
/// whose history holds the flat layout of revision v1.8: it refuses before
/// reading anything, names the first flat commit, and never reports the
/// repository conformant (LP-3.35; rulings 82, 97). Exit 2: the gate could
/// not run, which is the verifier's limit and not a finding about the store.
fn refuse_flat_history(root: &std::path::Path, base: Option<&str>) -> Result<(), String> {
    let mut revs = vec!["HEAD"];
    revs.extend(base);
    match ledger_core::layout::flat_history(root, &revs) {
        None => Ok(()),
        Some(commit) => Err(format!(
            "this repository's history holds the flat layout of revision v1.8, first at commit {commit}, and this run has no \
             legacy capability to read it (ruling 82) — the repository cannot be verified by it; a verifier with the capability \
             (this one, without `--no-legacy-layout`) reads both path patterns in history (LP-3.35)"
        )),
    }
}

fn parse_gate(name: &str) -> Result<Gate, String> {
    match name {
        "readiness" => Ok(Gate::Readiness),
        "completeness" => Ok(Gate::Completeness),
        other => Err(format!("`{other}` is not a gate — expected `readiness` or `completeness`")),
    }
}

fn parse_today(raw: Option<&str>) -> Result<NaiveDate, String> {
    match raw {
        None => Ok(Utc::now().date_naive()),
        Some(text) => text
            .parse()
            .map_err(|_| format!("`{text}` is not a date — expected YYYY-MM-DD")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_gates_parse_by_name() {
        assert_eq!(parse_gate("readiness").expect("gate"), Gate::Readiness);
        assert_eq!(parse_gate("completeness").expect("gate"), Gate::Completeness);
        let err = parse_gate("release").expect_err("unknown");
        assert!(err.contains("is not a gate"), "{err}");
    }

    #[test]
    fn an_absent_date_means_today() {
        assert_eq!(parse_today(None).expect("today"), Utc::now().date_naive());
    }

    #[test]
    fn a_malformed_date_says_what_it_wanted() {
        let err = parse_today(Some("10-08-2026")).expect_err("bad date");
        assert!(err.contains("expected YYYY-MM-DD"), "{err}");
    }
}
