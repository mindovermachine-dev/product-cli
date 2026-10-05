//! The selection file's shape, with the manifest over it.

use super::*;

fn file() -> BatchFile {
    BatchFile {
        form: BATCH_FORM.to_string(),
        actor: "owner@customer.example".to_string(),
        as_role: None,
        rows: vec![
            Row {
                repository: "billing".into(),
                branch: Some("agent/12".into()),
                decision: "dec:ns/01A".into(),
                version: "sha256:aa".into(),
                grant: Some("grant:01G".into()),
            },
            Row {
                repository: "ledger".into(),
                branch: None,
                decision: "dec:ns/01B".into(),
                version: "sha256:bb".into(),
                grant: None,
            },
        ],
    }
}

#[test]
fn the_manifest_covers_every_row_its_branch_and_its_grant() {
    let base = manifest(&file());
    let mut moved = file();
    moved.rows[1].version = "sha256:cc".into();
    assert_ne!(manifest(&moved), base, "a moved hash");
    let mut regranted = file();
    regranted.rows[0].grant = Some("grant:01H".into());
    assert_ne!(manifest(&regranted), base, "another grant");
    let mut fewer = file();
    fewer.rows.pop();
    assert_ne!(manifest(&fewer), base, "a missing row");
    let mut other_branch = file();
    other_branch.rows[0].branch = Some("agent/13".into());
    assert_ne!(manifest(&other_branch), base, "another branch");
    let mut no_branch = file();
    no_branch.rows[0].branch = None;
    assert_ne!(manifest(&no_branch), base, "a branch dropped");
    let mut other_role = file();
    other_role.as_role = Some("acceptor".into());
    assert_ne!(manifest(&other_role), base, "`--as`");
    let mut reordered = file();
    reordered.rows.reverse();
    assert_eq!(manifest(&reordered), base, "order is not content");
}

#[test]
fn the_manifest_is_a_manifest_not_a_version_hash() {
    assert!(crate::batch::is_manifest_shaped(&manifest(&file())));
}

#[test]
fn a_file_with_no_rows_or_a_duplicate_row_is_refused() {
    let mut empty = file();
    empty.rows.clear();
    assert!(check(&empty).is_err());
    let mut twice = file();
    twice.rows.push(twice.rows[0].clone());
    assert!(check(&twice).is_err());
    let mut wrong = file();
    wrong.form = "ledger.something-else".into();
    assert!(check(&wrong).is_err());
}

#[test]
fn the_file_round_trips_through_yaml() {
    let text = serde_yaml::to_string(&file()).expect("yaml");
    assert!(text.contains("form: ledger.acceptance-batch.v1"), "{text}");
    let back: BatchFile = serde_yaml::from_str(&text).expect("parse");
    assert_eq!(back, file());
}
