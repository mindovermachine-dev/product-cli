//! Serialising the graph as sorted, canonical N-Triples.
//!
//! The committed export is this form: one triple per line, every term
//! written in full (no prefixes, no `a`), literals escaped the RDF 1.2
//! canonical way, lines sorted by code point and each ended by one LF. The
//! triples are the same ones the Turtle index carries — both serialisations
//! read one [`Triples`] map — so the export can never say something the
//! index does not. Sorting by line is what makes the bytes a function of the
//! graph alone, which `verify --export` relies on.

use std::collections::BTreeSet;

use crate::store::Store;

use super::turtle::{triples, NS, PROV};

const RDF_TYPE: &str = "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>";
const XSD: &str = "http://www.w3.org/2001/XMLSchema#";

/// The whole store as sorted N-Triples, one LF-terminated line per triple.
pub fn emit(store: &Store) -> String {
    let graph = triples(store);
    let mut lines = BTreeSet::new();
    for (subject, predicates) in &graph.0 {
        for (predicate, objects) in predicates {
            for object in objects {
                lines.insert(format!(
                    "{} {} {} .",
                    expand(subject),
                    expand(predicate),
                    expand(object)
                ));
            }
        }
    }
    let mut out = String::new();
    for line in lines {
        out.push_str(&line);
        out.push('\n');
    }
    out
}

/// One term from its Turtle spelling to its N-Triples spelling.
fn expand(term: &str) -> String {
    if term == "a" {
        return RDF_TYPE.to_string();
    }
    if term.starts_with('"') {
        return expand_literal(term);
    }
    if term.starts_with('<') {
        return term.to_string();
    }
    expand_name(term)
}

/// A literal keeps its (already canonical) lexical form; only a prefixed
/// datatype after the closing quote is expanded. The emitter escapes every
/// `"` inside the lexical form, so the last quote is the closing one.
fn expand_literal(term: &str) -> String {
    let Some(close) = term.rfind('"') else {
        return term.to_string();
    };
    let (lexical, suffix) = term.split_at(close + 1);
    match suffix.strip_prefix("^^") {
        Some(datatype) => format!("{lexical}^^{}", expand_name(datatype)),
        None => term.to_string(),
    }
}

/// A prefixed name to a full IRI. The emitter writes three prefixes and no
/// others; an unknown one is returned unchanged rather than guessed at, and
/// the round-trip test would then fail on the unparsable line.
fn expand_name(name: &str) -> String {
    let full = [("ledger:", NS), ("prov:", PROV), ("xsd:", XSD)]
        .iter()
        .find_map(|(prefix, iri)| name.strip_prefix(prefix).map(|local| format!("<{iri}{local}>")));
    full.unwrap_or_else(|| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rdf_type_and_prefixed_names_are_written_in_full() {
        assert_eq!(expand("a"), RDF_TYPE);
        assert_eq!(expand("ledger:Decision"), "<urn:ledger:ns#Decision>");
        assert_eq!(expand("prov:Entity"), "<http://www.w3.org/ns/prov#Entity>");
        assert_eq!(expand("<urn:ledger-set:x>"), "<urn:ledger-set:x>");
    }

    #[test]
    fn a_typed_literal_expands_only_its_datatype() {
        assert_eq!(
            expand("\"2026-08-10\"^^xsd:date"),
            "\"2026-08-10\"^^<http://www.w3.org/2001/XMLSchema#date>"
        );
        assert_eq!(expand("\"ledger:not-a-name\""), "\"ledger:not-a-name\"");
        assert_eq!(expand("\"a \\\"quoted\\\" x\"^^xsd:date"), "\"a \\\"quoted\\\" x\"^^<http://www.w3.org/2001/XMLSchema#date>");
    }
}
