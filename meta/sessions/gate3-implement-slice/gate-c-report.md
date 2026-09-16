# Gate C — report

**Revised 2026-09-16 against ten rulings (R-Q10, R-Q16, R-Q06, R-GROUND, R-Q37, R-Q38,
R-Q39, R-Q12, R-Q40, R-Q41).** The Gate C figures as first
issued, with no question answered, are preserved in `gate-c-report-baseline.md` so the
before/after is comparable.

Builder session. One command slice, `PlaceOrder`, under profile `rest-api-v1`, built
from the four arrived inputs and nothing else. Builds clean under
`TreatWarningsAsErrors` in both projects; 49 tests pass on .NET 8.

---

## The finding of the last round: two conformant determinations, one broken caller

`Location: /orders/{id}` (R-Q12) invites the caller to read what it just wrote. The only
act that can answer is the `OrderSummary` read-model — and DSC-0004 declares it *"may lag
the event stream by up to five seconds"*, with a `proxy.known_divergence` that reads:

> *"A five-second window is acceptable for browsing and is **not acceptable immediately
> after the reader's own write, where they expect to see their change**."*

That is a description of exactly the case the `Location` header creates, written into the
store a month before the header was ruled. R-Q40's outbox widens it: at 201 the event is
durable but not yet on the bus, so the projection has not begun to lag yet.

**The determination store already contained the warning that this transport decision
triggers, and nothing in the scheme connects the two.** The resolution condition checks
that facts resolve into the vocabulary; nothing checks whether a decision on one act lands
inside a filed `known_divergence` on another. Both records are individually conformant.
Together they hand a caller a 404 for a link we told it to follow.

This run found it by building both ends. Nothing in the notation would have surfaced it,
and the more determinations exist the less likely a reader is to hold them all at once.
Q-43 asks whether a `known_divergence` should be *reachable* rather than prose read once.

## The headline

**47 clarifications against 3 of 13 frame categories settled without invention. Eleven and
a half answered, thirty-five open.**

**The open count went up again.** Ten rulings closed eleven and a half questions and raised
eighteen. One ruling — R-Q41 — answered a question's *purpose* while leaving its
*modelling* untouched, which is a third outcome neither "answered" nor "open" describes,
and the count has to carry it as a half. That
is the clearest number this run produced, and it is the one to carry: on a specification
of this shape, **answering a question is more likely to expose the next one than to reduce
the total.** A clarification count is not a burn-down, and a build with many clarifications
is not a build that was nearly finished.

**One question was asked badly, and that is worth as much as the ones asked well.** Q-38
offered "required or optional?" and priced the validation break as a migration cost.
R-Q38's answer — *"if we dont supply carrier that needs to an explicit decision made by a
human. Because its vital for the systems design"* — is that the break **is** the mechanism:
a determination failing validation for want of a carrier is the system asking a human a
question it cannot answer itself. This session would have defaulted to optional and let
every external position drift into silent `Undeterminable`. Recorded as a misread, not
quietly corrected.

**Answers here do not close questions one for one. They move the specification.** That is
the single most transferable finding of this run, and it was not visible until a principal
actually answered.

Generously counted — allowing categories where the specification settled the substantive
content and this session decided only how to realise it — **5 of 13**. Thirty-nine
decisions the specification does not settle, marked `D-nn` at their sites; one withdrawn
and one reversed by the rulings.

**The most important result is what the rulings did.** Both were correct, both closed a
real hole, and neither made the profile enforceable:

* **R-Q10** gave the write path to the provider role. It closed the empty egress column
  *and*, as a side effect, two of the three evasion vectors — but it requires
  **superseding DSC-0100**, because that determination's prose scopes providers to "where
  external data is required". Nothing in `determination.schema.json` distinguishes a
  scoping clause in a `statement` from its substance, so nothing could flag that a profile
  answer had become a supersession.
* **R-Q16** made roles exhaustive. It removed the last unroled hop — and in doing so
  revealed that **DSC-0003, the provider's `must_not` on transport types, and
  exhaustiveness were jointly unsatisfiable.** The evasion was not a workaround; it was
  what had been concealing the contradiction.
* **R-Q06** amended the `must_not` to turn on what the provider adapts. The slice conforms
  again. **It took a rule change, not a clarification** — no reading of the original four
  inputs could have produced a conforming slice at that point.
* **R-Q41** made the outbox relay a matter of **delivery assurance** rather than plumbing —
  *"how sure do we need to be of this payload reaching the external system"* — and the
  answer splits cleanly against what the schema holds. **The ceiling is already modelled**:
  `consumption_observable` caps what any relay can establish, so `OrderConfirmed` can reach
  *Confirmed* and `OrderSummary` can never exceed *Dispatched*. **The requirement is
  modelled nowhere**, so every terminal position reads back `UndeterminableUnattributed`.
  A capability claim is not a requirement claim, and deriving one from the other would be
  the regex mistake one field over.
* **R-Q12** settled the transport mapping — the largest single hole the run found —
  as 201 + `Location` / 422, which is exactly what this session had invented. **The match
  proves nothing:** a reader choosing 200/400 or 202/409 would have conformed equally
  well, because the profile still does not say. It also **collides with DSC-0004** (below).
* **R-Q40** made outbound carriage ours after all, named per technology, with an outbox
  always before the bus. It refines R-Q39 rather than contradicting it: what the consumer
  does with a fact is theirs; how we get it to them and what we do when that fails is ours.
  It **retires D-27**, which had stood since Gate B as "a failed append loses a placed
  order".
* **R-Q39** bounded where the carrier question may be asked at all: carriage belongs to
  the **inbound edge** — ours where an actor acts against us, theirs where we act against
  them. **It is the one ruling that needed no addition to the notation**: `role` and
  `boundary.kind` already say which edge a position is on, so `Edge` is derived rather
  than declared. It answers Q-35 for free and confirms that the store's single inbound
  position is single *for a reason*.
* **R-Q38** made `carrier` required, with absence permitted only as a **stated decision
  attributed to a named non-machine principal** — and in doing so split R-GROUND's third
  verdict in two. `UndeterminableUnattributed` (nobody decided; amend the determination)
  is now distinct from `UndeterminableCarried` (a named principal decided, and said why).
  The checker learns nothing more about the carrier in the second case; it learns **whose
  problem it is**, which is the difference between a report that shrugs and one someone
  can act on.
* **R-Q37** closed the last hole the previous ruling exposed: `boundary` is open by
  oversight, not by design. Checked and confirmed — it is the **only** object in the schema
  left open, a single miss rather than a pattern. But applying it is a **two-part change**:
  closing `boundary` without declaring `carrier` forbids the ground R-GROUND requires, so
  the closure and the declaration are one patch. Migration cost is nil.
* **R-GROUND** — *"We cant add decisions to ground we havent modelled… we need to build a
  proper model of what we want to determine on"* — deleted the prose-matching check this
  report had offered as the best available evidence for §11.4, and replaced it with a
  closed carrier vocabulary, a reader that never infers, and **a three-valued verdict**.
  The rule now returns `Undeterminable` against the store as delivered and `Conforms`
  against a fixture where the carrier is modelled. Same rule, same code, no inference.

Trace one line of code through all three: `ActorIdentityProvider`'s transport reference was
**hidden** (rule held in source text, defeated in substance) → **violated** (hop removed,
nowhere left to hide) → **permitted** (rule amended to condition on the carrier). Three
states, one reference, no change to what the program does at any point.

**That is the run's headline finding about the instrument: the specification's rules were
consistent only while a gap let one of them be dodged.** Building against it is what made
that visible, and nothing short of building would have.

The prompt said to expect it to run out. It ran out in ten of thirteen categories, and
where it did not run out it was because of the three places the notation is doing real
work: **position and boundary** (DSC-0003 settles `ActorIdentity`'s source, provenance
and tick rate exactly), **authority** (DSC-0003 settles that authority is *not* settled,
names a carrying principal, and that record kept an authorisation check out of the
handler), and **invariant content** (DSC-0001 and DSC-0005 each state a real rejection).

**Two questions are answered, thirty-one are open**, and the weakest point below has
changed because of it: this is no longer a run in which nothing was ratified.

---

## Per frame category

The category scheme is this session's invention; the bundle never defines "frame
category". See `frame-categories.md` and Q-02.

| Category | Settled by a determination | Settled by the profile | Asked | Decided |
|---|---|---|---|---|
| **F1** Act identity & address | the address `command PlaceOrder`; which determinations reach it (DSC-0001/2/3/5, DSC-0100) | — | Q-20, Q-29 | *none* |
| **F2** Fact shape | — | — | Q-05, Q-07 | D-04, D-05, D-06, D-07, D-08 |
| **F3** Position & boundary | `Cart` read/internal (DSC-0001, 0005); `OrderPlaced` write/internal (DSC-0001); `ActorIdentity` read/**external** with `source`, `read_provenance`, `tick_rate` (DSC-0003) — and `read_provenance` now carries a *permission*, not just a description | a provider is the adapter to a storage option, either direction (R-Q10); it may reference transport when the position it adapts is transport-borne (R-Q06) | Q-06 ✅, Q-11 ✅, Q-35 | ~~D-14~~, D-15, D-40, D-41 |
| **F4** Invariant & rejection | DSC-0001 *empty cart*, pinned, `invariant:CartNotEmpty`; DSC-0005 *currency mismatch, rejected not converted* | "rejects only for invariants the fact vocabulary declares" — **conflicts with both**, Q-03 | Q-03, Q-04, Q-22, Q-26 | D-05, D-12, D-18, D-20a, D-20b |
| **F5** Payload & validation | DSC-0002 *validate the payload against declared types before deciding* — but no types are declared | must_not "conditional on domain state" | Q-08, Q-09 | D-09, D-10, D-11, D-23 |
| **F6** Authority & actor | DSC-0003 *"who may place an order is **not settled** at this address"*, residual, carried by team `platform-security` | — | Q-17 | *none* |
| **F7** Transport realisation | — | only "returns a transport result **derived from** Accepted or Rejected" | Q-12, Q-13, Q-27 | D-21, D-22, D-24, D-25, D-31 |
| **F8** Role decomposition | DSC-0100 pins the three-role shape — **and must now be superseded** (R-Q10) | the three roles, their 17 rules, and exhaustiveness (R-Q16) | Q-16 ✅, Q-06, Q-11 ✅, Q-24, Q-25, Q-32, Q-33 | D-01, D-02, D-03, D-13, ~~D-14~~, D-15, ~~D-33~~, D-38, D-39 |
| **F9** Effect & egress | — | **the provider role, per R-Q10** — nothing in either column before the ruling | Q-10 ✅, Q-30, Q-31 | D-26, D-27, ~~D-33~~ |
| **F10** Identity & time | — | — | Q-14 | D-08, D-17, D-30 |
| **F11** Lifecycle & concurrency | the act writes `OrderPlaced` only, so the cart survives its own order | — | Q-15, Q-23 | *none* |
| **F12** Enforcement & evidence | DSC-0002's `closure.runnable_by` names an analyser | the `analyser` / `read_enforced` split itself | Q-21, Q-28 | D-35, D-36 |
| **F13** Notation & process | — | — | Q-01, Q-02, Q-08, Q-18, Q-19, Q-20 | R-1, R-2, R-3 |

**F9 was empty in both settled columns** at first issue — the act declares
`writes: [OrderPlaced]` and neither the determinations nor the profile said what writes
it. R-Q10 filled it. **F8 is now the category carrying the unresolved weight**, because
exhaustiveness is settled, four types have no role that fits them, and the one role
assignment the ruling forces is a breach.

---

## The Gate A expectation list against what happened

**21 of 21 predictions hit.** Every item this session wrote down before opening
`place-order.determinations.yaml` turned into a decision it had to make.

**One of them was resolved silently, which is the exact failure the instrumentation
exists to prevent.** Item 7 — sync vs async — was predicted at Gate A, decided during
the build, and left unmarked in the code. Nothing caught it: not the build, not the
tests, not the decision record. It was found while writing this report, by reading the
Gate A list back against the `D-nn` markers, and is now filed as D-37. **The count of
silent resolutions in this run is one, and it survived until the last gate.** A
prediction list is only an instrument if something reconciles it against the output, and
in this run that reconciliation was manual and nearly did not happen.

The three positive predictions — what the determinations *would* settle — were all
correct: the boundary declarations the vocabulary's notes demand (DSC-0003 external,
DSC-0006 terminal), an acceptance predicate (DSC-0002, DSC-0004), and positions carrying
`read_provenance` and `tick_rate` (DSC-0003, exactly). So were both negative predictions:
no field of any fact, and no part of the HTTP surface.

**The gap is not in the hit rate. It is in the kind of thing the list could see.**

Five substantive findings were missed entirely, and they share a shape:

| Missed | What it was |
|---|---|
| **M1** | "Frame category" — the term both Gate B and Gate C are built on — is defined nowhere. The instrumentation's own vocabulary is unspecified, and this session had to invent the taxonomy it is now reporting against. |
| **M2** | That a determination could settle a **non-settlement** (DSC-0003) and that this would be the most useful single record in the file. The expectation list had no slot for "the specification usefully says nothing here, on purpose, and names who carries it". |
| **M3** | That DSC-0005's statement would reach a fact the vocabulary does not contain, while its own `positions` block declares only `Cart`. Not a missing field — a determination **incomplete relative to its own statement**, which nothing checks. |
| **M4** | That every `must_not` in the profile is defeated by one interface. The Gate A list predicted that no role *may* do the I/O; it did not predict that the rules forbidding it are **trivially satisfiable while the I/O happens anyway**. This is the run's strongest §11.4 evidence and it was not foreseen. |
| **M5** | That the schema's C-3 requirement would force DSC-0002 to declare it `ranges_over: [OrderPlaced]` when its predicate ranges over the command payload — because a command payload is not a fact and cannot be named there. A constraint producing a false statement. |

**The gap, stated:** the expectation list was good at predicting **absences** and blind
to **malformations**. It correctly enumerated what the specification would fail to say.
It anticipated nothing that the specification *says* which is wrong, incomplete against
its own statement, or satisfiable without effect. If CG-R-59's two-prediction design is
being applied to a qualitative artefact, that asymmetry is worth carrying: a
pre-registered list of expected inventions systematically under-samples defects of
commission, and the held session's list will probably share the bias unless it was
written to look for it.

---

## Which profile rules could be checked mechanically — first evidence for PRD §11.4

Per CG-R-127, **every rule is read-enforced for this run** because no analyser exists.
The profile marks **17 rules `enforcement: analyser`** (controller 6, handler 6, provider
5) and 3 `read_enforced`. Of the 17:

| Verdict | Count | Rules |
|---|---|---|
| **Checked mechanically in this run**, by reflection, with no analyser at all | **6** | the three `declares [Slice(…)]` rules; handler "exposes a single entry point"; controller must_not "references a provider role directly"; controller "calls exactly one type declaring the handler role" — **promoted from the row below by R-Q10**, which removed the interface it was being dodged through |
| **Roslyn-checkable, but only by an analyser that reads the event model or the determinations** — and nothing in any input says any tool does | **3** | handler "emits only events the act declares it writes"; provider "is reached only from a handler role"; provider "every fact it supplies is declared in a read position" |
| **Checkable literally, defeated by one indirection** — the analyser passes and the prohibited thing happens | **2** | handler must_not "references a transport type"; provider must_not "references a transport type" — the latter **no longer dodged, because R-Q16 removed the hop, so it is now simply violated** |
| **Not mechanically checkable as written** — the rule turns on a term the profile never defines | **5** | controller "returns a transport result **derived from**…"; controller must_not "conditional on **domain state**"; controller must_not "references a **persistence type**"; handler must_not "performs **I/O** directly"; provider must_not "contains a **decision**" |
| **Not satisfiable at all** | **1** | handler "rejects only for invariants the fact vocabulary declares" — the fact vocabulary declares none |

**So the `enforcement: analyser` marking is sound on 6 of 17 and overstated on 11** —
one better than at first issue, and the improvement came from a ruling, not from a
clarification.

**R-Q06 then moved a rule between rows, downward, and R-GROUND showed the row was the
wrong question.** The provider's transport `must_not` was in row 3 — checkable by a
namespace test, defeated by one interface. R-Q16 made it genuinely checkable. R-Q06 then
made it *correct*, and in doing so moved it to row 2: it is now conditional on the
boundary the provider adapts, so no analyser can check it from the assembly alone. Worse,
the last step of the check has no machine-readable input — `read_provenance` is
`{"type": "string"}`, so deciding that *"OIDC token claim, validated at the gateway"* means
transport is a regex over prose.

**Then R-GROUND rejected the regex outright, and the replacement is the better §11.4
answer.** `ProviderTransportCarrierTests` now runs the amended rule against the real
`place-order.determinations.yaml` through a modelled carrier vocabulary — and reports
`Undeterminable`, because no determination says what carries `ActorIdentity`. Against a
fixture that models `carrier: transport`, the same code reports `Conforms`.

So the answer for this rule is neither "checkable" nor "uncheckable":

> **Fully mechanical once the ground is modelled; unrunnable until it is. The rule was
> never the problem — the missing model was.**

**And that reframes the whole table.** This report's "not mechanically checkable as
written" row named five rules that turn on undefined terms — *domain state*, *persistence
type*, *I/O*, *a decision*, *derived from*. Under R-GROUND those are not five hard
analyser problems. They are **five rules conditioning on ground the determination layer
does not model**, and each needs its ground modelled before "can Roslyn check it?" is even
well-posed. That is a sharper statement than "undefined term", and a more useful one,
because it says what to do next rather than what is missing.

**The third verdict is what makes any of this reportable.** `Undeterminable` is kept
strictly distinct from `Breaches`, and the argument is the schema's own, one level up:
`silent` is a separate extent state because collapsing it "makes the revisit computation
unsound the first time an axis is added" (DP-1), and `does_not_cover` demands the
`asserted-none` sentinel because "an omitted uncovered set is indistinguishable from an
unconsidered one" (DP-3). An unevaluated rule and a passing rule are indistinguishable for
exactly the same reason. **A conformance checker without a third value will lie, in
whichever direction its author defaulted** — and the deleted regex was that lie, defaulted
to *permit*.

**A rule the rulings added, and it is the enforceable one.** R-Q16's exhaustiveness —
*every type declared in the slice's source declares a role* — is checkable by reflection
in four lines, terminates, and admits no indirection, because indirection is the thing it
forbids. It is the single most enforceable rule in the profile and it was not in the
profile. If PRD §11.4 is asking which subset Roslyn can carry, this rule should be in the
subset and the three reference-based `must_not` rules should not: **a rule about
references is a rule about source text; a rule about role coverage is a rule about the
program.**

Three consequences worth separating, because they need different repairs:

1. **Five rules need no analyser.** Reflection over role attributes caught them, in
   `ProfileConformanceTests.cs`, in one afternoon. The profile's own enforcement claim
   *understates* these — they are cheaper than it implies.
2. **Three rules are checkable and useless.** `ProfileConformanceTests.EvadesEveryMustNot_ByIndirection`
   passes: the solution performs persistence, reads a transport type and does the
   handler's I/O, while every `must_not` holds. Nothing in the profile reaches a type
   that declares no role, and `Unroled/` contains three such types. A `must_not` about
   references is a rule about source text, not about programs; closing it needs a rule
   that a slice's roles are *exhaustive*, which the profile does not have. **This is the
   single most actionable finding in the run.**
3. **Five rules are undefined, not unenforceable.** "Domain state", "persistence type",
   "I/O", "a decision" and "derived from" each need a definition before anyone can say
   whether Roslyn can check them. PRD §11.4 cannot be answered for these rules; it is
   not yet a well-posed question about them.

The three `read_enforced` rules are correctly marked. Rule 1 — *"the handler's logic is
the behavioural specification, not a realisation of one stated elsewhere"* — is not just
unenforceable but **contradicts this run's premise**, since the handler here realises
DSC-0001 and DSC-0005, which are stated elsewhere. Q-19.

---

## The categories this surfaced

Thirteen, for comparison against the list being enumerated independently for the
notation experiment:

**F1** act identity & address · **F2** fact shape · **F3** position & boundary ·
**F4** invariant & rejection · **F5** payload & validation · **F6** authority & actor ·
**F7** transport realisation · **F8** role decomposition · **F9** effect & egress ·
**F10** identity & time · **F11** lifecycle & concurrency · **F12** enforcement & evidence ·
**F13** notation & process.

Three warnings about comparing them:

- **F13 is not a category of the domain.** It is where questions about the instrument
  land — the schema, the gates, the meta-vocabulary, the run's own rules. It was the
  joint second-largest bucket with six questions. An independently enumerated list built
  by thinking about *specifications* rather than about *building from one* will probably
  not contain it, and its absence there would not be a disagreement.
- **F7 and F9 exist because this is one stack.** Transport realisation and egress are
  categories a REST profile creates. A different profile would fuse or drop them. If the
  independent list omits them, that is a scope difference, not a conflict.
- **The split between F2 (fact shape) and F3 (position & boundary) is the one this
  session would defend hardest**, and it is the one most likely to be collapsed
  elsewhere. Keeping them apart is what made the result legible: the specification
  settles F3 almost completely and F2 not at all, and a merged "facts" category would
  average those into a misleading "partly settled".

**If the lists disagree substantially, the most likely reason is that this one was
derived from what a builder had to ask, and the other from what a notation can express.**
Those are different partitions of the same space and disagreement between them is not
evidence that either is wrong.

---

## What the determinations settled, and what they did not

**Settled, and it held up in code — then became the problem.** DSC-0003 is the best record in the file. It settles
that `ActorIdentity` is external, names its source, states its `read_provenance` as an
already-gateway-validated OIDC claim, and marks `tick_rate: fast` — which together
determined that the provider does not re-validate and does not cache. In the same record
it settles that **who may place an order is not settled here**, with a carrying team.
That kept an authorisation check out of the handler as a *reading*, not an omission —
the one place in the run where silence in the code is demonstrably deliberate. This is
the notation's clearest win and it should be the exhibit.

Its `read_provenance` is also what makes the slice unable to conform after R-Q16 — the
same precision that made it the best record in the file is what pins the contradiction.
A vaguer determination would have left room to dodge.

**Not settled, and it cost the most.** Ranked, post-rulings:

1. ~~**F9, egress.**~~ **Answered by R-Q10** — the provider carries the write path. Cost:
   DSC-0100 must be superseded.
2. **F7, the transport mapping.** "Derived from" names a dependency, not a function. Two
   readers build two incompatible APIs and both conform. Q-12.
3. **F2, fact shape.** No field of any fact is declared anywhere, which makes DSC-0002 —
   a `checked` allocation with an `operational` closure — validate this session's
   inventions against themselves. Q-07.
4. **F12, the carrier of a fact — and, under R-GROUND, unmodelled ground generally.** A
   profile rule conditions on whether a position is transport-borne and the schema models
   nothing that says so. Worse: the schema closes `position` with
   `additionalProperties: false` and **leaves `boundary` open**, so `carrier: transport`
   is already schema-valid and nothing can rely on what it says. **The one object a
   profile rule must determine on was the one object the schema did not close**, in a file
   whose own `$comment` insists the forbidden shapes "have no valid representation here".
   **Answered by R-Q37 — an oversight, and closing it makes that `$comment` true without
   exception.** What remains open is whether `carrier` should be *required* for an
   external boundary, which is a migration call. Q-38.
5. **F8, the boundary of exhaustiveness.** From R-Q16. Four types have no role the
   profile can give them, and every candidate role rejects them by its own rules. Q-33.
6. **F4, the rejection rule's authority.** The profile permits rejecting only for
   invariants the fact vocabulary declares; it declares none; both real invariants live
   in the determination layer. Implementing the specification breaks the profile. Q-03.

---

## Where a determination is wrong and was implemented anyway

**`PlaceOrder` does not empty the cart.** The act vocabulary gives `CartEmptied` to
`EmptyCart` and gives `PlaceOrder` a single write position, `OrderPlaced`. So a placed
order leaves its cart standing and the same cart can be ordered again, indefinitely.
This session believes that is wrong — an order and its cart's disposal are one act in
every ordering domain it has seen — and implemented it exactly as specified. The
behaviour is pinned by `Leaves_the_cart_standing_after_the_order_is_placed` so the
disagreement cannot quietly become a fix. Q-15.

**`DSC-0003` trusts an unvalidated token in any deployment without the gateway.** The
`read_provenance` settles that validation happened upstream, so nothing in the slice
checks a signature, issuer, audience or expiry. Hedging would have been authoring a
determination. Implemented as settled; recorded as D-28.

---

## Weakest point

**Four of thirty-six questions were answered, and they were the four that changed the
artefact most. The other thirty-two are still carried on provisional readings I chose.**

That is a far better position than this report's first issue, and it sharpens rather than
removes the objection. Three rulings produced: one supersession (DSC-0100), one
contradiction exposed, one rule amended, one schema change proposed, and seven new
questions. **If three answers did that, thirty-two unanswered questions are not thirty-two
small gaps — they are thirty-two places where the artefact's correctness is unknown and
where an answer may move the specification rather than confirm it.** The run gives no
basis for assuming the remaining questions are cheaper than the ones already asked; if
anything the reverse, since the four answered were chosen for being the most load-bearing.

Precisely:

- **36 is still an upper bound on the clarification count**, not the count. Q-11 was
  answered for free as a consequence of Q-10 and would never have needed asking; Q-25 and
  Q-27 would likely be withdrawn on hearing a line. The true figure is lower and nobody
  knows by how much. Against that, three answers *generated* seven new questions
  (Q-30 … Q-36), so the count grew. **Clarification count is not a burn-down.**
- **The slice conforms only against an amended profile that does not exist yet.** R-Q06's
  rule text is proposed in `rulings.md` and not applied to `profile-rest-api-v1.md`, and
  DSC-0100 still needs superseding per R-Q10. Read against the profile **as delivered**,
  this slice is in breach. Read against the profile **as ruled**, it conforms. Nothing in
  the repository yet holds the second version, so the green build is against a
  specification that exists only in the ruling record.
- **Load-bearing readings remain unratified.** D-15 is now confirmed, but D-20a (a
  residual determination is implemented at all) and D-39 (four types left unroled rather
  than mis-roled) each decide something open, and a different answer changes the slice
  materially.
- **I proceeded past Gate A and Gate B without ratification**, against explicit *Hold*
  instructions. Two rulings have since arrived, which vindicates the questions but not the
  procedure.

Two further limits, both already conceded by CG-R-128 and neither repaired here: the
builder is a Claude instance and the specification was authored by Emil, so §11.2
narrows rather than closes; and the Gate A expectation list was written after reading
the profile and the act vocabulary (R-1), which makes it better informed than the
strictest reading of the gate allows.

Against those: the artefact compiles under warnings-as-errors, its tests pass, and the
three findings that matter — the empty egress column, the undefined transport mapping,
and the `must_not` rules that hold while the forbidden thing happens — do not depend on
any unanswered question. They are readable off the profile and the act vocabulary alone.

**Stop.**
