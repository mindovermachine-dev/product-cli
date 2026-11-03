# ADR-0005: One record per node kind, with a source generator

**Status:** Accepted (2026-10-30)
**Date:** 2026-10-30
**Definition:** chapter 7, Node kinds

## Context

The Rust tool kept five hand-maintained parallel enumerations per node kind. It failed silently when one was missed. The definition designs that failure mode out rather than testing for it.

## Decision

- **D1. One record per kind.** Each node kind of the pinned `sdlc:` edition is one C# record in `ProductCli.Vocabulary`, annotated with its class IRI and the IRI of each property.
- **D2. Generated code.** A source generator emits, from those records alone:
  - the RDF writer to triples;
  - the parser from triples;
  - the kind registry used by `domain new`, `show` and `list`;
  - the shape binding (the shape IRI a record is checked against).
- **D3. Mismatches are compile errors.** A record whose IRIs do not exist in the pinned edition fails the build. The generator reads the edition's vocabulary file as an additional input. A kind in the edition with no record is a build warning listing it, so coverage of the vocabulary is visible.
- **D4. Canonicalisation is not generated.** The canonicaliser is not per kind: RDFC-1.0 runs over the graph, from Varve (ADR-0002, D5). Draft 1 listed the canonicaliser among the generated outputs; it does not need to be, and it should not be.

## Alternatives considered

- **Generate the records from the ontology.** Deferred. Records generated from `sdlc:` would make the vocabulary the single source outright. But the C# shape of a record (naming, optionality, collections) needs authoring decisions the ontology does not carry. Revisit once the `sdlc:` edition is pinned and its annotations are known.
- **Reflection over attributes at runtime.** Rejected by ADR-0003.

## Consequences

- Adding a node kind is one record.
- An edition change that renames a term breaks the build where the term is used, not silently at export.
