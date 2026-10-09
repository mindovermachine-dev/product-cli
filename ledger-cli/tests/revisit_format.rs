//! A file declares the format `revisit_if` needs (protocol §3.2 and §7.2, #81).
//!
//! Three of this repository's change-sets carried `revisit_if` under
//! `format: 1`: `revise` wrote them before `Author::append` stamped
//! `format::needed_for` (#67). The principal ruled (2026-10-05) that a
//! landed file's `format:` may be corrected — raised only, only to the
//! lowest format its content needs, nothing else changed. These tests hold
//! the writer, the correction and the repository's own store to that.

mod common;

use std::path::{Path, PathBuf};

use common::{hand, Repo};
use ledger_core::format;
use ledger_core::verify::{self, view::View, Options};

/// The change-sets the correction raised to `format: 4`.
const CORRECTED: [&str; 3] = [
    "01KZX70EMPA47TBR0PFKX4M32Z",
    "01KZX70EQGQCB1B190TS9FZ1A2",
    "01KZX70ET1GMR2012XKEP5EWDW",
];

/// Listed in #81 beside them, but `revisit_if` appears there only inside its
/// `note` and `statement` text: it needs format 1 and stays there.
const NOT_REOPENING: &str = "01KZX70S86QGXVCA5GW5WSY6XA";

const REOPEN: &str = "claim:DDD-gates-01@sha256:6f500ee8";

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn declared(path: &Path) -> String {
    let text = std::fs::read_to_string(path).expect("read");
    text.lines().next().unwrap_or_default().to_string()
}

fn verify(repo: &Repo) -> (i32, String) {
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

/// A decision revised onto a reopen edge, the revision accepted. Returns the
/// revision's log file.
fn revised_onto_a_reopen_edge(repo: &Repo) -> PathBuf {
    repo.declare();
    let id = repo.add("Money is decimal.", &[]);
    let before = repo.log_files();
    repo.ok(&["revise", &id, "--statement", "Money is decimal.", "--revisit-if", REOPEN, "--note", "edge re-typed"]);
    let file = repo.log_files().into_iter().find(|f| !before.contains(f)).expect("the revision's file");
    repo.ok_tty(&["accept", &id]);
    repo.log_dir("fixture.ledger").join(file)
}

/// Before #67 every next-version verb wrote `format: 1`, an inherited edge
/// included; `Author::append` now stamps `format::needed_for` for each.
#[test]
fn every_version_carrying_a_reopen_edge_is_written_at_format_4() {
    let repo = Repo::human();
    let file = revised_onto_a_reopen_edge(&repo);
    assert_eq!(declared(&file), "format: 4", "stated by `revise`");
    let store = ledger_core::store::load(repo.path());
    let id = store.log.iter().flat_map(|l| l.file.versions.iter()).map(|v| v.decision.to_string()).next().expect("id");
    let before = repo.log_files();
    repo.ok(&["revise", &id, "--statement", "Money is decimal, always."]);
    let inherited = repo.log_files().into_iter().find(|f| !before.contains(f)).expect("file");
    assert_eq!(declared(&repo.log_dir("fixture.ledger").join(inherited)), "format: 4", "inherited");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
}

/// The case #81 corrects, replayed: a reopen edge landed under `format: 1`,
/// then the declaration alone raised. The landed-entity rule (#89) reads
/// `format:` as no entity, so the correction passes and the acceptance
/// stands; an edit to an entity of the same file still fails `L007`.
#[test]
fn a_landed_declaration_may_be_raised_but_an_entity_beside_it_may_not_change() {
    let repo = Repo::human();
    let file = revised_onto_a_reopen_edge(&repo);
    let as_written = std::fs::read_to_string(&file).expect("read");
    std::fs::write(&file, as_written.replacen("format: 4", "format: 1", 1)).expect("write");
    hand::commit(&repo, "landed as the pre-#67 writer left it");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("carries `revisit_if`, a format 4 field — declare `format: 4`"), "{text}");
    assert!(!text.contains("[L007]"), "{text}");

    std::fs::write(&file, &as_written).expect("write");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "the correction, uncommitted: {text}");
    hand::commit(&repo, "format declaration corrected");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "the correction, landed: {text}");
    assert!(!text.contains("awaiting acceptance"), "the acceptance still signs the version: {text}");

    std::fs::write(&file, as_written.replacen("note: edge re-typed", "note: edge re-typed, quietly", 1)).expect("write");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[L007]") && text.contains("the change-set header") && text.contains("has changed"), "{text}");
}

/// Ruling 58: the correction is a raise with **nothing else** changed. The
/// same raise in one step with an edit beside it is not a correction, and
/// fails `L007` on the declaration as well as on the entity.
#[test]
fn a_raise_beside_another_edit_is_not_a_correction() {
    let repo = Repo::human();
    let file = revised_onto_a_reopen_edge(&repo);
    let as_written = std::fs::read_to_string(&file).expect("read");
    std::fs::write(&file, as_written.replacen("format: 4", "format: 1", 1)).expect("write");
    hand::commit(&repo, "landed as the pre-#67 writer left it");
    std::fs::write(&file, as_written.replacen("note: edge re-typed", "note: edge re-typed, quietly", 1)).expect("write");
    hand::commit(&repo, "raised, and the note edited");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("its format declaration was 1 at") && text.contains("and is 4 at"), "{text}");
}

#[test]
fn the_corrected_files_declare_exactly_what_they_need() {
    // This repository's store: `.decisions/ns/<ns>/log` (LP-3.34), or the
    // flat `.decisions/log` until it is re-laid out.
    let log = |stem: &str| {
        let root = workspace();
        ledger_core::layout::namespaces(&root)
            .into_iter()
            .map(|ns| ledger_core::layout::log_dir(&root, &ns).join(format!("{stem}.yml")))
            .chain(std::iter::once(root.join(".decisions/log").join(format!("{stem}.yml"))))
            .find(|p| p.is_file())
            .unwrap_or_else(|| panic!("{stem} is not in this repository's store"))
    };
    for stem in CORRECTED {
        let path = log(stem);
        let cs: ledger_core::changeset::ChangeSet =
            serde_yaml::from_str(&std::fs::read_to_string(&path).expect("read")).expect("parse");
        assert_eq!(cs.format, format::REVISIT_FORMAT, "{stem}");
        assert_eq!(format::needed_for(&cs), cs.format, "{stem}: raised past what it needs");
    }
    let path = log(NOT_REOPENING);
    let cs: ledger_core::changeset::ChangeSet =
        serde_yaml::from_str(&std::fs::read_to_string(&path).expect("read")).expect("parse");
    assert!(cs.versions.iter().all(|v| v.revisit_if.is_empty()), "{NOT_REOPENING} carries no reopen edge");
    assert_eq!(cs.format, format::CURRENT_FORMAT, "{NOT_REOPENING} needs format 1 and keeps it");
}

/// The four versions #81 names are each still signed by an unrevoked
/// acceptance of their own hash, and the gate finds nothing about them.
#[test]
fn the_four_acceptances_still_verify() {
    let staged = common::workspace_copy();
    let store = ledger_core::store::load(staged.path());
    let view = View::build(&store);
    let report = verify::verify(&store, &Options::offline(chrono::Utc::now().date_naive()));
    let mut seen = 0;
    for stem in CORRECTED.iter().chain([&NOT_REOPENING]) {
        let logged = store.log.iter().find(|l| l.file.id.ulid() == *stem).expect("in the store");
        for v in &logged.file.versions {
            seen += 1;
            let decision = v.decision.to_string();
            let signed = view.acceptances.iter().any(|a| {
                a.acceptance.version == v.hash && !view.is_revoked(a.acceptance) && view.signs_latest(a.acceptance)
            });
            assert!(signed, "{stem}: no unrevoked acceptance signs {}", v.hash);
            assert!(!report.awaiting_acceptance.contains(&decision), "{decision}");
            let about: Vec<_> = report.findings.iter().filter(|f| f.subject == decision || f.subject.contains(*stem)).collect();
            assert!(about.is_empty(), "{about:#?}");
        }
    }
    assert_eq!(seen, 4);
}
