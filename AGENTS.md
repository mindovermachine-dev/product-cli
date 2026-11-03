# AGENTS.md — product-cli

The single source for agents working in this repository (ADR-0007). `CLAUDE.md`
and any other tool's instruction file point here and repeat nothing.

## What this repository is

The rewrite of product-cli: one Native AOT .NET binary that authors the What
of a product as a graph with shapes, simulates its behaviour, and exports one
canonical document for Writ to land. This repository holds **the tool, its
ADRs and its tests**, and its release descriptors (D1.24).

- `docs/decisions/` holds the ADRs, ADR-0001 onwards (below).
- `docs/traceability/` holds one record per agent session (ADR-0007 D2).
- `releases/` holds the release descriptors (ADR-0007 D3).
- `eng/gates/` holds the checks `.github/workflows/gates.yml` runs.

There is no source code yet. Slice 0 (ADR-0002, ADR-0003, ADR-0004) brings
the eight layer projects, the layering analyzer, the AOT publish gate and the
package register, in its own pull request.

**The Rust workspace is frozen at the tag `rust-final`** (commit `70395df`).
Nothing at the tag changes. Anything the rewrite reads from the Rust tool — the
lift input, the framework CLI scenarios, the behaviour of the old crates, the
`.decisions/` ledger that `writ import` reads once — is read there.

## Where the definition lives

**The definition is the authority**, not summarised accurately anywhere,
including here: decision-driven-design,
[`applications/sdlc/product-cli/product-definition.md`](https://github.com/mindovermachine-dev/decision-driven-design/blob/main/applications/sdlc/product-cli/product-definition.md).
Read it before proposing anything structural. Chapters 1 to 6 are the product;
chapter 7 is how it is built and its order of work.

**Outside-in crosses two repositories** (D1.24, ADR-0001 D4). A change to the
surface, a format, a guarantee or a contract is a pull request to the
definition in decision-driven-design first, ratified by merge there. The
product-cli change that relies on it comes after, and cites that merged
commit's SHA. A vocabulary change is an `sdlc:` edition requested in
decision-driven-design, never an extension made here.

## ADRs

**"ADR-NNNN" means `docs/decisions/ADR-NNNN-<slug>.md` on `main`.** The Rust
workspace had its own ADR-0001 to ADR-0005; cite those as
`rust-final:conventions/adr/ADR-NNNN`, never by the bare number.

Until Writ #7 exists, the ADR files are authoritative by review (D1.22, the
interim rule of Writ ADR 0008); `docs/decisions/README.md` states the rule.

- Each ADR has a number, a status, Context, Decision as a numbered set (D1,
  D2, …), Alternatives considered, and Consequences. `adr-shape` checks it.
- **An accepted ADR is not edited.** A proposal that contradicts one says so
  and supersedes it, naming it (ADR-0001 D3). A dated note may be added under
  its status lines when the definition moves under it, as the Draft 4 notes
  were.
- ADR-0001 to ADR-0012 are held byte for byte against
  `docs/decisions/collected-2026-11-03.md`; `adr-text` checks it.
- Every `PackageReference` carries `Adr=` naming the ADR that admits it
  (ADR-0004), from slice 0.

## Gates

`.github/workflows/gates.yml` runs on every pull request to `main`. Each check
is a Bash script in `eng/gates/` and runs locally the same way:

| Job | Script | Fails when |
|---|---|---|
| `commits` | `eng/gates/commits.sh <base> <head>` | a commit lacks `Refs #N` or a `Signed-off-by:` naming a person, or names a model anywhere in its message, author or committer |
| `signatures` | `eng/gates/signatures.sh <commits.json>` | GitHub does not report a commit of the pull request as verified |
| `adr-shape` | `eng/gates/adr-shape.sh [dir]` | an ADR lacks its number, status, alternatives or consequences, or an unexpected file is in `docs/decisions/` |
| `adr-text` | `eng/gates/adr-text.sh [dir]` | ADR-0001 to ADR-0012 differ by a byte from their sections of the collected file, with the Draft 4 notes |
| `no-toolchain` | `eng/gates/no-toolchain.sh [root]` | a Rust, Node or Python toolchain file is at the root |
| `selftest` | `eng/gates/selftest.sh` | any check above passes on the case planted for it |

`selftest` plants a failing case for every check in a scratch repository on
the runner, so each run shows that each check still fails when it should.

```bash
eng/gates/selftest.sh
eng/gates/commits.sh origin/main HEAD
eng/gates/adr-shape.sh && eng/gates/adr-text.sh && eng/gates/no-toolchain.sh
```

**No rule is suppressed** (ADR-0007 D6). A check that is wrong is changed, by
ADR where it encodes one, and its plant in `selftest` changes with it.

## Every commit

Conventional commits, one logical change each (ADR-0007 D1):

- **Signed**, and verified by GitHub. A session signs as its agent identity,
  `Claude (Emil) <claude+emil@okkels-klein.dk>`, with the key its environment
  carries.
- **`Refs #N`** in the body. No issue covers the work? Open one first.
- **`Signed-off-by:`**, the DCO, naming the responsible human:
  `Signed-off-by: Emil Okkels Klein <emil@okkels-klein.dk>`.
- **No model identifier anywhere**: not in the message, not in a trailer
  (`Co-Authored-By:` included), not in the author or committer name or email.
  A `Claude-Session:` trailer is a session link and names no model.

## Every session

- **Plan first, and wait for approval.** Read the definition and the ADRs
  before proposing anything structural.
- **File a record** in `docs/traceability/`, named
  `YYYY-MM-DD-issue-N-<slug>.md`, in Writ's shape: the prompt, the tool, what
  was read, the approved plan, the commits, the checks run and their results,
  and any deviation from the plan with its reason (ADR-0007 D2). The model is
  written "not recorded (ADR-0007 D1)".
- **One pull request** per slice or ADR change, with a release descriptor
  (ADR-0007 D3).
- **`main` is fast-forwarded only by the maintainer's merge**, at an approved,
  checked head (ADR-0007 D4). Once Writ can land, through `writ land`.
- **Parallel sessions:** whichever lands second rebases, re-runs every check,
  and records the reconciliation in its record (ADR-0007 D5).

## Tests

**A guarantee test observes bytes, never the return value of the verb under
test** (ADR-0006): the draft log read back at a position, an export's bytes,
the runner's stdout, or the `dpp/1` records. A test of a refused write asserts
that the log's head and its bytes are unchanged. A test that asserts only on a
verb's result is rejected in review.

## Never

- **Anything beyond the What**: the How, work units, features, releases,
  delivery targets, or code generation of any kind (chapter 8).
- **Authority**: a key, a grant, `land`, or an acceptance under a grant, ever
  (W10, chapter 8).
- **Auto-adoption**, under any setting, or an adopt or reject verb on the agent
  surface (W12, ADR-0012 D7).
- **An extension of `sdlc:` here.** The vocabulary, its shapes and its rules
  are decision-driven-design's (D1.1, D1.9).
- **A runtime fetch of the edition**, or an edited copy of its files; the
  binary embeds them byte for byte, hash-checked at build (D1.20).
- **Product-specific code in Varve, or generic RDF here**: a missing generic
  capability is a Varve issue (ADR-0002 D5). **A reference to Writ's code**
  (chapter 6).
- **A Node toolchain**, or any toolchain but .NET: 100% managed .NET, no
  native dependency (chapter 7).
- **Reflection-based serialisation**, or reflection, `Reflection.Emit` or
  `dynamic` in shipped code (ADR-0003 D4).
- **A suppression of any rule** (ADR-0007 D6).
- **Copied Rust code.** The Rust tool is read at `rust-final` for its
  behaviour, and its bytes are never carried.
- **A package without `Adr=`**, or a version taken from memory (ADR-0004).
- **A commit** unsigned, without `Refs #N` or a DCO sign-off, or naming a model.
- **Rewriting or deleting a tag or a branch**, force-pushing `main`, or
  changing anything at `rust-final`.
- **An acceptance written by a session**: a session never writes
  `accepted-by`, never merges, and never tags.
