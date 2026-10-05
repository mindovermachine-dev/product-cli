//! The gate — the whole of L0's checking surface.
//!
//! One pass over the log produces every finding: the parse gate, then the
//! twelve classes. Failing for exactly those reasons is the format's measure of
//! success, so the orchestration here is deliberately flat — there is no
//! place for a rule to hide.
//!
//! Pendency is not a violation. A decision that is enumerated and allocated
//! but not yet signed is the normal state of work in progress; it is
//! reported as status, never as a failure, because a gate that fires on
//! ordinary work is a gate people learn to ignore.

pub mod acts;
pub mod authority;
pub mod disposition;
pub mod history;
pub mod integrity;
pub mod keys;
pub mod state;
pub mod view;

use chrono::NaiveDate;
use serde::Serialize;

use crate::finding::{Finding, Gate};
use crate::store::Store;

use view::View;

/// What to check, as of when.
#[derive(Debug, Clone)]
pub struct Options {
    /// Which gate to apply. `None` runs every class.
    pub gate: Option<Gate>,
    /// The date expiry is judged against, so the rule is testable without
    /// waiting for a clock.
    pub today: NaiveDate,
    /// Whether to consult git for blame consistency (class `L009`).
    pub blame: bool,
    /// Whether to read landing order from git (D6). Without it every
    /// entity is at the tip and `at` alone orders acts — the reading a
    /// write-time gate and the export-only verifier both have.
    pub history: bool,
    /// The base the landing is computed against (`verify --base`).
    pub base: Option<String>,
}

impl Options {
    /// Every class, as of `today`, with blame and landing consulted.
    pub fn full(today: NaiveDate) -> Self {
        Self { gate: None, today, blame: true, history: true, base: None }
    }

    /// Every class as of `today`, without git: the write-time gate.
    pub fn offline(today: NaiveDate) -> Self {
        Self { gate: None, today, blame: false, history: false, base: None }
    }
}

/// What one run found.
#[derive(Debug, Default, Serialize)]
pub struct Report {
    pub findings: Vec<Finding>,
    /// The L2 graph stage: cross-entry shape findings (`G001`–`G004`),
    /// distinct from the file gate's closed classes.
    pub graph: Vec<crate::graph::GraphFinding>,
    /// Entries that loaded cleanly, for the summary line.
    pub entries: usize,
    /// Decisions the log carries.
    pub decisions: usize,
    /// Allocated, awaiting acceptance — a visible status, not a failure.
    pub awaiting_acceptance: Vec<String>,
    /// Acceptances with no introducing commit yet, so `L009` was skipped.
    pub blame_uncommitted: usize,
    /// Whether git could be consulted at all.
    pub blame_unavailable: bool,
    /// The export stage (`verify --export`): committed exports that do not
    /// match the log. `None` when the stage was not asked for, so an unrun
    /// check never reads as a clean one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<Vec<crate::graph::export::ExportFinding>>,
    /// The trust-root stage: a committed `allowed_signers` that is not the
    /// log's derivation (spec v1.7). Always run; empty when nothing binds
    /// a key and no file is committed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub signers: Vec<crate::graph::export::ExportFinding>,
    /// Acceptances under a since-closed key, dated and landed before the
    /// close, awaiting re-acceptance — a review item until the policy's
    /// deadline, `L012` after it.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reaccept: Vec<crate::signing::check::Reaccept>,
    /// What the run notices about the store's governance — never a failure.
    #[serde(flatten)]
    pub notices: Notices,
}

/// The notices `verify` prints: what holds and what does not, by namespace.
#[derive(Debug, Default, serde::Serialize)]
pub struct Notices {
    /// Namespaces the log speaks with no policy: nothing in them is
    /// role-checked or signature-checked (D5 (c)).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unchecked: Vec<String>,
    /// Namespaces whose policy in force is `[none]`: governed and
    /// role-checked, but no signature is required.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unsigned: Vec<String>,
    /// The genesis holder, while they have no trusted key: until one is
    /// bound, the first self-bound binding to land for the address is the
    /// one trusted (D7, #96). With the governed namespaces it bears on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genesis_unbound: Option<GenesisUnbound>,
}

/// A genesis holder with no trusted key, and the governed namespaces.
#[derive(Debug, Default, serde::Serialize)]
pub struct GenesisUnbound {
    pub holder: String,
    pub namespaces: Vec<String>,
}

fn notices(store: &Store, trusted: &[&crate::authority::KeyBinding]) -> Notices {
    Notices { unchecked: unchecked(store), unsigned: unsigned(store), genesis_unbound: genesis_unbound(store, trusted) }
}

/// The genesis holder, when no key of theirs is trusted, with every
/// namespace under policy.
fn genesis_unbound(store: &Store, trusted: &[&crate::authority::KeyBinding]) -> Option<GenesisUnbound> {
    let auth = crate::authority::Authority::build(store);
    let genesis = auth.genesis()?;
    if trusted.iter().any(|k| k.principal == genesis.holder && k.act.opens()) {
        return None;
    }
    let mut namespaces: Vec<String> = auth.policies.iter().map(|p| p.namespace.clone()).collect();
    namespaces.sort();
    namespaces.dedup();
    Some(GenesisUnbound { holder: genesis.holder.to_string(), namespaces })
}

impl Report {
    /// Whether the gate passes: no file-stage, graph-stage or (when run)
    /// export-stage findings. Three stages, one exit discipline.
    pub fn is_conformant(&self) -> bool {
        self.findings.is_empty()
            && self.graph.is_empty()
            && self.export_findings().is_empty()
            && self.signers.is_empty()
    }

    /// The export stage's findings; empty when it was not run.
    pub fn export_findings(&self) -> &[crate::graph::export::ExportFinding] {
        self.export.as_deref().unwrap_or_default()
    }
}

/// Run the gate over a loaded store.
pub fn verify(store: &Store, opts: &Options) -> Report {
    let view = View::build(store);
    let mut findings = store.schema_findings.clone();
    findings.extend(view.findings.iter().cloned());
    findings.extend(disposition::unallocated(&view));
    findings.extend(disposition::expired(&view, opts.today));
    findings.extend(disposition::stranded(&view, store));
    findings.extend(disposition::model_acceptor(&view));
    findings.extend(disposition::model_judge(&view));
    findings.extend(integrity::hash_mismatch(&view).into_iter().chain(integrity::dangling_acceptance(&view)));
    findings.extend(keys::key_changed(&view));
    findings.extend(keys::key_collision(&view));
    findings.extend(authority::findings(store));
    let landing = landing(store, opts);
    let signing = crate::signing::check::check(store, &landing, opts.today);
    findings.extend(signing.findings.iter().cloned());
    findings.extend(history::findings(store, &landing));

    let mut report = Report {
        entries: store.entry_count(),
        decisions: view.latest.len(),
        awaiting_acceptance: awaiting(&view),
        reaccept: signing.reaccept.clone(),
        notices: notices(store, &signing.trusted),
        ..Report::default()
    };
    if opts.blame {
        let outcome = integrity::blame_consistency(&view, store);
        findings.extend(outcome.findings);
        report.blame_uncommitted = outcome.uncommitted;
        report.blame_unavailable = outcome.no_repository;
    }
    if let Some(gate) = opts.gate {
        findings.retain(|f| f.class.in_gate(gate));
    }
    findings.sort_by(|a, b| (a.class, &a.subject).cmp(&(b.class, &b.subject)));
    findings.dedup();
    report.findings = findings;
    // The graph stage: structural integrity, so it runs under both gates.
    report.graph = graph_stage(store, &landing);
    report.signers = crate::authority::signers::check(store, &signing.trusted);
    report
}

/// The graph stage: the SPARQL shapes, then `A006` over history.
fn graph_stage(store: &Store, landing: &crate::landing::Landing) -> Vec<crate::graph::GraphFinding> {
    let mut graph = crate::graph::shapes::graph_findings(store);
    graph.extend(acts::unauthorised(store, landing));
    graph.sort_by(|a, b| (a.class, &a.subject).cmp(&(b.class, &b.subject)));
    graph
}

/// Landing order from git when the run consults history (D6), else none.
fn landing(store: &Store, opts: &Options) -> crate::landing::Landing {
    match opts.history {
        true => crate::landing::Landing::compute(&store.root, opts.base.as_deref()).unwrap_or_default(),
        false => crate::landing::Landing::unknown(),
    }
}

/// Every namespace a decision is filed in whose log carries no policy.
fn unchecked(store: &Store) -> Vec<String> {
    let auth = crate::authority::Authority::build(store);
    let mut out: Vec<String> = store
        .log
        .iter()
        .flat_map(|l| l.file.versions.iter())
        .map(|v| v.decision.namespace().to_string())
        .filter(|ns| auth.policy(ns).is_none())
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Every namespace whose policy in force is `[none]`.
fn unsigned(store: &Store) -> Vec<String> {
    let auth = crate::authority::Authority::build(store);
    let mut out: Vec<String> = auth
        .policies
        .iter()
        .map(|p| p.namespace.clone())
        .filter(|ns| auth.policy(ns).is_some_and(|p| p.schemes == [crate::authority::Scheme::None]))
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Decisions that are allocated but carry no live acceptance of their
/// current version. Status, not a finding.
fn awaiting(view: &View) -> Vec<String> {
    view.latest_versions()
        .filter_map(|v| v.parsed.as_ref())
        .filter(|p| p.allocation.is_some())
        .filter(|p| {
            !view
                .acceptances
                .iter()
                .map(|a| a.acceptance)
                .any(|a| !view.is_revoked(a) && view.signs_latest(a) && a.decision == p.decision)
        })
        .map(|p| p.decision.to_string())
        .collect()
}

#[path = "mod_tests.rs"]
#[cfg(test)]
mod tests;
