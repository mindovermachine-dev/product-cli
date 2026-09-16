# Separating what a model did from what judged it

A small .NET library for a pattern worth having wherever models do work:

> **Save the run. Judge it afterwards, separately, with a different model — and
> record who judged, what they saw, and when.**

Built on [Microsoft.Extensions.AI.Evaluation][mseval] for its `IEvaluator` and
metric types, and designed to sit under an agent built on the
[Microsoft Agent Framework][maf]. It takes one NuGet package and knows nothing
about the repo it happens to live in.

[mseval]: https://learn.microsoft.com/dotnet/ai/conceptual/evaluation-libraries
[maf]: https://learn.microsoft.com/agent-framework/

## Why bother

The usual shape is to evaluate inline: get a response, score it, log the score.
It is easy, and it quietly destroys the two things that make evaluation useful
later.

**It merges measurement with opinion.** Some of what you want to record is
arithmetic — how long it took, how much a reviewer changed. Anyone can
recompute it from the same inputs, so it needs no attribution. The rest is a
model's *judgment*, which is an opinion held on one occasion. File them
together and the second borrows the first's standing: a reader cannot tell
which numbers were recorded and which were asserted.

**It merges the executor with the judge.** A model scoring its own output
inherits its own blind spots, and you cannot tell afterwards whether a good
score meant good work or a generous marker. Asking a *different* model, *later*,
makes the disagreements visible — and the disagreements are the signal.

So: two records, two moments, two configurations.

```
run                                    judge (later, separately)
────────────────────────────────       ──────────────────────────────────
model does the work                    a different model reads the record
  ↓                                      ↓
runs/<id>.json                         judgements/<run>/<judge>.<ctx>.json
  what it did                            who judged, what they saw, when
  deterministic metrics only             the verdict
```

## What goes in a run record

```json
{
  "form": "eval.run-record.v1",
  "id": "01M2MMTY7CY42T4BFJ5QDHAEW7",
  "tool": "spec-flow",
  "task": "act/settle-a-basket",
  "subject": "item-totals-v3",
  "model": "qwen3-coder-30b-a3b-instruct",
  "endpoint_host": "api.scaleway.ai",
  "duration_ms": 112848,
  "proposed": ["det/basket-rounding-is-half-even"],
  "kept": ["det/basket-rounding-is-half-even"],
  "reply": "…",
  "metrics": [ … ]
}
```

**`proposed` and `kept` are the pair to steal even if you take nothing else.**
What the model put forward, and what a person kept of it. The distance between
them is a human judgment on model output, collected on every run, supplied by
no model and costing nothing. Most teams have this information passing through
a review UI and throw it away.

A pipeline with no human in the loop writes them equal, and says so by their
being equal rather than by omitting the field.

**The reply is kept whole**, because a run that proposed nothing is the
interesting case and a character count cannot tell a model that answered
"nothing" from one whose answer could not be parsed. Both happened here, and
only one was a bug.

**The endpoint is kept as a host and the key is never kept.** A store that
accumulates credentials is one nobody can share.

## What goes in a judgment

```json
{
  "form": "eval.judgement.v1",
  "judges": "01M2MMTY7CY42T4BFJ5QDHAEW7",
  "judge": { "model": "mistral-medium-3.5-128b", "identity": "model:mistral-medium-3.5-128b" },
  "context": { "digest": "sha256:b737930b…", "shown": { "act_settles": "…", "proposed": "…" } },
  "verdicts": [ { "name": "Determination warrant", "value": "4/5", "reason": "…" } ],
  "ratifies_nothing": true
}
```

Three things or it is not written: **who judged, what they saw, and when.** Any
two leave a number that reads as fact and cannot be checked.

**The context is pinned before the model is asked anything** — a digest over
the exact inputs, stored beside them. Recompute it later and a mismatch tells
you the verdict was about a different state. Without this, a verdict six months
old is uninterpretable, because you cannot reconstruct what the judge could see.

**One run keeps every opinion of it.** Judgments are filed by judge and context
digest, so a second model lands beside the first rather than replacing it. Two
judges disagreeing over one pinned context is the outcome you want; a single
verdict per run is one nobody can argue with.

Here, two models over the same pinned context returned **4/5** and **5/5**, the
first naming one proposal as describing the work rather than settling a
question. That disagreement is the whole return on the pattern.

**`ratifies_nothing` is always true.** A judgment is evidence a person may read
before deciding. It decides nothing, it gates nothing, and there is no
principal field for a machine to fill.

## Comparing runs

A single run cannot be graded where the predicate is open. What *can* be read is
behaviour across runs — but only if runs carry coordinates, which is why a run
may declare two pinned sets:

```csharp
await RunObservation.ObserveAsync(store, run,
    address:     Pinned.Of(("task", ticket.Id), ("ground", whatTheSpecSettles)),
    arrangement: Pinned.Of(("model", modelId), ("endpoint_host", host)));
```

```csharp
foreach (var reading in Behaviour.Read(store.ReadRuns()))
{
    // reading.Agreement          — the same worker, asked twice
    // reading.AcrossArrangements — a different worker, same question
    // reading.Drift              — the gap between them
}

Behaviour.Collapsed(store.ReadRuns());  // two questions, one answer
```

| Held fixed | Varied | Reads as |
|---|---|---|
| address, arrangement | — | run-to-run variance |
| address | arrangement | drift in the arrangement, not the world |

**Keep the worker out of the address.** It is the thing you want to vary. And
keep the address coarse enough to repeat: an address containing a per-attempt
label gives every run its own coordinates, compares nothing, and looks like it
is working.

**Neither reading is a verdict on any single run.** A shift at fixed
coordinates says the ground moved or something undeclared got resolved — not
that one act was wrong.

## Declaring before acting

The forbidden state is a resolution nobody can be shown to have held. Catching
it needs one thing: a worker that says what it is about to do *before* it does
it, so that what it did can contradict what it said.

```csharp
// A separate turn, before the work. Asked afterwards it is a summary,
// and a summary cannot be contradicted by the run it summarises.
var declared = new Declaration(
    Decision: "resolve the basket total against what the act settles",
    Ground:   ["spec_check", "spec_records"]);

// Tolerance and assurance are the arrangement's, never the worker's.
declared = declared.BoundedBy(tolerance, assurance);

foreach (var finding in Escape.Check(run))
{
    // undeclared · incomplete-declaration · declared-but-unread
    // read-but-undeclared · unattributed-claim
}
```

`GroundRead` is **observed, not asked for** — the tools actually invoked, not
the worker's account of what it consulted. The two disagreeing is the finding.

This is the check that earns its keep. On a live run one model declared it
needed the act's text and never read it: `declared-but-unread`, caught
mechanically, on work that otherwise looked fine.

**Findings, not a gate.** Whether an escape is tolerable is a judgement, and a
judgement needs an owner; nothing here decides for them.

## Using it

```csharp
// 1. During the run — map your own shapes onto the pattern's vocabulary.
var store = new EvalStore(Backend.FromEnvironment(".eval").Open());

await RunObservation.ObserveAsync(store, new ObservedRun(
    Id: correlationId,
    Tool: "my-pipeline",
    Task: ticket.Id,
    Subject: slice.Name,
    Model: modelId,
    Endpoint: endpoint,
    Duration: stopwatch.Elapsed,
    Proposed: whatTheModelSuggested,
    Kept: whatTheReviewerKept,
    Reply: response.Text ?? ""));

// 2. Later, separately, with a different model.
var judgement = new Judgement(
    Judgement.FormV1, runId, DateTimeOffset.UtcNow,
    new Judge(judgeModel, host), JudgementContext.Pin(whatToShow), verdicts);

store.WriteJudgement(judgement);
```

`ObservedRun` names nothing from any particular tool, which is what lets one
store hold runs from an agent workflow, a batch job and a one-shot script
without any of them knowing about the others.

### Where it is stored

`EVAL_STORE` takes a path, or `azure:<account>/<container>[/<prefix>]`. The
layout is key-shaped on purpose — `runs/<id>.json` is a file path on disk and a
blob name in object storage without changing a character — so `IBlobs` is the
whole seam, three methods. Start on disk; move to a container when you want the
history shared, and nothing above that line changes.

A backend named but not built **refuses rather than falling back to disk**: a
tool that silently writes somewhere other than where it was told loses exactly
the run nobody knows to look for. Azure is declared and not yet implemented
here — it is three methods, and the interface is the contract.

### The deterministic evaluators

Three ship, and **none of them asks a model anything**:

| Metric | What it observes |
|---|---|
| Draft coherence | whether the reply's prose agrees with the structured answer it ended on |
| Reviewer amendment | Jaccard distance between `proposed` and `kept` |
| Determination shape | whether the proposed addresses are well-formed |

They are `IEvaluator`s, so a judged evaluator from
`Microsoft.Extensions.AI.Evaluation.Quality` composes beside them — but think
before you do. An LLM-judged metric written into a run record is an opinion
filed among facts, which is the thing this pattern exists to prevent. Put it in
a judgment instead.

Draft coherence exists because a live model wrote *"I did not need to make any
determinations"* and then listed one. The reviewer was shown a proposal the
reply said was never made.

## On Microsoft.Extensions.AI.Evaluation.Reporting

`Reporting` offers `IEvaluationResultStore` with disk and Azure Blob backends
and an HTML report writer, and its key triple maps onto this pattern almost
exactly:

| Reporting | Here |
|---|---|
| `ScenarioName` | what was attempted |
| `IterationName` | the run |
| `ExecutionName` | **the occasion** — `build@…` vs `judge:<model>@…` |

Executing and judging become two *executions* over one scenario and iteration,
which is a genuinely elegant fit. Two caveats before adopting it wholesale:

- Its grain is **inline**: `ReportingConfiguration.CreateScenarioRunAsync` →
  `ScenarioRun.EvaluateAsync` → flush on dispose. Deferred judging works
  against `IEvaluationResultStore` directly, but you are using half the package
  against its documented shape.
- It is **.NET only**. If anything in your stack that runs models is not .NET,
  the store cannot be the shared home. That is why the format here is a
  [document](../docs/eval-format-v1.md) with two implementations rather than a
  library with one.

Projecting these records to `ScenarioRunResult` to get the HTML report is a
reasonable thing to add on top. Making it the store is not.

## Honest limits

- **Azure is declared, not built.** `IBlobs` × 3.
- **No aggregation.** A store is per-repo; reading several together is a
  separate question the flat-JSON-under-stable-keys layout is meant to keep
  answerable without changing the format.
- **A judge needs enough context to answer.** Asked without the specification's
  own text, two different models each correctly replied that nothing could be
  warranted. An unanswerable question gets an honest refusal, which looks like
  a broken evaluator until you read the reason.
- **Nothing here says the judge is right.** It says who said it and what they
  were looking at, which is the part you cannot reconstruct later.

## Layout

```
src/Eval/
  RunRecord.cs        the run, and its metrics
  Judgement.cs        the judgment, its judge, and the pinned context
  Evaluators.cs       three deterministic IEvaluators
  Judging.cs          asking a model, and reading what it said
  RunObservation.cs   ObservedRun → a filed record
  EvalStore.cs        the key layout
  Blobs.cs            the backend seam: put / get / list
  Backend.cs          which backend, from configuration
```

[`../docs/eval-format-v1.md`](../docs/eval-format-v1.md) is normative. The Rust
implementation is `../eval-core`; both are held to the same digest fixture, so
neither can drift from the document alone.

Build and test:

```bash
dotnet test eval-dotnet/Eval.slnx
```
