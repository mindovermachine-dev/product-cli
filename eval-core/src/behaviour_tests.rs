use super::*;

use crate::pinned::Pinned;

fn run(id: &str, address: &str, arrangement: &str, proposed: &[&str]) -> RunRecord {
    let mut record = RunRecord::new(id, "tool", "act/a", "subject")
        .at(Pinned::of([("q", address)]))
        .arranged_as(Pinned::of([("model", arrangement)]));
    record.proposed = proposed.iter().map(|s| (*s).to_string()).collect();
    record
}

#[test]
fn runs_without_an_address_are_not_compared() {
    let bare = RunRecord::new("01", "tool", "act/a", "subject");
    assert!(read(&[bare]).is_empty());
}

#[test]
fn runs_at_one_address_are_read_together() {
    let readings = read(&[run("01", "q1", "m", &["det/a"]), run("02", "q1", "m", &["det/a"])]);
    assert_eq!(readings.len(), 1);
    assert_eq!(readings[0].runs, 2);
}

#[test]
fn runs_at_different_addresses_are_not_comparable() {
    assert_eq!(read(&[run("01", "q1", "m", &[]), run("02", "q2", "m", &[])]).len(), 2);
}

#[test]
fn a_repeated_answer_agrees_with_itself() {
    let readings = read(&[run("01", "q1", "m", &["det/a"]), run("02", "q1", "m", &["det/a"])]);
    assert_eq!(readings[0].agreement, Some(1.0));
}

#[test]
fn a_changed_answer_at_one_address_disagrees() {
    let readings = read(&[run("01", "q1", "m", &["det/a"]), run("02", "q1", "m", &["det/b"])]);
    assert_eq!(readings[0].agreement, Some(0.0));
}

/// Two runs that both proposed nothing agree. Silence is a real answer.
#[test]
fn two_silences_agree() {
    let readings = read(&[run("01", "q1", "m", &[]), run("02", "q1", "m", &[])]);
    assert_eq!(readings[0].agreement, Some(1.0));
}

/// One run at an address is not a low score; it is the absence of one.
#[test]
fn a_single_run_yields_no_agreement_rather_than_zero() {
    let readings = read(&[run("01", "q1", "m", &["det/a"])]);
    assert_eq!(readings[0].agreement, None);
    assert_eq!(readings[0].drift(), None);
}

/// The earthquake reading: same address, changed worker, moved answer.
#[test]
fn a_changed_arrangement_at_a_fixed_address_shows_as_drift() {
    let readings = read(&[
        run("01", "q1", "old", &["det/a"]),
        run("02", "q1", "old", &["det/a"]),
        run("03", "q1", "new", &["det/b"]),
        run("04", "q1", "new", &["det/b"]),
    ]);

    assert_eq!(readings[0].arrangements, 2);
    assert_eq!(readings[0].agreement, Some(1.0), "each worker agrees with itself");
    assert_eq!(readings[0].across_arrangements, Some(0.0), "they do not agree with each other");
    assert_eq!(readings[0].drift(), Some(1.0));
}

#[test]
fn a_stable_worker_change_shows_no_drift() {
    let readings = read(&[
        run("01", "q1", "old", &["det/a"]),
        run("02", "q1", "old", &["det/a"]),
        run("03", "q1", "new", &["det/a"]),
        run("04", "q1", "new", &["det/a"]),
    ]);
    assert_eq!(readings[0].drift(), Some(0.0));
}

/// Declared-distinct ground must yield distinct behaviour.
#[test]
fn two_addresses_answered_identically_are_reported_as_collapsed() {
    let collapse = collapsed(&[run("01", "q1", "m", &["det/a"]), run("02", "q2", "m", &["det/a"])]);
    assert_eq!(collapse.len(), 1, "the caller declared two questions and got one answer");
}

#[test]
fn addresses_answered_differently_have_not_collapsed() {
    assert!(collapsed(&[run("01", "q1", "m", &["det/a"]), run("02", "q2", "m", &["det/b"])]).is_empty());
}

/// An address that answered two different ways has not collapsed onto another.
#[test]
fn an_unstable_address_is_not_reported_as_collapsed() {
    let collapse = collapsed(&[
        run("01", "q1", "m", &["det/a"]),
        run("02", "q1", "m", &["det/b"]),
        run("03", "q2", "m", &["det/a"]),
    ]);
    assert!(collapse.is_empty());
}
