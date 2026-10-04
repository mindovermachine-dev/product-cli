//! `allowed_signers` — the trust root as a derived file (#65; PRD §0 item 11).
//!
//! One line per key window, in OpenSSH's `allowed_signers` format, so
//! `ssh-keygen -Y verify -f .decisions/allowed_signers` reads it directly:
//!
//! ```text
//! <principal> namespaces="ledger-accept@<ns>",valid-after="<t>"[,valid-before="<t>"] <key-type> <key>
//! ```
//!
//! Options are comma-separated, as OpenSSH's `allowed_signers` grammar
//! requires (spec v1.8 corrects the space-separated form spec v1.7 wrote,
//! which `ssh-keygen` refuses as an invalid key). `valid-after` is the
//! opening binding's time; `valid-before` is the time
//! of the `rotate` or `revoke` that closed it, when one did. The principal
//! is the bare address (ruling 6: files keep the bare address). The file is
//! never edited by hand: every `ledger identity` verb rewrites it, and
//! `verify` re-derives it and holds the committed bytes identical — the
//! export's discipline, applied to the trust root.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::graph::export::ExportFinding;
use crate::store::Store;
use crate::STORE_DIR;

use super::binding::KeyBinding;

/// The derived file's name under `.decisions/`.
pub const FILE: &str = "allowed_signers";

const HEADER: &str = "# Derived from the key-binding entries in .decisions/log by `ledger identity`.\n\
# Never edit by hand: `ledger verify` holds this file byte-identical to the log.\n";

/// Where the derived file lives.
pub fn path(root: &Path) -> PathBuf {
    root.join(STORE_DIR).join(FILE)
}

fn when(at: &DateTime<Utc>) -> String {
    at.format("%Y%m%d%H%M%SZ").to_string()
}

/// The file the log implies — every binding the signature gate trusts
/// (an unsigned or wrongly signed binding never enters it, spec v1.8);
/// `None` when nothing binds a key.
pub fn derive(store: &Store) -> Option<String> {
    let landing = crate::landing::Landing::compute(&store.root, None).unwrap_or_default();
    derive_from(&crate::signing::check::trusted_bindings(store, &landing))
}

/// The allowed-signers text for exactly these bindings.
pub fn derive_from(bindings: &[&KeyBinding]) -> Option<String> {
    let closes: BTreeMap<String, DateTime<Utc>> = bindings
        .iter()
        .filter_map(|b| b.closes.as_ref().map(|c| (c.to_string(), b.at)))
        .collect();
    let mut lines: Vec<String> = bindings
        .iter()
        .filter(|b| b.act.opens())
        .filter_map(|b| {
            let (key_type, key) = (b.key_type.as_deref()?, b.key.as_deref()?);
            let before = closes
                .get(&b.id.to_string())
                .map(|t| format!(",valid-before=\"{}\"", when(t)))
                .unwrap_or_default();
            Some(format!(
                "{} namespaces=\"ledger-accept@{}\",valid-after=\"{}\"{before} {key_type} {key}",
                b.principal,
                b.namespace,
                when(&b.at)
            ))
        })
        .collect();
    if lines.is_empty() {
        return None;
    }
    lines.sort();
    Some(format!("{HEADER}{}\n", lines.join("\n")))
}

/// Rewrite the derived file from the log (or remove it when nothing binds).
pub fn write(store: &Store) -> Result<(), String> {
    let target = path(&store.root);
    match derive(store) {
        Some(text) => product_core::fileops::write_file_atomic(&target, &text).map_err(|e| e.to_string()),
        None => Ok(()),
    }
}

/// The verify stage: the committed file equals the log's derivation.
pub fn check(store: &Store, trusted: &[&KeyBinding]) -> Vec<ExportFinding> {
    let label = format!("{STORE_DIR}/{FILE}");
    let committed = std::fs::read_to_string(path(&store.root)).ok();
    let regenerate = "regenerate it with `ledger identity sync` — it is never edited by hand";
    let message = match (derive_from(trusted), committed) {
        (None, None) => return Vec::new(),
        (Some(want), Some(have)) if want == have => return Vec::new(),
        (Some(_), None) => format!("the log binds keys but no {FILE} is committed — {regenerate}"),
        (None, Some(_)) => format!("{FILE} is committed but the log binds no key — it trusts what nothing filed"),
        (Some(_), Some(_)) => format!("does not match the key bindings in the log — {regenerate}"),
    };
    vec![ExportFinding { subject: label, message }]
}
