# D1.24: the Rust tree leaves `main`

> **Recorded contemporaneously**, by the session that did the work, under
> ADR-0007 D2, in the shape of Writ's `docs/traceability/` records (Writ ADR
> 0008). The maintainer's messages are verbatim except where the attached file
> is named rather than inlined. The transcript is held by the maintainer.

| | |
|---|---|
| **Issue** | [#165](https://github.com/mindovermachine-dev/product-cli/issues/165), opened by this session |
| **Date** | 2026-11-03 |
| **Tool** | Claude Code 2.1.317, a cloud session |
| **Model** | not recorded (ADR-0007 D1) |
| **Session identifier** | `session_01NbWSYSVJASSN4tw4nMJ9kU` |
| **Branch** | `claude/lucid-hamilton-jxk6pv`, from `main` at `70395df` (the commit `rust-final` names) |
| **Identity** | `Claude (Emil) <claude+emil@okkels-klein.dk>`, SSH-signed with key `SHA256:xDu5EyJd05ROum1g+8Aj1Up3MjE6RbHnI70/xNHljDY` |

## The prompts

The first message attached `product-cli-adrs.md`, the collected text of
ADR-0001 to ADR-0012, and read:

> Session prompt: the first rewrite PR, removing the Rust tree from `main` (D1.24)
> You work in `mindovermachine-dev/product-cli`. This session holds Emil's signing key. Every commit and tag you make is signed and must verify on GitHub.
> Stop and ask before you start if any of these is false:
>
> * the tag `rust-final` exists on GitHub at `70395dfe317cb9a00665433813766ca763f3c58a` and verifies;
> * `origin/main` is still at `70395df`. If it has moved, stop: something landed on the Rust tree after the freeze;
> * you have `product-cli-adrs.md`, the collected text of `ADR-0001` to `ADR-0012`, from Emil or from the Product-Cli project (`claude/product-cli-adrs.md`). No repository holds them. Do not write or reconstruct any ADR.
>
> Read first
>
> 1. decision-driven-design `main`, `applications/sdlc/product-cli/product-definition.md`, Draft 4. Read all of it, and above all:
>    * D1.22, governance until Writ #7;
>    * D1.24, where it lives, which this PR carries out;
>    * chapter 7, the order of work.
> 2. Writ ADR 0008, the commit traceability and session pattern, and Writ's `docs/traceability/` records, for the shape of a session record.
> 3. Varve's `AGENTS.md`, `eng/` and `releases/`, for the build policy, the commit gates and the release descriptor. Adopt the pattern (ADR-0007). Copy nothing that names Varve's own packages.
> 4. This repository at `70395df`: the top-level tree, `.github/workflows/`, `LICENSE`, `README.md`, `CLAUDE.md`. Read it only to know what you are removing.
>
> Plan first
> Open the tracking issue and post your plan as a comment on it. Wait for Emil's approval. Write no files before then.
>
> * The issue: "D1.24: remove the Rust tree from `main`". The body links D1.24 and the tag.
> * The plan names:
>    * every path you remove and every path you add;
>    * the licence change;
>    * the CI you keep, which is none, and the CI you add;
>    * the issue and PR disposition list (deliverable 5).
>
> Deliverables (one PR, `Refs #<issue>`)
> 1. Remove the Rust tree. After this PR, `main` contains only what deliverable 2 adds. Remove:
>
> * every crate and workspace file: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `deny.toml`, `dist-workspace.toml`, `.cargo/`, `xtask/`, `dagger/`;
> * `schema/`, `spec/`, `conventions/`, `examples/`, `scripts/`, `docs/`;
> * `.product/`, `.decisions/`, `.ddd/`, `.claude/`, `.mcp.json`, `.htmlvalidate.json`, `.stylelintrc.json`;
> * `.github/workflows/`: `product-ci.yml`, `publish-mcp.yml`, `release.yml`;
> * `CLAUDE.md`. It is replaced in deliverable 2.
>
> Everything removed stays readable at `rust-final`, and the PR description says so. Nothing is moved, renamed or edited first: the removal is one commit, separate from the additions.
> 2. Add, in separate commits:
>
> * `AGENTS.md`, the single source for agents (ADR-0007). It covers:
>    * what this repository is: the tool, its ADRs and tests;
>    * where the definition lives, and the rule that a definition change merges in decision-driven-design first, with the commit cited here;
>    * the gates;
>    * the commit rules: signed, `Refs #N`, a DCO sign-off, and no model identifier anywhere, including `Co-Authored-By` trailers and author names;
>    * the never list, from the project prompt's constraints.
> * `CLAUDE.md`: one line that points to `AGENTS.md`.
> * `README.md`:
>    * what the rewrite is, in two paragraphs from the definition's chapter 1;
>    * that the Rust tool is frozen at `rust-final`, and its releases keep working;
>    * a link to the definition.
> * `LICENSE`: Apache-2.0, as the definition requires, replacing MIT. Add a `NOTICE`. The MIT text stays at the tag and governs everything there. State this in the PR. Emil confirms the relicensing in review, since he is the copyright holder named in the MIT file.
> * `docs/decisions/`: `ADR-0001` to `ADR-0012`, split from `product-cli-adrs.md`, one file per ADR, under the interim rule of D1.22.
>    * The file names are:
>       * `ADR-0001-decisions-in-embedded-writ.md`
>       * `ADR-0002-layers.md`
>       * `ADR-0003-native-aot-single-binary.md`
>       * `ADR-0004-package-policy.md`
>       * `ADR-0005-node-kind-source-generator.md`
>       * `ADR-0006-tests-observe-bytes.md`
>       * `ADR-0007-commit-discipline.md`
>       * `ADR-0008-exporters-over-canonical-graph.md`
>       * `ADR-0009-rust-migration-acceptance.md`
>       * `ADR-0010-workbench-blazor-webassembly.md`
>       * `ADR-0011-proposals-and-dpp-decision-records.md`
>       * `ADR-0012-adopting-on-the-fly.md`
>    * Each file is its section of the collected file, with the heading promoted to `#` and its subheadings to `##`. The text is otherwise byte for byte.
>    * A test, or a step in `gates.yml`, checks that each file's body equals its section of the collected file, with the notes removed and the heading levels restored. The collected file is committed at `docs/decisions/collected-2026-11-03.md` as that check's source.
>    * Add a `README.md` that states the interim rule: review keeps ADR discipline, `writ import` reads these files once Writ #7 exists, and every package entry carries `Adr=`.
>    * ADR-0001, ADR-0008, ADR-0009 and ADR-0011 each get the note that the collected file's preamble gives for them, as a blockquote under the status lines, headed "Note (2026-11-03, Draft 4)". Change nothing else in them.
> * `docs/traceability/<date>-issue-<n>-rust-removal.md`: this session's record, in Writ's shape.
> * `releases/<descriptor>`: the release descriptor, in Varve's shape. It names:
>    * `rust-final`, and that no Rust release follows;
>    * the sdlc edition this repository will pin. It is not yet tagged, so it is stated as pending.
> * `.github/workflows/`: one workflow, `gates.yml`, that checks real things on every PR:
>    * every commit is signed, carries `Refs #N` and a DCO sign-off, and names no model;
>    * every file under `docs/decisions/` has the ADR shape: number, status, alternatives, consequences;
>    * no Rust, Node or Python toolchain file exists at the root.
> Each check must fail on a planted case. Show the failing runs in the PR. Add no .NET solution, project or SDK pin. Those come in slice 0 (ADR-0002, ADR-0003, ADR-0004): the eight projects, the layering analyzer, the AOT publish gate and the package register, in their own PR after this one.
>
> 3. Gate.
>
> * `gates.yml` is green on the PR head.
> * Every commit verifies on GitHub; check each SHA's verification through the API and paste the results in the PR.
> * `git ls-tree -r --name-only HEAD` lists only the added files.
>
> 4. In decision-driven-design: a second PR, one commit, signed. It updates the definition's cross-repository table:
>
> * the `rust-final` row becomes met, with the tag object's SHA;
> * add the row "the Rust tree removed from `main`", met with this PR's merge SHA. If this PR is not yet merged, mark it "in review" with the PR number and update the row after the merge;
> * the `README.md` row's state stays as it is.
>
> The same PR adds step 0 to chapter 7's order of work, as a new first item, verbatim:
>
> 0. `[D1.25]` (2026-11-03) Slice 0, the repository skeleton (ADR-0002, ADR-0003, ADR-0004):
>    * the eight layer projects, each with its public API baseline and a test that the layering analyzer refuses an upward reference;
>    * the Native AOT publish of the host on every supported runtime in CI, with a smoke test of the published binary;
>    * the package register: Varve, the analyzers, and the test-only packages.
>  It needs no `sdlc:` edition, so it does not wait for step 1's upstream.
>
> In the header, "D1.16 to D1.24" becomes "D1.16 to D1.25", and the sentence on where D1.24 comes from gains "D1.25 adds step 0 to the order of work".
> Follow decision-driven-design's `AGENTS.md`: a `Basis: DDD-dec-72` line, and `python3 scripts/ci.py` green. Do not merge: Emil ratifies by merging.
> 5. Disposition of the Rust tool's open work, as a list in the PR description, executed only after approval:
>
> * Open PRs #143 and #154 to #164 (the namespace work on the Rust ledger): close them unmerged, each with a comment that cites D1.24 and `rust-final`. The ledger is Writ's authority graph now. Delete no branch.
> * Open issues, about thirty of them: list each with a proposed disposition.
>    * Close as not carried, citing D1.24 and the definition's chapter 8, any issue that concerns:
>       * the ledger, keys or namespaces;
>       * `ddd`, `spec`, `eval`, codegen, design systems or the How;
>       * the Rust build.
>    * Propose re-filing, never re-filing yourself, an issue that names a What concern the rewrite still has, for example:
>       * #58: Decider branching outcomes;
>       * #59: value objects as elements.
> Say which chapter 7 step it would belong to.
>
> Rules
>
> * Commit identity: `Claude (Emil) <claude+emil@okkels-klein.dk>`, signed with this session's key.
> * Each commit carries `Refs #<issue>` and `Signed-off-by:`, and names no model, including in trailers.
> * No suppression of any check.
> * Nothing is written to Varve or Writ.
> * Do not rewrite or delete any tag or branch.
> * Do not force-push `main`.
> * `main` is fast-forwarded only by Emil's merge.
>
> Done when
>
> * The PR is approved and merged by Emil.
> * The decision-driven-design table PR is open with its row in its final state.
> * The disposition list has been executed as approved.
>
> Report:
>
> * the merge SHA;
> * each commit's verification;
> * the decision-driven-design PR number;
> * what you closed and what you proposed to re-file.
>
> Reconciliation
> If anything lands on `main` in this repository while you work, stop and ask. The tree is frozen, and a landing means someone else is working on it.

The session checked the preconditions, read, opened #165 and posted its
[plan](https://github.com/mindovermachine-dev/product-cli/issues/165#issuecomment-6600913057)
with five questions. The second message answered:

> Approved, with these answers and three additions.
>
> **Answers**
> 1. Never list: build it from the sources you listed, plus these project constraints:
>    - the What alone: no How, work units, features, releases, delivery targets or code generation;
>    - no key, grant, `land` or acceptance under a grant, ever;
>    - no auto-adopt, and no adopt verb on the agent surface;
>    - no extension of `sdlc:` here;
>    - no product-specific code in Varve, and no reference to Writ's code;
>    - no Node toolchain;
>    - no suppression of rules;
>    - no reflection-based serialisation;
>    - no runtime fetch of the edition;
>    - no copying of Rust code.
> 2. Model field: write "not recorded (ADR-0007 D1)" and nothing else. I won't fill it in either.
> 3. No `planted/*` branches. The `selftest` job proves on every run that each check fails on its plant, which is stronger evidence than red runs shown once, and it keeps fake model ids and unverified commits out of the repository. Paste one `selftest` run in the PR.
> 4. `v0.8.0-preview.1`. `1.0` is for the release at which the Rust binary retires.
> 5. Agreed: open the decision-driven-design PR after my merge here, as one commit with the row met.
>
> **Additions**
> - `AGENTS.md`, ADR numbering: "ADR-NNNN" means `docs/decisions/` on `main`. The Rust workspace's own ADR-0001 to ADR-0005 are cited as `rust-final:conventions/adr/ADR-NNNN`.
> - PR description: `docs/decisions/hafeok.ddd.nt` and `hafeok.ledger.nt` are the Rust ledger's exports. They leave with the tree, and `writ import` reads the ledger at `rust-final`, so no decision is lost.
> - The decision-driven-design PR also corrects D1.21. In "The lift map", replace the "Attribute blank nodes" bullet with:
>
>   > - **Blank nodes.** All 32 become elements (P-3), each keyed by (owning subject IRI, predicate, its own description, sorted). The key is unique over the frozen input.
>   >   - The 15 `pf:hasAttribute` nodes become `sdlc:Field` elements: `attrName` becomes `rdfs:label`, and `attrType` becomes `sdlc:fieldType`.
>   >   - The 7 `pf:offers` and 10 `pf:surfaces` nodes become `sdlc:Interaction` elements within their UI step.
>   > - **Stored, not minted.** The ULIDs and the adopted wire names are stored in the map, each wire name with the reference to the review that adopted it. Lifting again therefore reproduces the golden bytes.

## What was read

- decision-driven-design `fcda5ed` (`main`):
  `applications/sdlc/product-cli/product-definition.md`, Draft 4, whole;
  `AGENTS.md`; `LICENSING.md`; the edition's `owl:versionIRI` in
  `applications/sdlc/what/ontology.ttl`.
- Writ `6737ac9`: ADR 0008, `docs/traceability/README.md` and the record of
  2026-10-28; `LICENSE` and `NOTICE`.
- Varve `3024c7a`: `AGENTS.md`, `eng/` (listing and `identities.json`),
  `releases/` and `docs/releases.md` §1 to §3; `NOTICE`.
- This repository at `70395df`: the top-level tree (1,939 tracked files),
  `.github/`, `LICENSE`, `README.md`, `CLAUDE.md`; the open issues and pull
  requests; #58 and #59 in full.

## What the session found

- **Preconditions held.** `rust-final` is tag object
  `d0cf1f1ec40eb55c271b8d4dadfefcd260c03312` on
  `70395dfe317cb9a00665433813766ca763f3c58a`, verified by GitHub (`valid`);
  `origin/main` was `70395df` at the start and again before the first commit.
- **No `Cargo.lock`** is tracked at `70395df`, so the brief's entry for it
  removed nothing.
- **The definition names no licence.** The brief says Apache-2.0 is "as the
  definition requires", but Draft 4 at `fcda5ed` does not mention a licence,
  and decision-driven-design's own code is MPL-2.0 (`DDD-dec-79`). The first
  draft of `NOTICE` repeated the brief's claim; it was taken out before the
  branch was first pushed, and the question is in the pull request.
- **The environment's identity overrides `-c user.name`.** The cloud
  environment exports `GIT_AUTHOR_*` and `GIT_COMMITTER_*`, so the self-test's
  first author-name plant was in fact authored as the agent and passed. The
  self-test now sets the plant identity through those variables.
- **A plant can fail for the wrong reason.** In the self-test's first run
  every commit plant "failed" because `commits.sh` ran in the wrong
  repository. Each plant now names the finding it must produce, and fails the
  self-test if the check fails without it.

## What the session did

1. Opened #165 and posted the plan; after approval, removed the Rust tree in
   one commit and added the files below, one commit each.
2. Split the collected file with a throwaway script outside the repository,
   and checked the result with `adr-text`, which rebuilds the files by an
   independent route (awk) and agreed byte for byte.
3. Ran every gate locally on the branch, and the self-test (below).

## Commits

Each carries `Refs #165`, `Signed-off-by: Emil Okkels Klein
<emil@okkels-klein.dk>` and the `Claude-Session:` trailer, no co-author
trailer and no model identifier, is committed as the agent identity, and is
SSH-signed.

- `95bb957` chore: remove the Rust tree from main (D1.24)
- `875a513` docs: AGENTS.md, the single source for agents (ADR-0007)
- `48f5c9a` docs: CLAUDE.md points to AGENTS.md
- `ed5008d` docs: README.md for the rewrite
- `59553ee` licence: Apache-2.0 replaces MIT; add NOTICE
- `eb270d9` docs(decisions): the collected ADR text of 2026-11-03
- `cc827e0` docs(decisions): ADR-0001 to ADR-0012, split from the collected file
- `3fd396b` docs(decisions): the Draft 4 notes on ADR-0001, 0008, 0009 and 0011
- `d3bdf88` docs(decisions): README, the interim rule (D1.22)
- `2ffd36a` ci: gates.yml, the checks on every pull request
- `e14a897` releases: v0.8.0-preview.1, the rewrite begins
- this record

## Checks

Run locally on the branch before the first push:

- `eng/gates/commits.sh origin/main HEAD`: every commit passes.
- `eng/gates/adr-shape.sh`: 12 ADRs have the shape.
- `eng/gates/adr-text.sh`: 12 ADRs equal their sections of the collected file.
- `eng/gates/no-toolchain.sh`: no toolchain file at the root.
- `eng/gates/selftest.sh`: 5 clean cases pass and 31 plants fail, each with
  its finding.
- `cmp` of `docs/decisions/collected-2026-11-03.md` against the attachment:
  identical (sha256 `3dcf7e5b…c096faffd`).
- `cmp` of `LICENSE` against the apache.org text and Writ's `LICENSE`:
  identical.

The `gates.yml` runs on the pull request, each commit's verification read
through the API, and `git ls-tree` of the head are in the pull request's
description.

## Deviations from the plan

- **The self-test changed after the plan.** It binds each plant to its
  finding, and sets the plant identity through the environment (above). The
  plan's checks are otherwise as approved.
- **`NOTICE`** does not say the definition requires Apache-2.0, for the
  reason above.
- **The default commit attribution** of this environment adds a co-author
  trailer naming a model; it was left out, as the rules require.
