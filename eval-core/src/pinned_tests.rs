use super::*;

#[test]
fn a_pin_holds_its_own_digest() {
    assert!(Pinned::of([("task", "act/a")]).holds());
}

#[test]
fn an_altered_pin_does_not_hold() {
    let mut pinned = Pinned::of([("task", "act/a")]);
    pinned.shown.insert("task".into(), "act/b".into());
    assert!(!pinned.holds());
}

#[test]
fn the_same_facts_pin_alike() {
    assert_eq!(Pinned::of([("a", "1")]), Pinned::of([("a", "1")]));
}

#[test]
fn different_facts_pin_apart() {
    assert_ne!(Pinned::of([("a", "1")]).digest, Pinned::of([("a", "2")]).digest);
}

#[test]
fn a_short_digest_is_twelve_hex() {
    assert_eq!(Pinned::of([("a", "1")]).short().len(), 12);
}
