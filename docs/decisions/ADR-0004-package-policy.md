# ADR-0004: Package policy and register

**Status:** Accepted (2026-10-30)
**Date:** 2026-10-30
**Definition:** chapter 7, Constraints

## Context

The project is 100% managed .NET, Varve's constraints 1 to 5 apply, and every third-party package needs an ADR. Versions are resolved from nuget.org, never from memory.

## Decision

- **D1. The build policy and register.** The repository adopts Varve's build policy and package register. Every `PackageReference` carries `Adr=` naming the ADR that admits it. The build fails on a package without one.
- **D2. Starting register.** The register starts with:
  - Varve, admitted by this ADR;
  - the test framework and the property-testing library, as test-only packages, admitted by this ADR once named in slice 0's plan, with the versions resolved from nuget.org.
- **D3. Two open choices, each deferred to its own ADR in the slice that needs it:**
  - **CLI argument parsing.** System.CommandLine, if its current release is trim- and AOT-clean, or a small hand-written parser over the fixed verb table. Decided in the nodes slice.
  - **MCP transport.** The official C# MCP SDK, if it is AOT-clean, or a minimal JSON-RPC over stdio and HTTP using System.Text.Json source generation. Decided in the MCP slice. My default is the hand-written one: the protocol surface is small, and AOT certainty outweighs the code saved.
- **D4. Central versions.** Package versions are pinned centrally. Upgrades are PRs with the register updated.

## Alternatives considered

- **Admit common packages up front.** Rejected: every package is a trust and AOT cost, and the register is only worth having if it records a reason for each entry.

## Consequences

- Slice 0 has exactly Varve plus test-only packages.
- Each later dependency arrives with its own ADR and its AOT evidence.
