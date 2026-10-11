//! Only `based_on` crosses a namespace (LP-3.31, ruling 43).
//!
//! A `supersedes` edge never names a decision of another namespace. In a
//! store that exists today the rule is judged on live claims only: a
//! decision's latest version (ruling 75). Earlier versions are history, so a
//! next version that drops the edge repairs the store — the same withdrawal
//! LP-8.21 records for a losing claimant under `G005`. A `SCHEMA` fault, no
//! class added; a target no change-set filed stays `G001`'s, whichever
//! namespace it names.

use crate::finding::Finding;
use crate::verify::view::View;

/// Every decision whose latest version supersedes a decision of another
/// namespace.
pub fn findings(view: &View) -> Vec<Finding> {
    view.latest_versions()
        .filter_map(|v| v.parsed.as_ref())
        .filter_map(|p| {
            let old = p.supersedes.as_ref()?;
            (old.namespace() != p.decision.namespace()).then(|| {
                Finding::schema(
                    &p.decision.to_string(),
                    format!(
                        "its latest version {} supersedes `{old}`, a decision of `{}` — only `based_on` crosses a namespace; `supersedes` never does (LP-3.31, ruling 43), judged on the live claim alone (ruling 75)",
                        p.stored_hash.short(),
                        old.namespace()
                    ),
                )
            })
        })
        .collect()
}

#[path = "crossing_tests.rs"]
#[cfg(test)]
mod tests;
