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
