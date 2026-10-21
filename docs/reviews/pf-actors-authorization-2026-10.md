# Review — actors, authorization and the actor's place in the What (pf 1.11.0, §3.7)

*2026-10-21. An evaluation of the proposal to add actors and authorization to the
Product Framework, made against §3.2.0, §3.2.1, §3.3, §3.4, §4.2, §4.4 and §6.3.
The proposal survives with three corrections. The section is drafted as
[§3.7](../product-framework-open.md#37-actors-and-authorization--who-may-act).*

## Where the text lives

The canonical specification is
[Hafeok/product-framework](https://github.com/Hafeok/product-framework), and §3.7
is merged there as **1.11.0** on branch `claude/pf-actors-authorization-xq2sap`:
the spec text, a CHANGELOG entry, the `preview/conformance/` request schemas, and
the `actor-kind` authoring-scope kind. Canonical had already released **1.10.0**
(§2.2 the allocation reading, §5.3 seam demand, 2026-07-11), so §3.7 is numbered
1.11.0 upstream. The mirror here has since been brought up to canonical 1.11.0 by
a three-way merge (base canonical 1.9.1): it takes all of 1.10.0 and 1.11.0 and
keeps its own differences from 1.9.1.

Two adaptations were made in the canonical copy. Canonical has no
blueprint/DeployableUnit vocabulary (that rename is product-cli's alone), so
`authenticated_by` and provider coverage are stated **per deployment** of the
infrastructure/runtime contract. Canonical also already uses *actor* for whoever
realises a work unit (§5, "a model actor"), so §3.7 gains a sentence, in both
copies, keeping a principal of the product distinct from the realising actor.

The ontology (`schema/ontology/`) and SHACL shapes (`schema/shapes/`) have no
counterpart in the canonical repo, so their §3.7 vocabulary stays here.

Differences from canonical that the merge kept, all present before 1.9.1, are
product-cli's own: the blueprint/DeployableUnit vocabulary (canonical says
archetype/deployment), schema links to `schema/json/` rather than `preview/`, the
§8 seam row, and the §14 package paragraph.

The merge also kept §5 and §5.1 wording that predated canonical 1.7.0/1.8.0. That
was lag, not intent, and it has since been brought up to canonical: the SPMC
bundle with cells, a fully pinned Model binding and a content-addressed Context
pool, which is the reading 1.10.0's capacity check assumes; and Build-seam schemas
owned by the contracts tier (ai-development-contracts), not by the framework. §5.2
already matched. The vendored `schema/json/build-seam/` files, until then a
pre-1.7.0 snapshot (snake_case, one `executor_extension` slot), were refreshed
from canonical's `preview/build-seam/`, so §5.1 takes canonical's sentence as is,
with only the links rewritten. `product build --emit-seam` output and the worked
pair validate against the refreshed schemas.

## Verdict

The premise holds. Authentication is How, already named by §4.4 (OIDC / OAuth2)
and bound per DeployableUnit (§4.2). Authorization splits cleanly by the
framework's own test: if two systems must agree, it is domain-level. Actor kinds,
Decider rejections over the actor, and View scopes are domain-level; which actor
kinds a screen is offered to is system-level. Adding them required no change to
the derivation contract's existing links, only new ones beside them.

Three parts of the proposal had to change:

1. **The trigger source does not become an actor kind.** The source stays the
   three-valued channel, and a new `acts_as` edge says who acts. Reasons in Q1.
2. **A View scope is keyed by actor kind.** As proposed, scope coverage said
   "every claim a scope reads is carried by every actor kind the step offers to".
   That forces staff to carry `customer_id` so the shop's "own orders" rule can be
   read for them. The scope is therefore a set of **rules, one per actor kind**. A
   rule with no keys is a *declared* "every row". An actor kind with no rule sees
   nothing.
3. **A seam-applied scope needs a keyed fold.** Filtering a shared fold fails for
   cross-row aggregates. The fix is still one fold, but a *keyed* one (Q3), so the
   Projector gains a `keyed_by` declaration.

## Where it fits, and where it contradicts

| Section | Fit | Contradiction found / resolution |
|---|---|---|
| §3.2.0 Trigger | `acts_as` is one more typed edge on the block | Replacing `source` breaks the Automation pattern and the per-node shape (Q1). Kept `source`; `acts_as` must agree with it in nature |
| §3.2.1 UI step | `offered_to` is an annotation on the step, like its other edges | None. Restricted to `human` actor kinds, because a step is "where a person sees something and acts" |
| §3.3 Decider | The actor alphabet is derived exactly like the command alphabet. Claim reads are `reads` edges. Actor coverage is the twin of command coverage | `decide(state, command)` changes signature. Resolved as a declared fourth input (Q2) |
| §3.4 Projector | The fold is unchanged. Claim justification is the twin of state justification | A seam filter over a shared fold fails for aggregates. Resolved with a keyed fold (Q3) |
| §4.2 Contracts | `authenticated_by` is a per-DeployableUnit infrastructure fact, the same as deployment identity | None |
| §4.4 Interfaces | OIDC is already the named standard | The canonical REST projection must not put the actor in a request schema. Resolved: one `openIdConnect` security scheme per actor kind |
| §6.3 Verification | Four checks, each a twin of an existing one | The checks are inert on a What with no actor kind, so graphs from before §3.7 are judged as before |
| §6.3.1 Wire | Additive optional `actor` (decision) and `key` (projection) | Set-valued claims need lists, which the scalar alphabet lacks. Resolved: arrays are admitted in the actor block only |

## The four questions

### Q1 — Does making the trigger source an actor kind break any `--strict` check or the round-trip?

**Yes, both, if done as a replacement. No, if done additively, which is what §3.7 does.**

- `source` is the **pattern discriminator**. `check_trigger` enforces `source ∈
  {user, external, automated}`, and the Automation rule (`source == "automated"`
  ⇒ `watches` a View) reads it. With `source` an open set of actor-kind IRIs, that
  rule has nothing to test unless the actor kind carries a closed nature. That
  just moves the enum somewhere else and adds an indirection to every pattern
  check.
- **Round-trip:** `pf:source` is emitted as a literal. Re-pointing it at an IRI
  changes what `seed.rs` must parse. `lit()` on a named node returns its local
  name, so old and new graphs would both parse, but silently mean different
  things: a `user` literal vs a `d:user` actor kind. It also changes the emitted
  Turtle, and with it the **graph content hash**, for every existing What. Every
  reified tree would then report drift (§7.3.1) without its meaning having changed.
- **Additively** (`acts_as`, `offered_to`, `public`/`scoped_by`, and the
  `ActorKind` node), an actor-free graph emits byte-identical Turtle and keeps its
  hash. The new kind is wired through all five parallel enumerations: struct +
  `counts()`/`ids()`, Turtle emit, seed parse, `seed_canon`, and `maximal()`.
  `full_graph_round_trips_losslessly` covers every new field. Viz (step 5,
  optional) is deferred because `viz.rs` sits at 389 of 400 lines.

### Q2 — A fourth argument of `decide`, or a reserved payload field with `supplied_by: system`?

**A fourth argument.** Both forms put the actor in the decision request, so the
argument is about *where* in the request.

- The payload is the **caller's** wire contract: §4.4's canonical REST projection
  makes the request schema equal to the payload schema. A reserved payload field
  would appear in every generated OpenAPI document as something the caller can
  set. Each generator would then need a negative rule ("strip fields marked
  `supplied_by: system`"), and one generator that forgets it lets the caller choose
  who they are. That is a security property resting on every consumer remembering
  an exception.
- Adding a payload field also changes the payload schema and so the graph hash
  (§7.3.1). An actor field on every command would churn every Command's wire
  contract.
- On the wire, the actor sits **beside** `when`, not inside its `with`:
  `{ given, when, actor?: { kind, claims } }`. Payload equality and the
  `CommandRef` shape are unchanged. The REST projection gets the actor from the
  security scheme, which is where OIDC puts it anyway.

### Q3 — Is a View scope a Projector concern or a seam concern?

**A seam concern, but it needs one declaration from the Projector.** The fold stays
shared and takes no actor. A per-actor fold would make the oracle depend on who
asks, multiply the scenarios by the actor alphabet, and let two actors' views of
the same facts drift apart in logic.

The real case where "filter a shared fold" fails is a **cross-row aggregate**:
"the total I have spent" cannot be recovered by filtering a global total. The fix
is not a per-actor fold but a **keyed** one. The Projector folds per customer and
the scope selects the partition, so one fold still serves every actor. The
condition becomes checkable when the Projector declares `keyed_by` and scope
coverage requires every scope key's field to be one of those keys.

Disjunctive scopes ("mine, or shared with me") are not expressible as a rule. They
are modelled as a recorded share event and a View keyed on it. The relationship
then lands in the domain as data, rather than as an access list in the
realisation.

Also, today's projection wire answers a single flat object, so there are no rows
to filter. The wire therefore gains `key` (to name a partition), not `actor`.

### Q4 — Does anything belong in the Two Pillars mapping (§8)?

**Almost all of it is within this framework's instantiation.** The §8 "What
specification" row gains actor kinds, and the "How" row gains `authenticated_by`;
both are mapping text, not new concepts. One thing is offered upward, as §7.4
offers the composition verdict: the separation of **authority within the
specified system** from **authority over the specification**. That separation is
exactly why Writ's grants stay out of `pf:`. §3.7 states it as "two authorities,
never one" and carries the upward note.

## Status of the §3.7 checks

§3.7 and §6.3 rule every check below. **Ruled — not implemented** means the
spec requires the check but no code runs it yet. A What can therefore pass
`product domain validate --strict` while failing a ruled check, and a green
strict run does not mean the What conforms to §3.7. Each unbuilt check needs a
model change outside the What graph, named in the last column.

| Check (§6.3 Actor conformance) | Status | Where it runs / what it needs |
|---|---|---|
| Per-node actor shapes: closed nature; claim names, types and `values_from`; trigger nature agreeing with source; steps offered to humans only; View public xor scoped; one rule per actor kind; a rule reads only claims its actor kind carries | Implemented | `pf::rules_actor` (in-loop and `validate`); `schema/shapes/shapes.shacl.ttl` |
| Actor coverage, surface half: every step offered to ≥1 actor kind, every automated/external trigger acting as one | Implemented | `rules_actor::actor_conformance` under `--strict`; inert without actor kinds |
| Actor coverage, Decider half: every offered (command, actor kind) accepted or reachably rejected in simulation | **Ruled — not implemented** | an actor in Decider scenarios' `when` and claim reads in the logic (`decider_logic`, `decider_sim`, `codegen_scenarios`) |
| Scope coverage: every surfaced View public, or scoped for every actor kind its step is offered to | Implemented | `rules_actor::actor_conformance` under `--strict` |
| Scope coverage, `keyed_by` clause: every scope key's field is a key the View's Projector is keyed by | **Ruled — not implemented** | a `keyed_by` declaration on the Projector (`pf::projector`) |
| Claim justification: a claim no Decider reads and no scope uses is a finding | **Ruled — not implemented** | Decider claim reads (as the Decider half above); the scope-use half alone would report every Decider-read claim as dead, so it is not shipped on its own |
| Provider coverage: every actor kind a system serves is `authenticated_by`, every claim mapped, in each DeployableUnit | **Ruled — not implemented** | `authenticated_by` in the How contract / DeployableUnit schema (`pf::how`, `pf::deployable_unit`) |
| Wire: `actor` beside `when` (decision), `key` (projection) | Schema only | `schema/json/conformance/`; no runner or codegen emits them yet |

Also built in this change: the `ActorKind` node and its edges (`acts_as`,
`offered_to`, `public`, `scoped_by`), Turtle emit/parse, canonical order, CLI flags
(`--nature --claims --acts-as --offered-to --public --scoped-by`), the generated
MCP field schema, and the ontology (`pf:ActorKind`, plus `pf:Trigger`, which was
missing).

Not checks, and not built: rendering actor kinds in `pf::viz`, and generating the
`openIdConnect` security schemes §4.4 rules in `codegen_openapi` (also **ruled —
not implemented**).

Open, not ruled: whether `ActorKind` is a governed boundary kind in
`WHAT_POLICY` (ddd). It is internal today, and changing that is a ddd decision,
not a framework one.
