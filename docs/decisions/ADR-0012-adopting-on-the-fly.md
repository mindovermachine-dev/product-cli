# ADR-0012: Adopting on the fly — one action per adoption, wherever the author is

**Status:** Accepted (2026-10-30)
**Date:** 2026-10-30
**Definition:** D1.15 (chapter 3, "Adopting on the fly")
**Refines:** ADR-0010 (workbench), ADR-0011 (proposals and decision records)

## Context

Under ADR-0011, every agent write is a proposal, and nothing reaches the What until the author adopts it (W12). In an authoring session the agent may propose dozens of changes an hour. If adopting costs a typed reason and a context switch, authors will either stop adopting, which stalls the draft, or stop reading, which empties W12 of meaning. Adoption has to be the cheapest act in the tool while each one stays a real, separate decision.

`dpp/1` allows `why` to be a `ref` to another decision, provided it is also pinned in `basedOn` (§3.4). It requires `why` to be non-empty, so a rejection cannot be reasonless.

## Decision

- **D1. Adoption needs no typing.** By default an adoption's `why` is `{ "ref": <proposal version> }`, with that version pinned in `basedOn`. A typed reason replaces the default when the author gives one.
- **D2. Rejection takes one line, and it goes back to the agent.** The workbench and `review` offer a few one-press reasons (out of scope, wrong, duplicate, not yet) beside free text. The reason is returned to the agent as the outcome of its proposal through MCP, so the next proposal can answer it.
- **D3. Batches are many records.** Adopting a selection, everything pending in a phase, or `adopt --all` writes one adoption record per proposal. It is confirmed first, with the count stated, and each adoption is re-checked against the head as it lands. A proposal that fails its check is left pending and reported.
- **D4. The workbench is built for the keyboard:**
  - next and previous proposal, adopt, reject, and choose an alternative, each one key;
  - the change set drawn on the graph in place;
  - the findings the proposal would add or close shown on its card;
  - stale proposals marked, with adopt disabled.
- **D5. The CLI.**
  - `product proposal list` numbers pending proposals.
  - `adopt` takes numbers, ids or `--all`.
  - `review` is an interactive walk with single-key adopt, reject, skip and quit, like `git add -p`.
  - All of these are verbs of the authoring session as the author, so they write the same records as the workbench.
- **D6. Elicitation in the conversation, where the client supports it.** The tool may send an MCP elicitation request asking the author to adopt or reject pending proposals. The answer travels from the client to the tool, not through the model, so the adoption's `by` is the author.
  - The profile states that such an adoption trusts the client to have shown the request to the author.
  - The model never calls adopt; the agent surface still has no adopt verb.
  - Where the client lacks elicitation, the agent points the author to the workbench or to `review`.
- **D7. No auto-adoption.** There is no setting that adopts proposals without an author action, not for a phase, a kind, or a time window. Batch adoption (D3) is the fast path, and it is still an act of the author.

## Alternatives considered

- **Require a typed reason on every adoption.** Rejected: it makes the common act expensive, and the reasons typed under pressure would be "ok". The proposal's own reason, pinned by reference, is the better record.
- **Let the agent adopt when the author says so in chat.** Rejected: the model would perform the act, so by `dpp/1` §3.4 `by` would have to be the agent, and W12 would be empty. Elicitation keeps the act with the person.
- **An auto-adopt mode for trusted phases or kinds.** Rejected: it is W12 switched off by configuration. If it is ever wanted, it is a change to W12 and to the promise, not a setting.
- **One record for a batch.** Rejected: a batch record would hide which proposals were adopted and make a later withdrawal of one of them a change to all of them.

## Consequences

- Adopting costs one key or one click; rejecting costs one more.
- Rejection reasons become a feedback channel to the agent, and they become provenance too.
- Batch adoption is possible and recorded per proposal, so the provenance shows the author adopted forty things in a minute. That is honest, and visible to whoever lands the export.
- **Open: elicitation support.** Which MCP clients implement elicitation, Claude Code and claude.ai included, is not verified here. D6 is built only when a target client supports it, and its absence degrades to D4 and D5.
