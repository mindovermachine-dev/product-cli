//! `allowed_signers` — the trust root as a derived file (#65; PRD §0 item 11).
//!
//! One file per namespace, `.decisions/ns/<ns>/allowed_signers` (LP-3.34),
//! one line per key window of that namespace, in OpenSSH's
//! `allowed_signers` format, so `ssh-keygen -Y verify -f` reads it directly:
//!
//! ```text
//! <principal> namespaces="ledger-accept@<ns>",valid-after="<t>"[,valid-before="<t>"] <key-type> <key>
//! ```
//!
//! Options are comma-separated, as OpenSSH's `allowed_signers` grammar
//! requires (spec v1.8 corrects the space-separated form spec v1.7 wrote,
//! which `ssh-keygen` refuses as an invalid key). `valid-after` is the
//! opening binding's time; `valid-before` is the time of the earliest
//! `rotate` or `revoke` that closed its key, when one did — in any
//! namespace, since a close ends the key, not the binding (ruled 2026-10-06). The principal
//! is the bare address (ruling 6: files keep the bare address). The file is
//! never edited by hand: every `ledger identity` verb rewrites it, and
//! `verify` re-derives it and holds the committed bytes identical — the
//! export's discipline, applied to the trust root.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::graph::export::ExportFinding;
use crate::store::Store;

use super::binding::KeyBinding;

/// The derived file's name under each namespace's directory.
pub const FILE: &str = crate::layout::SIGNERS_FILE;

/// The derived file's two comment lines, as the ledger protocol states them (§4.9).
pub const HEADER: &str = "# Derived from the key-binding entries in .decisions/log by `ledger identity`.\n\
# Never edit by hand: `ledger verify` holds this file byte-identical to the log.\n";

/// Where a namespace's derived file lives.
pub fn path(root: &Path, namespace: &str) -> PathBuf {
    crate::layout::signers_path(root, namespace)
}

fn when(at: &DateTime<Utc>) -> String {
    at.format("%Y%m%d%H%M%SZ").to_string()
}

/// The files the log implies, by namespace — every binding the signature
/// gate trusts (an unsigned or wrongly signed binding never enters one,
/// spec v1.8). A namespace that binds nothing has no entry.
pub fn derive(store: &Store) -> BTreeMap<String, String> {
    let landing = crate::landing::Landing::compute(&store.root, None, true).unwrap_or_default();
    derive_each(&crate::signing::check::trusted_bindings(store, &landing), &filed(store))
}

/// Each namespace's file from exactly these bindings: the lines of its
/// bindings, under the header. A namespace none of them opens a key in has
/// no entry. A close is looked up among all of `bindings`, in whichever
/// namespace it was filed, as [`derive_from`] does.
pub fn derive_each(bindings: &[&KeyBinding], filed: &[&KeyBinding]) -> BTreeMap<String, String> {
    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (namespace, line) in lines(bindings, filed) {
        grouped.entry(namespace).or_default().push(line);
    }
    grouped
        .into_iter()
        .map(|(namespace, mut lines)| {
            lines.sort();
            (namespace, format!("{HEADER}{}\n", lines.join("\n")))
        })
        .collect()
}

/// Every binding filed in the store, trusted or not.
pub fn filed(store: &Store) -> Vec<&KeyBinding> {
    store.log.iter().flat_map(|l| l.file.key_bindings.iter()).collect()
}

/// The allowed-signers text for exactly these bindings, every namespace's
/// line in one text — what the signature check hands `ssh-keygen`; a
/// close's target is looked up in `filed`.
pub fn derive_from(bindings: &[&KeyBinding], filed: &[&KeyBinding]) -> Option<String> {
    let mut lines: Vec<String> = lines(bindings, filed).into_iter().map(|(_, line)| line).collect();
    if lines.is_empty() {
        return None;
    }
    lines.sort();
    Some(format!("{HEADER}{}\n", lines.join("\n")))
}

/// One line per opening binding, with the namespace it belongs to.
fn lines(bindings: &[&KeyBinding], filed: &[&KeyBinding]) -> Vec<(String, String)> {
    // A close ends the key in every namespace (ruled 2026-10-06): every
    // line of a closed key ends at its key's earliest close.
    let closes: BTreeMap<String, DateTime<Utc>> = bindings
        .iter()
        .filter_map(|b| super::key_close::closes_among(bindings, filed, b).iter().map(|c| c.at).min().map(|t| (b.id.to_string(), t)))
        .collect();
    bindings
        .iter()
        .filter(|b| b.act.opens())
        .filter_map(|b| {
            let (key_type, key) = (b.key_type.as_deref()?, b.key.as_deref()?);
            let before = closes
                .get(&b.id.to_string())
                .map(|t| format!(",valid-before=\"{}\"", when(t)))
                .unwrap_or_default();
            let line = format!(
                "{} namespaces=\"ledger-accept@{}\",valid-after=\"{}\"{before} {key_type} {key}",
                b.principal,
                b.namespace,
                when(&b.at)
            );
            Some((b.namespace.clone(), line))
        })
        .collect()
}

/// Rewrite each namespace's derived file from the log.
pub fn write(store: &Store) -> Result<(), String> {
    for (namespace, text) in derive(store) {
        let target = path(&store.root, &namespace);
        if let Some(dir) = target.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        product_core::fileops::write_file_atomic(&target, &text).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// The verify stage: each namespace's committed file equals the log's
/// derivation for that namespace.
pub fn check(store: &Store, trusted: &[&KeyBinding]) -> Vec<ExportFinding> {
    let derived = derive_each(trusted, &filed(store));
    let namespaces: BTreeSet<String> = derived.keys().cloned().chain(store.namespaces()).collect();
    let regenerate = "regenerate it with `ledger identity sync` — it is never edited by hand";
    namespaces
        .into_iter()
        .filter_map(|ns| {
            let label = format!("{}/{FILE}", crate::layout::relative_dir(&ns, crate::layout::Kind::Signers).trim_end_matches('/'));
            let committed = std::fs::read_to_string(path(&store.root, &ns)).ok();
            let message = match (derived.get(&ns), committed) {
                (None, None) => return None,
                (Some(want), Some(have)) if *want == have => return None,
                (Some(_), None) => format!("the log binds keys in `{ns}` but no {FILE} is committed there — {regenerate}"),
                (None, Some(_)) => format!("{FILE} is committed in `{ns}` but the log binds no key there — it trusts what nothing filed"),
                (Some(_), Some(_)) => format!("does not match the key bindings of `{ns}` in the log — {regenerate}"),
            };
            Some(ExportFinding { subject: label, message })
        })
        .collect()
}
