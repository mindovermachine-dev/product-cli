//! The committed export: `ledger export --format ntriples` writes it, and
//! `verify --export` holds it byte-identical to the log — a hand edit, a
//! stale file, or no file at all fails the gate with exit 1.

use std::path::{Path, PathBuf};
use std::process::Output;

use assert_cmd::Command;

const TODAY: &str = "2026-08-10";
const EXPORT: &str = "docs/decisions/fixture.ledger.nt";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn ledger(root: &Path, args: &[&str]) -> Output {
    let mut cmd = Command::cargo_bin("ledger").expect("binary");
    cmd.arg("--root").arg(root).args(args);
    cmd.output().expect("run")
}

fn verify_export(root: &Path) -> Output {
    ledger(root, &["verify", "--export", "--no-blame", "--today", TODAY])
}

fn text(out: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))
}

/// A scratch copy of the conformant fixture, so tests can write exports.
fn scratch() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let from = fixtures().join("pass/.decisions");
    let to = dir.path().join(".decisions");
    for sub in ["sets", "log"] {
        std::fs::create_dir_all(to.join(sub)).expect("mkdir");
        for entry in std::fs::read_dir(from.join(sub)).expect("read").flatten() {
            std::fs::copy(entry.path(), to.join(sub).join(entry.file_name())).expect("copy");
        }
    }
    dir
}

fn namespace_of_fixture(dir: &Path) -> String {
    let out = ledger(dir, &["export", "--format", "ntriples", "--out", "-"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let start = stdout.find("<urn:dec:").expect("a decision IRI") + "<urn:dec:".len();
    stdout[start..].split('/').next().expect("namespace").to_string()
}

#[test]
fn an_export_written_then_verified_passes_and_says_so() {
    let dir = scratch();
    assert_eq!(namespace_of_fixture(dir.path()), "fixture.ledger", "fixture namespace moved");
    let out = ledger(dir.path(), &["export", "--format", "ntriples"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    assert!(dir.path().join(EXPORT).is_file());
    let out = verify_export(dir.path());
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    assert!(text(&out).contains("every committed export matches the log byte for byte"));
}

#[test]
fn the_export_is_byte_identical_across_runs() {
    let dir = scratch();
    let first = ledger(dir.path(), &["export", "--format", "ntriples", "--out", "-"]);
    let second = ledger(dir.path(), &["export", "--format", "ntriples", "--out", "-"]);
    assert!(!first.stdout.is_empty());
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn a_hand_edited_export_fails_verify_with_exit_one() {
    let dir = scratch();
    ledger(dir.path(), &["export", "--format", "ntriples"]);
    let path = dir.path().join(EXPORT);
    let mut body = std::fs::read_to_string(&path).expect("read");
    body.push_str("<urn:x> <urn:y> \"forged\" .\n");
    std::fs::write(&path, body).expect("write");
    let out = verify_export(dir.path());
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    let report = text(&out);
    assert!(report.contains("export stage — 1 finding(s):"), "{report}");
    assert!(report.contains("[EXPORT] docs/decisions/fixture.ledger.nt"), "{report}");
}

#[test]
fn verify_export_with_nothing_committed_fails_rather_than_passing() {
    let dir = scratch();
    let out = verify_export(dir.path());
    assert_eq!(out.status.code(), Some(1), "{}", text(&out));
    assert!(text(&out).contains("no committed export found"));
}

#[test]
fn plain_verify_does_not_run_the_export_stage() {
    let dir = scratch();
    let out = ledger(dir.path(), &["verify", "--no-blame", "--today", TODAY, "--json"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(json.get("export").is_none(), "an unrun stage is absent, never an empty pass");
    let out = ledger(dir.path(), &["verify", "--export", "--no-blame", "--today", TODAY, "--json"]);
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(json["export"].as_array().map(Vec::len), Some(1));
}

#[test]
fn an_unknown_format_or_namespace_exits_two() {
    let dir = scratch();
    let out = ledger(dir.path(), &["export", "--format", "turtle"]);
    assert_eq!(out.status.code(), Some(2), "{}", text(&out));
    assert!(text(&out).contains("expected `ntriples`"));
    let out = ledger(dir.path(), &["export", "--format", "ntriples", "--namespace", "hafeok.none"]);
    assert_eq!(out.status.code(), Some(2), "{}", text(&out));
    assert!(text(&out).contains("speaks no namespace `hafeok.none`"));
}
