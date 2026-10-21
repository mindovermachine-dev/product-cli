//! The §3.7 actor node model — who may act in the product being specified.
//!
//! Split from model.rs for the 400-line gate. An actor kind is a domain-level
//! kind of principal carrying typed claims; the system's authentication supplies
//! the actor, never the caller. The edges that hang authorization off existing
//! nodes (a trigger's `acts_as`, a UI step's `offered_to`, a View's scope) live
//! on those nodes; only their shapes are declared here.

use serde::{Deserialize, Serialize};

/// The closed set of actor natures (§3.7). A nature is not a role: it fixes
/// which §3.2.0 trigger source may act as the kind (a `human` through a UI
/// step, a `process` through an automated trigger, an `external` system through
/// an API call). Roles are the adopter's — the framework ships none.
pub const ACTOR_NATURES: [&str; 3] = ["human", "process", "external"];

/// §3.7 — one claim an actor of this kind carries: a named field typed from the
/// §3.2 datatype vocabulary, optionally set-valued (`many`) and optionally drawn
/// from declared reference data (§3.1) — the constitutive value space, such as
/// the role set or the tenant list.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
pub struct Claim {
    pub name: String,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,
    /// A set-valued claim (a role set, a tenant list). On the conformance wire
    /// it is the one list-valued position, confined to the actor block (§6.3.1).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub many: bool,
    /// The ReferenceSet (§3.1) the claim's values are drawn from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub values_from: Option<String>,
}

/// §3.7 — an actor kind: a kind of principal (a customer, staff, an automated
/// process, an external system). Domain-level, because two systems over the
/// domain must agree on who it is and what it carries; the identity provider
/// that authenticates it is How (`authenticated_by`, §4.2).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
pub struct ActorKind {
    pub id: String,
    pub label: String,
    /// human | process | external (see [`ACTOR_NATURES`]).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub nature: String,
    /// The claims an authenticated actor of this kind carries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub claims: Vec<Claim>,
}

/// §3.7 — one conjunct of a scope rule: a row is visible when its `field`
/// equals the actor's `claim` (or is a member of it, when the claim is
/// set-valued). The field is a key of the Projector's fold, so one fold serves
/// every actor and the scope selects at the seam.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
pub struct ScopeKey {
    pub field: String,
    pub claim: String,
}

/// §3.7 — what one actor kind may see of a View: the rows matching every key
/// (all rows when `keys` is empty — a declared, not a defaulted, "everything").
/// An actor kind with no rule on a scoped View sees nothing.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
pub struct ScopeRule {
    pub actor: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keys: Vec<ScopeKey>,
}
