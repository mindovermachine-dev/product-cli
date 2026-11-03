# ADR-0013: Build-time analyzers and the test stack

**Status:** Proposed
**Date:** 2026-11-03
**Definition:** chapter 7, Constraints; D1.25
**Amends:** ADR-0004 D1 (its wording only, D4 below)

## Context

ADR-0002 D3 requires layering to be enforced by an analyzer that fails the build, and every library to have a public API baseline. ADR-0003 D4 bans reflection in shipped code. ADR-0004 D2 admits Varve, and "the test framework and the property-testing library … once named in slice 0's plan". Slice 0 (D1.25, #167) named them, and needs the analyzers, which no ADR has yet admitted.

Every package here was resolved from nuget.org on 2026-11-03.

Varve consumes `DecisionDriven.Analyzers` (source `Hafeok/decision-driven-analyzers`) for its layering. Slice 0's plan checked its `0.1.0-preview.10` package against two questions:

- **Does its layering rule apply to a family other than Varve's?** Yes. DD0001 reads `ArchFamily=F` and `ArchLayer=n` from the project. It checks every referenced assembly whose name begins `F.`, reading that assembly's `[assembly: ArchLayer]` from metadata, and requires it strictly below `n`. Nothing in the rule names Varve.
- **Is it AOT-neutral?** Yes. The package holds `analyzers/dotnet/cs/*.dll` (netstandard2.0) and `build/` props and targets, and no `lib/`. It is marked `developmentDependency`.

## Decision

- **D1. The test stack.** Both packages are test-only: `Scope="tests/"` in the register, referenced only by the `*.Tests` projects, and never published with AOT. ADR-0004 D2 admits them, so their register entries cite `Adr="0004"`.
  - **xunit v3 (`xunit.v3` 4.0.2)**, on Microsoft.Testing.Platform. `global.json` names the platform as the test runner, and `xunit.v3` 4.x brings its v2 adapter, `xunit.v3.mtp-v2`. This is what Varve uses.
  - **CsCheck (5.0.1)** for property tests. It is Apache-2.0 and has no dependencies of its own.
- **D2. The analyzers.** These are admitted by this ADR, and their entries cite `Adr="0013"`.
  - **`DecisionDriven.Analyzers` (0.1.0-preview.10)**, for ADR-0002 D3's layering rule: DD0001 under `ArchFamily=ProductCli`, with every project's layer declared and `dd_require_layer = true`. It is referenced from `Directory.Build.props` by every project, with `PrivateAssets="all" IncludeAssets="analyzers;build"`. It is a preview, so a bump is a register change, made in a PR.
  - **`Microsoft.CodeAnalysis.PublicApiAnalyzers` (5.6.0)**, for ADR-0002 D3's public API baselines. Every library carries `PublicAPI.Shipped.txt` and `PublicAPI.Unshipped.txt`, and the build fails on a library without them, since the analyzer is silent when the files are absent.
  - **`Microsoft.CodeAnalysis.BannedApiAnalyzers` (5.6.0)**, for ADR-0003 D4. `eng/BannedSymbols.txt` bans, in every project under `src/`, the reflection that defeats trimming and Native AOT: `System.Reflection.Emit`, `Type.GetType` by name, reflective `Activator.CreateInstance`, and `Assembly.Load`, `LoadFrom` and `LoadFile`. Each entry cites ADR-0003 D4.
- **D3. Nothing ships.** No analyzer assembly reaches the binary or any output: every analyzer reference is `PrivateAssets="all"`, and none of the three packages has a `lib/` asset. The analyzers do add bytes to our assemblies, and only these: the `DecisionDriven.Analyzers` generator compiles internal attribute types into each assembly it analyzes (`DecisionDriven.ArchLayerAttribute` and the decision-citation attributes), and one `[assembly: ArchLayer(n)]`. They are plain attributes, read by no code of ours at run time, with no reflection and no effect on AOT.
- **D4. The register entry is the `PackageVersion`.** Under central package management, the register is `Directory.Packages.props`, Varve's form, which ADR-0004 D1 adopts. A `PackageReference` there carries no version.
  - This ADR therefore **amends the wording of ADR-0004 D1**, "Every `PackageReference` carries `Adr=`". It now reads: every `PackageVersion` in `Directory.Packages.props` carries `Adr=` naming the ADR that admits it, and a test-only entry also carries `Scope=`. The build fails on an entry without `Adr=`, on an `Adr=` that names no ADR in `docs/decisions/`, and on a reference from a project outside the entry's `Scope=`.
  - ADR-0004 is cited from here on as amended by this ADR. Its file is not edited (ADR-0001 D3).

## Alternatives considered

- **FsCheck for property tests.** Rejected. FsCheck 3.6.0 depends on `FSharp.Core`, which puts a second language's runtime library in every test closure for no capability CsCheck lacks.
- **No property-testing library in slice 0.** Rejected: ADR-0004 D2 names one in the starting register, and slice 0 has a property to test (every argument but `--version` exits 2).
- **A local layering analyzer**, the fallback ADR-0002 D3 allows. Rejected: the package's DD0001 applies to our family unchanged, and a local analyzer is code to keep for no gain.
- **Enabling only DD0001 of `DecisionDriven.Analyzers`.** Not possible without lowering the other rules' severity, which is a suppression: DD0008 reports it, and ADR-0007 D6 forbids it.
- **Requiring `Adr=` on each `PackageReference` as well.** Rejected: it would repeat on every reference what the register already states once, and two copies of one fact can disagree.
- **Compiling the layering probes in process.** Rejected: that needs `Microsoft.CodeAnalysis.CSharp` as one more test package, and an analyzer configuration built by hand to imitate MSBuild. The probes are built with `dotnet build` under the real build files instead, which tests the wiring as well as the rule.

## Consequences

- **Every rule of `DecisionDriven.Analyzers` is on as an error**, not only DD0001. That includes, among others, `[Contract]` on every public interface (DD0009), the contract type vocabulary (DD0010), no naked primitives on contract and model surfaces (DD0013), and no mutable static state (DD0004). This is ADR-0002 D3's intent. None fires on the slice 0 skeleton. **The first public interface needs a decision set in the package's format** (YAML front matter in a markdown file, read through `DdLedgerDirectory`), so that `[Contract]` can cite it. That is a question for the slice that adds the interface.
- The register is a gate in the build, not a convention. Its failing cases are permanent tests (`BuildPolicyTests`), built on every run.
- A new package needs an ADR and a `PackageVersion` entry with `Adr=`, and `Scope=` when it is test-only. `AGENTS.md` states the rule.
