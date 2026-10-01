//! `ledger export` — the committed per-namespace N-Triples export.
//!
//! With `--namespace`, writes that namespace; without it, writes every
//! namespace the log speaks, each to its default path, so regenerating all
//! exports after a ledger act is one command. `--out -` prints instead.

use std::path::{Path, PathBuf};

use ledger_core::graph::export;
use ledger_core::store::{self, Store};

use super::{resolve_root, EXIT_OK};

/// What one invocation exports, and where to.
pub struct Flags {
    pub format: String,
    pub namespace: Option<String>,
    pub out: Option<PathBuf>,
}

pub fn run(root: Option<PathBuf>, flags: Flags) -> Result<i32, String> {
    if flags.format != "ntriples" {
        return Err(format!("`{}` is not an export format — expected `ntriples`", flags.format));
    }
    let repo_root = resolve_root(root)?;
    let loaded = store::load(&repo_root);
    let targets: Vec<String> = match &flags.namespace {
        Some(ns) => vec![ns.clone()],
        None => export::namespaces(&loaded).into_iter().collect(),
    };
    if targets.is_empty() {
        return Err("the log speaks no namespace yet — there is nothing to export".to_string());
    }
    if flags.out.is_some() && targets.len() > 1 {
        return Err(format!(
            "--out names one file, but the log speaks {} namespaces — pass --namespace",
            targets.len()
        ));
    }
    for namespace in &targets {
        write_one(&repo_root, &loaded, namespace, flags.out.as_deref())?;
    }
    Ok(EXIT_OK)
}

fn write_one(root: &Path, loaded: &Store, namespace: &str, out: Option<&Path>) -> Result<(), String> {
    let text = export::export(loaded, namespace)
        .ok_or_else(|| format!("the log speaks no namespace `{namespace}` — nothing to export"))?;
    if out == Some(Path::new("-")) {
        print!("{text}");
        return Ok(());
    }
    let path = out.map_or_else(|| export::default_path(root, namespace), Path::to_path_buf);
    export::write(&path, &text)?;
    println!("wrote {} — {} triple(s) of `{namespace}`", path.display(), text.lines().count());
    Ok(())
}
