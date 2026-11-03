# ADR-0011: Agent writes are proposals; every change at the head is a `dpp/1` decision record

**Status:** Accepted (2026-10-30)
**Date:** 2026-10-30
**Definition:** D1.11, D1.12, D1.14 (chapter 3 "Proposals and decisions", chapter 4 "Decision records and the profile", W10 to W12)
**Rests on:** `dpp/1`, specification 1.0, draft of 2026-10-27 (decision-driven-design `spec/decision-provenance/dpp-1.md`)

> **Note (2026-11-03, Draft 4)**
>
> D1.14 is superseded by D1.17. The What-version record takes `DDD-dec-63` §6's ruled shape:
>
> - `dk.decision-driven-design.sdlc` holds `edition`, `canon`, `graphHash` and `elements`;
> - this tool's own dotted member holds the format IRI, the simulation status and the logic artefacts;
> - `basedOn` pins every decision in force. That reverses the alternative this ADR rejected ("`basedOn` lists every adoption ever"). D1.17 accepts the size, which stays well under §3.1 for several thousand decisions, in exchange for a record whose `basedOn` is complete on its own.

## Context

The author should adopt each step the agent proposes, and that adoption should create provenance (D1.10). The protocol now lives in decision-driven-design as `dpp/1`, shared by any system that records decisions. What it fixes and what it leaves out decides the design:

- **One conformance class** (§8): a conformant system. There is no provenance class beside an authority class. Draft 2's W11 assumed one; it does not exist.
- **Authority is a profile matter.** Who may decide or accept, whether a decision binds, positions, sessions and any clock other than `at` are non-goals (§1.3), documented by each system in its profile (§9.4). Appendix B shows Writ-style acceptance as one such profile.
- **`why` is required.** A record without a non-empty `why` is not a record (§3.4).
- **Software is `by`.** Where software made the decision, `by` MUST be the software's identity, with the responsible person in `attributedTo` (§3.4).
- **Status values carry no force.** `status` is one of `proposed`, `accepted`, `superseded`, `withdrawn`; its meaning is the writer's profile's, and a reader MUST NOT infer force from it (§3.4).
- **Signatures are optional** and, when present, show only that a key signed the content, not entitlement (§11.3).
- **The file binding** (§6.6) lets a directory of records be a conformant system with no code.

So product-cli can record decisions as a conformant system and stay outside authority, with no key.

## Decision

- **D1. The draft log is a `dpp/1` system.** Each product's draft log exposes its records through the file binding at `.product/products/<name>/decisions/`, with a descriptor naming this tool's profile. Its system identifier is chosen once per draft log and never changes (§3.3).
- **D2. Proposals.** Every write verb on the agent surface creates a proposal: a change set, checked against the shapes (W1) and the phase gate (W9) on submission, and held beside the head. Its record is version 1 of a new decision:
  - `by`: the agent's identity as the MCP client declares it;
  - `attributedTo`: the author;
  - `status`: `proposed`;
  - `why`: the agent's reason. The MCP tool schema makes it a required argument.

  The change set's hash is pinned in a dotted member.
- **D3. Alternatives.** An agent may submit two or three proposals as alternatives. A dotted member names the set. They are separate decisions, so each keeps its own reason.
- **D4. Adoption.** Adoption is the next version of the proposal's decision:
  - `by`: the author;
  - `status`: `accepted`;
  - `why`: the author's reason, or `{ "ref": <the proposal> }` with the proposal pinned in `basedOn` (§3.4);
  - `supersedes`: the proposal version.

  The change set is re-checked against the current head before the commit. If it no longer holds, the adoption is refused and the proposal stays pending. Adopting one alternative writes a `withdrawn` version for each sibling, with `why` referring to the adopted one.
- **D5. Rejection.** Rejection is the next version with `status: withdrawn` and the author's reason. The workbench asks for a reason in one line. An empty reason is not offered, because the protocol does not allow one.
- **D6. Direct writes.** A write by the author at the CLI is a single record: `by` the author, `status: accepted`, `why` given with `--why`. There is no proposal step.
- **D7. The What-version record.** An export writes a What-version record (D1.14):
  - `basedOn` is the previous What-version plus every adoption and direct write since it, each pinned. The chain keeps records small and the walk complete.
  - The edition, format IRI, canonical hash and simulation status are in dotted members.
  - The draft position, the Varve header hash and the session identifier are in `extensions`.
- **D8. The profile.** This tool publishes its profile. It states:
  - `proposed` means an agent's suggestion;
  - `accepted` means adopted into this author's draft;
  - `withdrawn` means rejected or displaced by an alternative;
  - no record binds anyone, and an edge into this system from another carries provenance only;
  - the dotted members and `extensions` it writes;
  - the identity schemes in `by`: the author's comes from configuration as a URI; the agent's is self-declared and unauthenticated.
- **D9. No signatures.** Records are unsigned. Integrity rests on pins and the Git history of the decisions directory (§11.2). Signing by the author's key would be a later ADR; it would add integrity, never authority.
- **D10. Hashing.** The `dpp/1` hash law (§5) is implemented once and checked against the protocol's canonicalisation vectors. It is generic, so if Varve offers it, it comes from Varve (ADR-0002 D5); if not, that is a Varve issue first.

## Alternatives considered

- **Avoid `status: accepted`, to keep clear of Writ's meaning of acceptance.** Rejected: the protocol defines the value set and makes meaning per-profile. Inventing a dotted status would hide the decision from every `dpp/1` reader, and §3.7 forbids putting protocol-defined information only in dotted members. The cost is that a reader must know profiles differ; the protocol already requires readers to infer no force from a status.
- **One record per commit, with no proposals.** Rejected: the agent would write to the head, and W12 (nothing an agent wrote is in the What unless a person adopted it) would not hold.
- **Adoption as a separate decision with `basedOn` to the proposal, not a new version.** Workable. Rejected because a decision's versions already express "the same decision, now taken". Separate decisions would double the records per step and split one decision's history across two ids.
- **The What-version `basedOn` lists every adoption ever.** Rejected: it grows without bound past the §3.1 size recommendation. The chain gives the same walk.
- **Signing with an author key now.** Deferred: optional in the protocol, and it would reopen the no-key boundary for integrity the Git history already gives.

## Consequences

- Every element of the What traces, by a `dpp/1` walk, to the author who adopted it and the agent proposal it came from, with reasons at both steps. A Writ decision that lands an export by naming it in `basedOn` reaches all of them (W11).
- The agent surface needs a reason on every write. That is extra input per call, but it is the reason the agent already has, and it becomes the record's `why`.
- The `dpp/1` conformance suite becomes part of this repository's CI.
- **The system identifier.** Ruled with this ADR: one `dpp/1` system per author's draft, identified as the product IRI P plus `/draft/<author>`. Two authors on one product are two systems, joined by `basedOn` like any others.
- **`status: accepted`.** Ruled with this ADR, with the meaning given by this tool's profile (D8).
- **Open: the protocol is a draft.** `dpp/1` is a draft of 2026-10-27; a change to it before 1.0 is final may change this ADR.
