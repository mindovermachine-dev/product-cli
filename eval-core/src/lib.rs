//! Observation of model runs, kept apart from the judgments made about them.
//!
//! [`docs/eval-format-v1.md`](../../docs/eval-format-v1.md) is normative; this
//! crate follows it. Two records, separate on purpose: a [`RunRecord`] says
//! what a model did, a [`Judgement`] says what another model made of it. The
//! model that executes is never the model that judges, and never judges at the
//! same time — merging them leaves a reader unable to tell which model was
//! confident about what.
//!
//! Nothing here gates. No record is signed, none names a principal, and a
//! number nobody signed must not be able to fail a build. Where a decision is
//! owed it is owed to a store that names one; this is not that store.
//!
//! Where records are kept is [`Backend`]'s to say and [`Blobs`]' to do. The
//! key layout lives in [`store`] and nowhere else, so moving a tool from disk
//! to object storage is an edit to configuration rather than to a call site.
//!
//! [`escape`] looks for the forbidden state — a resolution nobody can be shown
//! to have held — by checking a run against what it declared before acting.
//! [`behaviour`] reads runs against each other at a fixed address. It produces
//! no verdict on any single run — a shift at fixed coordinates says the ground
//! moved or something undeclared was resolved, never that the one act was
//! wrong.

#![deny(clippy::unwrap_used)]

pub mod backend;
pub mod behaviour;
pub mod blobs;
pub mod declaration;
pub mod digest;
pub mod escape;
pub mod judgement;
pub mod pinned;
pub mod run;
pub mod store;

pub use backend::Backend;
pub use behaviour::Reading;
pub use declaration::{Attribution, Declaration};
pub use escape::Finding;
pub use blobs::{Blobs, DiskBlobs};
pub use judgement::{Judge, Judgement};
pub use pinned::Pinned;
pub use run::{Metric, RunRecord};
pub use store::EvalStore;
