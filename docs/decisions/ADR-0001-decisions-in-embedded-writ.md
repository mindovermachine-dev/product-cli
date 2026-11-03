# ADR-0001: Record this repository's decisions as ADRs governed by an embedded Writ

**Status:** Accepted (2026-10-30)
**Date:** 2026-10-30
**Definition:** chapter 8, "Decision-driven, like Writ"

> **Note (2026-11-03, Draft 4)**
>
> D1 now reads `docs/decisions/ADR-####-<slug>.md` (D1.22). The first PR carries ADR-0001 to ADR-0012.

## Context

product-cli signs nothing (W10), but the repository that builds it is governed. Chapter 8 requires an embedded `writ` from the first commit, ADRs in its log, code landed through `writ land`, and a baseline on day one. Writ's `land` may not exist when this repository starts. Writ resolved the same bootstrap problem with ADR-level discipline by review (Writ ADR 0008).

The old `.decisions/` ledger is Writ's authority graph now and is read once by `writ import`. Nothing here writes to it.

## Decision

- **D1. Where ADRs live.** Every architectural decision is an ADR under `adr/ADR-####-<slug>.md`, numbered from 0001. Each has Context, Decision as a numbered decision set (D1, D2, …), Alternatives considered, and Consequences.
- **D2. Writ from the first commit.** The repository embeds `writ` from the first commit that can hold it. Until then the ADR files are authoritative, by review, as in Writ ADR 0008. When the log can hold them, every accepted ADR is imported and a baseline is taken. From that point the log is authoritative and the files are its rendering.
- **D3. No silent divergence.** A proposal that contradicts an accepted ADR says so and supersedes it, naming the superseded ADR. It never diverges silently.
- **D4. Outside-in.** An ADR that changes the surface, formats, guarantees or contracts names the definition change it rides on, and the definition changes first. An ADR that needs a vocabulary change names the `sdlc:` edition requested in decision-driven-design.

## Alternatives considered

- **Keep `.decisions/`.** Rejected: it is Writ's authority graph now (chapter 8), and writing to it would make this tool a decision ledger.
- **Defer governance until Writ can land.** Rejected: decisions taken before the baseline would have no record. Review discipline costs little and is already proven in Writ.

## Consequences

- The first PR carries ADR-0001 to ADR-0011 as files.
- Importing them into the embedded log is a tracked follow-up, done once Writ can hold them.
