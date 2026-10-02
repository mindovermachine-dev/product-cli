//! The N-Triples export: canonical form, namespace selection, freshness.

use std::path::Path;

use crate::id::DecisionId;
use crate::store::Store;
use crate::testkit;

use super::export::{check, default_path, export, namespaces, EXPORT_DIR};
use super::{ntriples, turtle};

const OTHER: &str = "dec:hafeok.other/01K2C4YQJ3F8M0PT5W7NZ9RDY0";

fn accepted_store() -> Store {
    let sealed = testkit::sealed(testkit::version());
    let acceptance = testkit::acceptance(&sealed);
    testkit::store(testkit::changeset(vec![sealed], vec![acceptance]))
}

/// One change-set filing a decision in each of two namespaces, each
/// accepted, with the `hafeok.ledger` acceptance revoked.
fn two_namespace_store() -> Store {
    let ours = testkit::sealed(testkit::version());
    let mut theirs = testkit::version();
    theirs.decision = OTHER.parse::<DecisionId>().expect("id");
    let theirs = testkit::sealed(theirs);
    let ours_acc = testkit::acceptance(&ours);
    let mut theirs_acc = testkit::acceptance(&theirs);
    theirs_acc.id = "acc:01K2C4YQJ3F8M0PT5W7NZ9RDY1".parse().expect("acc id");
    let mut cs = testkit::changeset(vec![ours, theirs], vec![ours_acc, theirs_acc]);
    cs.revocations.push(testkit::legacy_revocation("2026-08-11T09:00:00Z", "filed against the wrong version"));
    testkit::store(cs)
}

fn count(text: &str) -> usize {
    let rows = product_core::pf::sparql_rules::select(text, "SELECT ?s ?p ?o WHERE { ?s ?p ?o }")
        .expect("parses as RDF");
    rows.len()
}

#[test]
fn the_export_carries_exactly_the_index_triples() {
    let store = accepted_store();
    let nt = ntriples::emit(&store);
    assert_eq!(count(&nt), count(&turtle::emit(&store)));
    assert_eq!(count(&nt), nt.lines().count(), "one triple per line, none repeated");
}

#[test]
fn lines_are_full_terms_sorted_by_code_point_and_lf_terminated() {
    let nt = ntriples::emit(&accepted_store());
    assert!(nt.ends_with(" .\n"), "{nt}");
    assert!(!nt.contains('\r'), "LF only");
    let lines: Vec<&str> = nt.lines().collect();
    let mut sorted = lines.clone();
    sorted.sort_unstable();
    assert_eq!(lines, sorted, "sorted by code point");
    for line in &lines {
        assert!(line.starts_with('<'), "every subject is an IRI: {line}");
        assert!(!line.contains(" a ") && !line.contains("ledger:Decision"), "no Turtle shorthand: {line}");
    }
    assert!(nt.contains("<http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:ledger:ns#Decision> ."));
    assert!(nt.contains("^^<http://www.w3.org/2001/XMLSchema#dateTime>"));
}

#[test]
fn the_set_stays_an_iri() {
    let nt = ntriples::emit(&accepted_store());
    assert!(nt.contains("<urn:ledger:ns#set> <urn:ledger-set:ledger-design> ."), "{nt}");
}

#[test]
fn literals_are_escaped_the_rdf_1_2_canonical_way() {
    let mut raw = testkit::version();
    raw.statement = "tab\there \"quoted\" back\\slash bell\u{7} del\u{7f} ff\u{c} é".into();
    let store = testkit::store(testkit::changeset(vec![testkit::sealed(raw)], Vec::new()));
    let nt = ntriples::emit(&store);
    assert!(
        nt.contains("\"tab\\there \\\"quoted\\\" back\\\\slash bell\\u0007 del\\u007F ff\\f é\""),
        "{nt}"
    );
    assert_eq!(count(&nt), nt.lines().count());
}

#[test]
fn a_namespace_export_holds_only_that_namespace() {
    let store = two_namespace_store();
    assert_eq!(
        namespaces(&store).into_iter().collect::<Vec<_>>(),
        ["hafeok.ledger", "hafeok.other"]
    );
    let ours = export(&store, "hafeok.ledger").expect("spoken");
    assert!(!ours.contains("hafeok.other"), "{ours}");
    assert!(!ours.contains("RDY1"), "the other namespace's acceptance stays out: {ours}");
    assert!(ours.contains("<urn:ledger:ns#revokes>"), "our revocation comes along: {ours}");
    assert!(!ours.contains("revokedAt"), "no triple lands on the acceptance (ruling 3): {ours}");
    let theirs = export(&store, "hafeok.other").expect("spoken");
    assert!(!theirs.contains("Revocation"), "a revocation follows its acceptance: {theirs}");
    assert!(export(&store, "hafeok.nobody").is_none());
}

#[test]
fn two_exports_of_one_store_are_the_same_bytes() {
    let store = two_namespace_store();
    assert_eq!(export(&store, "hafeok.ledger"), export(&store, "hafeok.ledger"));
}

fn rooted(dir: &Path, mut store: Store) -> Store {
    store.root = dir.to_path_buf();
    store
}

fn write_exports(store: &Store) {
    for ns in namespaces(store) {
        let path = default_path(&store.root, &ns);
        std::fs::create_dir_all(path.parent().expect("dir")).expect("mkdir");
        std::fs::write(&path, export(store, &ns).expect("export")).expect("write");
    }
}

#[test]
fn fresh_exports_pass_and_a_citation_projection_is_ignored() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = rooted(dir.path(), two_namespace_store());
    write_exports(&store);
    std::fs::write(dir.path().join(EXPORT_DIR).join("hafeok.ledger.citations.nt"), "x\n")
        .expect("write");
    assert_eq!(check(dir.path(), &store), Vec::new());
}

#[test]
fn nothing_committed_is_a_finding_not_a_pass() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = rooted(dir.path(), accepted_store());
    let findings = check(dir.path(), &store);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("no committed export"), "{findings:?}");
}

#[test]
fn a_hand_edited_export_fails_and_names_the_regeneration() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = rooted(dir.path(), accepted_store());
    write_exports(&store);
    let path = default_path(dir.path(), "hafeok.ledger");
    let edited = std::fs::read_to_string(&path).expect("read").replace("never double", "never float");
    std::fs::write(&path, edited).expect("write");
    let findings = check(dir.path(), &store);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].subject, "docs/decisions/hafeok.ledger.nt");
    assert!(findings[0].message.contains("1 line(s) the log does not produce, 1 line(s) it lacks"));
    assert!(findings[0].message.contains("ledger export --format ntriples --namespace hafeok.ledger"));
}

#[test]
fn reordered_lines_are_the_same_triples_but_not_the_same_bytes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = rooted(dir.path(), accepted_store());
    write_exports(&store);
    let path = default_path(dir.path(), "hafeok.ledger");
    let mut lines: Vec<String> =
        std::fs::read_to_string(&path).expect("read").lines().map(str::to_string).collect();
    lines.reverse();
    std::fs::write(&path, lines.join("\n") + "\n").expect("write");
    let findings = check(dir.path(), &store);
    assert!(findings[0].message.contains("not its bytes"), "{findings:?}");
}

#[test]
fn a_spoken_namespace_without_an_export_and_an_unspoken_export_both_fail() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = rooted(dir.path(), two_namespace_store());
    write_exports(&store);
    std::fs::remove_file(default_path(dir.path(), "hafeok.other")).expect("rm");
    std::fs::write(default_path(dir.path(), "hafeok.gone"), "").expect("write");
    let findings = check(dir.path(), &store);
    let text = format!("{findings:?}");
    assert_eq!(findings.len(), 2, "{text}");
    assert!(text.contains("which the log does not speak"), "{text}");
    assert!(text.contains("no export of it is committed"), "{text}");
}

#[test]
fn a_namespace_export_carries_its_own_authority_records_only() {
    use crate::authority::fixture::{self, accepted, genesis, grant, role};
    use crate::authority::Capability;
    let g = genesis("1", "steward");
    let elsewhere = grant("2", "steward", "other@x", "ns:hafeok.other", 0);
    let mut cs = fixture::changeset(vec![g.clone(), elsewhere.clone()], vec![accepted("3", &g)]);
    let sealed = testkit::sealed(testkit::version());
    cs.decisions = testkit::changeset(vec![sealed.clone()], Vec::new()).decisions;
    cs.versions = vec![sealed];
    let mut store = fixture::store(vec![role("steward", Capability::ALL)], cs);
    store.roles.push(role("unrelated", &[Capability::AcceptDecision]));
    let text = super::export::export(&store, "hafeok.ledger").expect("the namespace is spoken");
    assert!(text.contains(&format!("<urn:{}>", g.id)), "the genesis (*) reaches every namespace");
    assert!(text.contains("<urn:ledger-role:steward>"), "the role its grant names");
    assert!(!text.contains(&format!("<urn:{}>", elsewhere.id)), "another namespace's grant stays out");
    assert!(!text.contains("ledger-role:unrelated"), "a role nothing here names stays out");
}
