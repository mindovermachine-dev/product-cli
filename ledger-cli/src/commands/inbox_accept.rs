//! `ledger inbox accept` — one sitting, one confirmation, one signature per
//! acceptance, committed and pushed per branch.
//!
//! The inbox writes the batch selection file (`ledger.acceptance-batch.v1`)
//! beside its configuration and prints the rows, the grant per row and the
//! manifest. Confirmed with that manifest — at a terminal only — it takes
//! each branch in turn: fetch (an item is re-read before it is signed),
//! a fresh worktree at the remote branch, `ledger accept --batch … --confirm`
//! there (which refuses if a version hash moved), the exports regenerated,
//! a commit as the holder, a push, and `ledger verify --base <default>`.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use ledger_core::batch_file::{self, BatchFile, Row, BATCH_FORM};
use ledger_core::inbox::git;

use super::inbox::{self, Sitting};
use super::{EXIT_OK, EXIT_VIOLATIONS};

/// What the holder chose from the list.
pub struct Selection {
    pub all: bool,
    pub decisions: Vec<String>,
}

pub fn run(config: Option<PathBuf>, selection: Selection, as_role: Option<String>, confirm: Option<String>) -> Result<i32, String> {
    let started = Instant::now();
    let sitting = inbox::open(config, as_role.as_deref())?;
    if let Some(why) = sitting.holder.model_or_bot_reason() {
        eprintln!("refused — {} is not a principal who can accept: {why}", sitting.holder);
        return Ok(EXIT_VIOLATIONS);
    }
    let rows = match rows(&sitting, &selection, as_role.as_deref()) {
        Ok(rows) => rows,
        Err(why) => {
            eprintln!("refused — {why}");
            return Ok(EXIT_VIOLATIONS);
        }
    };
    let file = BatchFile { form: BATCH_FORM.to_string(), actor: sitting.holder.to_string(), as_role, rows };
    let path = batch_path(&sitting);
    batch_file::write(&path, &file)?;
    let manifest = batch_file::manifest(&file);
    print!("{}", plan(&file, &manifest, &path));
    let Some(confirmed) = confirm else {
        println!("\ndry run — nothing written. Having read the above, sign this sitting with:\n  ledger inbox accept {} --confirm {manifest}", flags(&selection));
        return Ok(EXIT_OK);
    };
    if confirmed != manifest {
        eprintln!("refused — the inbox moved since it was read: you confirmed {confirmed}, it now pins {manifest}");
        return Ok(EXIT_VIOLATIONS);
    }
    let code = sign(&sitting, &file, &path, &manifest)?;
    println!("elapsed    {:.2}s", started.elapsed().as_secs_f64());
    Ok(code)
}

/// The rows of the sitting: the selected items, and the affirmations of
/// acceptances under a since-closed key.
fn rows(s: &Sitting, sel: &Selection, as_role: Option<&str>) -> Result<Vec<Row>, String> {
    if !sel.all && sel.decisions.is_empty() {
        return Err("name what to accept: --all, or --decision <dec> (repeatable)".to_string());
    }
    let wanted = |d: &str| sel.all || sel.decisions.iter().any(|x| x == d);
    let mut out = Vec::new();
    for i in s.listing.items.iter().filter(|i| wanted(&i.decision)) {
        writable(s, &i.repository, &i.branch)?;
        if i.grant.starts_with("needs --as") {
            return Err(format!("{} on {} {}: name the role with --as", i.decision, i.repository, i.grant));
        }
        out.push(Row { repository: i.repository.clone(), branch: Some(i.branch.clone()), decision: i.decision.clone(), version: i.version.clone(), grant: Some(i.grant.clone()) });
    }
    for r in s.reviews.iter().filter(|r| r.reason == "key-closed" && wanted(&r.decision)) {
        writable(s, &r.repository, &r.branch)?;
        let branch = s.index.branches.iter().find(|b| b.repository == r.repository && b.name == r.branch).ok_or("unknown branch")?;
        let grant = ledger_core::inbox::list::grant_on(&s.index, branch, &s.holder, &r.decision, as_role)
            .ok_or_else(|| format!("{}: no grant to re-accept it under", r.decision))?;
        out.push(Row { repository: r.repository.clone(), branch: Some(r.branch.clone()), decision: r.decision.clone(), version: r.version.clone(), grant: Some(grant) });
    }
    for d in &sel.decisions {
        if !out.iter().any(|r| &r.decision == d) {
            return Err(format!("{d} is not in your list"));
        }
    }
    if out.is_empty() {
        return Err("nothing in your list to accept".to_string());
    }
    out.sort();
    out.dedup_by(|a, b| a.repository == b.repository && a.branch == b.branch && a.decision == b.decision);
    Ok(out)
}

fn writable(s: &Sitting, repository: &str, branch: &str) -> Result<(), String> {
    match s.index.branches.iter().find(|b| b.repository == repository && b.name == branch).and_then(|b| b.read_only.clone()) {
        Some(why) => Err(format!("{repository} @ {branch} is read-only: {why}")),
        None => Ok(()),
    }
}

fn batch_path(s: &Sitting) -> PathBuf {
    s.config_path.with_file_name("inbox-batch.yml")
}

fn plan(file: &BatchFile, manifest: &str, path: &std::path::Path) -> String {
    let mut out = format!("batch      {} — {} row(s), as {}\nmanifest   {manifest}\n", path.display(), file.rows.len(), file.actor);
    for r in &file.rows {
        out.push_str(&format!(
            "  {} @ {}  {}  {}\n    under {}\n",
            r.repository,
            r.branch.clone().unwrap_or_default(),
            r.decision,
            r.version,
            r.grant.clone().unwrap_or_default()
        ));
    }
    out
}

fn flags(sel: &Selection) -> String {
    if sel.all {
        "--all".to_string()
    } else {
        sel.decisions.iter().map(|d| format!("--decision {d}")).collect::<Vec<_>>().join(" ")
    }
}

/// Each branch in turn: re-read, sign, export, commit, push, verify.
fn sign(s: &Sitting, file: &BatchFile, path: &std::path::Path, manifest: &str) -> Result<i32, String> {
    let mut groups: BTreeMap<(String, String), usize> = BTreeMap::new();
    for r in &file.rows {
        *groups.entry((r.repository.clone(), r.branch.clone().unwrap_or_default())).or_default() += 1;
    }
    let mut failed = 0;
    for ((repository, branch), count) in &groups {
        match sign_branch(s, repository, branch, *count, path, manifest) {
            Ok(line) => println!("{line}"),
            Err(why) => {
                failed += 1;
                println!("{repository} @ {branch}: not signed — {why}");
            }
        }
    }
    println!("\n{} branch(es) signed and pushed, {failed} not", groups.len() - failed);
    Ok(if failed == 0 { EXIT_OK } else { EXIT_VIOLATIONS })
}

fn sign_branch(s: &Sitting, repository: &str, branch: &str, count: usize, path: &std::path::Path, manifest: &str) -> Result<String, String> {
    let repo = s.index.repository(repository).ok_or("unknown repository")?;
    let indexed = s.index.branches.iter().find(|b| b.repository == repository && b.name == branch).ok_or("not indexed")?;
    git::fetch(repo)?;
    let wt = git::worktree(repo, branch, &indexed.rev)?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let p = path.display().to_string();
    let status = Command::new(&exe)
        .arg("--root")
        .arg(&wt)
        .args(["accept", "--batch", &p, "--repository", repository, "--branch", branch, "--confirm", manifest])
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("`ledger accept --batch` refused — see above".to_string());
    }
    let export = Command::new(&exe).arg("--root").arg(&wt).args(["export", "--format", "ntriples"]).output().map_err(|e| e.to_string())?;
    if !export.status.success() {
        return Err(format!("export: {}", String::from_utf8_lossy(&export.stderr).trim()));
    }
    git::git(&wt, &["add", "-A", ".decisions", "docs/decisions"])?;
    git::git(&wt, &["commit", "-q", "-m", &format!("ledger: accept {count} decision(s) — ledger inbox")])?;
    git::git(&wt, &["push", "-q", &repo.remote, &format!("HEAD:refs/heads/{branch}")])?;
    let (stdout, stderr, code) = inbox::verify(&wt, &indexed.base, &["--export"])?;
    let report = format!("{stdout}{stderr}");
    let verdict = if code == 0 {
        "green".to_string()
    } else {
        let findings: Vec<&str> = report.lines().filter(|l| l.trim_start().starts_with("- [")).collect();
        format!("NOT green:\n{}", findings.join("\n"))
    };
    Ok(format!("{repository} @ {branch}: {count} accepted, committed as {}, pushed — verify --base {} {verdict}", s.holder, indexed.base))
}
