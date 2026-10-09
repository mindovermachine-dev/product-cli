//! A float, or an explicit null in a required field, in hashed content is a schema fault (rulings 55, 56).
//!
//! The verification report's section 8 and section 13.1, committed with
//! their digests: `statement: 1.5` hashed as its source text and verified,
//! and `statement: null` and `statement: ~` hashed as the texts `null` and
//! `~`. Each store is now refused, while the quoted spellings stay text.

mod common;

use std::path::Path;

use common::{hand, Repo};

const CS: &str = "01K2C4YQJ3F8M0PT5W7NZ9RDXW";

/// A version whose `statement` is written as `scalar`, its digest the one
/// the loader computes, committed beside the `pass` fixture's set.
fn committed(scalar: &str) -> (i32, String) {
    let repo = Repo::human();
    let sets = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pass/.decisions/ns/fixture.ledger/sets");
    std::fs::create_dir_all(repo.sets_dir("fixture.ledger")).expect("sets dir");
    for entry in std::fs::read_dir(sets).expect("fixture").flatten() {
        std::fs::copy(entry.path(), repo.sets_dir("fixture.ledger").join(entry.file_name())).expect("copy");
    }
    std::fs::create_dir_all(repo.log_dir("fixture.ledger")).expect("log dir");
    let file = |hash: &str| {
        format!(
            "format: 1\nid: cs:{CS}\ncreated_at: 2026-08-10T09:14:22Z\ncreated_by: fixture-human@example\n\
decisions:\n  - id: dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV\n    created_at: 2026-08-10T09:14:22Z\n    created_by: fixture-human@example\n\
versions:\n  - decision: dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV\n    hash: {hash}\n    set: ledger-fixture\n    statement: {scalar}\n\
\x20   allocation: constraint\n    discharge: [analyzer:DEC001-no-float-money]\n    tolerance_floor_at_creation: T1\n"
        )
    };
    let draft: ledger_core::changeset::ChangeSet = serde_yaml::from_str(&file(&format!("sha256:{}", "0".repeat(64)))).expect("parse");
    let hash = ledger_core::hash::version_hash(&draft.versions[0]).to_string();
    std::fs::write(repo.log_dir("fixture.ledger").join(format!("{CS}.yml")), file(&hash)).expect("write");
    hand::commit(&repo, "a version");
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

#[test]
fn a_plain_float_statement_is_a_schema_fault() {
    for scalar in ["1.5", "1.50", "1e3", ".inf"] {
        let (code, text) = committed(scalar);
        assert_eq!(code, 1, "{scalar}: {text}");
        assert!(text.contains("[SCHEMA]") && text.contains("versions[0].statement") && text.contains("float"), "{scalar}: {text}");
    }
}

#[test]
fn an_explicit_null_statement_is_a_schema_fault() {
    for scalar in ["null", "~"] {
        let (code, text) = committed(scalar);
        assert_eq!(code, 1, "{scalar}: {text}");
        assert!(text.contains("`versions[0].statement` is an explicit null"), "{scalar}: {text}");
    }
}

/// The controls: a quoted float and an integer are text, and verify.
#[test]
fn quoted_and_integer_statements_stay_text() {
    for scalar in ["'1.5'", "\"null\"", "42"] {
        let (code, text) = committed(scalar);
        assert_eq!(code, 0, "{scalar}: {text}");
    }
}
