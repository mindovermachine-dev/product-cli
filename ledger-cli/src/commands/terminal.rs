//! Which invocations run only at a terminal (#71, #85).
//!
//! Two sides, decided here once for the whole surface and checked in
//! [`super::run`] before anything else runs, so a refused invocation opens
//! no store and writes nothing:
//!
//! - **Terminal** — every verb that signs or unsays a signature, and every
//!   verb that writes an authority record: `accept <dec>`,
//!   `accept --set|--group --confirm`, `revoke`, `init --namespace`,
//!   `role declare`, `grant new|accept|revoke`, `unavailable`, `available`,
//!   `identity add|rotate|revoke`, `policy set`.
//! - **Scriptable** — the verbs that file decisions (`declare`, `add`,
//!   `allocate`, `escape`, `revise`, `supersede`), the derived-file verbs
//!   (`identity sync`, `reindex`, `merge --install`, `merge-driver`), a bare
//!   `init`, `merge --resolve` (it files reconciled *versions* only, never an
//!   acceptance or a revocation — `ledger_core::merge::resolve`), the
//!   selection dry run, and every read.
//!
//! The match has no wildcard arm: a new subcommand does not compile until
//! it is put on one side. There is no override — no flag, no environment
//! variable, no injected predicate.

use super::authority_enum::{GrantCmd, IdentityCmd, PolicyCmd, RoleCmd};
use super::cli_enum::Commands;

/// The side an invocation is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Refused unless stdin is a terminal; the verb as the refusal names it.
    Terminal(&'static str),
    Scriptable,
}

/// Refuse a terminal-only invocation whose stdin is not a terminal.
pub fn gate(command: &Commands) -> Result<(), String> {
    match mode(command) {
        Mode::Terminal(verb) => super::common::require_terminal(verb),
        Mode::Scriptable => Ok(()),
    }
}

/// Which side `command` is on.
pub fn mode(command: &Commands) -> Mode {
    match command {
        Commands::Accept { decision, confirm, .. } => accept(decision.is_some(), confirm.is_some()),
        Commands::Revoke { .. } => Mode::Terminal("revoke"),
        Commands::Init { namespace, .. } => match namespace {
            Some(_) => Mode::Terminal("init --namespace"),
            None => Mode::Scriptable,
        },
        Commands::Available { .. } => Mode::Terminal("available"),
        Commands::Unavailable { .. } => Mode::Terminal("unavailable"),
        Commands::Grant { cmd } => grant(cmd),
        Commands::Identity { cmd } => identity(cmd),
        Commands::Policy { cmd } => policy(cmd),
        Commands::Role { cmd } => role(cmd),
        Commands::Add { .. }
        | Commands::Allocate { .. }
        | Commands::Declare { .. }
        | Commands::Escape { .. }
        | Commands::Revise { .. }
        | Commands::Supersede { .. } => Mode::Scriptable,
        Commands::Merge { .. } | Commands::MergeDriver { .. } | Commands::Reindex => Mode::Scriptable,
        Commands::Blame { .. }
        | Commands::Coverage { .. }
        | Commands::Diff { .. }
        | Commands::Export { .. }
        | Commands::Log { .. }
        | Commands::Show { .. }
        | Commands::Status { .. }
        | Commands::Verify { .. } => Mode::Scriptable,
    }
}

/// Naming a decision signs it; a selection signs only with `--confirm`.
fn accept(names_decision: bool, confirms: bool) -> Mode {
    match (names_decision, confirms) {
        (true, _) => Mode::Terminal("accept"),
        (false, true) => Mode::Terminal("accept --confirm"),
        (false, false) => Mode::Scriptable,
    }
}

fn grant(cmd: &GrantCmd) -> Mode {
    match cmd {
        GrantCmd::Accept { .. } => Mode::Terminal("grant accept"),
        GrantCmd::New { .. } => Mode::Terminal("grant new"),
        GrantCmd::Revoke { .. } => Mode::Terminal("grant revoke"),
    }
}

fn identity(cmd: &IdentityCmd) -> Mode {
    match cmd {
        IdentityCmd::Add { .. } => Mode::Terminal("identity add"),
        IdentityCmd::Revoke { .. } => Mode::Terminal("identity revoke"),
        IdentityCmd::Rotate { .. } => Mode::Terminal("identity rotate"),
        IdentityCmd::Sync => Mode::Scriptable,
    }
}

fn policy(cmd: &PolicyCmd) -> Mode {
    match cmd {
        PolicyCmd::Set { .. } => Mode::Terminal("policy set"),
        PolicyCmd::Show { .. } => Mode::Scriptable,
    }
}

fn role(cmd: &RoleCmd) -> Mode {
    match cmd {
        RoleCmd::Declare { .. } => Mode::Terminal("role declare"),
    }
}

#[path = "terminal_tests.rs"]
#[cfg(test)]
mod tests;
