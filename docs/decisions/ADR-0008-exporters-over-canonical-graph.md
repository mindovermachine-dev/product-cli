# ADR-0008: Exporters over the canonical graph

**Status:** Accepted (2026-10-30)
**Date:** 2026-10-30
**Definition:** D1.2, D1.3, D1.5, D1.7

## Context

The canonical document is the only thing that crosses the seam to Writ (W7, W8, W10). By D1.3, it is the RDFC-1.0 canonical N-Quads of the What graph. Other renderings of the same What are wanted, starting with graph serialisations, and adding one must be cheap. Three constraints apply:

- Native AOT rules out runtime plugins (ADR-0003).
- An export seam is where code generation would re-enter (chapter 8).
- Exporting must not become a sixth contract (chapter 6).

Changes from the 2026-10-27 draft:

- the canonical document is N-Quads, not Turtle;
- the manifest is the `dpp/1` What-version record (D1.1), not invented fields;
- the authored logic is in the graph (D1.7), so the input to exporters is complete;
- sorted Turtle replaces N-Triples as the second exporter, because N-Triples is near-identical to canonical N-Quads for the default graph and would prove nothing.

## Decision

- **D1. One pipeline.** Node records → triples → RDFC-1.0 → `CanonicalGraph` → exporter → bytes. `CanonicalGraph` is the pinned `sdlc:` edition plus the sorted, RDFC-labelled triple set of the default graph. Canonicalisation happens once.
- **D2. Exporters are pure functions over triples:**

```csharp
public interface IGraphExporter
{
    ExportFormat Format { get; }   // format IRI, media type, supported editions, stability
    void Write(CanonicalGraph graph, WhatVersionRecord source, IBufferWriter<byte> output);
}
```

  An exporter reads neither the draft log nor node-record internals.
- **D3. The canonical writer is exporter one.** The canonical N-Quads writer implements `IGraphExporter` with no special path. Its format IRI is the one Writ lands.
- **D4. Sorted Turtle is exporter two.** It ships in the same slice, with a spec page and a conformance suite: a fixed prefix set, subjects and predicates sorted. It proves the seam on a format with real serialisation choices (prefixes, grouping, literal forms). The exit check is that both writers run over `CanonicalGraph` with no special cases.
- **D5. Registration.** A source generator builds a static registry of exporters. No reflection.
- **D6. Edition refusal.** An exporter declares its supported editions and refuses any other, with a diagnostic.
- **D7. Derivation in the record.** Every export carries a `dpp/1` What-version record with its format IRI. Derived exports add `prov:wasDerivedFrom` naming the canonical content hash. These fields are fixed in the record's specification before the first release.
- **D8. Scope.** An exporter describes the What. Output that is executable, or that is parameterised by a realisation target, is code generation and is refused. This is a review rule, and every exporter ADR states which side of it the exporter falls on.
- **D9. Stability tiers.** Only formats with a spec page and a conformance suite are read-forever: canonical N-Quads and sorted Turtle. Others are marked unstable.
- **D10. Out of process.** `product domain export --canonical` on stdout is the interface for anything outside the binary.

## Alternatives considered

- **Exporters over typed records.** Rejected: it couples every exporter to edition-generated record shapes. A typed view, if ever needed, is a projection computed from `CanonicalGraph`.
- **Runtime plugins.** Impossible under AOT.
- **Canonical writer as a special path.** Rejected: the hardest case would be the untested one.
- **N-Triples as exporter two.** Rejected: near pass-through from canonical N-Quads, so it proves nothing.
- **JSON-LD as exporter two.** Deferred: context and compaction rules are a specification question of their own.

## Consequences

- W7 is enforced in one place.
- Adding an exporter costs one class, one spec page and its fixtures.
- Writ must accept N-Quads as the landable document (cross-repository dependency, D1.3).

## Open

- **IRI base for format identifiers.** Tool-owned, beside `sdlc:`, not in it. To be fixed in the `dpp/1` record's specification.
