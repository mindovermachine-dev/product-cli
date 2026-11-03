# ADR-0009: Acceptance through a one-time migration of the self-hosted What

**Status:** Accepted (2026-10-30)
**Date:** 2026-10-30
**Definition:** D1.4 (chapter 7, Acceptance)

## Context

Draft 1's acceptance test was byte identity with the Rust tool's export. D1.1 made that impossible: the Rust export uses `pf:` and its own IRIs, while the new tool exports `sdlc:` with flat ULID IRIs. Chapter 8 still carries the self-hosted What over as the first fixture, so it needs a path in.

## Decision

- **D1. A repository tool, not a verb.** `tools/migrate-pf` is a repository tool, not a product verb and not part of the shipped binary. It reads the Rust tool's canonical export of the self-hosted What. It maps `pf:` terms to `sdlc:` terms of the pinned edition, mints one element IRI per node, and writes three files:
  - the migrated document;
  - the term map;
  - the IRI map (old IRI → new IRI).
- **D2. Run once, commit the outputs.** The migration runs once. Its three outputs are committed as fixtures under `fixtures/self-hosted/` and never regenerated: ULIDs are minted once.
- **D3. The acceptance criterion.**
  1. The migrated document imports through W1 and passes the shapes of its edition.
  2. Its canonical export round-trips byte-identically (W8).
  3. The conformance suite passes.

  When all three hold, the Rust binary is retired, and `tools/migrate-pf` is deleted in the same PR.
- **D4. No mapping means no migration.** A `pf:` term with no `sdlc:` mapping fails the migration and names the term. It is resolved in decision-driven-design (an edition question), never by dropping triples.
- **D5. The Rust tool is untouched.** The Rust tool is read for its export bytes only; nothing in it changes.

## Alternatives considered

- **Graph isomorphism under the term and IRI map.** Rejected: it gives a weaker gate than byte identity, and the migrated document is needed as a fixture anyway.
- **Migration as a product verb (`product import --from-pf`).** Rejected: it would make `pf:` part of the surface, and the surface would then be read-forever for a retired vocabulary.

## Consequences

- Slice 3 of the order of work depends on the pinned edition having a mapping for every `pf:` term the self-hosted What uses.
- The term map doubles as a list of what `sdlc:` must cover.
