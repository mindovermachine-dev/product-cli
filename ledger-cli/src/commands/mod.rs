//! Subcommand surface for the `ledger` binary.
//!
//! L0 was the format plus the gate; L1 adds the authoring verbs and L2 the
//! graph. Every authoring verb is a thin shell over the same `ledger-core`
//! validators the gate calls — a verb refuses at write exactly what
//! `verify` would fail one command later, and no second copy of the rules
//! exists on this side of the crate boundary.

mod add;
mod basis_text;
mod cli_enum;
mod common;
mod declare;
mod diff_cmd;
mod evolve;
mod export;
mod graph_cmds;
mod init;
mod inspect;
mod merge_cmd;
mod resolve_cmd;
mod sign;
mod verify;

use std::path::PathBuf;

pub use cli_enum::Commands;

/// Exit codes, CI-conventional: a gate that says no is a different outcome
/// from a gate that could not run, and CI has to be able to tell them apart.
pub const EXIT_OK: i32 = 0;
pub const EXIT_VIOLATIONS: i32 = 1;
pub const EXIT_ERROR: i32 = 2;

/// Dispatch, mapping every outcome to an exit code.
pub fn run(command: Commands, root: Option<PathBuf>) -> i32 {
    let result = dispatch(command, root);
    match result {
        Ok(code) => code,
        Err(message) => {
            eprintln!("error: {message}");
            EXIT_ERROR
        }
    }
}

/// Dispatch. Split in two along the line the store itself draws: verbs
/// that append to the log, and reads that never touch it.
fn dispatch(command: Commands, root: Option<PathBuf>) -> Result<i32, String> {
    match command {
        Commands::Accept { decision, set, group, expires, confirm, json } => {
            sign::accept(root, sign::AcceptFlags { decision, set, group, expires, confirm, json })
        }
        Commands::Add {
            set, statement, namespace, store, discharge, stage, expectation, actor,
            tolerance_override, based_on, revisit_if, note, key, exported,
        } => add::run(root, add::Flags {
            set, statement, namespace, store, discharge, stage, expectation, actor,
            tolerance_override, based_on, revisit_if, note, key, exported,
        }),
        Commands::Allocate { decision, store, discharge, stage, expectation, actor } => {
            evolve::allocate(root, &decision, &store, &discharge, stage.as_deref(), expectation, actor.as_deref())
        }
        Commands::Declare { set, title, tolerance_floor, ground, owner, notes } => {
            declare::run(root, declare::Flags { set, title, tolerance_floor, ground, owner, notes })
        }
        Commands::Escape { decision, exposure, review_by } => {
            evolve::escape(root, &decision, exposure, &review_by)
        }
        Commands::Init => init::run(root),
        Commands::Merge { rev, resolve, install, json } => {
            merge_cmd::run(root, merge_cmd::Flags { rev, resolve, install, json })
        }
        Commands::MergeDriver { base, ours, theirs, path } => {
            merge_cmd::driver(&base, &ours, &theirs, &path)
        }
        Commands::Reindex => graph_cmds::reindex(root),
        Commands::Revise {
            decision, statement, based_on, revisit_if, no_revisit_if, note, parent, key,
        } => {
            let edges = evolve::ReviseEdges {
                based_on: &based_on, revisit_if: &revisit_if, no_revisit_if, note, key,
            };
            evolve::revise(root, &decision, statement, edges, parent.as_deref())
        }
        Commands::Revoke { acceptance, reason } => sign::revoke(root, &acceptance, reason),
        Commands::Supersede { decision, by, reason } => {
            evolve::supersede(root, &decision, &by, reason)
        }
        read => dispatch_read(read, root),
    }
}

/// The read-only projections: nothing here appends to the log.
fn dispatch_read(command: Commands, root: Option<PathBuf>) -> Result<i32, String> {
    match command {
        Commands::Blame { decision } => inspect::blame(root, &decision),
        Commands::Coverage { set, json, today } => {
            graph_cmds::coverage(root, set.as_deref(), json, today.as_deref())
        }
        Commands::Diff { spec, json } => diff_cmd::run(root, &spec, json),
        Commands::Export { format, namespace, out } => {
            export::run(root, export::Flags { format, namespace, out })
        }
        Commands::Log { set } => inspect::log(root, set.as_deref()),
        Commands::Show { decision, set, group, json, today } => {
            inspect::show(root, inspect::ShowFlags { decision, set, group, json, today })
        }
        Commands::Status { today } => inspect::status(root, today.as_deref()),
        Commands::Verify { gate, json, today, no_blame, export } => {
            verify::run(root, verify::Args { gate, json, today, blame: !no_blame, export })
        }
        // Every writing verb is handled by `dispatch`, which routes here
        // only for what is left. Reaching this arm means a variant was
        // added to the enum and to neither match — reported, never panicked.
        _ => Err("internal: this subcommand is not wired into either dispatch half".to_string()),
    }
}

/// Resolve the repo root: the flag, else the first ancestor with a store.
pub fn resolve_root(root: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(explicit) = root {
        return Ok(explicit);
    }
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    ledger_core::store::find_root(&cwd).ok_or_else(|| {
        format!(
            "no {}/ found here or in any parent — run `ledger init` to scaffold one",
            ledger_core::STORE_DIR
        )
    })
}
