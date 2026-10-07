//! A landed change-set's header, `parents` included, is never edited (LP-8.25).
//!
//! Ruling 60, the verification report's section 10: a parent appended to a
//! landed change-set's header was keyed as a new entity landing, and the
//! store stayed conformant, while a `note` appended the same way failed
//! `L007`. `parents` is now part of the header entity.

mod common;

use std::path::Path;

use common::{hand, Repo};

const CS: &str = "01K2C4YQJ3F8M0PT5W7NZ9RDXW";

/// The `pass` fixture's store, committed.
fn landed() -> Repo {
    let repo = Repo::human();
    let from = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pass/.decisions");
    for dir in ["sets", "log"] {
        for entry in std::fs::read_dir(from.join(dir)).expect("fixture").flatten() {
            std::fs::copy(entry.path(), repo.path().join(".decisions").join(dir).join(entry.file_name())).expect("copy");
        }
    }
    hand::commit(&repo, "landed");
    repo
}

/// Add `line` to the landed header, commit, and verify.
fn header_gains(line: &str) -> (i32, String) {
    let repo = landed();
    let path = repo.path().join(format!(".decisions/log/{CS}.yml"));
    let text = std::fs::read_to_string(&path).expect("log file");
    let edited = text.replacen("created_by: fixture-human@example\n", &format!("created_by: fixture-human@example\n{line}\n"), 1);
    assert_ne!(text, edited, "the header was edited");
    std::fs::write(&path, edited).expect("write");
    hand::commit(&repo, "edit the landed header");
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

#[test]
fn a_parent_added_to_a_landed_header_fails_l007() {
    let (code, text) = header_gains("parents: [cs:01K2C4YQJ3F8M0PT5W7NZ9RDX0]");
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[L007]") && text.contains("the change-set header landed"), "{text}");
}

/// The control the report ran: a `note` added to a landed header.
#[test]
fn a_note_added_to_a_landed_header_fails_l007() {
    let (code, text) = header_gains("note: added later");
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("[L007]") && text.contains("the change-set header landed"), "{text}");
}
