# ADR-0007: Commits, sessions and the main branch

**Status:** Accepted (2026-10-30)
**Date:** 2026-10-30
**Definition:** chapter 8, "Decision-driven, like Writ"

## Context

The repository follows the Writ and Varve pattern: signed agent commits, traceability per session, and a `main` that accepts only checked commits.

## Decision

- **D1. Commits.** Every commit is signed. Agent sessions sign as the agent identity, with no model identifiers in commits, trailers or records. Every commit carries `Refs #N` and a DCO sign-off.
- **D2. Sessions.** Every agent session leaves a traceability record:
  - what was read;
  - the approved plan;
  - the commits;
  - the checks run and their results;
  - anything deviating from the plan, with the reason.
- **D3. Pull requests.** One PR per slice or ADR change, with a release descriptor.
- **D4. Main.** `main` accepts only checked commits and is fast-forwarded at an approved head. Once Writ can land, through `writ land`.
- **D5. Parallel sessions.** When two sessions run in parallel, whichever lands second reconciles: it rebases, re-runs every check, and records the reconciliation in its traceability record.
- **D6. No suppression.** No rule is suppressed. A rule that is wrong is changed by ADR.

## Alternatives considered

None. This ADR adopts the existing Writ and Varve pattern rather than deciding a new one.

## Consequences

- `AGENTS.md` is the single source for these rules.
- The session prompt template refers to this ADR.
