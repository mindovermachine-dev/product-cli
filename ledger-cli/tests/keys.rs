//! Decision keys through the binary: filing, carrying, and the two refusals.

mod common;

use common::Repo;

#[test]
fn add_files_a_keyed_exported_decision_as_format_5() {
    let repo = Repo::human();
    repo.declare();
    let out = repo.ok(&[
        "add", "--set", "ledger-design", "--namespace", "fixture.ledger", "--statement",
        "Money is decimal.", "--store", "constraint", "--key", "MoneyIsDecimal", "--exported",
    ]);
    assert!(out.contains("key `MoneyIsDecimal`"), "{out}");
    let file = repo.log_files().pop().expect("a log file");
    let text = std::fs::read_to_string(repo.log_dir("fixture.ledger").join(file)).expect("read");
    assert!(text.contains("format: 5"), "{text}");
    assert!(text.contains("key: MoneyIsDecimal") && text.contains("exported: true"), "{text}");
    repo.ok(&["verify", "--no-blame"]);
}

#[test]
fn an_unkeyed_add_stays_a_format_1_file() {
    let repo = Repo::human();
    repo.declare();
    repo.add("Money is decimal.", &[]);
    let file = repo.log_files().pop().expect("a log file");
    let text = std::fs::read_to_string(repo.log_dir("fixture.ledger").join(file)).expect("read");
    assert!(text.contains("format: 1"), "a file declares only what it uses: {text}");
}

#[test]
fn a_bad_key_is_refused_before_anything_is_written() {
    let repo = Repo::human();
    repo.declare();
    let out = repo.ledger(&[
        "add", "--set", "ledger-design", "--namespace", "fixture.ledger", "--statement", "x",
        "--key", "money_is_decimal",
    ]);
    assert_eq!(out.status.code(), Some(2));
    assert!(repo.log_files().is_empty());
}

#[test]
fn a_revision_carries_the_key_and_refuses_a_rename() {
    let repo = Repo::human();
    repo.declare();
    let id = repo.add("Money is decimal.", &["--key", "MoneyIsDecimal"]);
    repo.ok(&["revise", &id, "--statement", "Money is decimal, always."]);
    let err = repo.refused(&["revise", &id, "--statement", "Renamed.", "--key", "MoneyIsExact"]);
    assert!(err.contains("[L013]"), "{err}");
    repo.ok(&["verify", "--no-blame"]);
}

#[test]
fn a_keyless_decision_is_given_its_key_by_a_new_version() {
    let repo = Repo::human();
    repo.declare();
    let id = repo.add("Money is decimal.", &[]);
    repo.ok(&["revise", &id, "--statement", "Money is decimal.", "--key", "MoneyIsDecimal"]);
    repo.ok(&["verify", "--no-blame"]);
}

#[test]
fn a_second_live_decision_cannot_take_a_held_key() {
    let repo = Repo::human();
    repo.declare();
    repo.add("Money is decimal.", &["--key", "MoneyIsDecimal"]);
    let args = [
        "add", "--set", "ledger-design", "--namespace", "fixture.ledger", "--statement",
        "Prices are decimal.", "--store", "constraint", "--key", "MoneyIsDecimal",
    ];
    let err = repo.refused(&args);
    assert!(err.contains("[L014]"), "{err}");
}
