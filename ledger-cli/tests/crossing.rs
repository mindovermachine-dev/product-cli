//! Only `based_on` crosses a namespace; `supersedes` never does (LP-3.31,
//! ruling 43), judged on live claims alone (ruling 75; AC-43).
//!
//! The writer refuses the verb by the gate it runs over the store as it
//! would be, and the gate fails a claim filed by hand past the verb. A next
//! version that drops the edge repairs the store; the version that made the
//! claim stays in history, unjudged.

mod common;

use common::{hand, Repo};
use ledger_core::changeset::ChangeSet;
use ledger_core::mint::UlidMint;
use ledger_core::version::VersionRaw;

const OURS: &str = "fixture.ledger";
const THEIRS: &str = "other.ledger";

/// A decision in `ns`, returning its id.
fn decision(repo: &Repo, ns: &str, statement: &str) -> String {
    let out = repo.ok(&["add", "--set", "ledger-design", "--namespace", ns, "--statement", statement, "--store", "constraint", "--discharge", "analyzer:DEC001"]);
    common::decision_id(&out)
}

/// The latest version of `decision`, as filed.
fn latest(repo: &Repo, decision: &str) -> VersionRaw {
    let store = ledger_core::store::load(repo.path());
    let view = ledger_core::verify::view::View::build(&store);
    view.latest.get(decision).and_then(|i| view.versions.get(*i)).map(|v| v.raw.clone()).expect("a filed version")
}

/// File the next version of a decision by hand, past every verb: `mutate`
/// edits the copy of its latest version; the change-set lands in the
/// decision's own namespace.
fn next_version_by_hand(repo: &Repo, decision: &str, mutate: impl FnOnce(&mut VersionRaw)) -> String {
    let mut raw = latest(repo, decision);
    raw.parent = Some(raw.hash.clone());
    mutate(&mut raw);
    raw.hash = ledger_core::hash::version_hash(&raw);
    let hash = raw.hash.to_string();
    let mut mint = UlidMint::system();
    let mut cs = ChangeSet::empty(ledger_core::format::CURRENT_FORMAT, mint.mint_id("cs").expect("cs id"), chrono::Utc::now(), "fixture-human@example".parse().expect("identity"), None);
    cs.versions.push(raw);
    let log = repo.log_dir(decision.split('/').next().and_then(|d| d.strip_prefix("dec:")).expect("namespace"));
    std::fs::create_dir_all(&log).expect("log dir");
    std::fs::write(log.join(cs.file_name()), serde_yaml::to_string(&cs).expect("yaml")).expect("write");
    hash
}

fn verify(repo: &Repo) -> (i32, String) {
    let out = repo.ledger(&["verify", "--no-blame"]);
    (out.status.code().unwrap_or(-1), common::both(&out))
}

/// Two namespaces, a decision in each, committed and conformant.
fn two_namespaces() -> (Repo, String, String) {
    let repo = Repo::human();
    repo.declare();
    repo.declare_in(THEIRS);
    let old = decision(&repo, OURS, "The one to be superseded.");
    let successor = decision(&repo, THEIRS, "The would-be successor, in another namespace.");
    hand::commit(&repo, "two namespaces");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
    (repo, old, successor)
}

#[test]
fn the_verb_refuses_to_supersede_across_namespaces_and_writes_nothing() {
    let (repo, old, successor) = two_namespaces();
    let before = repo.log_files();
    let refused = repo.refused(&["supersede", &old, "--by", &successor, "--reason", "a crossing"]);
    assert!(refused.contains("[SCHEMA]") && refused.contains(&successor) && refused.contains("ruling 43"), "{refused}");
    assert_eq!(repo.log_files(), before, "nothing written");
    // The same edge the other way round is refused the same way.
    let refused = repo.refused(&["supersede", &successor, "--by", &old]);
    assert!(refused.contains("[SCHEMA]") && refused.contains(&old) && refused.contains("ruling 43"), "{refused}");
}

#[test]
fn a_hand_filed_crossing_is_a_schema_fault_until_a_next_version_drops_the_edge() {
    let (repo, old, successor) = two_namespaces();
    let claim = next_version_by_hand(&repo, &successor, |raw| raw.supersedes = Some(old.parse().expect("id")));
    hand::commit(&repo, "a supersedes into another namespace, by hand");
    let (code, text) = verify(&repo);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains(&format!("[SCHEMA] {successor}:")) && text.contains(&old) && text.contains("ruling 43"), "{text}");
    // The next version withdraws the claim (ruling 75); the claim stays in
    // history and the store is conformant again.
    let withdrawn = next_version_by_hand(&repo, &successor, |raw| raw.supersedes = None);
    hand::commit(&repo, "the edge dropped");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("conformant"), "{text}");
    let store = ledger_core::store::load(repo.path());
    assert!(store.log.iter().flat_map(|l| l.file.versions.iter()).any(|v| v.hash.to_string() == claim && v.supersedes.is_some()), "history keeps the claim");
    assert_eq!(latest(&repo, &successor).hash.to_string(), withdrawn);
}

#[test]
fn a_supersession_within_a_namespace_still_stands() {
    let (repo, old, _) = two_namespaces();
    let successor = decision(&repo, OURS, "The successor, at home.");
    repo.ok(&["supersede", &old, "--by", &successor]);
    hand::commit(&repo, "superseded at home");
    let (code, text) = verify(&repo);
    assert_eq!(code, 0, "{text}");
}
