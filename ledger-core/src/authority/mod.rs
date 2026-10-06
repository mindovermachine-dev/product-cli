//! The authority records — who may accept, revoke and grant (format 6).
//!
//! The file schema the authority vocabulary projects
//! (`ledger/spec/authority/ledger-authority.ttl`, draft-2026-09-22 as
//! amended for ruling 3): role files under `.decisions/roles/`, and as log
//! entries grants, grant acceptances, unavailability and availability,
//! revocations, key bindings and namespace policy versions. The files are
//! the truth; the graph is the read model (`graph::authority`), and the
//! SHACL shapes' gate classes `A003`/`A005` run in the graph stage.
//!
//! Nothing here is consulted for a namespace without a policy: such a
//! namespace is a pre-v2 store, and its acceptances are judged exactly as
//! before. `init --namespace` is what puts a namespace under policy.

pub mod availability;
pub mod binding;
pub mod check;
pub mod choice;
pub mod filing;
pub mod key_close;
#[cfg(test)]
pub(crate) mod fixture;
pub mod grant;
pub mod payload;
pub mod policy;
pub mod references;
pub mod revocation;
pub mod role;
pub mod signers;
pub mod structure;
pub mod view;

pub use availability::{Availability, Basis, Unavailability};
pub use binding::{BindingAct, KeyBinding};
pub use check::{authorize, authorize_named, Authorized, Denial, Target};
pub use grant::{Grant, GrantAcceptance, GrantScope, Limit, Order};
pub use policy::{Policy, Scheme};
pub use revocation::{Revocable, Revocation};
pub use role::{Act, Capability, Role};
pub use view::Authority;
