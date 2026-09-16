//! The delegable half, reached by launching the agent host.
//!
//! `import` and `build` are not implemented here. They belong to the agent
//! host — a separate executable that links no code able to write a closure —
//! so this module only finds it, hands it its arguments, then returns its exit
//! code unchanged. Launching a process is packaging; it is not linking. The
//! boundary the flow rests on is where it was: `spec` is where a principal
//! acts, `specflow` is where a model does, and nothing reachable from here
//! closes a record.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Args;
use product_core::error::{ProductError, Result};

use crate::{exit, render::Report};

/// The agent host's executable, without a platform suffix.
const HOST: &str = "specflow";

/// Where to find the host, when it is not beside this binary.
///
/// Mirrors `SPEC_BIN` on the other side of the seam, which is how the host
/// already finds *this* binary in a test repo.
const HOST_ENV: &str = "SPECFLOW_BIN";

/// Verbs this launcher may never hand the host.
///
/// The host cannot perform them — nothing it links offers either — so this
/// list is belt to that braces, in the shape `SpecCli.ForbiddenVerbs` and
/// `tools::WITHHELD` already take. A launcher that forwards whatever it is
/// given is a launcher that inherits the next caller's mistake.
const NEVER_DELEGATED: &[&str] = &["accept", "reject", "close", "policy"];

#[derive(Args)]
pub struct ImportArgs {
    /// The codebase to scan (default: the repo root).
    #[arg(long, value_name = "PATH")]
    pub source: Option<PathBuf>,
}

#[derive(Args)]
pub struct BuildArgs {
    /// The slice to build.
    #[arg(long, value_name = "ID")]
    pub slice: String,
    /// The ratified act the slice is built against.
    #[arg(long = "act", value_name = "ACT-REF")]
    pub act_ref: String,
    /// Whose identity opens the record. Defaults to the host's own.
    #[arg(long, value_name = "IDENTITY")]
    pub by: Option<String>,
    /// Extra instructions for the slice builder.
    #[arg(long, value_name = "TEXT")]
    pub instructions: Option<String>,
}

/// Re-scan a codebase into `.spec/inventory.json`.
pub fn import(root: &Path, args: &ImportArgs) -> Result<Report> {
    launch(root, "import", &import_forwarded(args))
}

/// What `import` hands the host.
fn import_forwarded(args: &ImportArgs) -> Vec<OsString> {
    let mut forwarded: Vec<OsString> = Vec::new();
    if let Some(source) = &args.source {
        forwarded.push("--source".into());
        forwarded.push(source.clone().into_os_string());
    }
    forwarded
}

/// Build a slice, opening its act-time record. Exits 3: the closure is pending.
///
/// Named `build` rather than `implement` because `spec implement` is already
/// the record-opening primitive the host calls back into. The chain reads
/// `spec build` → `specflow implement` → `spec implement`, and each hop names
/// something different from the one before it.
pub fn build(root: &Path, args: &BuildArgs) -> Result<Report> {
    launch(root, "implement", &build_forwarded(args))
}

/// What `build` hands the host.
fn build_forwarded(args: &BuildArgs) -> Vec<OsString> {
    let mut forwarded: Vec<OsString> = vec![
        "--slice".into(),
        args.slice.clone().into(),
        "--act".into(),
        args.act_ref.clone().into(),
    ];
    push_pair(&mut forwarded, "--by", args.by.as_deref());
    push_pair(&mut forwarded, "--instructions", args.instructions.as_deref());
    forwarded
}

/// Append a `--key value` pair, when there is a value to append.
///
/// The host parses strictly in pairs, so a flag without a value is not a flag
/// it ignores — it is a parse failure that prints usage.
fn push_pair(forwarded: &mut Vec<OsString>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        forwarded.push(key.into());
        forwarded.push(value.into());
    }
}

/// Run the host, returning what it returned.
///
/// The child inherits this process's streams, so its output is the output: no
/// buffering, no re-rendering, and an exit code that reaches CI unchanged —
/// 3 stays 3, which is the whole point of the code existing.
fn launch(root: &Path, verb: &str, forwarded: &[OsString]) -> Result<Report> {
    guard(verb)?;
    let host = locate()?;
    let me = std::env::current_exe().map_err(|e| {
        ProductError::ConfigError(format!("could not resolve this binary's own path: {e}"))
    })?;

    let status = Command::new(&host)
        .arg(verb)
        .arg("--root")
        .arg(root)
        // Pin the host to *this* binary rather than to whatever `spec` PATH
        // resolves. A launcher that hands over an ambiguous chain has not
        // removed the ambiguity, only moved it one process along.
        .arg("--spec")
        .arg(&me)
        .args(forwarded)
        .status()
        .map_err(|e| unavailable(&host, &e))?;

    Ok(Report::silent(status.code().unwrap_or(exit::COULD_NOT_RUN)))
}

/// Refuse to hand the host a verb that names a principal.
fn guard(verb: &str) -> Result<()> {
    if NEVER_DELEGATED.contains(&verb) {
        return Err(ProductError::ConfigError(format!(
            "`{verb}` names a principal; the agent host cannot be one. \
             Run `spec {verb}` yourself."
        )));
    }
    Ok(())
}

/// Where the host is: an explicit override, then beside this binary, then PATH.
///
/// The sibling lookup is what makes one installer enough — `cargo install` and
/// the release installers both land every binary in the same directory.
fn locate() -> Result<PathBuf> {
    if let Some(supplied) = std::env::var_os(HOST_ENV) {
        let path = PathBuf::from(supplied);
        if !path.is_file() {
            return Err(ProductError::NotFound(format!(
                "{HOST_ENV} points at {}, which is not a file",
                path.display()
            )));
        }
        return Ok(path);
    }
    if let Some(sibling) = sibling() {
        if sibling.is_file() {
            return Ok(sibling);
        }
    }
    Ok(PathBuf::from(executable_name()))
}

/// The host as it would sit next to this binary.
fn sibling() -> Option<PathBuf> {
    let me = std::env::current_exe().ok()?;
    Some(me.parent()?.join(executable_name()))
}

/// The host's file name on this platform.
fn executable_name() -> String {
    format!("{HOST}{}", std::env::consts::EXE_SUFFIX)
}

/// What to say when the host will not start.
///
/// An absent host is not a broken install of `spec`; it is the other half of
/// the flow missing, which is a different thing to go and fix.
fn unavailable(host: &Path, error: &std::io::Error) -> ProductError {
    if error.kind() == std::io::ErrorKind::NotFound {
        return ProductError::NotFound(format!(
            "the agent host `{}` — the delegable half of the flow.\n  \
             Build it with `dotnet publish spec-flow/src/SpecFlow.Cli -o <dir>`, put it \
             beside `spec`, or point {HOST_ENV} at it.",
            host.display()
        ));
    }
    ProductError::ConfigError(format!("could not run `{}`: {error}", host.display()))
}

#[path = "host_tests.rs"]
#[cfg(test)]
mod tests;
