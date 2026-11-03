# ADR-0006: Guarantee tests observe bytes, not return values

**Status:** Accepted (2026-10-30)
**Date:** 2026-10-30
**Definition:** chapter 5

## Context

In the Rust tool, two MCP status tools returned a success envelope without writing the file. Their tests checked the envelope and passed. Eleven updates were lost before anyone noticed. The cause was test scope, not implementation quality: a test that observes the return value accepts a stub.

## Decision

- **D1. What a guarantee test observes.** Every W-property test observes one of:
  - the draft log, read back through the draft-store contract at a position;
  - the bytes of an export;
  - the bytes on the runner's stdout;
  - the `dpp/1` records in the decisions directory (ADR-0011).
  None observes the return value of the verb under test.
- **D2. Failed writes.** A test of a refused write (W1, W2, W5) asserts that the log's head position and its bytes are unchanged, not only that an error was returned.
- **D3. Recorded in `AGENTS.md`.** The rule is in `AGENTS.md` and is checked in review. A test that only asserts on a verb's result is rejected.

## Alternatives considered

- **Leave it to review.** Rejected: that is what failed before.
- **An analyzer.** Desirable but not yet specified: it is hard to tell mechanically what a test observes. It stays a review rule until a pattern for an analyzer emerges.

## Consequences

- Guarantee tests are slightly heavier: they read back.
- A stub cannot pass them.
