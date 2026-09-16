use super::*;

fn store() -> (tempfile::TempDir, DiskBlobs) {
    let dir = tempfile::tempdir().expect("tempdir");
    let blobs = DiskBlobs::new(dir.path());
    (dir, blobs)
}

#[test]
fn a_body_put_under_a_key_comes_back() {
    let (_dir, blobs) = store();
    blobs.put("runs/01ABC.json", "{}").expect("puts");
    assert_eq!(blobs.get("runs/01ABC.json").expect("gets").as_deref(), Some("{}"));
}

#[test]
fn an_absent_key_holds_nothing_rather_than_failing() {
    let (_dir, blobs) = store();
    assert_eq!(blobs.get("runs/nope.json").expect("gets"), None);
}

#[test]
fn a_prefix_lists_what_is_under_it() {
    let (_dir, blobs) = store();
    blobs.put("runs/a.json", "{}").expect("puts");
    blobs.put("runs/b.json", "{}").expect("puts");
    blobs.put("judgements/x/c.json", "{}").expect("puts");

    let mut listed = blobs.list("runs");
    listed.sort();
    assert_eq!(listed, ["runs/a.json", "runs/b.json"]);
}

#[test]
fn an_absent_prefix_lists_nothing() {
    let (_dir, blobs) = store();
    assert!(blobs.list("runs").is_empty());
}

/// A store that can be talked into writing elsewhere is not a store.
#[test]
fn a_key_cannot_climb_out_of_the_root() {
    let (_dir, blobs) = store();
    assert!(blobs.put("../escaped.json", "{}").is_err());
    assert!(blobs.put("runs/../../escaped.json", "{}").is_err());
}

#[test]
fn an_empty_segment_is_not_a_key() {
    let (_dir, blobs) = store();
    assert!(blobs.put("runs//a.json", "{}").is_err());
}
