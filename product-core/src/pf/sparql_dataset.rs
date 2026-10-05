//! An in-memory RDF dataset of named graphs, queried one graph at a time.
//!
//! The decision registry's inbox (`ledger inbox`, R0) indexes each
//! repository branch's committed export as its own named graph. A query runs
//! with exactly one named graph as its default graph — never over the union
//! — so shapes that hold per store (one genesis per store) hold per graph.
//! Kept here so oxigraph stays this crate's dependency alone (OD-2).

use std::collections::BTreeMap;

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::{GraphName, NamedNode, NamedNodeRef, NamedOrBlankNode};
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;

/// A dataset of named graphs.
pub struct Dataset {
    store: Store,
}

impl Dataset {
    /// An empty dataset.
    pub fn new() -> Result<Self, String> {
        Ok(Self { store: Store::new().map_err(|e| e.to_string())? })
    }

    /// Load N-Triples text into the named graph `graph` (an IRI).
    pub fn load_graph(&self, graph: &str, ntriples: &str) -> Result<(), String> {
        let name = NamedNodeRef::new(graph).map_err(|e| e.to_string())?;
        let parser = RdfParser::from_format(RdfFormat::NTriples).without_named_graphs().with_default_graph(name);
        self.store.load_from_reader(parser, ntriples.as_bytes()).map_err(|e| e.to_string())
    }

    /// The named graphs, sorted.
    pub fn graphs(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .store
            .named_graphs()
            .filter_map(Result::ok)
            .filter_map(|g| match g {
                NamedOrBlankNode::NamedNode(n) => Some(n.into_string()),
                NamedOrBlankNode::BlankNode(_) => None,
            })
            .collect();
        out.sort();
        out
    }

    /// Run a SELECT with `graph` as the default graph — that graph alone.
    /// Rows map variable names to raw term strings (IRIs in `<>`, literals
    /// quoted), as [`super::sparql_rules::select`] returns them.
    pub fn select_in(&self, graph: &str, query: &str) -> Result<Vec<BTreeMap<String, String>>, String> {
        let name = NamedNode::new(graph).map_err(|e| e.to_string())?;
        let mut prepared = SparqlEvaluator::new().parse_query(query).map_err(|e| e.to_string())?;
        prepared.dataset_mut().set_default_graph(vec![GraphName::NamedNode(name)]);
        let QueryResults::Solutions(solutions) = prepared.on_store(&self.store).execute().map_err(|e| e.to_string())? else {
            return Ok(Vec::new());
        };
        let vars: Vec<String> = solutions.variables().iter().map(|v| v.as_str().to_string()).collect();
        let mut out = Vec::new();
        for sol in solutions {
            let sol = sol.map_err(|e| e.to_string())?;
            let mut row = BTreeMap::new();
            for var in &vars {
                if let Some(term) = sol.get(var.as_str()) {
                    row.insert(var.clone(), term.to_string());
                }
            }
            out.push(row);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_query_sees_one_named_graph_never_the_union() {
        let d = Dataset::new().expect("dataset");
        d.load_graph("urn:g:a", "<urn:x> <urn:p> \"a\" .\n").expect("a");
        d.load_graph("urn:g:b", "<urn:x> <urn:p> \"b\" .\n").expect("b");
        assert_eq!(d.graphs(), ["urn:g:a", "urn:g:b"]);
        let a = d.select_in("urn:g:a", "SELECT ?v WHERE { <urn:x> <urn:p> ?v }").expect("query");
        assert_eq!(a.len(), 1);
        assert_eq!(a[0]["v"], "\"a\"");
    }
}
