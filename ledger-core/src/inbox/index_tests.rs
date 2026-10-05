//! The index's graph-stage shapes, per named graph.

use product_core::pf::sparql_dataset::Dataset;

use crate::authority::fixture::{self, genesis};
use crate::graph::GraphClass;

/// One store: its own genesis, accepted.
fn store(tail: &str) -> String {
    let g = genesis(tail, "steward");
    let accepted = fixture::accepted(&format!("{tail}9"), &g);
    crate::graph::ntriples::emit(&fixture::store(Vec::new(), fixture::changeset(vec![g], vec![accepted])))
}

#[test]
fn two_stores_each_with_its_genesis_pass_per_graph_and_fail_a005_as_a_union() {
    let (a, b) = (store("1"), store("2"));
    let d = Dataset::new().expect("dataset");
    d.load_graph(&super::index::graph_iri("billing", "main"), &a).expect("a");
    d.load_graph(&super::index::graph_iri("ledger", "main"), &b).expect("b");
    for g in d.graphs() {
        let classes: Vec<GraphClass> = crate::graph::shapes::graph_findings_in(&d, &g).iter().map(|f| f.class).collect();
        assert!(!classes.contains(&GraphClass::A005), "{g}: one genesis per store holds per graph: {classes:?}");
    }
    let union = Dataset::new().expect("dataset");
    union.load_graph("urn:ledger-inbox:union", &format!("{a}{b}")).expect("union");
    let classes: Vec<GraphClass> =
        crate::graph::shapes::graph_findings_in(&union, "urn:ledger-inbox:union").iter().map(|f| f.class).collect();
    assert!(classes.contains(&GraphClass::A005), "two stores in one graph fail A005: {classes:?}");
}

#[test]
fn a_graph_name_is_an_iri_whatever_the_branch() {
    let d = Dataset::new().expect("dataset");
    d.load_graph(&super::index::graph_iri("my repo", "feature/ünïcode x"), "<urn:a> <urn:b> <urn:c> .\n").expect("load");
    assert_eq!(d.graphs().len(), 1);
}
