//! `ledger inbox` — the R0 decision registry over local clones (#79).
//!
//! `inbox list` shows, for the holder, every proposed decision they may
//! accept across the configured clones, grouped by repository and pull
//! request (or branch), newest first; the acceptances awaiting re-acceptance
//! in their own state; and every namespace with no policy, named as
//! unchecked. `inbox accept` writes the batch selection file, prints its
//! manifest, and — confirmed at a terminal — signs each branch with `ledger
//! accept --batch` in a worktree, commits as the holder, runs `ledger verify
//! --base <default branch>`, and pushes only when that is green. Every
//! `verify` the inbox runs names its base explicitly; none relies on
//! `origin/HEAD`. Every run first sweeps the worktrees an earlier one left.

use std::path::{Path, PathBuf};

use clap::Subcommand;
use ledger_core::identity::Identity;
use ledger_core::inbox::config::{self, Config};
use ledger_core::inbox::index::{self, Branch, Index};
use ledger_core::inbox::list::{self, Listing, Review};

use super::EXIT_OK;

/// `ledger inbox …`
#[derive(Subcommand)]
pub enum InboxCmd {
    /// List every proposed decision you may accept, across the clones
    List {
        /// The clones to read (default: $XDG_CONFIG_HOME/ledger/inbox.yml)
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Accept listed decisions in one sitting: enumerate, then confirm
    Accept {
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,
        /// Every item in the list, re-acceptances included
        #[arg(long, conflicts_with = "decision")]
        all: bool,
        /// One decision of the list (repeatable)
        #[arg(long, value_name = "DEC")]
        decision: Vec<String>,
        /// The role the whole batch is made as (D9)
        #[arg(long = "as", value_name = "ROLE")]
        as_role: Option<String>,
        /// The manifest the dry run printed. Without it nothing is written
        #[arg(long, value_name = "MANIFEST")]
        confirm: Option<String>,
    },
}

pub fn run(cmd: InboxCmd) -> Result<i32, String> {
    match cmd {
        InboxCmd::List { config, json } => list_cmd(config, json),
        InboxCmd::Accept { config, all, decision, as_role, confirm } => {
            super::inbox_accept::run(config, super::inbox_accept::Selection { all, decisions: decision }, as_role, confirm)
        }
    }
}

/// Everything one sitting reads.
pub struct Sitting {
    pub config_path: PathBuf,
    pub holder: Identity,
    pub index: Index,
    pub listing: Listing,
    pub reviews: Vec<Review>,
}

/// Load the configuration, the holder, the index, the list and the reviews.
pub fn open(config_path: Option<PathBuf>, as_role: Option<&str>) -> Result<Sitting, String> {
    let config_path = config_path.unwrap_or_else(config::default_path);
    let config = config::load(&config_path)?;
    let holder = holder(&config)?;
    for repo in &config.repositories {
        ledger_core::inbox::git::sweep(repo).map_err(|e| format!("{}: {e}", repo.name))?;
    }
    let index = index::build(&config)?;
    let listing = list::list(&index, &holder, as_role)?;
    let mut reviews = Vec::new();
    for branch in &index.branches {
        reviews.extend(review(&index, branch, &holder)?);
    }
    Ok(Sitting { config_path, holder, index, listing, reviews })
}

/// The holder: the git identity every clone is configured with. A sitting
/// is one principal's; clones configured as different people refuse.
fn holder(config: &Config) -> Result<Identity, String> {
    let mut who: Option<Identity> = None;
    for repo in &config.repositories {
        let id = ledger_core::whoami::git_identity(&repo.path).map_err(|e| format!("{}: {e}", repo.name))?;
        match &who {
            Some(w) if *w != id => {
                return Err(format!(
                    "{} is configured as {id}, another clone as {w} — a sitting is one principal's",
                    repo.name
                ))
            }
            _ => who = Some(id),
        }
    }
    who.ok_or_else(|| "no repository configured".to_string())
}

/// The holder's acceptances on one branch awaiting re-acceptance, read
/// from `ledger verify --base <default> --json` run in a worktree.
fn review(index: &Index, branch: &Branch, holder: &Identity) -> Result<Vec<Review>, String> {
    let repo = index.repository(&branch.repository).ok_or("unknown repository")?;
    let wt = ledger_core::inbox::git::worktree(repo, &branch.name, &branch.rev)?;
    let out = verify(&wt, &branch.base, &["--json", "--no-blame"])?;
    let store = ledger_core::store::load(&wt);
    ledger_core::inbox::git::discard_worktree(repo, &wt);
    let report: serde_json::Value = serde_json::from_str(&out.0).map_err(|e| format!("verify --json: {e}"))?;
    let acceptance = |id: &str| store.log.iter().flat_map(|l| l.file.acceptances.iter()).find(|a| a.id.to_string() == id).cloned();
    // An acceptance the default branch already holds is listed there, once,
    // not again on every branch cut after it.
    let on_base: std::collections::BTreeSet<String> = match branch.default {
        true => Default::default(),
        false => ledger_core::revision::load_at(&repo.path, &branch.base)
            .map(|s| s.log.iter().flat_map(|l| l.file.acceptances.iter()).map(|a| a.id.to_string()).collect())
            .unwrap_or_default(),
    };
    let mut out_items = Vec::new();
    let mut push = |id: &str, reason: &'static str, deadline: Option<String>| {
        if let Some(a) = acceptance(id).filter(|a| a.actor == *holder && !on_base.contains(id)) {
            out_items.push(Review {
                repository: branch.repository.clone(),
                branch: branch.name.clone(),
                decision: a.decision.to_string(),
                version: a.version.to_string(),
                acceptance: id.to_string(),
                reason,
                deadline,
            });
        }
    };
    for r in report["reaccept"].as_array().into_iter().flatten() {
        let deadline = r["deadline"].as_str().map(str::to_string);
        push(r["acceptance"].as_str().unwrap_or_default(), "key-closed", deadline);
    }
    for f in report["findings"].as_array().into_iter().flatten().filter(|f| f["class"] == "L011") {
        let subject = f["subject"].as_str().unwrap_or_default();
        if subject.starts_with("acc:") {
            push(subject, "fails-on-base", None);
        }
    }
    Ok(out_items)
}

/// `ledger verify --base <base> …` in `dir`: (stdout, stderr, exit code).
/// The base is always named.
pub fn verify(dir: &Path, base: &str, extra: &[&str]) -> Result<(String, String, i32), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let out = std::process::Command::new(exe)
        .arg("--root")
        .arg(dir)
        .args(["verify", "--base", base])
        .args(extra)
        .output()
        .map_err(|e| e.to_string())?;
    Ok((
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(2),
    ))
}

fn list_cmd(config: Option<PathBuf>, json: bool) -> Result<i32, String> {
    let sitting = open(config, None)?;
    if json {
        let value = serde_json::json!({
            "holder": sitting.holder.to_string(),
            "branches": super::inbox_render::branches_json(&sitting.index),
            "items": sitting.listing.items,
            "needs_reacceptance": sitting.reviews,
            "unchecked": sitting.listing.unchecked,
        });
        println!("{}", serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?);
    } else {
        print!("{}", super::inbox_render::listing(&sitting));
    }
    Ok(EXIT_OK)
}
