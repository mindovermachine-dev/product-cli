//! A landed `format:` declaration is compared across history (LP-3.16, ruling 58).
//!
//! The verification report's section 6: a landed declaration was lowered,
//! then raised past what the content needs, and both edits passed, because
//! `format` is no entity and history never compared it. The one change a
//! declaration may undergo is the #81 correction: a raise to exactly the
//! lowest format the content needs, with nothing else changed. That case
//! stays green in `revisit_format.rs`, and on this repository's own store.

mod common;

use std::path::{Path, PathBuf};

use common::{hand, Repo};

const CS: &str = "01K2C4YQJ3F8M0PT5W7NZ9RDXW";

/// The `pass` fixture's store (its change-set needs format 1), landed
/// declaring `format: N`. Returns the log file.
fn landed_at(repo: &Repo, n: u32) -> PathBuf {
    common::copy_fixture_into("pass", repo.path());
    let file = repo.log_dir("fixture.ledger").join(format!("{CS}.yml"));
    redeclare(&file, 1, n);
    hand::commit(repo, &format!("landed at format {n}"));
    file
}

fn redeclare(file: &Path, from: u32, to: u32) {
    let text = std::fs::read_to_string(file).expect("read");
    let edited = text.replacen(&format!("format: {from}\n"), &format!("format: {to}\n"), 1);
    assert!(from == to || edited != text, "format: {from} declared");
    std::fs::write(file, edited).expect("write");
}

fn verify(repo: &Repo) -> (i32, String) {
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

fn names_the_declaration(text: &str, was: u32, is: u32) -> bool {
    text.contains("[L007]") && text.contains(&format!("its format declaration was {was} at")) && text.contains(&format!("and is {is} at"))
}

#[test]
fn a_landed_declaration_that_still_meets_the_need_is_not_lowered() {
    let repo = Repo::human();
    let file = landed_at(&repo, 3);
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "format 3 is a valid declaration on its own: {text}");
    redeclare(&file, 3, 1);
    hand::commit(&repo, "lowered to 1");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(names_the_declaration(&text, 3, 1), "{text}");
}

#[test]
fn a_landed_declaration_is_not_raised_past_what_the_content_needs() {
    let repo = Repo::human();
    let file = landed_at(&repo, 1);
    redeclare(&file, 1, 5);
    hand::commit(&repo, "raised to 5");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(names_the_declaration(&text, 1, 5), "{text}");
}

/// Judged in the working tree too, before anything is committed.
#[test]
fn an_uncommitted_change_of_declaration_is_judged() {
    let repo = Repo::human();
    let file = landed_at(&repo, 1);
    redeclare(&file, 1, 2);
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("its format declaration was 1 at") && text.contains("and is 2 at the working tree"), "{text}");
}

/// A declaration left as it landed is no finding.
#[test]
fn an_unchanged_declaration_is_no_finding() {
    let repo = Repo::human();
    landed_at(&repo, 3);
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
}
