# product-cli

A machine-checkable statement of what a product does, with an executable
oracle for its behaviour, exported as one graph that someone else can sign.
product-cli is being rewritten as one Native AOT .NET binary that authors that
statement — the What of a product, in the `sdlc:` vocabulary — as a graph with
shapes, never prose with diagrams. Anything that cannot be stated as a node,
an edge or a shape is not yet specified, and the tool says so rather than
accepting it.

The What is authored in conversation: an agent proposes, the author watches
the graph, its findings and its open questions change live, and adopts or
rejects each proposal, so every element traces to the person who adopted it.
Behaviour is executable before any code exists — Deciders and Projectors are
simulated against scenarios drawn from the flows, and the same scenarios later
judge realised code through a wire protocol. The tool puts nothing in force
and decides nothing about realisation: it produces a canonical document, and
what that document is held against, and who accepts it, belong to
[Writ](https://github.com/Hafeok/writ).

## Definition

The product is defined in decision-driven-design:
[`applications/sdlc/product-cli/product-definition.md`](https://github.com/mindovermachine-dev/decision-driven-design/blob/main/applications/sdlc/product-cli/product-definition.md).
This repository holds the tool, its ADRs (`docs/decisions/`) and its tests.
Agents start at [`AGENTS.md`](AGENTS.md).

## The Rust tool

The Rust workspace is frozen at the tag
[`rust-final`](https://github.com/mindovermachine-dev/product-cli/tree/rust-final)
(commit `70395df`), where all of it stays readable under the MIT licence it
was published with. No Rust release follows it. Its published releases, up to
[`v0.7.0`](https://github.com/mindovermachine-dev/product-cli/releases/tag/v0.7.0),
keep working.

## Licence

Apache-2.0 ([`LICENSE`](LICENSE), [`NOTICE`](NOTICE)). Developed with AI
assistance under human review.
