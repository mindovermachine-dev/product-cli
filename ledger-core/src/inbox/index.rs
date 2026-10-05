//! The inbox index: one named graph per (repository, branch), built from
//! each branch's committed export (registry PRD §8).
//!
//! **Which branches.** Git does not record which branches have an open
//! pull request, so R0 indexes the default branch plus every
//! remote-tracking branch whose committed export differs from the default
//! branch's. A pull-request number is a label: read from `refs/pull/<n>/head`
//! when the clone has fetched those refs and the head commit matches, absent
//! otherwise; grouping then falls back to the branch.
//!
//! **Rebuilt every invocation.** R0 has no incremental path: the dataset is
//! built from the exports each time the inbox runs, so there is no
//! maintained index for a rebuild to be compared with (that equivalence test
//! belongs to R1.5, where an index is kept).
//!
//! **Per graph, never the union.** The graph-stage shapes run over each named
//! graph alone: with one genesis per store, two repositories in one graph
//! would fail `A005`.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use product_core::pf::sparql_dataset::Dataset;

use super::config::{Config, Repository};
use super::git;
use crate::graph::GraphFinding;

/// One indexed branch.
#[derive(Debug, Clone)]
pub struct Branch {
    pub repository: String,
    /// The branch's name on the remote.
    pub name: String,
    /// The revision its export was read at (`<remote>/<name>`).
    pub rev: String,
    pub head: String,
    pub default: bool,
    /// The default branch's revision: the base every `verify` names.
    pub base: String,
    pub pr: Option<u32>,
    /// The named graph holding this branch's export.
    pub graph: String,
    pub indexed_at: DateTime<Utc>,
    /// Graph-stage findings over this graph alone.
    pub findings: Vec<GraphFinding>,
    /// Why the remote cannot be reached for a push here, when it cannot
    /// (`git::push_unreachable`: a dry run, not a test of write permission).
    pub push_unreachable: Option<String>,
}

/// The index over every configured clone.
pub struct Index {
    pub dataset: Dataset,
    pub branches: Vec<Branch>,
    pub repositories: Vec<Repository>,
}

impl Index {
    /// The clone a branch belongs to.
    pub fn repository(&self, name: &str) -> Option<&Repository> {
        self.repositories.iter().find(|r| r.name == name)
    }

    /// A clone's path, for loading a branch's store.
    pub fn path(&self, name: &str) -> Option<PathBuf> {
        self.repository(name).map(|r| r.path.clone())
    }
}

/// Build the index from the clones' committed exports.
pub fn build(config: &Config) -> Result<Index, String> {
    let dataset = Dataset::new()?;
    let mut branches = Vec::new();
    for repo in &config.repositories {
        branches.extend(index_clone(&dataset, repo)?);
    }
    Ok(Index { dataset, branches, repositories: config.repositories.clone() })
}

fn index_clone(dataset: &Dataset, repo: &Repository) -> Result<Vec<Branch>, String> {
    let default = git::default_branch(repo);
    let remote = git::remote_branches(repo).map_err(|e| format!("{}: {e}", repo.name))?;
    let base = if remote.contains_key(&default) { format!("{}/{default}", repo.remote) } else { default.clone() };
    let base_export = git::exports_at(&repo.path, &base).map_err(|e| format!("{} `{base}`: {e}", repo.name))?;
    let prs = git::pull_requests(&repo.path);
    let mut out = Vec::new();
    let mut add = |name: &str, rev: String, export: &str, is_default: bool| -> Result<(), String> {
        let head = git::commit_of(&repo.path, &rev)?;
        let graph = graph_iri(&repo.name, name);
        dataset.load_graph(&graph, export).map_err(|e| format!("{} `{name}`: {e}", repo.name))?;
        out.push(Branch {
            repository: repo.name.clone(),
            name: name.to_string(),
            findings: crate::graph::shapes::graph_findings_in(dataset, &graph),
            push_unreachable: git::push_unreachable(repo, &head, name),
            pr: prs.get(&head).copied(),
            rev,
            head,
            default: is_default,
            base: base.clone(),
            graph,
            indexed_at: Utc::now(),
        });
        Ok(())
    };
    add(&default, base.clone(), &base_export, true)?;
    for name in remote.keys().filter(|n| **n != default) {
        let rev = format!("{}/{name}", repo.remote);
        let export = git::exports_at(&repo.path, &rev).unwrap_or_default();
        if export != base_export {
            add(name, rev, &export, false)?;
        }
    }
    Ok(out)
}

/// The named graph for one branch of one repository.
pub fn graph_iri(repository: &str, branch: &str) -> String {
    let enc = |s: &str| -> String {
        s.chars()
            .map(|c| if c.is_ascii_alphanumeric() || "-._~/".contains(c) { c.to_string() } else { format!("%{:02X}", c as u32 & 0xff) })
            .collect()
    };
    format!("urn:ledger-inbox:{}:{}", enc(repository), enc(branch))
}
