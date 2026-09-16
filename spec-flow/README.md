# spec-flow — the agent host

The delegable half of the specification flow, built on the
[Microsoft Agent Framework](https://learn.microsoft.com/agent-framework/)
(`Microsoft.Agents.AI` 1.21.0). The other half is the Rust `spec` binary
(`spec-core` / `spec-cli`), and the split between them is not a packaging
decision — it is the accountability boundary of the flow itself.

## The split

| Verb | Where | Why |
|---|---|---|
| `import` | **.NET, here** | Roslyn scan → `.spec/inventory.json`; re-derivable, so delegating it is straightforwardly good |
| `implement` | **.NET, here** | builds a slice; produces a **pending** record |
| `candidates`, `map`, `check` | Rust | read-only over the store; the gate is a closed class set |
| **`accept`**, **`reject`**, **`close`**, **`policy set`** | **Rust only** | each names a principal, and a machine cannot be one |

## MCP

Two servers, one subset. Both withhold every verb that names a principal.

```bash
specflow mcp --root . --spec target/debug/spec     # .NET — the complete surface
spec-mcp .                                          # Rust — no .NET required
```

`specflow mcp` is the complete one because it adds **`spec_import`** natively.
Its five read tools are *proxied* to the `spec` binary rather than
reimplemented — one implementation of what the store means, so an MCP client
and a CI run cannot be told different things about the same repo. Tools carry
MCP annotations: `ReadOnly` on the reads, `Idempotent` on import.

Its boundary is the strong one. The assembly contains no code that writes a
closure, every call out goes through `SpecCli`, and `ForbiddenVerbs` throws on
a withheld verb. The Rust server's is weaker and says so: a registry boundary,
not a linkage one.

**The agent is an MCP client too.** `GovernedTools.ConnectAsync` connects the
slice-building `AIAgent` to `spec-mcp` and gives it those tools and nothing
else. An agent holding only governed surfaces cannot escape through
un-governed tooling; what it can still do is mismanage its own context, and
that is an honest limit rather than a gap. The client re-filters
`Server.Withheld` instead of trusting the server — a surface that trusts what
it is handed inherits the other side's next mistake.

A model may do everything up to the decision. It may not commit it.

## `import`

```bash
dotnet run --project spec-flow/src/SpecFlow.Cli -- import --root . --source ../their-repo
```

Syntax-first over parsed trees, with whatever references happen to resolve. It
does not need a restored, buildable project: an importer that only works on a
green build is one that does not run on the codebases most worth importing.

Recognised: controller routes (`[HttpGet]`/`[HttpPost]`/`[Route]`), minimal-api
maps, `IHostedService`/`BackgroundService`, `IConsumer`/`IRequestHandler` and
friends, `static Main`. A convention it misses shows up as a *missing* entry
point, never a wrong one — an under-reporting importer loses candidates, an
over-reporting one manufactures acts nobody has.

**It never guesses an act name.** Candidates carry observed transport fields
and the unfilled slots `name` and `settles`. A route's name accepted as an
act's name is how every act becomes an endpoint with a better label, which is
the whole reason the candidate vocabulary is graded measurement-only.

The inventory is a **projection**: rebuilt wholesale each run, carrying no
verdict and no ratification, and ordered so an unchanged codebase scans
byte-identically. Without that, every re-run would read as drift.

## Why the process boundary is the point

§14.2 of the PRD asks for the restriction to be *structural, not instructed* —
"a rule held in prose is not enforced, and an MCP surface told not to call a
verb is prose." A process boundary is the cheapest structural version of that:
the agent host does not link the code that writes a closure, so there is no
call it could make.

Three things hold it in place, each independently checkable:

1. `SpecCli.ForbiddenVerbs` throws when a `close` is assembled, wherever in the
   argument list it appears.
2. `BoundaryTests.No_public_member_of_the_flow_assembly_closes_a_record`
   reflects over the exported surface and fails if one ever appears.
3. The workflow graph has no edge to a closure — `HandOffExecutor` renders a
   command string and stops.

On the Rust side the same boundary is `S002`, which delegates to the ledger's
`Identity::model_or_bot_reason` — the identical test `L006` applies to an
acceptor. One identity law, two gates.

## Why a workflow rather than an agent loop

The g-track PRD (§7.8) ruled against ceding the loop to an agent runtime
because "the inner planner's context management and prompting are the
runtime's own — an arrangement whose absorbable forms and retrieval habits we
do not author."

MAF's **Workflows** layer does not have that problem. The graph is authored
here: executors, edges, and the order they run in are ours. The framework
supplies the superstep scheduler (deterministic traversal, type-checked edges
at `Build()`, checkpoints at superstep boundaries) and stays out of the
ordering. That is the part worth switching for; the agent layer is incidental.

The one place a model enters is `BuildSliceExecutor`, and its strategy is
*injected* — which model, which endpoint, which instructions stay arrangement
parameters, so the model ladder remains an experiment rather than a constant.

## The graph

```
ImplementRequest
   ↓
[open-record]    → `spec implement` opens the act-time record, first, before any work
   ↓ ActRecordOpened
[build-slice]    → the delegable half; an AIAgent, or anything else you inject
   ↓ SliceBuilt
[draft-closure]  → shapes what arose into a draft
   ↓ ClosureDraft
[closure-draft-review]  ← RequestPort<ClosureDraft, DraftReview>: the human-in-the-loop port
   ↓ DraftReview
[hand-off]       → yields the `spec close …` command. Renders it; never runs it.
```

The record opens *first* on purpose: a record opened after the work is a record
a crashed run never opens, and the write-back leg exists precisely because a
write that can be skipped will be skipped.

## Running it

Put the host beside the `spec` binary and one command name covers the flow:

```bash
cargo build -p spec-cli                       # the Rust half
dotnet publish spec-flow/src/SpecFlow.Cli -c Release -o target/debug

spec import                                   # → specflow import
spec build --slice checkout-totals --act act/settle-basket
```

`spec build` always exits **3** — work completed, closure pending. Then, as
yourself:

```bash
spec close <id> --principal you@example.com --nothing-arose
spec check                                     # 0 only once every record is closed
```

`spec import` and `spec build` are the only two verbs that leave the Rust
binary. Each launches `specflow`, which is found beside `spec`, or at
`SPECFLOW_BIN`, or on PATH — and which is handed the launching binary as its
`--spec` so the chain cannot resolve a different one. **Launching is not
linking:** the host still contains no code that writes a closure, `spec build`
reaches it as `specflow implement`, and the launcher refuses to forward a verb
that names a principal. The boundary is where it was; only the front door moved.

The host also stands alone, which is what the .NET tests drive:

```bash
dotnet test spec-flow/SpecFlow.slnx           # drives the real `spec` binary
dotnet run --project spec-flow/src/SpecFlow.Cli -- implement \
  --slice checkout-totals --act act/settle-basket --root . --spec target/debug/spec
```

To wire a model, set `SPECFLOW_MODEL_ENDPOINT` (any OpenAI-compatible endpoint
— Scaleway, Ollama, vLLM), plus `SPECFLOW_MODEL` and `SPECFLOW_MODEL_KEY`.

## Observing runs

Every model-backed `spec build` writes one record to `.spec/runs/<record-id>.json`
— named by the act-time record it observed, so the two join without an index.

**Measurement, never a verdict.** The journal is beside the store, not inside
it: not hashed, not signed, not closed, and `spec check` does not read it
(asserted in `spec-cli/tests/boundaries.rs`). No metric it keeps can fail
anything, and `Failed` is hard-coded false — a number nobody signed must not be
able to fail a build. The flow already has a place where judgment is filed
under a name, and this is deliberately not it.

The default evaluators are `IEvaluator`s from `Microsoft.Extensions.AI.Evaluation`,
but **none of them asks a model anything**:

| Metric | What it observes |
|---|---|
| Draft coherence | whether the reply's prose agrees with the array it ended on — a builder claiming it made no determinations while emitting one has contradicted itself, and the draft carries the contradiction to a reviewer as a proposal |
| Reviewer amendment | Jaccard distance between what was drafted and what the reviewer kept. 0 means the draft stood |
| Determination shape | whether the addresses are shaped `det/…`; one echoing the slice name is a diagnostic, not a failure |

These are arithmetic: anyone with the same inputs recomputes the same numbers,
which is why they can run inline and be trusted later without attribution.

**A judged evaluation is not one of these, and does not belong in this file.**
Asking a model to assess a run is itself an act, on an occasion, by a named
model, over a particular context — so it is pinned and recorded separately.
See *Judging a run* below.

**Reviewer amendment is the metric worth having.** It is the one grounded in a
human judgment: a person looked at a draft and said what they would actually
file. It costs nothing, it is collected on every run, and no model supplies it.

Both files follow [`docs/eval-format-v1.md`](../docs/eval-format-v1.md), shared
with the `eval-core` crate so one store can hold runs from tools in either
runtime. Where that store lives is configuration — `EVAL_STORE` takes a path,
or `azure:<account>/<container>[/<prefix>]`; unset, it is `.spec/` beside the
act store. The keys do not change with the backend, which is what makes moving
one a swap rather than a migration. A backend named but not built refuses
rather than falling back to disk.

The endpoint is kept as a host and the key is not kept at all, so a journal is
safe to share. It is **gitignored by default** all the same: a run record keeps
the model's reply verbatim, which is a description of your codebase, and that is
a decision to make deliberately rather than by not noticing. Un-ignore
`.spec/runs/` and `.spec/judgements/` when you want the history shared across a
team. Read it with `jq`:

```bash
jq -r '[.ran_at, .model, (.metrics[] | select(.name=="Reviewer amendment") | .value)] | @tsv' \
  .spec/runs/*.json
```

## Judging a run

**Asking a model to assess a run is itself an act**, so it is a separate verb
with its own record. It happens on its own occasion, under its own arrangement,
over a context pinned before the model is asked anything:

```bash
export SPECFLOW_JUDGE_ENDPOINT=https://api.scaleway.ai/v1
export SPECFLOW_JUDGE_MODEL=mistral-medium-3.5-128b
export SPECFLOW_JUDGE_KEY=…

spec judge <record-id>
```

Nothing defaults. With no judge configured the verb refuses rather than falling
back to the builder's model — a model marking its own work is the arrangement
least worth recording, so it is not the one you get by saying nothing.

Each judgment lands at `.spec/judgements/<record-id>/<model>.<context>.json`
and carries **who judged, what they saw, and when**:

```json
{
  "form": "spec.judgement.v1",
  "judges": "01M2MMTY7CY42T4BFJ5QDHAEW7",
  "judge": { "model": "mistral-medium-3.5-128b", "identity": "model:mistral-medium-3.5-128b" },
  "context": { "digest": "sha256:b737930b…", "shown": { "act_settles": "…", "drafted": "…" } },
  "verdicts": [ { "name": "Determination warrant", "value": "4/5", "reason": "…" } ],
  "ratifies_nothing": true
}
```

A verdict without those three is a number that reads as fact and cannot be
checked, so the record carries all of them or it is not written. The context
digest is what makes a verdict interpretable later: re-observe the run,
recompute, and a mismatch says the verdict was about a different state — the
move the policy's `basis_binds` makes, for the same reason.

**One run is judged more than once**, and the directory keeps every opinion: by
a second model, by the same model later, by a bigger one when the question turns
out to matter. Two judges over the same pinned context disagreeing is the signal,
not a fault — a single verdict per run would be one nobody could argue with.

**A judgment ratifies nothing.** There is no principal field and a machine could
not fill one: `S002` and `L006` say so on the other side of the seam. `spec check`
does not read `.spec/judgements/` any more than it reads `.spec/runs/`, asserted
in `spec-cli/tests/boundaries.rs`. It is evidence a person may read before
deciding; it decides nothing.

The judge is shown what the act settles, read back through `spec acts --id`
rather than parsed here — one reading of what the store means, or a judge and a
CI run can be told different things about the same act. It matters: asked
*without* the act text, two different models each answered that no determination
could be warranted, which was the right answer to an unanswerable question.

## The whole flow, end to end

```bash
cargo build -p spec-cli
scripts/checks/spec-flow-e2e.sh
```

Walks a throwaway repo through import → check (drift) → accept → reject →
implement → check (open record) → close → check (green), asserting the
refusals along the way: a machine cannot ratify, a machine cannot close,
`implement` exits 3. It runs in CI.

## Known gaps

- **Checkpointing is not wired.** MAF takes checkpoints at superstep
  boundaries, but only `CheckpointManager.CreateInMemory()` is documented for
  .NET — Python has a pluggable `CheckpointStorage`, .NET appears not to. It
  does not block this flow: the act-time record on disk *is* the durable pause,
  and it outlives the process by design. Wire checkpoints only for
  intra-run resilience on long builds, and expect to write the store.
- **Executor identity is fragile.** A checkpoint resumes only into a graph with
  matching executor identities, and ids assigned later do not repair earlier
  checkpoints. `ExecutorIds` holds logical-role constants for this reason; keep
  request and conversation ids out of them.
- **Signing says a key holder acted, not that the human did.** Custody,
  rotation and revocation are not modelled in v1, and the ledger's own
  `Acceptance.signature` is still empty at L0. See §4e of
  `docs/spec-flow-store-v1.md`.
