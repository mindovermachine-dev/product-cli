//! The derived reading of a log that every rule queries.
//!
//! Built once per run so the rules stay declarative: which version is the
//! latest for each decision, which hashes exist, which acceptances are still
//! live. Nothing here is cached to disk — L0 has no index, and a view
//! rebuilt from the log every run is exactly the property §5's rebuild test
//! exists to protect.
//!
//! **Latest derives from the parent DAG, never from ULID order.** ULIDs
//! order by one clock, and two writers' clocks prove nothing about
//! parenthood: the latest version of a decision is the unique version no
//! other version of the same decision names as `parent`. A chain with more
//! than one such tip is *forked* — two writers diverged — which the graph
//! stage reports as `G004` and only `ledger merge --resolve` may settle.
//! **A forked decision has no latest version** (LP-8.11, ruling 54): it is
//! absent from [`View::latest`] and its tips are in [`View::forked`], so no
//! reading of the store picks one. Every reader of the latest version
//! states what it does for a decision that has none: the latest-only gate
//! classes skip it, and `G004` is its one finding.
//!
//! Assembling each wire version into its domain form happens here too, so a
//! file that cannot describe a version reports once rather than once per
//! rule that trips over it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::acceptance::Acceptance;
use crate::finding::Finding;
use crate::store::Store;
use crate::version::{DecisionVersion, VersionRaw};

/// One version as the rules see it: its wire form, its path, and the domain
/// form when it assembled.
pub struct ViewedVersion<'a> {
    pub raw: &'a VersionRaw,
    pub path: &'a Path,
    pub parsed: Option<DecisionVersion>,
}

/// One acceptance with the file it was filed in.
pub struct ViewedAcceptance<'a> {
    pub acceptance: &'a Acceptance,
    pub path: &'a Path,
}

/// The whole log, read.
pub struct View<'a> {
    pub versions: Vec<ViewedVersion<'a>>,
    /// Decision id to the index of its latest version in `versions` — the
    /// tip of the parent DAG, not the last file visited. A forked decision
    /// has no entry: it has no latest version (ruling 54).
    pub latest: BTreeMap<String, usize>,
    /// Decisions whose parent DAG carries more than one tip: two writers
    /// diverged, and no reading of the store may resolve that silently.
    /// Indices of every tip, ordered by hash.
    pub forked: BTreeMap<String, Vec<usize>>,
    /// Every `(decision, hash)` pair a version was filed under.
    pub stored: BTreeSet<(String, String)>,
    pub acceptances: Vec<ViewedAcceptance<'a>>,
    pub revoked: BTreeSet<String>,
    /// Faults met while reading, already classified.
    pub findings: Vec<Finding>,
}

impl<'a> View<'a> {
    /// Read the store. Change-sets are visited in ULID order for a
    /// deterministic base, but "latest" is derived from the parent DAG
    /// afterwards — file order proves nothing about parenthood.
    pub fn build(store: &'a Store) -> Self {
        let mut view = Self {
            versions: Vec::new(),
            latest: BTreeMap::new(),
            forked: BTreeMap::new(),
            stored: BTreeSet::new(),
            acceptances: Vec::new(),
            revoked: BTreeSet::new(),
            findings: Vec::new(),
        };
        for logged in &store.log {
            for raw in &logged.file.versions {
                view.take_version(raw, &logged.path);
            }
            for acceptance in &logged.file.acceptances {
                view.acceptances.push(ViewedAcceptance { acceptance, path: &logged.path });
            }
            for revocation in &logged.file.revocations {
                if let Some(acceptance) = revocation.revoked_acceptance() {
                    view.revoked.insert(acceptance.to_string());
                }
            }
        }
        view.derive_latest();
        view.check_revocations(store);
        view.check_sets(store);
        view
    }

    fn take_version(&mut self, raw: &'a VersionRaw, path: &'a Path) {
        let parsed = match DecisionVersion::assemble(raw) {
            Ok(v) => Some(v),
            Err(f) => {
                self.findings.push(f);
                None
            }
        };
        self.stored.insert((raw.decision.to_string(), raw.hash.to_string()));
        self.versions.push(ViewedVersion { raw, path, parsed });
    }

    /// Derive each decision's latest version from its parent DAG.
    ///
    /// A version is a *tip* when no version of the same decision names its
    /// hash as `parent`. Content-identical filings (one hash filed twice —
    /// both sides of a merge carrying the same act) collapse to their first
    /// appearance. One tip is the latest; several tips are a fork, recorded
    /// for `G004` and the merge machinery, and the decision then has no
    /// latest version: no ordering heuristic may pick one (LP-8.11). No tip
    /// at all is only representable when stored hashes lie (`L007` fails
    /// such a store); the last filing stands in.
    fn derive_latest(&mut self) {
        let mut by_decision: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, viewed) in self.versions.iter().enumerate() {
            by_decision.entry(viewed.raw.decision.to_string()).or_default().push(i);
        }
        for (decision, indices) in by_decision {
            // hash -> first index filing it; BTreeMap order = hash order.
            let mut nodes: BTreeMap<String, usize> = BTreeMap::new();
            for &i in &indices {
                nodes.entry(self.versions[i].raw.hash.to_string()).or_insert(i);
            }
            // A hash is claimed — no longer a tip — when a version names it
            // as `parent` or closes it as `merged_from` (a reconciliation).
            let claimed: BTreeSet<String> = indices
                .iter()
                .flat_map(|&i| {
                    let raw = self.versions[i].raw;
                    raw.parent.iter().chain(raw.merged_from.iter())
                })
                .map(|h| h.to_string())
                .collect();
            let tips: Vec<usize> = nodes
                .iter()
                .filter(|(hash, _)| !claimed.contains(*hash))
                .map(|(_, &i)| i)
                .collect();
            match tips.as_slice() {
                [] => {
                    if let Some(&last) = indices.last() {
                        self.latest.insert(decision, last);
                    }
                }
                [tip] => {
                    self.latest.insert(decision, *tip);
                }
                [_, _, ..] => {
                    self.forked.insert(decision, tips);
                }
            }
        }
    }

    /// A revocation naming an acceptance nobody filed is a dangling
    /// reference: the parse gate, not one of the nine.
    fn check_revocations(&mut self, store: &Store) {
        let filed: BTreeSet<String> =
            self.acceptances.iter().map(|a| a.acceptance.id.to_string()).collect();
        for revocation in store.log.iter().flat_map(|c| &c.file.revocations) {
            let Some(acceptance) = revocation.revoked_acceptance() else { continue };
            if !filed.contains(&acceptance.to_string()) {
                self.findings.push(Finding::schema(
                    &acceptance.to_string(),
                    "revocation names an acceptance that is not filed",
                ));
            }
        }
    }

    /// A version naming an undeclared set has no floor to be judged against,
    /// so the stranding rule could not run. Also the parse gate.
    fn check_sets(&mut self, store: &Store) {
        for v in &self.versions {
            let Some(parsed) = &v.parsed else { continue };
            if store.set_in(parsed.decision.namespace(), &parsed.set).is_none() {
                self.findings.push(Finding::schema(
                    &parsed.decision.to_string(),
                    format!(
                        "names set `{}`, which is not declared under `{}`'s sets/ — a version names a set of its own namespace (LP-5.22)",
                        parsed.set,
                        parsed.decision.namespace()
                    ),
                ));
            }
        }
    }

    /// The latest version of every decision, in id order.
    pub fn latest_versions(&self) -> impl Iterator<Item = &ViewedVersion<'a>> {
        self.latest.values().filter_map(|i| self.versions.get(*i))
    }

    /// Whether this decision's chain has diverged into more than one tip.
    pub fn is_forked(&self, decision: &str) -> bool {
        self.forked.contains_key(decision)
    }

    /// How many decisions the log holds versions of, forked ones included.
    pub(crate) fn decision_count(&self) -> usize {
        self.latest.len() + self.forked.len()
    }

    /// Whether the log holds any version of this decision.
    pub(crate) fn has_version(&self, decision: &str) -> bool {
        self.latest.contains_key(decision) || self.is_forked(decision)
    }

    /// The tips of a decision: its latest version, or every tip of a
    /// forked chain in hash order. For readers that show what stands
    /// without picking among tips.
    pub(crate) fn tips(&self, decision: &str) -> Vec<&ViewedVersion<'a>> {
        let indices = match (self.latest.get(decision), self.forked.get(decision)) {
            (Some(i), _) => vec![*i],
            (None, Some(tips)) => tips.clone(),
            (None, None) => Vec::new(),
        };
        indices.into_iter().filter_map(|i| self.versions.get(i)).collect()
    }

    /// Whether this acceptance signs a tip of its decision — the latest
    /// version, or any tip of a forked chain. A display reading; the gate
    /// asks [`View::signs_latest`], which a forked decision never satisfies.
    pub(crate) fn signs_a_tip(&self, a: &Acceptance) -> bool {
        self.tips(&a.decision.to_string()).iter().any(|v| v.raw.hash == a.version)
    }

    /// Whether this acceptance has been revoked.
    pub fn is_revoked(&self, a: &Acceptance) -> bool {
        self.revoked.contains(&a.id.to_string())
    }

    /// Whether this acceptance signs the current state of its decision.
    /// An acceptance of a superseded version is history, not a live claim,
    /// and a forked decision has no current state to sign.
    pub fn signs_latest(&self, a: &Acceptance) -> bool {
        self.latest
            .get(&a.decision.to_string())
            .and_then(|i| self.versions.get(*i))
            .is_some_and(|v| v.raw.hash == a.version)
    }
}
