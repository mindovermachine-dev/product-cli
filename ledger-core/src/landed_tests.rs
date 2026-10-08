//! Entity keys over a change-set file's text.

use super::*;

const FILE: &str = "format: 6
id: cs:01A
created_at: 2026-10-01T00:00:00Z
created_by: owner@customer.example
acceptances:
  - id: acc:01B
    decision: dec:ns/01C
revocations:
  - acceptance: acc:01X
    at: 2026-10-01T00:00:00Z
versions:
  - decision: dec:ns/01C
    hash: sha256:aa
";

#[test]
fn every_item_is_keyed_by_its_list_and_identity() {
    let e = entities(FILE).expect("mapping");
    let keys: Vec<&str> = e.keys().map(String::as_str).collect();
    assert_eq!(keys, ["acceptances/acc:01B", "changeset", "revocations/acc:01X", "versions/sha256:aa"]);
}

#[test]
fn a_format_change_alone_changes_no_entity() {
    let bumped = FILE.replace("format: 6", "format: 7");
    assert_eq!(entities(FILE), entities(&bumped));
}

#[test]
fn an_edit_to_an_entity_shows_in_its_value() {
    let edited = FILE.replace("decision: dec:ns/01C\nrevocations", "decision: dec:ns/01D\nrevocations");
    assert_ne!(entities(FILE).expect("a")["acceptances/acc:01B"], entities(&edited).expect("b")["acceptances/acc:01B"]);
}

/// Ruling 60: `parents` is part of the header entity, so adding a parent to
/// a landed change-set changes the header, as adding a `note` does.
#[test]
fn parents_are_keyed_with_the_header() {
    let with_parent = FILE.replace("created_by: owner@customer.example\n", "created_by: owner@customer.example\nparents: [cs:01P]\n");
    let e = entities(&with_parent).expect("mapping");
    assert!(e.keys().all(|k| !k.starts_with("parents/")), "{:?}", e.keys());
    assert_ne!(entities(FILE).expect("a")[HEADER], e[HEADER]);
}
