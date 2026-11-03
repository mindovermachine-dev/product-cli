# ADR-0003: One Native AOT binary, gated in CI from the first commit

**Status:** Accepted (2026-10-30)
**Date:** 2026-10-30
**Definition:** chapter 7, Hosts and Constraints

## Context

The product is one Native AOT binary serving both CLI and MCP, with no server. AOT and trimming failures are cheap to fix when they appear and expensive once a dependency or a reflection pattern has spread. The Rust tool never had this constraint, so nothing in its behaviour reveals where .NET would break.

## Decision

- **D1. One binary.** `ProductCli.Host` publishes one Native AOT executable per supported runtime identifier. CLI, MCP (stdio and HTTP) and the workbench host (ADR-0010) are modes of that binary.
- **D2. Analyzers on everywhere.** Every library sets `IsAotCompatible` and `IsTrimmable`, and AOT and trim analyzer warnings are errors.
- **D3. CI publishes on every PR.** CI runs the AOT publish on every PR from the first commit and runs the published binary's smoke test. A PR that breaks AOT publish fails.
- **D4. No reflection.** No reflection-based serialisation. JSON uses System.Text.Json source generation. Registries (exporters, node kinds, verbs) are generated at compile time.
- **D5. Supported platforms.** The supported runtime identifiers are recorded in the build policy. W7 ("on every platform") is tested by running the canonical export fixtures on each of them in CI.

## Alternatives considered

- **JIT binary first, AOT later.** Rejected: the cost of AOT grows with every dependency and pattern added before the gate exists.
- **A separate MCP binary.** Rejected by chapter 7: one process, one binary.

## Consequences

- Package choices (ADR-0004) are filtered by AOT compatibility before anything else.
- W7's cross-platform clause becomes a CI matrix rather than an assertion.
