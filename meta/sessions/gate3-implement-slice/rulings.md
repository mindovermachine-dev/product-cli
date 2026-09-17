# Rulings — answers from the principal

Per CG-R-128, *"the questions are the datum and the answers become part of the record."*
Recorded verbatim, with what each changes.

---

## R-Q10 — Emil, 2026-09-16

**Question put (Q-10).** *"`PlaceOrder` declares `writes: [OrderPlaced]`. An event emitted
and never persisted is not written. But the handler `must_not` perform I/O, the
controller `must_not` reference a persistence type, the provider's every rule is about
supplying reads and it `must_not` decide — and there is no fourth role. The write
position has no realisation anywhere in the profile. What persists the event?"*

**Answer, verbatim.**

> we need a writer path as we have a reader path, I would argue that providers can supply
> writes as well as reads. They are the adapters to the storage options

**Ruled:** the provider role carries the write path. A provider is the adapter to a
storage option, in either direction. No fourth role.

### What it changes in the build

The unroled `EventAppendingPlaceOrderHandler` is gone. `OrderPlacedProvider` declares
`[Slice("PlaceOrder", SliceRole.Provider)]` and records the write position; the handler
reaches it exactly as it reaches the read providers, so "performs I/O directly" still
holds for the same reason it held before.

Two evasion vectors closed with it, and this is the ruling's real yield:

* **D-33 is withdrawn.** The controller no longer depends on an interface that DI
  resolves to an unroled type. It depends on `PlaceOrderHandler`, so the controller's
  *"calls exactly one type declaring the handler role"* is now true at run time and not
  only in source text.
* **D-27 narrows.** The append is still synchronous and non-transactional, but it now
  happens inside a roled type the profile constrains.

### What it answers beyond the question asked

**Q-11 is answered as a consequence.** If a provider is "the adapter to the storage
options", then whether the fact is internal or external never decided whether a provider
is used — storage did. `CartProvider` supplying an internal `Cart` was correct, and D-15's
"sufficiency, not restriction" reading is confirmed by a stronger statement than the one
this session guessed at.

**Q-06 is NOT answered and is now sharper.** `ActorIdentity`'s `read_provenance` is an
OIDC token claim on the request. Under this ruling the provider is the adapter that
fetches it — and a provider `must_not` "reference a transport type". If providers are
adapters to *sources* generally, an adapter to a request-borne claim must touch the
request. The conflict survives the ruling intact.

### Two amendments the ruling requires, proposed not authored

Per the standing rules — *you propose, Emil ratifies* — neither is applied to the inputs;
both are stated here for ratification.

**1. The profile's provider role text.** `must` currently covers only supply:

```yaml
    must:
      - "declares [Slice(<instance>, \"provider\")]"
      - "is reached only from a handler role for the same act instance"
      - "every fact it supplies is declared in a read position on the act"
      - "every fact it records is declared in a write position on the act"   # ← added
```

Without the added line the write path is roled but unconstrained: nothing stops a
provider recording a fact the act does not declare it writes. That is the mirror of the
rule that already exists for reads, and the asymmetry would be a new hole.

**2. `DSC-0100` must be superseded, not edited.** Its `statement` reads *"realised by a
controller, a handler and, **where external data is required**, a provider."* That clause
scopes providers to external data, which this ruling contradicts. The statement is a
**determination**, pinned, travelling to `all-command-slices` — so changing it is a
supersession under `provenance.supersedes`, not a profile revision. Proposed replacement
clause: *"…and, where the act has a read or write position requiring storage, a
provider."*

**This is the ruling's largest cost and it should be recorded as such.** A question about
a missing role turned out to require superseding the determination that pins the
architecture, because the determination's prose carried a scoping clause the profile body
did not. Prose in a `statement` field is load-bearing in a way the schema does not
expose: nothing in `determination.schema.json` distinguishes a statement's scoping clause
from its substance, so nothing could have flagged that this edit was a supersession.

### Three questions the ruling raises

Recorded per the Gate B instrumentation; all three are open.

**Q-30 (F9).**
> Does the handler call the write provider, or does it return `Accepted` and something
> else records it? I have the handler call it, so the write happens inside the decision
> act. That makes a provider failure occur *after* the decision is taken and *before* the
> caller is told — is that the intended shape, or is the write meant to be a consequence
> of `Accepted` rather than part of reaching it?

*Proceeded under:* the handler calls the provider, before returning `Accepted`.

**Q-31 (F9).**
> If the write provider throws, the act has decided but not written. That is not a
> rejection — no invariant is cited — so it cannot be `Rejected`, and the profile admits
> no third exit. What is the outcome of a decided-but-unwritten act?

*Proceeded under:* it throws, and the same unroled middleware maps it — the third exit
the profile does not admit, now on the write side as well as the read side (D-19).

**Q-32 (F8).**
> May one provider type serve both a read and a write position, or is it one provider per
> position? I have used three, one per position. Nothing in the ruling or the profile
> settles it, and "adapters to the storage options" suggests one adapter per *store*,
> which would group them differently again — by backing store rather than by position.

*Proceeded under:* one provider per position.

---

## R-Q16 — Emil, 2026-09-16

**Question put (Q-16).** *"Nothing in the profile forbids a slice from containing types
that declare no role, and no rule reaches such a type. I have built a conforming slice in
which an unroled decorator performs the handler's I/O, an unroled adapter holds the
provider's transport reference, and an unroled sink is the persistence the controller may
not name. Every `must_not` holds and every prohibited thing happens. Is the unroled type
intended to be outside the profile, and if not, what rule closes it?"*

**Answer, verbatim.**

> we need the role for the act. We cant have an act withour an actor and role is part of
> that.

**Ruled:** roles are exhaustive. Every type participating in an act declares a role;
there is no outside-the-profile.

### What it changes in the build

`HttpContextClaimSource` is gone. `ActorIdentityProvider` now references
`IHttpContextAccessor` itself and reads the claim off the request directly. There is no
unroled hop left for the reference to hide in.

### And that makes three rules jointly unsatisfiable

This is the ruling's result, and it is not a defect in the ruling.

| | |
|---|---|
| **DSC-0003** | `ActorIdentity` arrives as an "OIDC token claim, validated at the gateway" — it is on the request |
| **profile / provider** | `must_not` "references a transport type" |
| **R-Q16** | every type participating in the act declares a role |

Any two hold. All three cannot. Before the ruling, the `IClaimSource` indirection let all
three appear to hold, because the transport reference sat in a type no rule reached. **The
evasion was not a workaround; it was the thing concealing the contradiction.** Closing it
is what made the contradiction undeniable, which is the ruling doing exactly what it
should.

This session implements DSC-0003 and R-Q16 and **breaks the provider's `must_not`,
visibly**, asserted by
`ProfileConformanceTests.The_provider_references_a_transport_type_in_breach_of_its_own_rule`.
It broke that one because it is the only one of the three that is a rule about source
text rather than about what the act does.

**Q-06 is now forced.** It was sharpened by R-Q10 and is made unavoidable by R-Q16. It is
the next thing needing a ruling, and the options are narrow:

1. **Amend the provider `must_not`** — a provider adapting a transport-borne source may
   reference transport; providers adapting a store may not. Draws the line by what is
   adapted, which matches R-Q10's "adapters to the storage options".
2. **Rule that `ClaimsPrincipal` is not a transport type** — the claim is identity data
   that happens to arrive on a request. Narrow, and leaves `IHttpContextAccessor` still
   forbidden, so it needs a different accessor.
3. **Supersede DSC-0003's `read_provenance`** — the gateway writes the claim somewhere
   non-transport that the provider reads. Changes the architecture, not the profile.

### The boundary R-Q16 leaves open — Q-33

Four types still declare no role after the ruling, and **no role in the profile fits any
of them**:

| Type | Why no role fits |
|---|---|
| `InMemoryCartStore`, `InMemoryOrderPlacedStore` | the storage options a provider *adapts*. Under R-Q10 the adapter is the provider, so the store is behind it — but is a store "participating in the act"? |
| `GuidOrderIdentityMint` | supplies the `OrderId`. A provider `must` supply only "facts declared in a read position on the act", and an order identifier is not a fact in the vocabulary at all. Rolling it as a provider breaks that rule. |
| `ReadPositionUnavailableMiddleware` | produces a transport result, which is controller work — but the controller `must` "call exactly one type declaring the handler role", and this calls none. Rolling it as a controller breaks that rule, and would also make two controllers for one act. |

**Q-33 (F8).**
> Where does "participating in the act" stop? Taken at face value the rule reaches the DI
> container, `TimeProvider`, `ProblemDetails` and the ASP.NET pipeline. Taken narrowly it
> reaches only the types I chose to put in the slice's namespaces, which is circular. The
> exhaustiveness rule needs an edge, and the profile has no vocabulary for one.

*Proceeded under:* stores, mint and middleware left unroled and marked as breaches,
because every available role rejects them.

**D-38 — a small mechanical finding.** Written over "every type", the exhaustiveness rule
catches the compiler-generated async state machine behind the middleware's `InvokeAsync`.
An analyser implementing R-Q16 must scope to types declared in source, not types in the
assembly. Found by running it, not by reading it.

---

## R-Q06 — Emil, 2026-09-16

**Question put (Q-06).** *"DSC-0003 settles that `ActorIdentity`'s `read_provenance` is an
'OIDC token claim, validated at the gateway'. The only carrier of that claim is the HTTP
request. The profile makes the provider the role that supplies read-position facts and
says a provider `must_not` 'reference a transport type'. The one permitted supplier of
this fact may not touch the only thing that carries it. How is the claim meant to reach
the provider?"*

**Answer, verbatim.**

> Amend the provider must_not

**Ruled:** option 1 of the three put forward. A provider adapting a transport-borne source
may reference a transport type; a provider adapting a store may not. The line is drawn by
**what is adapted**, consistent with R-Q10's "adapters to the storage options".

### Proposed rule text, not authored

```yaml
  - name: provider
    required: false
    enforcement: analyser
    must:
      - "declares [Slice(<instance>, \"provider\")]"
      - "is reached only from a handler role for the same act instance"
      - "every fact it supplies is declared in a read position on the act"
      - "every fact it records is declared in a write position on the act"   # R-Q10
    must_not:
      - "contains a decision"
      - "references a transport type, unless the position it adapts declares a
         transport-borne carrier"                                            # R-Q06
```

### The slice conforms again

`ActorIdentityProvider` keeps its `IHttpContextAccessor` and is no longer in breach.
DSC-0003 carries the permission. The rule still bites: `Cart`'s boundary is plain
`internal`, `CartProvider` gets no permission and takes none.

**So the three-way contradiction is resolved, and it took a rule change rather than a
clarification.** Worth holding onto: no reading of the original four inputs could have
produced a conforming slice here. The specification was not under-specified at this point
— it was inconsistent, and only building against it made that visible.

### What the amendment costs — D-40, and it is the run's sharpest §11.4 result

**The rule left the code.** Before the amendment it was a flat prohibition: run a
namespace test over a type's references, done — checkable, and defeated by one interface.
After the amendment it is conditional on what the provider adapts, and that is not in the
assembly at all. It is in the determination that declares the read position's `boundary`.

So a checker must now:

1. read the determination store and find the read position for the fact this provider
   supplies — which needs a stated convention for matching a provider to its fact (**Q-34**;
   this session used the return type of the provider's single public method, D-41); and
2. decide from that position's `boundary` whether the carrier is transport.

**Step 2 has no machine-readable answer.** `determination.schema.json` gives
`read_provenance` as `{"type": "string"}` — free text. DSC-0003 says *"OIDC token claim,
validated at the gateway"*, and the only way to get "transport" out of that is to match on
prose. `ProviderTransportCarrierTests.IsTransportBorne` does exactly that and says so; it
is the method that should not exist.

`ProviderTransportCarrierTests` is nonetheless the first thing in this run to demonstrate
a profile rule being enforced *by reading the determinations*, against the real
`place-order.determinations.yaml`, rather than asserted. That is the evidence PRD §11.4
was missing, and it says: **the rule is enforceable, and the last step of the enforcement
is a regex over prose.**

### Proposed: a machine-readable carrier

Add to `$defs.position.boundary` — a change to the schema, hence proposed and not applied:

```json
"carrier": {
  "type": "string",
  "enum": ["transport", "store", "computed", "external-call"],
  "$comment": "What physically carries the fact to the act. read_provenance stays free
               text and says HOW it was established; carrier says WHAT delivers it, and
               is what a profile rule may condition on."
}
```

`read_provenance` and `carrier` answer different questions and should not be merged:
*"OIDC token claim, validated at the gateway"* is provenance; `transport` is carriage.
A rule that conditions on carriage currently has to infer it from a sentence about
provenance.

### Two gaps the amendment leaves

**Q-35 (F9).** The amended `must_not` is stated over "the position it adapts", and
`positions` distinguishes read from write. A provider that *records* to a transport sink —
posting to a webhook, say — adapts a write position, and the rule as proposed says nothing
about it. `OrderPlacedProvider` adapts a store, so this slice does not exercise it, but
the asymmetry is real: R-Q10 gave providers a write direction and R-Q06's amendment only
covers the read one.

**Q-36 (F12).** With the carrier enum added, the rule becomes conditional on a field the
*determination author* controls. An author who writes `carrier: transport` grants their
own provider the permission. Is that intended — the determination is the authority, so it
decides — or does it need a check that the carrier matches the fact's actual source, which
nothing can establish?

---

## R-GROUND — Emil, 2026-09-16

**Prompted by** this session reporting that `ProviderTransportCarrierTests.IsTransportBorne`
matched on prose — *"the method that should not exist"* — and proposing a `boundary.carrier`
enum as a remedy.

**Answer, verbatim.**

> we have the same issue for rules as all software. We cant add decisions to ground we
> havent modelled. We instead of using a regex, we need to build a proper model of what we
> want to determine on

**Ruled:** a determination may only be made over modelled ground. The regex is not a
pragmatic shortcut to be tolerated until the schema catches up — it is the symptom of a
rule determining on something nobody modelled, and the fix is the model.

### What it changes in the build

`IsTransportBorne` is **deleted**. `CarrierModel.cs` replaces it with:

* `Carrier` — a **closed** vocabulary: `transport | store | computed | external-call`;
* `DeterminationStore.ReadCarrier` — reads the modelled field and **never infers**. An
  absent field is unmodelled. A value outside the vocabulary is *also* unmodelled: an
  unrecognised string is not a licence to guess;
* `ProviderTransportRule.Evaluate` — returning **three** values, not two.

### The third verdict is the ruling's real content

`Verdict.Undeterminable` is kept strictly distinct from `Verdict.Breaches`.

**This is the schema's own argument, one level up.** `determination.schema.json` keeps
`silent` as a distinct extent state because collapsing it into `does-not-travel` *"makes
the revisit computation unsound the first time an axis is added"* (DP-1). Exactly the same
holds for enforcement: collapsing "I have no ground for this" into "this is broken" makes
a conformance report unsound the first time a carrier is modelled. **A checker that cannot
say *unmodelled* will lie, in whichever direction its author defaulted** — and the prose
regex was that lie, defaulted to *permit*.

The same distinction is already load-bearing elsewhere in the scheme: `does_not_cover`
requires the explicit `asserted-none` sentinel *"because an omitted uncovered set is
indistinguishable from an unconsidered one"* (DP-3). An unevaluated rule and a passing
rule are indistinguishable for the same reason, and need the same remedy.

### What the modelled check now reports

| Store | Carrier for `ActorIdentity` | Verdict |
|---|---|---|
| `place-order.determinations.yaml`, **as delivered** | not modelled | **Undeterminable** |
| fixture with `carrier: transport` | `transport` | **Conforms** |

Same rule, same code, no inference. So the honest §11.4 answer for this rule is neither
"checkable" nor "uncheckable": **fully mechanical once the ground is modelled, and
unrunnable until it is.** The rule was never the problem; the missing model was.

And the rule still bites without needing ground at all — a provider referencing no
transport type conforms whatever the carrier, so the prohibition is never vacuous. A
modelled `carrier: store` against a transport-referencing provider returns `Breaches`.

### The schema finding this exposes

`determination.schema.json` closes `position` with `additionalProperties: false` and
**leaves `boundary` open**. So:

1. `carrier: transport` is **already schema-valid today**. The proposal is a **closure,
   not a relaxation** — the field can be written right now, and nothing can rely on what
   it says.
2. More generally: **the one object a profile rule needs to determine on is the one object
   the schema does not close.** Everywhere else the schema is emphatic about closed unions
   — act types, extent states, allocation classes, boundary `kind`, `tick_rate` — and the
   `$comment` on the file says the prohibitions "are not advice: the forbidden shapes have
   no valid representation here". An open `boundary` is a hole in exactly that argument,
   and it is where the ground for this rule was supposed to live.

**Q-37 (F13).**
> Is `boundary` open deliberately — an extension point for carrier-like facts nobody has
> modelled yet — or is the missing `additionalProperties: false` an oversight? Under
> R-GROUND the two readings are not equivalent: an open object is unmodelled ground by
> construction, so anything written there can be read by a human and determined on by
> nothing.

### The fixture, and why it is not an authored determination

`tests/…/fixtures/dsc-0003.carrier-modelled.yaml` is DSC-0003 with one field added. It is
**not** in `inputs/`, nothing loads it but the test demonstrating the proposal, and its
`made_by` reads `gate3-fixture-not-a-determination`. The convention is borrowed from the
repository's own rule that fixture acceptances carry a fixture principal and live only
under `tests/`. The prohibition is on filling a specification gap by authoring; this
demonstrates what filling it would require, and leaves the filling to a principal.

### The generalisation, which outlives this rule

R-GROUND is not about carriers. Stated as this run met it:

> **A profile rule may only condition on ground the determination layer models. A rule
> that conditions on anything else is not enforceable — it is a reader's inference wearing
> an analyser's clothes, and it will pass or fail on a determination author's choice of
> words.**

Read back over the profile, this disqualifies more than the transport rule. `must_not`
"contains a conditional on **domain state**", "references a **persistence type**",
"performs **I/O** directly", "contains a **decision**" — each conditions on ground that is
nowhere modelled, in the determinations or the fact vocabulary. Under R-GROUND those four
are not "hard to check"; they are **not yet checkable at all**, and each needs its ground
modelled before the question "can Roslyn do it?" is even well-posed. That is a sharper
statement of what this report's §11.4 table called "not mechanically checkable as written",
and a more useful one, because it says what to do next.

---

## R-Q37 — Emil, 2026-09-16

**Question put (Q-37).** *"`determination.schema.json` closes `position` with
`additionalProperties: false` and leaves `boundary` open. So `carrier: transport` is
already schema-valid, and nothing can rely on what it says. Is `boundary` open
deliberately — an extension point for carrier-like facts nobody has modelled — or is the
missing `additionalProperties: false` an oversight?"*

**Answer, verbatim.**

> boundary should be closed, it's an oversight

**Ruled:** `boundary` is closed. The openness was not an extension point.

### The claim checks out, and checking it was worth doing

`SchemaClosureTests` walks the schema and confirms the ruling's premise: **`boundary` is
the only object the schema leaves open.** It is a single miss, not a pattern.

One near-miss worth recording, because it would otherwise look like a second hole:
`allocation` has no `additionalProperties` at its top level, but each of its three `oneOf`
branches carries `additionalProperties: false` over its own properties, so an unexpected
key fails every branch and the `oneOf` fails. **Closure by branch is still closure.**
`boundary` has no such structure.

Every other object — `address`, `extent`, the axis object, `acceptance`, `closure`,
`proxy`, `principal`, `position`, `provenance`, and the root — is closed directly.

### Applying it is a two-part change, not a one-liner

**Closing `boundary` alone would forbid the modelled carrier**, which is the ground
R-GROUND requires. `SchemaClosureTests.Closing_boundary_without_declaring_carrier_would_forbid_the_modelled_ground`
asserts exactly this: `carrier` is the *only* undeclared key in the R-GROUND fixture, so
the closure and the declaration are coupled and must land in the same change.

Stated plainly: **the open `boundary` is currently the only reason a modelled carrier can
be written at all — and is also the only reason it cannot be relied on.** Close it without
declaring `carrier` and the field becomes invalid; close it with the declaration and
R-GROUND's ground becomes representable *and* trustworthy. The two rulings are one patch.

**Migration cost: none.** Every boundary key in `place-order.determinations.yaml` as
delivered is already declared — `kind`, `source`, `read_provenance`, `tick_rate`,
`consumer`, `consumption_observable`. The store validates unchanged under the closure.
Asserted by `Closing_boundary_does_not_invalidate_the_delivered_store`.

### Proposed patch — `$defs.position.boundary`, not applied

```json
"boundary": {
  "$comment": "DP-7. external: read here, produced by no act in scope. terminal: written here, read by no act in scope. internal: resolves within scope.",
  "type": "object",
  "required": ["kind"],
  "additionalProperties": false,
  "properties": {
    "kind": { "type": "string", "enum": ["internal", "external", "terminal"] },
    "source": { "type": "string" },
    "consumer": { "type": "string" },
    "read_provenance": { "type": "string" },
    "carrier": {
      "type": "string",
      "enum": ["transport", "store", "computed", "external-call"],
      "$comment": "R-GROUND. What physically carries the fact to the act. read_provenance stays free text and says HOW the fact was established; carrier says WHAT delivers it, and is the only one of the two a profile rule may condition on."
    },
    "tick_rate": { "type": "string", "enum": ["static", "slow", "fast", "unknown"] },
    "consumption_observable": { "type": "boolean" }
  },
  "allOf": [
    { "if": { "properties": { "kind": { "const": "external" } } },
      "then": { "required": ["kind", "source", "read_provenance", "tick_rate"] } },
    { "if": { "properties": { "kind": { "const": "terminal" } } },
      "then": { "required": ["kind", "consumer", "consumption_observable"] } }
  ]
}
```

**Q-38 (F13).** Left open deliberately rather than resolved:
> Should `carrier` be **required** for an `external` boundary, alongside `source`,
> `read_provenance` and `tick_rate`? The `allOf` already obliges an external read to
> declare where it comes from and how fast it ticks. Under R-GROUND, a rule conditioning
> on carriage is unrunnable without it — so leaving `carrier` optional means every
> external position may silently produce `Undeterminable`. Making it required is a
> breaking change to the delivered store: DSC-0003 would fail validation until amended.
> That is a principal's call about migration cost, not a builder's.

### What this closes about the schema's own argument

The file's header `$comment` says the prohibitions *"are not advice: the forbidden shapes
have no valid representation here."* That claim was true of the whole schema except one
object — and the exception was the one object a profile rule turned out to need to
determine on. With R-Q37 applied it becomes true without exception, and R-GROUND's
requirement — that a rule may only condition on modelled ground — acquires a schema that
can actually enforce "modelled".

---

## R-Q38 — Emil, 2026-09-16

**Question put (Q-38).** *"Should `carrier` be **required** for an `external` boundary? …
Making it required is a breaking change to the delivered store: DSC-0003 would fail
validation until amended. That is a principal's call about migration cost, not a
builder's."*

**Answer, verbatim.**

> if we dont supply carrier that needs to an explicit decision made by a human. Because
> its vital for the systems design

**Ruled:** `carrier` is required. Not supplying one is permitted — as a **stated,
attributed decision carried by a named human**, never as an omission.

### The question was framed wrongly and the ruling says so

Q-38 offered required-or-optional and treated the validation break as a **cost**. It is
the **mechanism**. A determination that fails validation for want of a carrier is the
system asking a human a question it cannot answer itself — which, given the ruling's
reason, is precisely what should happen. DSC-0003 failing until amended is not migration
debt; it is the one place the carriage of an identity fact gets a decision.

This session recorded Q-38 as a migration call and would have defaulted to optional. That
default would have let every external position drift into silent `Undeterminable` — the
same failure mode as the deleted regex, one level up. **Filed as a misread: the builder
priced the break instead of reading what the break is for.**

### It splits the third verdict in two

R-GROUND gave the checker `Undeterminable`. R-Q38 says that value conflates two states
demanding different actions:

| | Means | Remedy |
|---|---|---|
| `UndeterminableUnattributed` | the field is absent; **nobody decided** | amend the determination — and under R-Q37's closed schema this is a validation failure, so it cannot pass silently |
| `UndeterminableCarried` | a **named principal** stated that no carrier is supplied, and why | none. A filed risk with an owner. |

The checker learns nothing more about the carrier in the second case. It learns **whose
problem it is**, and that is the difference between a report that shrugs and a report
someone can act on.

### The schema already has this device, twice, and has not named it

1. `does_not_cover` requires the explicit `asserted-none` sentinel, *"because an omitted
   uncovered set is indistinguishable from an unconsidered one"* (DP-3).
2. `allocation.residual` requires a carrying actor kind **and** an accountable principal,
   and `principal.kind` omits `machine` *"by design"* — *"a model identity cannot be an
   accepting principal"* (PR-2/DP-4).
3. R-Q38's carrier is the same device a third time.

**Proposed: name it.** The schema appears to hold an unnamed invariant —

> **No silent omission. Wherever a field's absence changes what a downstream consumer may
> conclude, absence must be *stated*, and stated absence must be *attributed to a
> non-machine principal*.**

Naming it would make the next instance derivable instead of rediscovered. This is the
third time it has been rediscovered, and this run found the third one by building against
the schema rather than by reading it.

### Proposed patch — supersedes the R-Q37 draft above

```json
"carrier": {
  "$comment": "R-GROUND / R-Q38. What physically carries the fact to the act. read_provenance stays free text and says HOW the fact was established; carrier says WHAT delivers it, and is the only one of the two a profile rule may condition on. Absence is not a value: it is stated and attributed, per the store's standing pattern (cf. does_not_cover's asserted-none, and residual's principal).",
  "oneOf": [
    { "type": "string", "enum": ["transport", "store", "computed", "external-call"] },
    {
      "type": "object",
      "required": ["state", "principal", "reason"],
      "additionalProperties": false,
      "properties": {
        "state": { "type": "string", "const": "not-supplied" },
        "principal": {
          "type": "object",
          "required": ["kind", "identifier"],
          "additionalProperties": false,
          "properties": {
            "kind": { "type": "string", "enum": ["human", "team", "external-party"] },
            "identifier": { "type": "string", "minLength": 1 }
          }
        },
        "reason": { "type": "string", "minLength": 1 }
      }
    }
  ]
}
```

and `carrier` joins the `external` branch of the `allOf`:

```json
{ "if":   { "properties": { "kind": { "const": "external" } } },
  "then": { "required": ["kind", "source", "read_provenance", "tick_rate", "carrier"] } }
```

**`principal.kind` omits `machine`, deliberately and for the schema's own stated reason.**
An agent may not decide that the carriage of a fact does not matter. A withholding that
names no acceptable principal reads back as **unattributed**, not as a decision — the safe
direction, asserted by
`ProviderTransportCarrierTests.A_withholding_without_an_accountable_principal_is_not_a_decision`.

### Migration, now that the break is the point

Applying R-Q37 + R-GROUND + R-Q38 as one patch invalidates exactly one record in the
delivered store: **DSC-0003**, whose `ActorIdentity` read position is `external` and
declares no carrier. Every other position is `internal` or `terminal` and is untouched.
So the total migration is one determination, and amending it is the human decision the
ruling demands.

**Q-39 (F3), and this one is a design question rather than a notation one.**
> Should `carrier` be required for `internal` boundaries too? `Cart` is internal and read
> through a provider that adapts *something* — a store, today. Under R-GROUND a rule
> conditioning on carriage is unrunnable for internal positions for exactly the same
> reason it was for external ones, and the amended provider rule applies to every
> provider. The `allOf` currently singles out `external` because that is where `source`
> and `tick_rate` matter; carriage may not follow the same line.

---

## R-Q39 — Emil, 2026-09-16

**Question put (Q-39).** *"Should `carrier` be required for `internal` boundaries too?
`Cart` is internal and read through a provider that adapts something — a store, today.
Under R-GROUND a rule conditioning on carriage is unrunnable for internal positions for
exactly the same reason it was for external ones…"*

**Answer, verbatim.**

> carrier is for actors acting against us - if we act against someone its their
> responsibility to name the carrier and take that into account for their system design

**Ruled:** carriage is a property of the **inbound edge**, not of every position. It is
ours to name where an actor acts against us, and theirs to name where we act against them.

### The boundary kinds already say which edge a position is on

No new field is needed. `role` and `boundary.kind` together give the edge:

| `role` + `kind` | Edge | Whose carrier |
|---|---|---|
| `read` + `external` | **inbound** — an actor acts against us | **ours.** Required, per R-Q38 |
| `write` + `terminal` | **outbound** — we act against them | **theirs**, in their store, for their design |
| anything `internal` | neither — the chain resolves in our own scope | nobody's; no outside party exists |

That is a derivation, not an assertion: `ModelledPosition.Edge` reads it off the two
fields the schema already has. **The ruling required no addition to the notation** — which
is itself worth recording, because the previous four rulings each did.

### Read back off the delivered store

`Only_the_inbound_edge_carries_a_carrier_question` walks
`place-order.determinations.yaml` and finds **exactly one** inbound position in the entire
file: `ActorIdentity:read` on DSC-0003 — the one position this whole carrier argument has
been about. Every other read is `internal`; the two writes, `OrderSummary` and
`OrderConfirmed`, are `terminal`.

So the migration from R-Q37 + R-GROUND + R-Q38 stands at **one record**, and R-Q39
confirms it is one record *for a reason* rather than by accident: DSC-0003 is the only
place in the store where anything acts against us.

### A fifth verdict, and it is not a sixth Undeterminable

`Verdict.NotApplicable`. The distinction the rulings have now forced, in full:

| Verdict | Means |
|---|---|
| `Conforms` | evaluated, holds |
| `Breaches` | evaluated, broken |
| `UndeterminableUnattributed` | in scope, unanswered, **nobody decided** — amend the determination |
| `UndeterminableCarried` | in scope, unanswered, **a named principal decided** — a filed risk with an owner |
| `NotApplicable` | **never ours to ask** — R-Q39 |

`Undeterminable` means the question is ours and open. `NotApplicable` means it was never
ours. Collapsing them would put every internal position into a queue of things to go and
model, which is the opposite of what the ruling says.

### It answers Q-35 as a consequence

Q-35 asked what the amended `must_not` says about a provider that *records* to a transport
sink. The answer is that the rule has no purchase there at all: a write to a terminal
consumer is us acting outward, so the carriage is theirs, so there is nothing for the rule
to condition on. **Not a hole — the rule's correct scope.**

### The compiler found something a reviewer would not have

The breach example in `ProviderTransportCarrierTests` used an **internal** position with
`carrier: store`. Under R-Q39 that position is out of scope, so the example silently
stopped demonstrating a breach and started demonstrating `NotApplicable`. It only surfaced
because the record's shape changed and the build broke.

**Bounding a rule's scope silently invalidates the examples that justified it.** Nothing
in the specification, the schema or the profile would have flagged that, and a
prose-and-review process would very likely have kept the stale example. Rewritten to the
only shape that can now breach: an inbound position whose carrier is modelled as
something other than transport, with the provider reaching for the request anyway.

### The gap it leaves — Q-40

**Q-40 (F9/F12).**
> An outbound provider that references a transport type — posting `OrderConfirmed` to
> fulfilment over HTTP — is now `NotApplicable`: nothing in the amended `must_not` permits
> it and nothing forbids it. It is **ungoverned**, and
> `An_outbound_transport_reference_is_ungoverned_not_permitted` asserts that rather than
> letting it read as conformance.
>
> Is ungoverned the intent? The schema is not silent about the outbound edge — a
> `terminal` boundary requires `consumer` and `consumption_observable`, so we *do* model
> claims about what happens after we act. If we model whether they can be observed
> consuming it, the reason carriage is different needs saying, or the outbound edge needs
> its own rule.

*Proceeded under:* ungoverned, asserted and named rather than resolved.

---

## R-Q12 — Emil, 2026-09-16

**Question put (Q-12).** *"The controller must 'return a transport result derived from the
handler's Accepted or Rejected'. Derived how? Nothing states the status for an accepted
command, the status for a rejected one, whether a Location header is owed, or what the
body is. Two readers produce two incompatible APIs from this profile and both conform."*

**Answer, verbatim.**

> Accepted is 201 with Location, Rejected is 422

**Ruled.** The largest single hole this run found in the profile is closed.

### The guess matched, and that is not evidence the profile was sufficient

This session invented exactly this mapping at Gate B and reported it as the run's biggest
hole. **The match proves nothing about legibility.** A second reader choosing 200/400, or
202/409, would have conformed equally well against the profile as delivered, because the
profile still does not say. Convergence between an author and a reader who share
REST convention is not the specification doing work. The gap was real and the ruling, not
the guess, is what closed it.

### Two paths the ruling does not reach

R-Q12 settles the two branches the profile's rule names. It leaves the exits that are
*neither* Accepted nor Rejected, and those are still this session's inventions:

| Path | Invented | Why the ruling does not cover it |
|---|---|---|
| invalid payload → **400** | D-23 | DSC-0002's check happens before the handler; it produces no outcome to derive from |
| unsuppliable read position → **401** / **404** | D-19, D-31 | the profile admits two handler exits; this is a third |

**Q-12b (F7)** remains open: what transport result is owed for an act that never reached
a verdict?

### And the ruling collides with DSC-0004

**This is the finding of this round.**

`Location: /orders/{id}` invites the caller to read what it just wrote. The only act that
can answer that read is the `OrderSummary` read-model — and DSC-0004 declares that it *"may
lag the event stream by up to five seconds"*, with a `proxy.known_divergence` reading:

> *"A five-second window is acceptable for browsing and is **not acceptable immediately
> after the reader's own write, where they expect to see their change**. The measured lag
> says nothing about that case."*

That is a description of precisely the case a `Location` header creates. R-Q40's outbox
widens the window further: at 201 the event is durable but not yet on the bus, so the
projection has not begun to lag yet.

**The determination store already contained the warning that this transport decision
triggers, and nothing in the scheme connects the two.** The resolution condition checks
that facts resolve into the vocabulary. Nothing checks whether a build-time decision on
one act lands inside a filed `known_divergence` on another. The two records are
individually conformant and jointly produce a 404 for a caller following a `Location` we
told it to follow.

**Q-43 (F12).**
> Should a `proxy.known_divergence` be *reachable* — something a later determination or a
> profile rule can be checked against? It is currently prose in one record, read by a
> human once, at the moment it is written. This run found the collision by building both
> ends; nothing in the notation would have surfaced it, and the more determinations exist
> the less likely a reader is to hold them all at once.

Implemented as ruled: the `Location` header is emitted. The collision is reported, not
worked around.

---

## R-Q40 — Emil, 2026-09-16

**Question put (Q-40).** *"An outbound provider that references a transport type — posting
`OrderConfirmed` to fulfilment over HTTP — is now `NotApplicable`: nothing in the amended
`must_not` permits it and nothing forbids it. It is ungoverned. Is that the intent?"*

**Answer, verbatim.**

> we need to name the transport and how to handle the cases if thats ours to own, this is
> pr technology. For a event driven system i would expect us to always have an outbox
> pattern before sending to the eventbus

**Ruled:** outbound carriage **is** ours to own. The transport and its case handling are
named **per technology** — which places them in the profile, not in the determination
layer. And for an event-driven system the shape is fixed: **an outbox, always, before the
bus.**

### The split this settles

R-Q39 said carriage of an inbound fact is ours and carriage of an outbound fact is the
consumer's. R-Q40 refines that, and the line is not where this session drew it:

* **What the consumer does with the fact** — theirs. R-Q39 stands.
* **How we get it to them, and what we do when that fails** — **ours**, named per
  technology, in the profile.

So Q-40's "ungoverned" was the right reading of the profile as delivered and the wrong
place to leave it. The outbound edge is governed; the profile simply has no section for it.

### What changed in the build

`OrderPlacedProvider` writes to an `IOutbox`, not to a sink. Entries land `Pending`.

This retires **D-27**, which has stood since Gate B as *"a failed append loses a placed
order"*. With an outbox the enqueue is the durable act: it either succeeds before the
caller is told anything, or the caller is told it failed. **The loss window does not
vanish — it moves to the relay**, where a crash leaves the event stored and unpublished,
which is recoverable rather than lost. That is the pattern working, and it is why the
ruling is right.

### What is deliberately not built

**The relay is a separate act and has no profile to conform to.** Draining the outbox onto
the bus is an automation slice, and `profile-rest-api-v1` says in as many words:

> *"No profile for read-model, automation or translation slices. One act type, one
> profile, one slice."*

So a relay built here would conform to nothing. `Nothing_in_this_slice_dispatches_the_outbox`
asserts the absence so it reads as a boundary rather than an omission.

**Q-41 (F1/F8).**
> The outbox relay is an act. `ordering.eventmodel.yaml` declares no automation slice for
> it — the act vocabulary has six slices, all `command` or `read-model`, and the schema's
> `act_type` enum carries `automation` with nothing using it. Should the relay be an act
> in the vocabulary? If it is, it needs a profile that does not exist. If it is not, then
> **the thing that actually delivers every event this context produces is outside the
> model entirely**, which is a large omission for a scheme whose point is that the act
> vocabulary is complete.

### Proposed profile section — named, not authored

R-Q40 says the transport is named per technology, so it belongs in the profile body
alongside `roles:`. Proposed shape, for ratification:

```yaml
outbound:
  # R-Q40. What carries our facts outward, and what happens when it fails.
  # Per technology: this block is what changes when the stack changes.
  pattern: outbox
  enforcement: analyser
  must:
    - "a write position is recorded to the outbox, never to the bus directly"
    - "the outbox entry is durable before the act's transport result is returned"
    - "the relay that drains the outbox is a separate act"
  cases:
    relay_unavailable:   "entries remain pending; no fact is lost, publication is delayed"
    duplicate_dispatch:  "at-least-once; consumers deduplicate on the event's identity"
    poison_entry:        "UNSPECIFIED — needs a ruling"
```

**`must` rule 1 is analyser-checkable today** by the same reflection the role rules use: a
provider adapting a write position may reference the outbox type and not a bus type. Rules
2 and 3 are not — "durable before the result is returned" is an ordering claim over a call
graph, and "a separate act" needs the act vocabulary, which brings back Q-41.

**Q-42 (F9).** `poison_entry` above is left `UNSPECIFIED` deliberately rather than filled:
> An entry the relay cannot publish — a schema the bus rejects, a consumer permanently
> gone — is neither lost nor delivered. Under the store's own standing pattern (*no silent
> omission*, R-Q38) this needs a stated disposition with an owner, not a default. What is
> it?

---

## R-Q41 — Emil, 2026-09-16

**Question put (Q-41).** *"The outbox relay is an act. `ordering.eventmodel.yaml` declares
no automation slice for it… Should the relay be an act in the vocabulary? If it is, it
needs a profile that does not exist. If it is not, then the thing that actually delivers
every event this context produces is outside the model entirely."*

**Answer, verbatim.**

> Q-41 is assurance for external delivery - and thats important for us to have an answer
> for. How sure do we need to be of this payload reaching the external system

**Ruled:** the relay is not plumbing. It is the mechanism that **discharges a delivery
assurance requirement**, and the requirement is a determination — a statement about risk
we are willing to carry, not about technology.

### The question splits, and the schema models exactly one half

**The ceiling is already modelled**, in `boundary.consumption_observable`. If we cannot
observe them consuming it, **no relay however good gets us above "we sent it"**. That is a
hard constraint and it is derivable from the store as delivered:

| Determination | Consumer | `consumption_observable` | Ceiling |
|---|---|---|---|
| DSC-0006 · `OrderConfirmed` | fulfilment | `true` | **Confirmed** — consumption is observable |
| DSC-0004 · `OrderSummary` | the order history screen | `false` | **Dispatched** — nothing above this is obtainable |

`Consumption_observability_sets_the_ceiling_on_assurance` asserts it against the real file.

**The requirement is modelled nowhere.** There is no field in
`determination.schema.json` that says how sure we need to be. Every terminal position in
the store reads back `UndeterminableUnattributed` — nobody decided, and under R-Q37's
closed schema that is exactly the state that should be impossible to reach silently.

**`consumption_observable` is a CAPABILITY claim — can we see it? The ruling asks for a
REQUIREMENT claim — how sure must we be?** Those are different questions and only the
first is written down. Deriving the second from the first would be the regex mistake one
field over, so `ReadRequiredAssurance` looks for a field, finds none, and says so.

### The most useful thing the model produces

A requirement above the ceiling is **not a demanding requirement — it is an unsatisfiable
one**, and no amount of relay engineering closes it. Demanding `Confirmed` delivery to the
order history screen, where `consumption_observable: false`, means the *boundary* has to
change, not the code. `A_requirement_above_the_ceiling_cannot_be_met_by_any_relay` pins all
four combinations.

That is a check worth having and it needs one new field to run.

### Proposed field — `boundary.delivery_assurance`

Required for a `terminal` boundary, with absence stated and attributed exactly as R-Q38
requires of `carrier`:

```json
"delivery_assurance": {
  "$comment": "R-Q41. How sure we need to be that the payload reached the consumer. A requirement, not a capability: consumption_observable says what we CAN establish, this says what we MUST. A requirement above what consumption_observable permits is unsatisfiable by any relay and the boundary must change instead.",
  "oneOf": [
    { "type": "string", "enum": ["unassured", "enqueued", "dispatched", "confirmed"] },
    { "$ref": "#/$defs/statedAbsence" }
  ]
}
```

with `terminal`'s `allOf` branch becoming
`["kind", "consumer", "consumption_observable", "delivery_assurance"]`.

**`$defs/statedAbsence` is proposed as a shared definition**, not a third copy. R-Q38's
carrier absence, and this one, are the same shape — `{state: not-supplied, principal,
reason}` — and `does_not_cover`'s `asserted-none` and `allocation.residual`'s principal are
the same *device*. Extracting it is the mechanical half of naming the **no silent
omission** invariant proposed under R-Q38. Fourth instance; time to name it.

### Where the question actually bites, and it is not this slice

`PlaceOrder` writes `OrderPlaced` with boundary `internal` — read in scope by
`ConfirmOrder` and `OrderSummary`. **This slice has no external delivery at all.** The
external delivery in this context is `OrderConfirmed` → fulfilment, one act downstream, at
`ConfirmOrder`. `This_slice_has_no_external_delivery_at_all` asserts it.

So the ruling names a real requirement that the built slice does not exercise, while the
store does. The assurance question is live at `ConfirmOrder` and nowhere in `PlaceOrder`,
and the outbox this slice writes to is feeding **internal** consumers.

**Q-44 (F9).**
> R-Q40 requires an outbox for this slice's write, whose delivery is internal. R-Q41 frames
> the relay as assurance for *external* delivery. Both can hold — an outbox for every
> write, assurance requirements only where we cross out — but the scoping is not stated.
> Is the outbox required for internal writes too, and if so on what grounds, given the
> assurance argument does not apply to them?

**Q-45 (F12/F13), and it is the one worth deciding first.**
> R-Q40 put the transport and its case handling in the **profile**, because they are "pr
> technology". R-Q41 puts *how sure we need to be* in the **determination layer**, because
> it is a statement about risk. That split looks right and it is not stated anywhere:
> **the requirement is a determination, the mechanism is a profile, and the profile must
> be able to show it discharges the requirement.** Nothing currently connects the two —
> an `outbound: pattern: outbox` profile section and a
> `delivery_assurance: confirmed` determination would sit in different files with no
> relation asserted between them. Confirming the split, and then requiring the link, is
> what would make R-Q41's answer enforceable rather than merely written down.

### What is still not answered about the relay

The ruling tells us *why* the relay matters. It does not settle Q-41's original question:
whether the relay is an **act in the vocabulary**. The act vocabulary still declares six
slices, all `command` or `read-model`; the schema's `act_type` enum still carries
`automation` with nothing using it; and the profile still covers no automation slice. A
relay built today conforms to nothing, so it remains deliberately unbuilt and
`Nothing_in_this_slice_dispatches_the_outbox` still asserts its absence.

**Q-41 is therefore answered as to purpose and open as to modelling**, and recorded that
way rather than closed.

---

## R-Q45 — Emil, 2026-09-16

**Question put (Q-45).** *"The requirement is a determination, the mechanism is a profile,
and the profile must be able to show it discharges the requirement. Nothing currently
connects the two… Confirming the split, and then requiring the link, is what would make
R-Q41's answer enforceable rather than merely written down."*

**Answer, verbatim.**

> the profile must declare which assurance level it discharges and the levels here matter -
> we have delivery_assurance every where and then we might have different profile
> implementations that support the given need. and that assurance. And its a common failure
> level, we think we only need it in event systems where we need to ensure that the event
> is delivered. But we dont do it in HTTP systems because we often assume that the external
> HTTP call is important to the current one we are serving. Even though that might not be
> the case

**Ruled:** the split is confirmed and the link is required. Three consequences, and the
third is about the profile this run was built against.

### 1. The join is checkable

The determination states `delivery_assurance`; the profile declares what it discharges;
conformance is `discharges >= required`. `AssuranceDischargeRule` implements it, and it
decides where both halves are present and reports `UndeterminableUnattributed` where either
is missing.

There is a second failure the join catches and it is the dangerous one: a profile that
**overclaims** — declares `confirmed`, implements a synchronous call — breaches whatever the
act requires, because it reads as assured and is not. That is checked before the
requirement is even consulted.

### 2. `delivery_assurance` is everywhere, which answers Q-44

Not only at terminal boundaries. Every write hands a fact to a later act and every such
handover can fail, so this slice's `internal` write has a requirement too. **R-Q40's outbox
is therefore not belt-and-braces for `OrderPlaced`** — Q-44 asked on what grounds an
internal write needs one, and this is the grounds.

The store as delivered declares a requirement on **none** of its three write positions —
one internal, two terminal.

### 3. The named failure mode, and this profile is an instance of it

> *"we think we only need it in event systems where we need to ensure that the event is
> delivered. But we dont do it in HTTP systems because we often assume that the external
> HTTP call is important to the current one we are serving. Even though that might not be
> the case"*

**"The caller is waiting" is not an assurance mechanism.** A synchronous outbound call
inside the request being served discharges *nothing*: a crash between the call and the
commit loses it as completely as a dropped event, and the premise that the call matters to
the request being served is frequently false — it is an assumption inherited from the
shape of the stack, not a decision anyone took.

Modelled as `AssuranceMechanism.SynchronousCallInRequest → Unassured` and asserted.

**And `rest-api-v1` is exactly the kind of profile that inherits the assumption.** It is an
HTTP profile; it declares no assurance level; and without R-Q40 its mechanism would have
been precisely this one. Computing what it actually discharges as built:

| | |
|---|---|
| Declared in the profile | **nothing** — the profile has no such field, which is what R-Q45 adds |
| Mechanism as built | outbox (R-Q40), relay deliberately unbuilt (Q-41) |
| **Actually discharges** | **`Enqueued`** — one step off zero, not four |

So R-Q40 is what moved this profile off `Unassured`, and it moved it exactly one level. A
consumer needing `Dispatched` or `Confirmed` is not served by `rest-api-v1` as it stands,
and until R-Q45's declaration exists nothing says so.

### 4. And it breaks DSC-0100's travel

This is the consequence worth carrying furthest.

> *"we might have different profile implementations that support the given need"*

If a profile is chosen by the assurance an act requires, then **profile selection varies per
act** — and `DSC-0100` declares the opposite:

```yaml
  extent:
    axes:
      slice-type:
        state: travels-to
        region: all-command-slices
        reason: >-
          A pattern-level determination. Every command slice ever written in
          this context collects it.
```

Under R-Q45, **every command slice does not collect `rest-api-v1`**. Each collects the
profile whose discharged assurance meets its need. A pattern-level determination cannot
pin a profile whose applicability is per-act, so DSC-0100's `slice-type` extent is wrong —
not stale, wrong at the point the ruling lands.

This is the **second** thing R-Q10 and R-Q45 between them require of DSC-0100: R-Q10 needs
its `statement` superseded (the "where external data is required" clause), and R-Q45 needs
its `extent` superseded. One supersession can carry both, and it should.

Note also what this does to the CG-R-53 anchor problem the profile already concedes.
DSC-0100 anchors on `PlaceOrder` "arbitrarily" and travels. If travel is now conditional on
a per-act requirement, the arbitrary anchor stops being a cosmetic defect and starts
selecting which act's needs the pinned profile happens to match.

**Q-46 (F9).** *"Everywhere"* is doing work and its edge is not stated:
> Does `delivery_assurance` apply to **read** positions as well as writes? A read can fail
> to arrive too — DSC-0003's `ActorIdentity` is an OIDC claim that may simply not be there,
> which this slice handles by throwing (D-19). That is an assurance question wearing a
> different hat. This session applied the field to writes only, because "reaching the
> external system" is a delivery phrase.

**Q-47 (F8/F1).**
> If several profiles exist per act type, differentiated by discharged assurance, **how is
> the profile selected for a given act?** Today one determination pins one profile to all
> command slices by an arbitrary anchor. Selection by requirement needs either a
> determination per act naming its profile, or a resolution rule that picks the profile
> whose discharge meets the act's need — and if two qualify, a tie-break. None of that
> exists, and it is the mechanism R-Q45 presupposes.

### Proposed profile field — not authored

```yaml
profile: rest-api-v1
act_type: command
stack: "C# / ASP.NET Core"
discharges:
  # R-Q45. The assurance level this profile's mechanism actually reaches. A profile may
  # not be selected for an act whose delivery_assurance exceeds it. Declaring more than
  # the mechanism delivers is a defect in the profile, not in the slices built to it.
  delivery_assurance: enqueued
  mechanism: outbox            # relay unbuilt; dispatched and confirmed are not reachable
```

---

## CG-R-134 … CG-R-137 — Emil, 2026-09-15 (received 2026-09-17)

Four rulings on the *answering method*, in response to `triage.md`. Filed verbatim at
`inputs/../rulings-cg-r-134-137.md`; what follows is what each changes here.

### CG-R-134 — answer the class, not the instance; breadth before depth

> *"a question is answered at the level of the rule that generates it. Where an instance
> answer would leave the family intact, the family is what gets ruled."*
> *"One question from each untouched category before going deep again."*

Both observations from `triage.md` accepted and generalised past this session. No change
to the artefact; it governs the next pass and is recorded so the next session inherits it.

One line in the ruling is worth carrying into the Gate C report as a method finding:

> *"The self-diagnosis is what makes this rulable. A session that reported eleven rulings
> without noticing they were one chain would have produced the same artefact and no
> finding."*

### CG-R-135 — the raw count is retired as a headline

> *"the raw count does not appear as a headline anywhere."*

Applied. The partition — **~10 specification gaps, ~14 ordinary design, ~11 about the
scheme** — is the reportable figure, and *nine of the ten gaps fall inside moves 1 and 3*
is the actionable statement. The raw count survives only inside `questions.md` as
bookkeeping.

The ruling's reason is recorded because it corrects a bias in this session's own
reporting: **both reframings of the count run the same way — the raw count overstates the
problem, and reporting it misrepresents the run in the direction of a worse result than
was found.** This session led with 29, then 49, on each revision.

### CG-R-136 — Q-02 answered, and a figure of this session's is void

> *"The Gate 3 prompt says per frame category in three places and defines it nowhere."*
> *"This is CG-R-63 in its seventh instance… a measurement whose denominator is defined
> after the measurement is not a measurement."*

**Q-02 is answered: the omission is the principal's, not a gap in the reading.** Three
consequences, all applied:

1. **The "3 of 13 settled" figure is VOID** — over a denominator invented after the
   measurement, and *"not partially salvaged"*. Struck from `README.md`,
   `gate-c-report.md` and `gate-c-report-baseline.md`. The "5 of 13 generously counted"
   figure goes with it, for the same reason.
2. **The Gate C comparison against the notation experiment's category list is DEFERRED,
   not repaired.** Inventing a frame now to compare against would make a disagreement
   between the lists indistinguishable from a disagreement about vocabulary. It waits on
   the notation list's ratification, which waits on the reconciliation, which is held.
3. **The thirteen categories are retained as this session's working scheme**, labelled
   invented-after-build, and **are not presented as a frame.**

### CG-R-137 — the four moves, ordered

| | Move | Owner |
|---|---|---|
| 1 | Declare the fact type space | **Emil.** *"The fact vocabulary is the domain model, I authored `ordering.eventmodel.yaml`, and the fields belong with it. I will produce it."* |
| — | The nine one-liners | this session, after move 1, before move 3 |
| 2 | `allocation.class` — principle ruled below; instance awaits the two readings | principle: Emil. Readings: **this session — see `allocation-readings.md`** |
| 3 | Individuation, carrying the DSC-0100 supersession | supersession is Emil's, *"lands with the profile repair"* |
| 4 | R-GROUND across the whole profile | *"correctly named as a modelling exercise rather than a ruling"* |

**The principle ruled, verbatim:**

> **`allocation.class` describes how a determination is held, not whether it has been acted
> on.** `residual` means nothing in the specification settles it and a named actor carries
> it — recording it as residual does not discharge it. `pinned` means it is settled;
> `checked` means a predicate settles acceptability.

The instance — whether DSC-0005's currency rejection belongs in the slice — is **not
ruled**, and is not to be ruled from a summary. The two readings and what each does to the
slice are in `allocation-readings.md`. Q-04 remains open.
