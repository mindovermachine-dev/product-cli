use super::*;

use crate::declaration::{Attribution, Declaration};

fn declared() -> Declaration {
    Declaration::new("settle the basket", vec!["act/settles".into(), "inventory".into()])
        .bounded_by("one penny", "high")
}

fn run() -> RunRecord {
    RunRecord::new("01", "tool", "act/a", "subject")
        .declaring(declared())
        .having_read(vec!["act/settles".into(), "inventory".into()])
}

fn kinds(run: &RunRecord) -> Vec<Kind> {
    check(run).into_iter().map(|f| f.kind).collect()
}

/// A run that predicted nothing cannot be contradicted by what it did.
#[test]
fn a_run_that_declared_nothing_is_itself_a_finding() {
    let bare = RunRecord::new("01", "tool", "act/a", "subject");
    assert_eq!(kinds(&bare), [Kind::Undeclared]);
}

#[test]
fn a_complete_declaration_matched_by_its_reads_finds_nothing() {
    assert!(check(&run()).is_empty());
}

#[test]
fn a_missing_declaration_field_is_found_by_name() {
    let mut record = run();
    record.declared = Some(Declaration::new("settle it", vec!["act/settles".into()]));
    record.ground_read = vec!["act/settles".into()];

    let found = check(&record);
    assert!(found.iter().all(|f| f.kind == Kind::IncompleteDeclaration));
    assert_eq!(found.len(), 2, "tolerance and assurance");
}

#[test]
fn ground_declared_and_never_read_is_found() {
    let mut record = run();
    record.ground_read = vec!["act/settles".into()];

    let found = check(&record);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].kind, Kind::DeclaredButUnread);
    assert_eq!(found[0].subject, "inventory");
}

#[test]
fn ground_read_and_never_declared_is_a_different_finding() {
    let mut record = run();
    record.ground_read.push("pricing-table".into());

    let found = check(&record);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].kind, Kind::ReadButUndeclared);
    assert_eq!(found[0].subject, "pricing-table");
}

#[test]
fn a_claim_resting_on_nothing_is_an_escape_candidate() {
    let mut record = run();
    record.attributions = vec![Attribution::unattributed("rounding is half-even")];

    let found = check(&record);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].kind, Kind::UnattributedClaim);
}

#[test]
fn a_claim_resting_on_declared_ground_is_not() {
    let mut record = run();
    record.attributions = vec![Attribution::to("the basket settles at close", "act/settles")];
    assert!(check(&record).is_empty());
}

/// One defect, reported once — the undeclared ground, not a second bad claim.
#[test]
fn a_claim_on_undeclared_ground_is_reported_as_the_undeclared_ground() {
    let mut record = run();
    record.attributions = vec![Attribution::to("prices are ex-VAT", "pricing-table")];

    let found = check(&record);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].kind, Kind::ReadButUndeclared);
    assert_eq!(found[0].subject, "pricing-table");
}

#[test]
fn findings_across_runs_name_the_run_they_are_in() {
    let bare = RunRecord::new("02", "tool", "act/a", "subject");
    let all = check_all(&[run(), bare]);
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].0, "02");
}
