//! What a run said it was doing, before it did it.
//!
//! A declaration is a closed object even where the acceptance predicate is
//! open: whether it is *complete* is mechanically checkable, whatever the
//! quality of the work that follows. Prediction before operation, applied to
//! the worker.
//!
//! Made before the act or not at all. A declaration written afterwards
//! describes what happened, which is a summary and not a prediction — and a
//! summary cannot be contradicted by the run it summarises.
//!
//! Only `decision` is the worker's. Ground, tolerance and assurance each have
//! their own owner, and a worker filling any of them in is that owner's
//! decision escaping into the thing it was supposed to constrain.

use serde::{Deserialize, Serialize};

/// What a worker declared before acting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Declaration {
    /// The decision being resolved, in the worker's own words.
    pub decision: String,
    /// The ground the act rests on: the addresses whose value can change how it
    /// resolves.
    ///
    /// **The ground author's, never the worker's.** A worker asked to name its
    /// own ground answers from introspection, and a worker's account of itself
    /// is not the arrangement's record. It is also the reading that quietly
    /// fails: the worker answers in whatever vocabulary it likes, the observed
    /// reads are in another, and the comparison finds a difference that was
    /// only ever a difference in naming.
    #[serde(default)]
    pub ground: Vec<String>,
    /// The declared bound on outcome-relevant variation.
    ///
    /// The arrangement's, not the worker's. A worker that set its own tolerance
    /// would be deciding how wrong it is allowed to be.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<String>,
    /// The assurance level the act is being performed at.
    ///
    /// Also the arrangement's. It is derived from the worst credible outcome by
    /// whoever bears that outcome; a worker declaring it would be pricing a
    /// consequence it does not carry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assurance: Option<String>,
}

impl Declaration {
    /// A declaration of what is being resolved, over the ground it rests on.
    ///
    /// The two arguments come from two different owners: the decision is the
    /// worker's, the ground is the ground author's. They meet here and nowhere
    /// earlier.
    pub fn new(decision: impl Into<String>, ground: Vec<String>) -> Self {
        Self { decision: decision.into(), ground, tolerance: None, assurance: None }
    }

    /// Add what the arrangement — not the worker — fixed.
    pub fn bounded_by(
        mut self,
        tolerance: impl Into<String>,
        assurance: impl Into<String>,
    ) -> Self {
        self.tolerance = Some(tolerance.into());
        self.assurance = Some(assurance.into());
        self
    }

    /// The fields a declaration must carry to be complete.
    pub fn missing(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.decision.trim().is_empty() {
            missing.push("decision");
        }
        if self.ground.is_empty() {
            missing.push("ground");
        }
        if self.tolerance.as_deref().map(str::trim).unwrap_or_default().is_empty() {
            missing.push("tolerance");
        }
        if self.assurance.as_deref().map(str::trim).unwrap_or_default().is_empty() {
            missing.push("assurance");
        }
        missing
    }
}

/// A claim the act made, and the declared ground it rests on.
///
/// A claim with no ground is an escape candidate: the worker resolved
/// something on a basis it did not declare, or on none.
///
/// **An attribution is the worker's own account of itself** and can be
/// confabulated. It is worth checking because it is cheap and because its
/// falsifier is sharp — attribution passing on acts where behaviour shows
/// dependence on undeclared ground means the attribution is decorative.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attribution {
    /// The claim, as the act stated it.
    pub claim: String,
    /// The declared ground element it rests on, where one was named.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ground: Option<String>,
}

impl Attribution {
    /// A claim resting on a named ground element.
    pub fn to(claim: impl Into<String>, ground: impl Into<String>) -> Self {
        Self { claim: claim.into(), ground: Some(ground.into()) }
    }

    /// A claim resting on nothing the worker named.
    pub fn unattributed(claim: impl Into<String>) -> Self {
        Self { claim: claim.into(), ground: None }
    }
}

#[path = "declaration_tests.rs"]
#[cfg(test)]
mod tests;
