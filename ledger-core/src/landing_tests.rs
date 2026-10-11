//! Landing on the first-parent line, including the branch/merge agreement.

use std::path::Path;

use super::*;
use crate::testkit::git;

fn file(root: &Path, name: &str) {
    let path = crate::layout::log_dir(root, "a.ns").join(name);
    std::fs::create_dir_all(path.parent().expect("dir")).expect("mkdir");
    std::fs::write(&path, name).expect("write");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", name]);
}

fn repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    git(dir.path(), &["init", "-q", "--initial-branch=main"]);
    git(dir.path(), &["config", "user.email", "fixture-human@example"]);
    git(dir.path(), &["config", "user.name", "Fixture"]);
    dir
}

#[test]
fn without_git_everything_is_at_the_tip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let l = Landing::compute(dir.path(), None, true).expect("compute");
    assert!(!l.available);
    assert_eq!(l.index(".decisions/ns/a.ns/log/a.yml"), l.index(".decisions/ns/a.ns/log/b.yml"));
}

#[test]
fn a_file_lands_at_the_first_first_parent_commit_containing_it() {
    let dir = repo();
    file(dir.path(), "a.yml");
    file(dir.path(), "b.yml");
    let l = Landing::compute(dir.path(), None, true).expect("compute");
    assert!(l.index(".decisions/ns/a.ns/log/a.yml") < l.index(".decisions/ns/a.ns/log/b.yml"));
    assert!(l.index(".decisions/ns/a.ns/log/b.yml") < l.index(".decisions/ns/a.ns/log/uncommitted.yml"), "uncommitted is the tip");
}

#[test]
fn a_branch_verified_against_its_base_agrees_with_the_merge() {
    let dir = repo();
    let root = dir.path();
    file(root, "a.yml");
    git(root, &["checkout", "-q", "-b", "topic"]);
    file(root, "branch.yml");
    git(root, &["checkout", "-q", "main"]);
    file(root, "main.yml");
    git(root, &["checkout", "-q", "topic"]);
    let local = Landing::compute(root, Some("main"), true).expect("against base");
    let naive = Landing::compute(root, None, true).expect("no base");
    assert!(
        naive.index(".decisions/ns/a.ns/log/branch.yml") < naive.index(".decisions/ns/a.ns/log/main.yml"),
        "without the base the branch file reads as earlier, which the merge will not say"
    );
    git(root, &["checkout", "-q", "main"]);
    git(root, &["merge", "-q", "--no-ff", "-m", "merge topic", "topic"]);
    let merged = Landing::compute(root, None, true).expect("merge");
    let order = |l: &Landing| {
        (l.index(".decisions/ns/a.ns/log/a.yml") < l.index(".decisions/ns/a.ns/log/main.yml"),
         l.index(".decisions/ns/a.ns/log/main.yml") < l.index(".decisions/ns/a.ns/log/branch.yml"))
    };
    assert_eq!(order(&local), (true, true));
    assert_eq!(order(&local), order(&merged), "local against the base and the merge ref agree");
    assert!(Landing::compute(root, Some("no-such-ref"), true).is_err());
}

#[test]
fn before_needs_landing_no_later_and_an_earlier_at() {
    let t = |s: &str| s.parse::<DateTime<Utc>>().expect("instant");
    let close = Position { index: 5, at: t("2026-10-10T00:00:00Z") };
    let backdated_late = Position { index: 6, at: t("2026-10-01T00:00:00Z") };
    let honest_early = Position { index: 4, at: t("2026-10-01T00:00:00Z") };
    let same_commit_later = Position { index: 5, at: t("2026-10-11T00:00:00Z") };
    assert!(!backdated_late.before(&close), "landed after the close: never before it");
    assert!(honest_early.before(&close));
    assert!(!same_commit_later.before(&close));
}
