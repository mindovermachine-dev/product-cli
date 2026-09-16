//! One command name over two executables: what the launcher hands the host.
//!
//! The agent host is a separate process on purpose, so these tests stand a
//! stub in its place and read the argv it was given. What is under test is the
//! seam — verb translation, the pinned `--spec`, and an exit code that reaches
//! a caller unchanged — not the host, which has its own tests in .NET.

use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;

fn spec(root: &Path) -> Command {
    let mut cmd = Command::cargo_bin("spec").expect("the spec binary builds");
    cmd.arg("--root").arg(root);
    // A stub must never be shadowed by a real host sitting beside the binary.
    cmd.env_remove("SPECFLOW_BIN");
    cmd
}

fn code(output: &std::process::Output) -> i32 {
    output.status.code().unwrap_or(-1)
}

/// A stand-in host that records its arguments, then exits as the real one does.
#[cfg(unix)]
fn stub_host(dir: &Path) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let log = dir.join("argv");
    let path = dir.join("specflow");
    std::fs::write(
        &path,
        format!("#!/bin/sh\nprintf '%s\\n' \"$@\" > {}\nexit 3\n", log.display()),
    )
    .expect("stub written");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
        .expect("stub is executable");
    path
}

#[cfg(unix)]
fn argv_of(dir: &Path) -> Vec<String> {
    std::fs::read_to_string(dir.join("argv"))
        .expect("the stub ran")
        .lines()
        .map(str::to_string)
        .collect()
}

#[cfg(unix)]
fn value_after(argv: &[String], flag: &str) -> Option<String> {
    let at = argv.iter().position(|a| a == flag)?;
    argv.get(at + 1).cloned()
}

#[test]
fn the_delegable_verbs_are_on_the_one_surface() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = spec(dir.path()).arg("--help").output().expect("help runs");
    let help = String::from_utf8_lossy(&out.stdout);
    for verb in ["import", "build", "close", "check"] {
        assert!(help.contains(verb), "`{verb}` should appear on the one surface:\n{help}");
    }
}

#[cfg(unix)]
#[test]
fn build_reaches_the_host_as_implement() {
    let dir = tempfile::tempdir().expect("tempdir");
    let host = stub_host(dir.path());
    let out = spec(dir.path())
        .env("SPECFLOW_BIN", &host)
        .args(["build", "--slice", "checkout-totals", "--act", "act/settle-basket"])
        .output()
        .expect("build runs");

    let argv = argv_of(dir.path());
    assert_eq!(argv.first().map(String::as_str), Some("implement"));
    assert_eq!(value_after(&argv, "--slice").as_deref(), Some("checkout-totals"));
    assert_eq!(value_after(&argv, "--act").as_deref(), Some("act/settle-basket"));
    assert_eq!(code(&out), 3, "closure pending must reach the caller unchanged");
}

#[cfg(unix)]
#[test]
fn the_host_is_pinned_to_the_binary_that_launched_it() {
    let dir = tempfile::tempdir().expect("tempdir");
    let host = stub_host(dir.path());
    spec(dir.path())
        .env("SPECFLOW_BIN", &host)
        .args(["build", "--slice", "s", "--act", "act/a"])
        .output()
        .expect("build runs");

    let pinned = value_after(&argv_of(dir.path()), "--spec").expect("--spec is forwarded");
    let expected = assert_cmd::cargo::cargo_bin("spec");
    assert_eq!(Path::new(&pinned), expected, "the chain must not resolve `spec` from PATH");
}

#[cfg(unix)]
#[test]
fn an_absent_option_reaches_the_host_as_absence() {
    let dir = tempfile::tempdir().expect("tempdir");
    let host = stub_host(dir.path());
    spec(dir.path())
        .env("SPECFLOW_BIN", &host)
        .args(["build", "--slice", "s", "--act", "act/a"])
        .output()
        .expect("build runs");

    let argv = argv_of(dir.path());
    assert!(!argv.contains(&"--by".to_string()), "the host's own default must stand: {argv:?}");
}

#[cfg(unix)]
#[test]
fn import_forwards_the_source_it_was_given() {
    let dir = tempfile::tempdir().expect("tempdir");
    let host = stub_host(dir.path());
    spec(dir.path())
        .env("SPECFLOW_BIN", &host)
        .args(["import", "--source", "src/Api"])
        .output()
        .expect("import runs");

    let argv = argv_of(dir.path());
    assert_eq!(argv.first().map(String::as_str), Some("import"));
    assert_eq!(value_after(&argv, "--source").as_deref(), Some("src/Api"));
}

#[test]
fn an_absent_host_could_not_run() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = spec(dir.path())
        .env("SPECFLOW_BIN", dir.path().join("nowhere"))
        .args(["build", "--slice", "s", "--act", "act/a"])
        .output()
        .expect("build runs");

    assert_eq!(code(&out), 2, "a missing host is `could not run`, not a finding");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("SPECFLOW_BIN"),
        "the refusal should name how to point at the host"
    );
}
