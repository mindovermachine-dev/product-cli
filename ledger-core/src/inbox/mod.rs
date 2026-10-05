//! The R0 decision registry: `ledger inbox` over local clones (#79,
//! registry PRD §9).
//!
//! A holder lists every proposed decision they may accept across the
//! configured clones, and accepts them in one sitting: the inbox writes the
//! batch selection file (`batch_file`), and each branch is signed by
//! `ledger accept --batch` in its own worktree, committed and pushed —
//! exactly as the CLI would in that repository. Reject and
//! changes-requested are R1.

pub mod config;
pub mod git;
pub mod index;
pub mod list;
pub mod terms;

#[cfg(test)]
#[path = "index_tests.rs"]
mod index_tests;
