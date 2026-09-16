//! Detecting the forbidden state: a resolution nobody can be shown to have held.
//!
//! Three checks, each closed even though the acceptance predicate is not.
//! Completeness asks whether the declaration says enough to be contradicted.
//! Ground reads ask whether what was consulted matches what was declared.
//! Attribution asks whether every claim rests on something declared.
//!
//! **These are findings, not verdicts, and they do not gate.** Whether a
//! particular escape is tolerable is a judgement, and a judgement needs an
//! owner. What the checks give that owner is a list they did not have to
//! assemble by reading transcripts.

use serde::{Deserialize, Serialize};

use crate::run::RunRecord;

/// What kind of escape was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// The run declared nothing at all, so nothing about it can be contradicted.
    Undeclared,
    /// A declaration is missing a field it must carry to be complete.
    IncompleteDeclaration,
    /// Ground was declared as needed and never read.
    DeclaredButUnread,
    /// Ground was read that the declaration never named.
    ReadButUndeclared,
    /// A claim rests on nothing the worker declared.
    UnattributedClaim,
}

impl Kind {
    /// What this kind is called in a report.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Undeclared => "undeclared",
            Self::IncompleteDeclaration => "incomplete-declaration",
            Self::DeclaredButUnread => "declared-but-unread",
            Self::ReadButUndeclared => "read-but-undeclared",
            Self::UnattributedClaim => "unattributed-claim",
        }
    }
}

/// One escape candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub kind: Kind,
    /// What the finding is about: a field, a ground element, a claim.
    pub subject: String,
    /// Why it was raised, in terms a reader can act on.
    pub message: String,
}

impl Finding {
    fn new(kind: Kind, subject: impl Into<String>, message: impl Into<String>) -> Self {
        Self { kind, subject: subject.into(), message: message.into() }
    }
}

/// Every escape candidate in one run.
pub fn check(run: &RunRecord) -> Vec<Finding> {
    let Some(declared) = &run.declared else {
        return vec![Finding::new(
            Kind::Undeclared,
            run.id.clone(),
            "the run declared nothing before acting, so nothing it did can be \
             contradicted by what it said it would do",
        )];
    };

    let mut found: Vec<Finding> = declared
        .missing()
        .into_iter()
        .map(|field| {
            Finding::new(
                Kind::IncompleteDeclaration,
                field,
                format!("the declaration carries no `{field}`"),
            )
        })
        .collect();

    found.extend(ground_findings(run));
    found.extend(attribution_findings(run));
    found
}

/// Declared against read, both directions.
///
/// **Only two of the document's three ground findings.** The third — a
/// fast-ticking axis with no read at act time — needs a tick rate per ground
/// element, and nothing in this format carries one. Reporting two findings and
/// naming the third as absent is honest; silently reporting two as though they
/// were all of them is not.
fn ground_findings(run: &RunRecord) -> Vec<Finding> {
    let Some(declared) = &run.declared else {
        return Vec::new();
    };
    let mut found = Vec::new();

    for element in &declared.ground {
        if !run.ground_read.contains(element) {
            found.push(Finding::new(
                Kind::DeclaredButUnread,
                element.clone(),
                format!("`{element}` was declared as needed and never read"),
            ));
        }
    }
    for element in &run.ground_read {
        if !declared.ground.contains(element) {
            found.push(Finding::new(
                Kind::ReadButUndeclared,
                element.clone(),
                format!("`{element}` was read but the declaration does not name it"),
            ));
        }
    }
    found
}

/// Claims resting on nothing declared.
///
/// A claim attributed to ground the declaration never named is *also* an
/// escape, and is reported as `read-but-undeclared` against that ground rather
/// than as a second unattributed claim — the defect is the undeclared ground,
/// and saying it twice would make one problem look like two.
fn attribution_findings(run: &RunRecord) -> Vec<Finding> {
    let Some(declared) = &run.declared else {
        return Vec::new();
    };
    let mut found = Vec::new();

    for attribution in &run.attributions {
        match &attribution.ground {
            None => found.push(Finding::new(
                Kind::UnattributedClaim,
                attribution.claim.clone(),
                "the claim rests on nothing the worker declared",
            )),
            Some(ground) if !declared.ground.contains(ground) => {
                found.push(Finding::new(
                    Kind::ReadButUndeclared,
                    ground.clone(),
                    format!(
                        "a claim rests on `{ground}`, which the declaration does not name"
                    ),
                ));
            }
            Some(_) => {}
        }
    }
    found
}

/// Every escape candidate across a set of runs, paired with the run it is in.
pub fn check_all(runs: &[RunRecord]) -> Vec<(String, Finding)> {
    runs.iter()
        .flat_map(|run| check(run).into_iter().map(move |f| (run.id.clone(), f)))
        .collect()
}

#[path = "escape_tests.rs"]
#[cfg(test)]
mod tests;
