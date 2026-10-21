//! §3.7 `--field` flags for the actor layer of `product domain new`/`edit`.
//!
//! Flattened into [`super::domain_fields_v16::V16Fields`] and merged into the
//! same JSON field map `pf::edit` consumes, so the CLI accepts exactly the
//! fields the `product_domain_*` MCP tools do (generated from the structs).

use clap::Args;
use serde_json::{json, Map, Value};

/// The §3.7 actor flags (actor kind, trigger `acts_as`, step `offered_to`,
/// View scope).
#[derive(Args, Default)]
pub struct ActorFields {
    /// §3.7 actor-kind nature: human | process | external
    #[arg(long = "nature")]
    nature: Option<String>,
    /// §3.7 actor-kind claims, `name[:type][[]][@reference-set]` (comma-separated;
    /// `[]` marks a set-valued claim, e.g. `customer_id:string,tenants:string[]@tenants`)
    #[arg(long = "claims", value_delimiter = ',')]
    claims: Option<Vec<String>>,
    /// §3.7 the actor kind a trigger acts as
    #[arg(long = "acts-as")]
    acts_as: Option<String>,
    /// §3.7 the (human) actor kinds a UI step is offered to (comma-separated)
    #[arg(long = "offered-to", value_delimiter = ',')]
    offered_to: Option<Vec<String>>,
    /// §3.7 the View is readable by every actor (no scope)
    #[arg(long = "public")]
    public: Option<bool>,
    /// §3.7 View scope rules, `actor` (every row) or `actor:field=claim`
    /// (comma-separated; several keys for one actor are a conjunction)
    #[arg(long = "scoped-by", value_delimiter = ',')]
    scoped_by: Option<Vec<String>>,
}

impl ActorFields {
    /// Merge the provided flags into the field map keyed by model field names.
    pub(crate) fn merge_into(&self, m: &mut Map<String, Value>) {
        let mut put = |k: &str, v: Value| { m.insert(k.to_string(), v); };
        if let Some(v) = &self.nature { put("nature", json!(v)); }
        if let Some(v) = &self.claims { put("claims", claims(v)); }
        if let Some(v) = &self.acts_as { put("acts_as", json!(v)); }
        if let Some(v) = &self.offered_to { put("offered_to", json!(v)); }
        if let Some(v) = &self.public { put("public", json!(v)); }
        if let Some(v) = &self.scoped_by { put("scoped_by", scope_rules(v)); }
    }
}

/// `name[:type][[]][@reference-set]` → a §3.7 claim object.
fn claims(items: &[String]) -> Value {
    let arr: Vec<Value> = items.iter().map(|s| {
        let (head, values_from) = s.split_once('@').map_or((s.as_str(), None), |(h, r)| (h, Some(r)));
        let (name, ty) = head.split_once(':').map_or((head, ""), |(n, t)| (n, t));
        let many = ty.ends_with("[]");
        let ty = ty.trim_end_matches("[]");
        let mut c = json!({ "name": name, "many": many });
        if !ty.is_empty() { c["type"] = json!(ty); }
        if let Some(r) = values_from { c["values_from"] = json!(r); }
        c
    }).collect();
    json!(arr)
}

/// `actor` / `actor:field=claim` items → one rule per actor, keys in order.
fn scope_rules(items: &[String]) -> Value {
    let mut rules: Vec<(String, Vec<Value>)> = Vec::new();
    for s in items {
        let (actor, key) = s.split_once(':').map_or((s.as_str(), None), |(a, k)| (a, Some(k)));
        let at = match rules.iter().position(|(a, _)| a == actor) {
            Some(i) => i,
            None => { rules.push((actor.to_string(), Vec::new())); rules.len() - 1 }
        };
        if let Some(k) = key {
            let (field, claim) = k.split_once('=').unwrap_or((k, ""));
            rules[at].1.push(json!({ "field": field, "claim": claim }));
        }
    }
    json!(rules.into_iter().map(|(actor, keys)| json!({ "actor": actor, "keys": keys })).collect::<Vec<_>>())
}
