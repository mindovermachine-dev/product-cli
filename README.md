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

## Build and test

The repository is the skeleton of slice 0 (D1.25): the eight layer projects
under `src/`, their tests under `tests/`, and a host whose only behaviour is
`product --version`. It needs the .NET SDK that `global.json` pins (10.0.401,
or a later feature band of 10.0) and nothing else.

```bash
dotnet build ProductCli.slnx -c Release
```

```bash
dotnet test -c Release --no-build
```

The tests build probe projects with `dotnet build` against the solution's
outputs, so build the solution first.

To publish the native binary, `product`, for your platform: Native AOT
compiles for the operating system it runs on, and needs that platform's
native toolchain (clang on Linux, Xcode on macOS, the Visual C++ build tools
on Windows).

```bash
dotnet publish src/ProductCli.Host/ProductCli.Host.csproj -c Release -r linux-x64 -o artifacts/publish/linux-x64
```

```bash
artifacts/publish/linux-x64/product --version
```

The supported runtime identifiers are `linux-x64`, `linux-arm64`, `osx-arm64`
and `win-x64`.

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
