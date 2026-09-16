//! Reading behaviour at a fixed address, across runs.
//!
//! Behaviour is prior to verdicts and lives precisely where the acceptance
//! predicate is open: you cannot grade a single act, but you can say whether
//! the same question asked twice was answered the same way, and whether two
//! different questions were answered identically.
//!
//! **None of this is a verdict on any single run.** A shift at a fixed address
//! means the ground moved or something was resolved that was not declared — it
//! never means the one act was wrong. Reading it the other way is the mistake
//! the whole separation exists to prevent.

use std::collections::{BTreeMap, BTreeSet};

use crate::run::RunRecord;

/// What was read at one address.
#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    /// The address these runs share.
    pub address: String,
    /// How many runs were seen there.
    pub runs: usize,
    /// How many distinct arrangements answered there.
    pub arrangements: usize,
    /// Mean pairwise agreement of what was proposed, within one arrangement.
    ///
    /// `None` when a single arrangement never answered twice, which is not a
    /// low score — it is the absence of one, and the two must not be confused.
    pub agreement: Option<f64>,
    /// Mean pairwise agreement *between* arrangements at this address.
    ///
    /// Read with [`Reading::agreement`]: the gap between them is what changed
    /// when the arrangement changed. `None` when fewer than two answered.
    pub across_arrangements: Option<f64>,
}

impl Reading {
    /// How much a changed arrangement moved the answer, where both are known.
    ///
    /// Positive means the arrangement agrees with itself more than with the
    /// other — behaviour moved when the worker did. The document's earthquake
    /// rule applies: this is arrangement drift unless an independent measure of
    /// the world moved the same way in the same window, and nothing here can
    /// tell you that.
    pub fn drift(&self) -> Option<f64> {
        Some(self.agreement? - self.across_arrangements?)
    }
}

/// Group runs by the address they declared, dropping those that declared none.
pub fn by_address(runs: &[RunRecord]) -> BTreeMap<String, Vec<&RunRecord>> {
    let mut grouped: BTreeMap<String, Vec<&RunRecord>> = BTreeMap::new();
    for run in runs {
        if let Some(address) = run.address_digest() {
            grouped.entry(address.to_string()).or_default().push(run);
        }
    }
    grouped
}

/// Read every address a set of runs covers.
pub fn read(runs: &[RunRecord]) -> Vec<Reading> {
    by_address(runs).into_iter().map(|(address, at)| read_one(address, &at)).collect()
}

/// Read one address.
fn read_one(address: String, at: &[&RunRecord]) -> Reading {
    let mut by_arrangement: BTreeMap<&str, Vec<&RunRecord>> = BTreeMap::new();
    for run in at {
        by_arrangement.entry(run.arrangement_digest().unwrap_or("(undeclared)")).or_default().push(run);
    }

    let within: Vec<f64> = by_arrangement
        .values()
        .flat_map(|group| pairwise(group.iter().map(|r| proposed(r))))
        .collect();

    let groups: Vec<Vec<BTreeSet<String>>> =
        by_arrangement.values().map(|g| g.iter().map(|r| proposed(r)).collect()).collect();
    let mut between: Vec<f64> = Vec::new();
    for (i, left) in groups.iter().enumerate() {
        for right in groups.iter().skip(i + 1) {
            for a in left {
                for b in right {
                    between.push(similarity(a, b));
                }
            }
        }
    }

    Reading {
        address,
        runs: at.len(),
        arrangements: by_arrangement.len(),
        agreement: mean(&within),
        across_arrangements: mean(&between),
    }
}

/// Addresses whose runs are indistinguishable from another address's.
///
/// §4.4: declared-distinct ground must yield distinct behaviour. Collapse means
/// the declared ground is not being read — the caller said two questions
/// differed and the worker answered them identically, so either the
/// declaration is decorative or the difference never reached the worker.
///
/// Reported as pairs, never as a verdict on either address.
pub fn collapsed(runs: &[RunRecord]) -> Vec<(String, String)> {
    let grouped = by_address(runs);
    let answers: BTreeMap<&String, BTreeSet<BTreeSet<String>>> =
        grouped.iter().map(|(a, at)| (a, at.iter().map(|r| proposed(r)).collect())).collect();

    let mut found = Vec::new();
    let addresses: Vec<&&String> = answers.keys().collect();
    for (i, left) in addresses.iter().enumerate() {
        for right in addresses.iter().skip(i + 1) {
            let (Some(one), Some(other)) = (answers.get(**left), answers.get(**right)) else {
                continue;
            };
            // Every answer at both addresses is the same single answer.
            if one.len() == 1 && one == other {
                found.push(((**left).clone(), (**right).clone()));
            }
        }
    }
    found
}

/// What a run proposed, as a set.
fn proposed(run: &RunRecord) -> BTreeSet<String> {
    run.proposed.iter().cloned().collect()
}

/// Jaccard similarity: 1.0 identical, 0.0 disjoint, 1.0 for two empty answers.
///
/// Two runs that both proposed nothing agree. It is a real answer and the
/// commonest one, so scoring it zero would make silence look like disagreement.
fn similarity(a: &BTreeSet<String>, b: &BTreeSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let union = a.union(b).count();
    if union == 0 {
        return 1.0;
    }
    a.intersection(b).count() as f64 / union as f64
}

/// Every pairwise similarity within one group.
fn pairwise<'a>(answers: impl Iterator<Item = BTreeSet<String>> + 'a) -> Vec<f64> {
    let collected: Vec<BTreeSet<String>> = answers.collect();
    let mut scores = Vec::new();
    for (i, a) in collected.iter().enumerate() {
        for b in collected.iter().skip(i + 1) {
            scores.push(similarity(a, b));
        }
    }
    scores
}

/// The mean of some scores, or nothing when there were none to take.
fn mean(scores: &[f64]) -> Option<f64> {
    (!scores.is_empty()).then(|| scores.iter().sum::<f64>() / scores.len() as f64)
}

#[path = "behaviour_tests.rs"]
#[cfg(test)]
mod tests;
