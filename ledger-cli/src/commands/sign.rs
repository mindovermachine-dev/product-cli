//! Adapters for the signature verbs: accept, revoke.
//!
//! Both verbs **refuse a non-interactive invocation** (#71, PRD §5 and
//! §10): a write happens only when stdin is a terminal. The check is
//! `terminal::gate`, run before dispatch for the whole surface. There is no
//! environment override and no flag — an override would be exactly the
//! bypass an agent harness reaches for. The selection form's dry run writes
//! nothing and stays scriptable; its `--confirm` write is refused like the
//! rest. The manifest step is kept beside the terminal check, not replaced
//! by it: the manifest binds what was read to what is signed, which a
//! terminal prompt does not (audit C8).
//!
//! `accept` has two shapes over one meaning. Naming a decision signs it
//! immediately, exactly as it always has. Naming a *selection* — `--set` or
//! `--group`, the same selector syntax `show` reads — enumerates it, pins
//! it, prints it, and writes nothing: signing then takes an explicit
//! `--confirm` carrying the manifest that read produced, which is a value
//! nobody types by accident because only a dry run can produce it.

use std::path::PathBuf;
use std::time::Instant;

use ledger_core::author::{AcceptArgs, AcceptBatchArgs, AcceptGroupArgs, AuthorError, RevokeArgs};
use ledger_core::show::Selector;

use super::common::{self, finish, open_author, parse_date};
use super::{EXIT_OK, EXIT_VIOLATIONS};

/// What one `accept` invocation covers, and how it reports.
pub struct AcceptFlags {
    pub decision: Option<String>,
    pub set: Option<String>,
    pub group: Option<String>,
    pub expires: Option<String>,
    pub confirm: Option<String>,
    pub json: bool,
    pub as_role: Option<String>,
}

pub fn accept(root: Option<PathBuf>, flags: AcceptFlags) -> Result<i32, String> {
    let selector =
        common::selector(flags.decision.as_deref(), flags.set.as_deref(), flags.group.as_deref())?;
    match selector {
        Selector::One(_) if flags.confirm.is_some() || flags.json => Err(
            "--confirm and --json belong to a selection (--set / --group); naming one decision signs it directly"
                .to_string(),
        ),
        Selector::One(decision) => {
            let mut author = open_author(root)?;
            author.as_role = flags.as_role;
            let expires_at = flags.expires.as_deref().map(parse_date).transpose()?;
            finish(author.accept(AcceptArgs { decision, expires_at }))
        }
        // Deliberately not "everything pending": a selection has to be
        // named. An implicit whole-store accept is the one convenience this
        // verb must not offer.
        Selector::All => Err(
            "name a decision, or --set, or --group — `accept` never covers the whole store implicitly"
                .to_string(),
        ),
        selector => grouped(root, selector, flags),
    }
}

/// `accept --batch <file>`: this clone's rows of a selection file, pinned
/// by the file's manifest; signing takes `--confirm` with that digest.
pub fn batch(root: Option<PathBuf>, file: PathBuf, (repository, branch): (Option<String>, Option<String>), flags: AcceptFlags) -> Result<i32, String> {
    let started = Instant::now();
    let mut author = open_author(root)?;
    author.as_role = flags.as_role.clone();
    let args = AcceptBatchArgs {
        file: ledger_core::batch_file::read(&file)?,
        path: file.display().to_string(),
        repository,
        branch: branch.or_else(|| current_branch(&author.root)),
        expires_at: flags.expires.as_deref().map(parse_date).transpose()?,
        confirm: flags.confirm,
    };
    report(author.accept_batch(args), flags.json, started)
}

/// The branch checked out, when HEAD is on one.
fn current_branch(root: &std::path::Path) -> Option<String> {
    let out = std::process::Command::new("git").arg("-C").arg(root).args(["symbolic-ref", "-q", "--short", "HEAD"]).output().ok()?;
    let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (out.status.success() && !name.is_empty()).then_some(name)
}

/// The batched act: enumerate, and only against a matching manifest, sign.
fn grouped(root: Option<PathBuf>, selector: Selector, flags: AcceptFlags) -> Result<i32, String> {
    let started = Instant::now();
    let mut author = open_author(root)?;
    author.as_role = flags.as_role.clone();
    let args = AcceptGroupArgs {
        selector,
        expires_at: flags.expires.as_deref().map(parse_date).transpose()?,
        confirm: flags.confirm,
    };
    report(author.accept_group(args), flags.json, started)
}

/// Print a batched run, enumeration and verdict, in either form.
fn report(result: Result<ledger_core::batch::Outcome, AuthorError>, as_json: bool, started: Instant) -> Result<i32, String> {
    let outcome = match result {
        Ok(outcome) => outcome,
        Err(err @ (AuthorError::Refused(_) | AuthorError::Conflict(_) | AuthorError::Unauthorized(_))) => {
            eprintln!("{err}");
            return Ok(EXIT_VIOLATIONS);
        }
        Err(AuthorError::Usage(m) | AuthorError::Io(m)) => return Err(m),
    };
    // The elapsed time is the requirements input L5 was promised: how long
    // a pass actually costs, per weight class. It is reported on every
    // path, refusals included, because a refused run cost time too.
    let elapsed = format!("{:.2}s", started.elapsed().as_secs_f64());
    if as_json {
        println!("{}", json(&outcome, &elapsed)?);
    } else {
        print!("{}", ledger_core::batch::render(&outcome, &elapsed));
    }
    Ok(if outcome.refused() { EXIT_VIOLATIONS } else { EXIT_OK })
}

/// The machine form: the same struct the text renders, with the wall-clock
/// alongside it, so neither form carries a fact the other lacks.
fn json(outcome: &ledger_core::batch::Outcome, elapsed: &str) -> Result<String, String> {
    let mut value = serde_json::to_value(outcome).map_err(|e| e.to_string())?;
    if let Some(object) = value.as_object_mut() {
        object.insert("elapsed".to_string(), serde_json::Value::String(elapsed.to_string()));
    }
    serde_json::to_string_pretty(&value).map_err(|e| e.to_string())
}

pub fn revoke(
    root: Option<PathBuf>,
    acceptance: &str,
    reason: String,
    as_role: Option<String>,
) -> Result<i32, String> {
    let mut author = open_author(root)?;
    author.as_role = as_role;
    let args = RevokeArgs { acceptance: acceptance.parse()?, reason };
    finish(author.revoke(args))
}

