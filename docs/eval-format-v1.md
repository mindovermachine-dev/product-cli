# Evaluation store, format v1

**Normative.** This document is what an outside implementation imports. The
code follows it, not the other way round.

Two implementations follow it here: [`eval-core/`](../eval-core) in Rust and
[`eval-dotnet/`](../eval-dotnet) in .NET. The .NET one is standalone and is
meant to be lifted into other codebases — its
[README](../eval-dotnet/README.md) is the pattern writeup, where this file is
the wire format.

Two files, and the separation between them is the whole point.

- `runs/<id>.json` — **what a model did.** Written at the time, by whatever ran
  it, before anything has been assessed.
- `judgements/<run>/<judge>.<context>.json` — **what a model said about a run.**
  Written later, by a separately configured judge, over a pinned context.

Neither is authority. Neither names a principal. No gate reads either.

---

## 1. Why they are separate files

A run record is an observation: it says what happened. A judgment is an
opinion: it says what someone made of what happened.

Keeping them in one file lets the second borrow the first's standing. A reader
who finds a verdict inline with the facts has no way to tell which parts were
recorded and which were asserted, and the verdict is the part that is least
reliable — rerun the same judge over the same inputs tomorrow and it may say
something else.

**The model that executes is never the model that judges, and never judges at
the same time.** Executing is delegable and happens first; judging is a
separate act, separately arranged, afterwards. A tool that evaluates inline has
merged the two and cannot tell you which model was confident about what.

## 2. Store layout

```
<store-root>/
  runs/<id>.json                        # one run
  judgements/<run>/<judge>.<ctx>.json   # one opinion of one run
```

`<store-root>` is the tool's to choose — `.spec/` for the specification flow,
somewhere else for another tool. One subject per file, named by its own id.
Nothing is edited after it is written.

A judgment's file name carries the judge and the first 12 hex of its context
digest, so **one run keeps every opinion of it**: a second model, the same
model later, a bigger one when the question turns out to matter. Re-asking the
same judge the same question over the same context overwrites in place rather
than accreting copies; a genuinely new occasion differs in the judge or the
context and lands beside the first.

## 3. The run record

```json
{
  "form": "eval.run-record.v1",
  "id": "01M2MMTY7CY42T4BFJ5QDHAEW7",
  "tool": "spec-flow",
  "task": "act/settle-a-basket",
  "subject": "item-totals-v3",
  "ran_at": "2026-09-16T08:11:21.655Z",
  "model": "qwen3-coder-30b-a3b-instruct",
  "endpoint_host": "api.scaleway.ai",
  "duration_ms": 112848,
  "proposed": ["det/basket-rounding-is-half-even"],
  "kept": ["det/basket-rounding-is-half-even"],
  "reply": "…",
  "metrics": []
}
```

| Field | Meaning |
|---|---|
| `id` | the run's identity, supplied by the tool. Joins to whatever the tool already calls this piece of work |
| `tool` | which tool ran it, so one store can hold several |
| `task` | what was attempted — an address, not prose |
| `subject` | what it was attempted on |
| `model` / `endpoint_host` | the arrangement. **Host only. The key is never written** |
| `proposed` | what the model put forward |
| `kept` | what a person kept of it |
| `reply` | the model's reply, whole |
| `metrics` | deterministic observations (§5) |
| `address` | **where the run happened** — the coordinates it is comparable at (§8). Optional |
| `arrangement` | **what answered** — the worker and how it was reached (§8). Optional |

**`proposed` and `kept` are the pair worth having.** The distance between them
is a human judgment on model output, collected on every run, supplied by no
model and costing nothing. A tool with no human in the loop writes `kept` equal
to `proposed` and says so by their being equal — not by omitting the field.

**`reply` is kept whole.** A run that proposed nothing is the interesting case,
and a length cannot distinguish a model that answered "nothing" from one whose
answer could not be read. It is also why a store is gitignored by default: the
reply describes the codebase it ran against.

## 4. The judgment

```json
{
  "form": "eval.judgement.v1",
  "judges": "01M2MMTY7CY42T4BFJ5QDHAEW7",
  "judged_at": "2026-09-16T08:30:15.597Z",
  "judge": {
    "model": "mistral-medium-3.5-128b",
    "endpoint_host": "api.scaleway.ai",
    "identity": "model:mistral-medium-3.5-128b"
  },
  "context": {
    "digest": "sha256:b737930b59a3…",
    "shown": { "task": "act/settle-a-basket", "proposed": "det/…" }
  },
  "verdicts": [
    { "name": "Determination warrant", "value": "4/5", "reason": "…",
      "rating": "Average", "diagnostics": [] }
  ],
  "ratifies_nothing": true
}
```

A verdict is filed with **who gave it, what they saw, and when**, or it is not
filed. Any two of the three leave a number that reads as fact and cannot be
checked.

**`judge.identity` is `model:<model>`.** A machine is named as one. There is no
principal field and a machine could not fill one.

**`ratifies_nothing` is always `true`** and is carried rather than implied. A
reader who finds the file should not have to know the directory layout to learn
that nothing in it was decided by anyone.

**A judge that would not answer did not judge.** Nothing is filed for a timeout
or a refusal: a missing verdict is honest, an empty one recorded as though it
were an opinion is not.

## 5. Metrics and verdicts

Both use the same shape:

```json
{ "name": "…", "value": "…", "reason": "…", "rating": "…", "diagnostics": [] }
```

`value` is text whatever the measured type — a store that must be migrated
before it can be read is a store nobody reads. `rating` is one of `Unknown`,
`Inconclusive`, `Unacceptable`, `Poor`, `Average`, `Good`, `Exceptional`.

**Nothing here can fail a build.** A metric in a run record is arithmetic; a
verdict in a judgment is an opinion. Neither is signed, and a number nobody
signed must not gate. Implementations hold this by what their gate reads, not
by a rule someone remembers.

## 6. Pinning a context

The digest ties a verdict to the state that produced it. Recompute it from a
run record later, and a mismatch says the verdict was about something else.

Build the **body**: for each key in ascending code-point order, the key, then
`U+001F`, then the normalised value — entries joined by `"\n"`, with no leading
or trailing newline.

A value is normalised by folding `\r\n` and `\r` to `\n`, applying Unicode NFC,
then trimming ASCII whitespace from both ends. A key whose value normalises to
empty is omitted entirely, so *absent* and *present but empty* cannot pin
differently: a distinction no reader could act on is not one worth hashing.

The **canonical form** is then `eval.judgement-context.v1`, `"\n"`, body — an
empty body still keeps the newline. The digest is `"sha256:"` followed by the
lowercase hex SHA-256 of that form encoded as UTF-8.

This is the same domain-separated construction the decision ledger uses, under
its own prefix. An implementation in another runtime must reproduce it byte for
byte; the two in this repo are held to each other by a shared fixture.

The record stores the digest **and** what it digested, so a reader can
disbelieve it without reconstructing the run.

## 7. Where a store lives

The layout in §2 is **key-shaped on purpose**: `runs/<id>.json` is a file path
on disk and a blob name in object storage without changing a character. So a
backend is a configuration choice, and the records do not know which one they
landed in.

`EVAL_STORE` names it, and both runtimes read the same spelling:

| Value | Backend |
|---|---|
| a path | files under that directory |
| `azure:<account>/<container>[/<prefix>]` | blobs in that container |
| unset | the tool's default path |

A prefix lets one container hold several tools, which is how a repo starts
per-repo and ends up central without the format moving.

**A backend that is named but not built refuses.** It does not fall back to
disk: a tool that silently writes somewhere other than where it was told is
worse than one that stops, because the run it lost is the one nobody knows to
look for. An implementation therefore either supports a backend or says so by
name.

**A key never climbs out of its root.** `..` and empty segments are refused
rather than normalised.

## 8. Comparing runs: address and arrangement

A single run cannot be graded where the acceptance predicate is open. What can
be read is **behaviour**: whether the same question asked twice was answered the
same way, and whether two questions declared different were answered
identically.

That requires coordinates, so a run may declare two pinned sets (§7):

- **`address`** — where it happened. The task instance together with the ground
  the caller declared. Two runs at the same address were asked the same
  question; runs at different addresses are not comparable.
- **`arrangement`** — what answered. The worker version, the endpoint, whatever
  else was arranged.

They are separate so they can vary independently:

| Held fixed | Varied | What the difference reads as |
|---|---|---|
| address, arrangement | — | run-to-run variance |
| address | arrangement | drift in the arrangement, not in the world |
| — | address | nothing; unlike things are not compared |

**Both are optional, and their absence is information.** A caller that declares
no address can still record what it did; its runs simply stand alone, and a
reader learns that from the missing field rather than by guessing.

**The grain is the caller's to choose and is recorded rather than assumed.**
What counts as "the same question" is a judgement nobody else can make. Leaving
it implicit is how comparison quietly compares unlike things — and an address
too fine gives every run its own coordinates, which looks like success and
compares nothing.

Two readings follow, and **neither is a verdict on any single run**:

- **Drift.** An arrangement agreeing with itself more than with another means
  behaviour moved when the worker did. It is drift in the arrangement unless an
  independent measure of the world moved the same way in the same window, and
  no store can tell you that.
- **Collapse.** Two addresses whose runs are indistinguishable mean the caller
  declared two questions and the worker answered one. Either the declaration is
  decorative or the difference never reached the worker.

A shift at a fixed address says the ground moved or something undeclared was
resolved. It never says the one act was wrong.

## 9. What this format does not do

- **It does not gate.** No conformance classes, no exit codes, no CI verdict.
- **It does not decide.** A judgment is evidence a person may read before
  deciding. Where a decision is owed, it is owed to a different store that
  names a principal.
- **It does not aggregate.** §7 says where a store lives, not how several are
  read together. Reporting across stores is a later question, and the format is
  deliberately flat JSON under stable keys so that answering it does not
  require changing this document.

## Migrations

| Version | Change |
|---|---|
| v1 | Initial. |
