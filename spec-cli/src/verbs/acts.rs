//! Reading back the acts a principal ratified.
//!
//! A read, and the only way anything outside this binary learns what an act
//! settles. The agent host asks here rather than parsing the store itself:
//! one implementation of what the store means is what keeps an MCP client, a
//! judge and a CI run from being told different things about the same repo.

use std::path::Path;

use clap::Args;
use product_core::error::{ProductError, Result};
use serde_json::json;
use spec_core::ratify;

use crate::exit;
use crate::render::Report;

#[derive(Args)]
pub struct ActsArgs {
    /// One act's address. Without it, every ratified act.
    #[arg(long, value_name = "ACT-REF")]
    pub id: Option<String>,
    #[arg(long)]
    pub json: bool,
}

/// List ratified acts, or show one.
pub fn acts(root: &Path, args: &ActsArgs) -> Result<Report> {
    let all = ratify::load_acts(root)?;
    let selected: Vec<_> = match &args.id {
        Some(id) => all.into_iter().filter(|a| &a.id == id).collect(),
        None => all,
    };

    if let Some(id) = &args.id {
        if selected.is_empty() {
            return Err(ProductError::NotFound(format!("no ratified act `{id}`")));
        }
    }

    let text = selected
        .iter()
        .map(|a| format!("{}  {}\n  settles: {}", a.id, a.name, a.settles))
        .collect::<Vec<_>>()
        .join("\n");

    let body = args.json.then(|| {
        json!(selected
            .iter()
            .map(|a| json!({
                "act": a.id,
                "name": a.name,
                "settles": a.settles,
                "realised_at": a.realised_at,
                "ratified_by": a.ratified_by.as_str(),
            }))
            .collect::<Vec<_>>())
    });

    Ok(Report::text(exit::CONFORMANT, text).with_json(body))
}
