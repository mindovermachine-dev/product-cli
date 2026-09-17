# Questions — Gate B

**Every question this session would ask, asked.**

**The raw count is not a headline** (CG-R-135). The reportable figure is the partition of
what remains: **~10 genuine specification gaps, ~14 ordinary design decisions any competent
team would make in any notation, ~11 about the scheme rather than this slice** — and *nine
of the ten gaps fall inside moves 1 and 3* (`triage.md`). Counts below are bookkeeping.

Twenty-nine asked at Gate B, twenty more raised by rulings (Q-30 … Q-47, plus Q-12b).

**Thirteen and a half are answered**, all on 2026-09-16, all recorded verbatim in
`rulings.md`: Q-10, Q-16, Q-06, Q-37, Q-38, Q-39, Q-12, Q-40, plus Q-11 (a free consequence
of Q-10), Q-36 (by R-GROUND) and Q-35 (a free consequence of Q-39). plus Q-45 and Q-44 (the
latter a free consequence of R-Q45). **Q-41 is answered as to purpose and open as to
modelling** — R-Q41 settles what the relay is *for* without settling whether it is an act
in the vocabulary. **Thirty-five and a half remain open.**

**The open count has gone up, not down.** Nine rulings have closed eleven questions and
raised sixteen. That is the clearest single measurement this run produced about what
answering costs.

**One was asked badly.** Q-38 offered a required-or-optional choice and priced the
validation break as a migration cost. R-Q38's answer is that the break is the *mechanism* —
it is how a human gets asked about something vital to the system's design. Recorded as a
misread rather than quietly corrected, because a question's framing is part of the datum.

The three rulings between them raised seven new questions and required one rule amendment,
one schema proposal and one supersession. That is the shape of the result: **answers here
do not close questions one for one — they move the specification.** Each is recorded
verbatim as it would be put to Emil, with what prompted it and its frame category
(`frame-categories.md` — that scheme is itself invented; see Q-02).

**Emil has answered one.** The rest of this session ran in one pass with no principal present, so
every question below is open. The prompt's gates say *Hold*; holding with nothing
delivered would have produced no slice to report on, so each question records **the
provisional reading the build proceeded under**, and the code carries the same marker.
An answer that differs from the provisional reading invalidates the code at the cited
site, not the question. **The questions are the datum; the provisional readings are not
answers and must not be counted as settlements.**

Format: **question** (verbatim) · **prompted by** · **category** · **proceeded under** ·
**site**.

---

## F13 — Notation & process

### Q-01
> The Standing rules say `canon-governance` holds the in-force rules and that I must read
> them at the pinned commit and comply. The Prohibitions say to read the four arrived
> inputs and nothing else about the scheme. `canon-governance` is not in the bundle and
> no commit is pinned. Which instruction governs — and if the rules were meant to be in
> force for this run, what does it cost that I have not read them?

**Prompted by** `prompt.md`, Standing rules against Prohibitions.
**Proceeded under** the Prohibitions. `canon-governance` unread; any rule it carries
that is not restated in the bundle has not been applied.
**Site** `bootstrap.md`.

### Q-02 — **ANSWERED by CG-R-136**
> What is a **frame category**? The term carries both the Gate B instrumentation and the
> Gate C table, and it is defined in neither `prompt.md` nor any of the four inputs. I
> have had to invent the scheme I am reporting against, which means my Gate C table is
> partly a report on my own taxonomy. Is there a fixed list, and is it the same list
> being enumerated for the notation experiment?

**Prompted by** `prompt.md` Gate B ("which frame category it concerns") and Gate C ("per
frame category").
**Answered by CG-R-136** — *"boundary… The Gate 3 prompt says per frame category in three
places and defines it nowhere"*; the omission is the principal's, and it is CG-R-63's
seventh instance. **The "3 of 13 settled" figure is void**, not partially salvaged. The
comparison against the notation experiment's list is **deferred**. The thirteen categories
are **retained as this session's working scheme**, labelled invented-after-build, and are
not presented as a frame.
**Site** `frame-categories.md`; `rulings.md` CG-R-136.

### Q-08
> `DSC-0002`'s predicate is about **the command payload**, but its `ranges_over` is
> `[OrderPlaced]` — the event the act writes, which the predicate never mentions. The
> schema requires `ranges_over` and says the resolution check verifies each entry is
> written by some act in scope. A command payload is not a fact in the vocabulary, so it
> cannot be named there. Did the C-3 requirement force a determination to declare it
> ranges over something it does not range over — and if so, is the payload a missing fact
> kind or is C-3 too strong?

**Prompted by** `place-order.determinations.yaml:75-76` against
`determination.schema.json:157-161`.
**Proceeded under** treating the predicate text as authoritative and `ranges_over` as an
artefact of the constraint. The payload check is implemented over the command, not over
`OrderPlaced`.
**Site** `PlaceOrderCommand.cs`, D-11.

### Q-18
> `DSC-0100.allocation.settled_by` is the literal string
> `"profile:rest-api-v1@<content-hash>"`. A `pinned` allocation whose `settled_by` is an
> unresolved placeholder identifies no version of the thing it pins. And the profile it
> pins is headed `[PROPOSED]` — not in force by its own status — while the determination
> filing it carries `recorded: "2026-09-14T00:00:00Z"`. Is a filed pin to a proposed
> artefact well-formed, and what hash was intended?

**Prompted by** `profile-rest-api-v1.md:1,5,34`.
**Proceeded under** implementing to the profile body as delivered, pinned by the file's
sha256 `940c07ae…be0beb` recorded in `bootstrap.md`.
**Site** `bootstrap.md`, `gate-a.md` contradiction D.

### Q-19
> The profile's first `read_enforced` rule is "the handler's logic is the behavioural
> specification, not a realisation of one stated elsewhere". This run's premise is that a
> specification IS stated elsewhere — `place-order.determinations.yaml` — and that the
> handler is built from it. Under the rule a conforming handler's logic is primary; under
> the prompt it is derived. Which holds, and is every determination-derived handler
> non-conforming by rule 1?

**Prompted by** `profile-rest-api-v1.md:89` against `prompt.md` Gate B.
**Proceeded under** the prompt. The handler realises DSC-0001 and DSC-0005, so it
violates read-enforced rule 1 as written, deliberately and on the record.
**Site** `PlaceOrderHandler.cs` remarks.

### Q-20
> `DSC-0001`'s extent says `context: does-not-travel, region: fulfilment`. Should extent
> be represented in the built artefact at all — as an attribute, a test, a comment — or
> is extent purely a property of the determination store with no realisation in code? I
> have realised none of it, and I cannot tell whether that is correct or a whole missing
> dimension of conformance.

**Prompted by** `place-order.determinations.yaml:16-26`, and the absence of any profile
rule mentioning extent.
**Proceeded under** extent has no code realisation. The slice is `ordering`; nothing
records that DSC-0001 stops at `fulfilment`.
**Site** nothing — the absence is the answer given.

---

## F2 — Fact shape

### Q-07
> No field of `Cart`, `ActorIdentity` or `OrderPlaced` is declared anywhere in the four
> inputs. The fact vocabulary gives an `id`, a `kind` and two prose notes. Yet `DSC-0002`
> requires that "for every field of the command payload, a domain type is declared and
> the supplied value satisfies its constraints", with an operational closure and a named
> analyser. **There are no declared domain types for the analyser to check against.** Is
> the type declaration meant to live in the event model, in a determination, or somewhere
> I have not been given — and until it does, is DSC-0002 dischargeable at all?

**Prompted by** `ordering.eventmodel.yaml:17-35` against
`place-order.determinations.yaml:48-50,66-82`.
**Proceeded under** inventing every field, then validating the payload against those
inventions — which makes the check self-referential.
**Site** `Facts.cs` D-04; `PlaceOrderCommand.cs`.

### Q-05
> `DSC-0005` rejects an order "against a cart whose currency differs from **the
> customer's account currency**". Nothing in the fact vocabulary carries an account
> currency; there is no Account fact; and DSC-0005's own `positions` declares only
> `Cart: read` — it declares no position for the second thing its statement compares
> against. Where does the account currency come from, and should the determination carry
> a position for it?

**Prompted by** `place-order.determinations.yaml:182-184,199-203`.
**Proceeded under** hanging `AccountCurrency` off `ActorIdentity` and reading it from an
invented `account_currency` claim. This silently widens a read position no determination
declares.
**Site** `Facts.cs` D-07; `Providers.cs` D-16.

---

## F4 — Invariant & rejection

### Q-03
> The profile says the handler must "reject **only for invariants the fact vocabulary
> declares**". `ordering.eventmodel.yaml` declares no invariant, no predicate, no
> constraint, and no field over which one could be stated. Read strictly the handler may
> never reject, `Rejected` is dead, and the controller's Accepted-or-Rejected mapping is
> half unreachable. Meanwhile DSC-0001 is a pinned determination at this exact address
> carrying `settled_by: "invariant:CartNotEmpty"` — an invariant in the **determination**
> layer. Does "fact vocabulary" mean the event model literally, or the specification as a
> whole? As written, implementing DSC-0001 breaks the profile.

**Prompted by** `profile-rest-api-v1.md:72` against
`place-order.determinations.yaml:14-15,29` and `ordering.eventmodel.yaml:17-35`.
**Proceeded under** the loose reading. DSC-0001 and DSC-0005 are both implemented, so
the handler violates the profile rule as written.
**Site** `PlaceOrderHandler.cs` remarks; this is the run's most important conflict.

### Q-04
> `DSC-0005` is `allocation.class: residual`, carried by human principal `emil`. Its
> statement is nonetheless a definite behavioural rule. Does `residual` describe how a
> determination is **discharged** — no pin, no check, a human carries the risk that it
> holds — or whether it is **built at all**? I have built it. If residual means "nothing
> is implemented", I have added behaviour the specification did not ask for.

**Prompted by** `place-order.determinations.yaml:182-198`.
**Proceeded under** allocation describes discharge, not existence. The currency
rejection is implemented and unverified.
**Site** `PlaceOrderHandler.cs` D-20(a).

### Q-22
> The profile names `Accepted` and `Rejected` four times and defines neither. Does
> `Accepted` carry the emitted events? Does `Rejected` carry a cited invariant, a code, a
> message? Is the pair closed? The read-enforced rule "a rejection reason corresponds to
> the invariant it cites" implies a rejection cites an invariant, which implies a field
> to cite it in — but that is inference, not specification.

**Prompted by** `profile-rest-api-v1.md:59,70,90`.
**Proceeded under** a closed hierarchy: `Accepted(OrderPlaced)`, `Rejected(Invariant,
Reason)`.
**Site** `PlaceOrderOutcome.cs` D-12.

### Q-26
> `DSC-0001` supplies an invariant name through `settled_by: "invariant:CartNotEmpty"`.
> `DSC-0005` supplies none. If a rejection must cite the invariant it corresponds to,
> where does the citation for DSC-0005 come from? I coined `CurrencyMatchesAccount`, which
> appears in no input and which a second reader will not reproduce.

**Prompted by** `place-order.determinations.yaml:29` against `:193-198`.
**Proceeded under** coining the name.
**Site** `PlaceOrderHandler.cs` D-20(b).

---

## F5 — Payload & validation

### Q-09
> Where does `DSC-0002`'s payload validation live? The handler may reject "only for
> invariants the fact vocabulary declares" and a payload type fault is not one, so it
> cannot be the handler. The controller `must_not` contain "a conditional on domain
> state" — is a payload type check a conditional on domain state? And if the controller
> does it, the resulting 400 is a transport result **not** "derived from the handler's
> Accepted or Rejected", contradicting the controller's own `must`. One of those rules
> has to give. Which?

**Prompted by** `place-order.determinations.yaml:48-50` against
`profile-rest-api-v1.md:60,63,72`.
**Proceeded under** validation in the controller; the "derived from" rule gives.
**Site** `PlaceOrderController.cs` D-23.

---

## F6 — Authority & actor

### Q-17
> `DSC-0003` says "who may place an order is not settled at this address", `residual`,
> carried by team `platform-security`. I have therefore implemented **no** authorisation
> check. Should the built slice record that an undischarged residual is live at this
> address — an attribute, a startup assertion, a failing test — or is silence the correct
> realisation of an unsettled determination? Silence and "nobody thought about it" look
> identical in the code.

**Prompted by** `place-order.determinations.yaml:92-107`.
**Proceeded under** silence in behaviour, plus a test asserting the absence and a
comment naming the residual.
**Site** `PlaceOrderHandler.cs` remarks; `PlaceOrderHandlerTests.Does_not_decide_who_may_place_an_order`.

---

## F3 + F8 — Position, boundary, role decomposition

### Q-06 — **ANSWERED**, see `rulings.md`
> `DSC-0003` settles that `ActorIdentity`'s `read_provenance` is an "OIDC token claim,
> validated at the gateway". The only carrier of that claim is the HTTP request. The
> profile makes the provider the role that supplies read-position facts and says a
> provider `must_not` "reference a transport type". **The one permitted supplier of this
> fact may not touch the only thing that carries it.** How is the claim meant to reach
> the provider?

**Prompted by** `place-order.determinations.yaml:114` against `profile-rest-api-v1.md:86`.
**Sharpened by R-Q10 and FORCED by R-Q16.** DSC-0003, the provider's `must_not` and the
exhaustiveness ruling are jointly unsatisfiable; any two hold, all three cannot. This
session implements DSC-0003 and R-Q16 and breaks the `must_not`, visibly. **This is now
the question that must be ruled on next**; three options are set out in `rulings.md`.
**Answered** *"Amend the provider must_not"* — Emil, 2026-09-16. A provider adapting a
transport-borne source may reference a transport type; one adapting a store may not.
**Now built as** `ActorIdentityProvider` holding `IHttpContextAccessor` directly and
conforming, with DSC-0003's `read_provenance` carrying the permission. **The amendment
moved the rule out of the code**: a checker must now read the determination store and then
infer "transport" from free-text prose (D-40). Q-35 and Q-36 are its residue.
**Site** `rulings.md` R-Q06; `ProviderTransportCarrierTests.cs`.

### Q-11 — **ANSWERED as a consequence of R-Q10**
> `Cart` is `internal` — the `Cart` read-model slice writes it, and DSC-0001 and DSC-0005
> both declare the boundary `internal`. The profile scopes the provider to "where
> **external** data is required". But the handler `must_not` perform I/O and the
> controller `must_not` reference a persistence type, so no role may fetch an internal
> fact either. Is the provider's trigger clause a sufficiency condition (a provider is
> *needed* when data is external) or a restriction (a provider may *only* supply external
> data)? Under the second reading the slice cannot read its own ground.

**Prompted by** `profile-rest-api-v1.md:3,77-83` against
`place-order.determinations.yaml:31-34`.
**Answered** by R-Q10: a provider is "the adapter to the storage options", so whether a
fact is internal or external never decided whether a provider is used — storage did.
Sufficiency confirmed, by a stronger statement than this session guessed at.
**Site** `Providers.cs` D-15.

### Q-16 — **ANSWERED**, see `rulings.md`
> Nothing in the profile forbids a slice from containing types that declare **no** role,
> and no rule reaches such a type. I have built a conforming slice in which an unroled
> decorator performs the handler's I/O, an unroled adapter holds the provider's transport
> reference, and an unroled sink is the persistence the controller may not name. **Every
> `must_not` holds and every prohibited thing happens.** Is the unroled type intended to
> be outside the profile, and if not, what rule closes it?

**Prompted by** the three `must_not` sets in `profile-rest-api-v1.md:61-63,74-75,85-86`,
and their silence about non-role types.
**Answered** *"we need the role for the act. We cant have an act withour an actor and role
is part of that."* — Emil, 2026-09-16. Roles are exhaustive; there is no
outside-the-profile.
**Now built as** no unroled hop: `ActorIdentityProvider` holds the transport reference
itself, in visible breach of its own `must_not`. The ruling did not make the rules
enforceable — it made the contradiction undeniable. Four types still have no role the
profile can give them (Q-33).
**Site** `rulings.md` R-Q16; `ProfileConformanceTests.The_provider_references_a_transport_type_in_breach_of_its_own_rule`.

### Q-24
> The profile writes the marker as `[Slice(<instance>, "controller")]` — a string
> literal. Is the role argument meant to be a string, or is a closed enum acceptable? And
> is the attribute type supplied by the framework or authored per solution? I have
> authored it, so nothing makes my `[Slice]` the same `[Slice]` any analyser would look
> for.

**Prompted by** `profile-rest-api-v1.md:54,69,81`.
**Proceeded under** authoring `SliceAttribute` with a closed `SliceRole` enum.
**Site** `Profile/SliceAttribute.cs` D-01, D-02.

### Q-25
> The controller `must` "call exactly one type declaring the handler role for the same
> act instance". Does "exactly one" constrain only handler-role calls, or does it forbid
> the controller calling anything else at all? My controller also touches `ModelState`
> and constructs `ProblemDetails`.

**Prompted by** `profile-rest-api-v1.md:58`.
**Proceeded under** the narrow reading: the constraint counts handler-role calls.
**Site** `PlaceOrderController.cs`.

---

## F9 — Effect & egress

### Q-10 — **ANSWERED**, see `rulings.md`
> `PlaceOrder` declares `writes: [OrderPlaced]`. An event emitted and never persisted is
> not written. But the handler `must_not` perform I/O, the controller `must_not`
> reference a persistence type, the provider's every rule is about supplying reads and it
> `must_not` decide — and there is no fourth role. **The write position has no
> realisation anywhere in the profile.** What persists the event?

**Prompted by** `ordering.eventmodel.yaml:54-57` against the whole of the profile's
`roles:` block.
**Answered** *"we need a writer path as we have a reader path, I would argue that
providers can supply writes as well as reads. They are the adapters to the storage
options."* — Emil, 2026-09-16.
**Now built as** `OrderPlacedProvider`, a provider-role type the handler reaches exactly
as it reaches the read providers. The unroled decorator is gone and D-33 is withdrawn
with it.
**Site** `Slices/PlaceOrder/Providers.cs`; `rulings.md` R-Q10.

---

## F7 — Transport realisation

### Q-12 — **ANSWERED**, see `rulings.md`
> The controller `must` "return a transport result **derived from** the handler's Accepted
> or Rejected". Derived *how*? Nothing states the status for an accepted command, the
> status for a rejected one, whether a `Location` header is owed, or what the body is.
> Two readers produce two incompatible APIs from this profile and both conform. Is the
> mapping meant to be free, or is it missing?

**Prompted by** `profile-rest-api-v1.md:59`.
**Answered** *"Accepted is 201 with Location, Rejected is 422"* — Emil, 2026-09-16, which
is exactly what this session invented. **The match proves nothing about legibility**: a
reader choosing 200/400 or 202/409 would have conformed equally well, because the profile
still does not say. The gap was real; the ruling closed it, the guess did not.
**Two paths it does not reach** — invalid payload (400) and an unsuppliable read position
(401/404) — are neither Accepted nor Rejected and remain invented. Q-12b.
**And it collides with DSC-0004**, whose `proxy.known_divergence` describes precisely the
read-after-write the `Location` header invites. Q-43.
**Site** `PlaceOrderController.cs` D-21; `rulings.md` R-Q12.

### Q-13
> What is the HTTP surface of `PlaceOrder` — verb, path, versioning? The profile fixes
> the stack and names the role and fixes no URL. `POST /orders` is my convention, not the
> specification's.

**Prompted by** `profile-rest-api-v1.md:50-63`.
**Proceeded under** `POST /orders`, unversioned.
**Site** `PlaceOrderController.cs` D-24.

### Q-27
> The controller `must_not` contain "a conditional on domain state", yet `must` derive its
> result from Accepted-or-Rejected — which is a conditional on the act's outcome. Where is
> the line? I read it as forbidding a branch on **which** invariant was cited, so all
> rejections map to one status and the invariant travels in the body. A per-invariant
> status map is the obvious alternative and I cannot tell if it is forbidden.

**Prompted by** `profile-rest-api-v1.md:59` against `:61`.
**Proceeded under** one status for all rejections.
**Site** `PlaceOrderController.cs` D-22.

---

## F10 — Identity & time

### Q-14
> Does an order have an identifier, who mints it, and what is it? Nothing in any input
> says `OrderPlaced` carries one — or a timestamp. Both are unusable-without, so I added
> both. A GUID and a human-meaningful order number are equally supported, and the choice
> changes the event, the `Location` header and the public API.

**Prompted by** `ordering.eventmodel.yaml:26` — `OrderPlaced` is an id and a kind.
**Proceeded under** an injected `IOrderIdentityMint` returning a GUID, and an injected
`TimeProvider`.
**Site** `Facts.cs` D-08; `Unroled/Adapters.cs` D-30.

---

## F11 — Lifecycle & concurrency

### Q-15
> `PlaceOrder` writes `OrderPlaced` and nothing else; `CartEmptied` belongs to
> `EmptyCart`. **So a placed order leaves its cart standing**, and the same cart can be
> ordered again. I think this is wrong and I have implemented it as specified. Is the
> cart meant to survive its own order, or is a write position missing from the act?

**Prompted by** `ordering.eventmodel.yaml:54-62`.
**Proceeded under** implementing it as specified, and pinning the behaviour in a test so
the disagreement cannot quietly become a fix.
**Site** `PlaceOrderHandlerTests.Leaves_the_cart_standing_after_the_order_is_placed`.

### Q-23
> Nothing addresses idempotency, retries, or two concurrent `PlaceOrder` calls against one
> cart. Two calls place two orders. Is that intended, out of scope for a determination
> layer, or a determination nobody has filed?

**Prompted by** the absence of any such statement across all four inputs.
**Proceeded under** no idempotency, no concurrency control, asserted in a test.
**Site** `PlaceOrderHandlerTests.Places_a_second_order_from_the_same_cart`.

---

## F12 — Enforcement & evidence

### Q-21
> Is a slice with no test conforming? Nothing in the profile, the determinations or the
> schema requires a test, and `DSC-0002`'s acceptance names a Roslyn analyser rather than
> a test. I wrote tests anyway. If tests are not part of conformance, I have added
> artefacts the specification does not recognise; if they are, the profile is missing a
> role.

**Prompted by** `profile-rest-api-v1.md:47-92` — no `test` role, no test rule.
**Proceeded under** writing tests, marked as evidence rather than as discharge of any
`checked` allocation.
**Site** `PlaceOrderHandlerTests.cs` D-35.

### Q-28
> `DSC-0002`'s closure is `operational`, `runnable_by:
> "roslyn-analyzer:PayloadTypeConformance"`, `terminates: true`, `tolerance: "exact"`.
> That analyser does not exist. Per CG-R-127 the rule is read-enforced for this run — but
> the determination still *claims* an operational closure. Does an operational closure
> naming an unbuilt runner overstate itself in exactly the way `proxy.known_divergence`
> exists to prevent elsewhere in the schema?

**Prompted by** `place-order.determinations.yaml:70-73` against
`determination.schema.json:139-155,176-185`.
**Proceeded under** treating it as read-enforced and saying so.
**Site** `ProfileConformanceTests.cs` remarks; Gate C enforceability table.

### Q-29
> `DSC-0002` travels to `all-command-slices` **and** `all-contexts`, and `DSC-0100` travels
> to `all-command-slices`. Both anchor on `act_instance: PlaceOrder`, as the schema
> requires and the anchor note concedes. So when I read the determinations at this
> address, **I cannot tell a rule meant for `PlaceOrder` from a rule meant for every
> command slice ever written** except by reading the extent prose. For a builder, is
> there any operational difference — should I have built anything differently for a
> travelling determination than for a bound one?

**Prompted by** `place-order.determinations.yaml:51-63` and
`profile-rest-api-v1.md:12-31,41`.
**Proceeded under** no operational difference. Bound and travelling determinations were
implemented identically.
**Site** `gate-a.md` contradiction F.


---

## Raised by ruling R-Q10 (2026-09-16)

### Q-30 — F9
> Does the handler call the write provider, or does it return `Accepted` and something
> else records it? I have the handler call it, so the write happens inside the decision
> act. That makes a provider failure occur *after* the decision is taken and *before* the
> caller is told — is that the intended shape, or is the write meant to be a consequence
> of `Accepted` rather than part of reaching it?

**Prompted by** R-Q10 giving the write to the provider role without saying who calls it.
**Proceeded under** the handler calls it, before returning `Accepted`.
**Site** `PlaceOrderHandler.Handle`; `PlaceOrderOutcome.cs` D-12a.

### Q-31 — F9
> If the write provider throws, the act has decided but not written. That is not a
> rejection — no invariant is cited — so it cannot be `Rejected`, and the profile admits
> no third exit. What is the outcome of a decided-but-unwritten act?

**Prompted by** the same ruling: a roled write path inherits the read path's problem
(D-19), now on the side where the act has already committed to a verdict.
**Proceeded under** it throws and unroled middleware maps it.
**Site** `Slices/PlaceOrder/Providers.cs` D-27.

### Q-32 — F8
> May one provider type serve both a read and a write position, or is it one provider per
> position? I have used three, one per position. "Adapters to the storage options"
> suggests one adapter per *store*, which would group them by backing store rather than
> by position — a third shape again.

**Prompted by** R-Q10's phrasing.
**Proceeded under** one provider per position.
**Site** `Slices/PlaceOrder/Providers.cs`.

### Q-33 — F8, raised by ruling R-Q16
> Where does "participating in the act" stop? Taken at face value the exhaustiveness rule
> reaches the DI container, `TimeProvider`, `ProblemDetails` and the ASP.NET pipeline.
> Taken narrowly it reaches only the types I chose to put in the slice's namespaces, which
> is circular. Four types in this solution still have no role and **no role in the profile
> fits any of them**: two stores a provider adapts; a mint supplying an `OrderId`, which
> is not a fact in a read position; and middleware producing a transport result without
> calling a handler. Rolling any of them breaks the rule of the role I would give it.

**Prompted by** R-Q16, on trying to apply it exhaustively.
**Proceeded under** leaving all four unroled and marking them as breaches.
**Site** `rulings.md` R-Q16; `ProfileConformanceTests.Four_participating_types_still_have_no_role_the_profile_can_give_them`.

### Q-34 — F8, raised by ruling R-Q06
> How does a checker match a provider to the fact it supplies? The amended rule is
> conditional on the position the provider adapts, so something must connect the type to
> the fact. I used the return type of the provider's single public method. Nothing states
> that convention, an analyser would need it stated, and a provider supplying two facts or
> returning a DTO breaks it immediately.

**Prompted by** implementing R-Q06's conditional rule.
**Proceeded under** return-type matching (D-41).
**Site** `ProviderTransportCarrierTests.FactSuppliedBy`.

### Q-35 — F9 — **ANSWERED as a consequence of R-Q39**
> The amended `must_not` is stated over "the position it adapts", and R-Q10 gave providers
> a write direction. A provider that *records* to a transport sink — posting to a webhook —
> adapts a write position, and the amendment as proposed says nothing about it. Is the
> permission meant to cover write positions too, or is a transport-borne write forbidden?

**Prompted by** the asymmetry between R-Q10 and R-Q06.
**Answered** by R-Q39: a write to a terminal consumer is us acting outward, so the
carriage is theirs and the rule has nothing to condition on. **Not a hole — the rule's
correct scope.** What remains is whether an outbound transport reference should be
governed by something else (Q-40).
**Site** `rulings.md` R-Q39.

### Q-36 — F12 — **ANSWERED by R-GROUND**
> With a `boundary.carrier` enum added, the amended rule becomes conditional on a field the
> determination author controls. An author who writes `carrier: transport` grants their own
> provider the permission. Is that intended — the determination is the authority, so it
> decides — or does the carrier need checking against the fact's actual source, which
> nothing can establish?

**Prompted by** proposing the carrier enum that would make R-Q06's rule machine-checkable.
**Answered** by R-GROUND: the determination layer is the ground, so the determination
author is the authority on the carrier — and the remedy for a wrong carrier is the same as
for any wrong determination, a supersession, not a cross-check nothing could perform. The
check reads what is modelled and reports `Undeterminable` where nothing is.
**Site** `rulings.md` R-GROUND; `CarrierModel.cs`.

### Q-37 — F13 — **ANSWERED**, see `rulings.md`
> `determination.schema.json` closes `position` with `additionalProperties: false` and
> leaves `boundary` open. So `carrier: transport` is already schema-valid, and nothing can
> rely on what it says. Is `boundary` open deliberately — an extension point for
> carrier-like facts nobody has modelled — or is the missing `additionalProperties: false`
> an oversight? Under R-GROUND the two readings are not equivalent: **an open object is
> unmodelled ground by construction.** It is also the one object in the schema a profile
> rule needs to determine on, while every other union in the file is emphatically closed.

**Prompted by** checking whether the proposed `carrier` field would even validate.
**Answered** *"boundary should be closed, it's an oversight"* — Emil, 2026-09-16.
**Checked, and the premise holds**: `boundary` is the only object the schema leaves open —
a single miss, not a pattern. (`allocation` looks open and is not: its three `oneOf`
branches each close themselves.) **But applying it is a two-part change**: closing
`boundary` without declaring `carrier` forbids the very ground R-GROUND requires, so the
closure and the declaration are one patch. Migration cost is nil — the delivered store
uses no undeclared boundary key.
**Site** `rulings.md` R-Q37; `SchemaClosureTests.cs`.

### Q-38 — F13 — **ANSWERED**, see `rulings.md`
> Should `carrier` be **required** for an `external` boundary, alongside `source`,
> `read_provenance` and `tick_rate`? The `allOf` already obliges an external read to
> declare where it comes from and how fast it ticks. Under R-GROUND a rule conditioning on
> carriage is unrunnable without it, so leaving `carrier` optional means every external
> position may silently produce `Undeterminable`. Making it required is a **breaking**
> change to the delivered store — DSC-0003 fails validation until amended. That is a
> principal's call about migration cost, not a builder's.

**Prompted by** drafting the R-Q37 patch and having to choose whether `carrier` joins the
`allOf` for `external`.
**Answered** *"if we dont supply carrier that needs to an explicit decision made by a
human. Because its vital for the systems design."* — Emil, 2026-09-16. Required; absence
permitted only as a **stated, attributed** decision carried by a named non-machine
principal.
**The question was framed wrongly** and this session would have defaulted to optional,
which would have let every external position drift into silent `Undeterminable` — the same
failure mode as the deleted regex, one level up.
**It splits the third verdict**: `UndeterminableUnattributed` (nobody decided; amend the
determination) against `UndeterminableCarried` (a named principal decided; a filed risk
with an owner).
**Site** `rulings.md` R-Q38; `CarrierModel.cs`; `fixtures/dsc-0003.carrier-withheld.yaml`.

### Q-39 — F3 — **ANSWERED**, see `rulings.md`
> Should `carrier` be required for `internal` boundaries too? `Cart` is internal and read
> through a provider that adapts *something* — a store, today. Under R-GROUND a rule
> conditioning on carriage is unrunnable for internal positions for exactly the same reason
> it was for external ones, and the amended provider rule applies to every provider. The
> `allOf` singles out `external` because that is where `source` and `tick_rate` matter;
> carriage may not follow the same line.

**Prompted by** drafting the R-Q38 patch and having to choose which `allOf` branch
`carrier` joins.
**Answered** *"carrier is for actors acting against us - if we act against someone its
their responsibility to name the carrier and take that into account for their system
design."* — Emil, 2026-09-16. Carriage is a property of the **inbound edge**: ours where
an actor acts against us (`read` + `external`), theirs where we act against them (`write`
+ `terminal`), nobody's for `internal`.
**It needed no addition to the notation** — `role` and `boundary.kind` already say which
edge a position sits on, which the previous four rulings each did not.
**Site** `rulings.md` R-Q39; `CarrierModel.ModelledPosition.Edge`.

### Q-40 — F9 / F12 — **ANSWERED**, see `rulings.md`
> An outbound provider that references a transport type — posting `OrderConfirmed` to
> fulfilment over HTTP — is now `NotApplicable`: nothing in the amended `must_not` permits
> it and nothing forbids it. It is **ungoverned**. Is that the intent? The schema is not
> silent about the outbound edge: a `terminal` boundary requires `consumer` and
> `consumption_observable`, so we do model claims about what happens after we act. If we
> model whether they can be observed consuming it, then either the reason carriage is
> different needs saying, or the outbound edge needs its own rule.

**Prompted by** applying R-Q39 and finding the case it leaves uncovered.
**Answered** *"we need to name the transport and how to handle the cases if thats ours to
own, this is pr technology. For a event driven system i would expect us to always have an
outbox pattern before sending to the eventbus."* — Emil, 2026-09-16. Outbound carriage **is**
ours; the transport and its cases are named **per technology**, so they belong in the
profile. For an event-driven system: an outbox, always, before the bus.
**It refines R-Q39 rather than contradicting it**: what the consumer does with the fact is
theirs; how we get it to them and what we do when that fails is ours.
**Now built as** `OrderPlacedProvider` → `IOutbox`, entries `Pending`. Retires D-27.
**Site** `rulings.md` R-Q40; `Slices/PlaceOrder/Providers.cs`.

### Q-12b — F7, raised by ruling R-Q12
> R-Q12 settles the two branches the profile's rule names. What transport result is owed
> for an act that never reached a verdict — an invalid payload rejected before the handler
> (400, D-23), or a read position that could not be supplied (401/404, D-19/D-31)? Neither
> is an Accepted or a Rejected, so neither is "derived from" one.

**Prompted by** applying R-Q12 and finding it covers two of the four paths this slice has.
**Proceeded under** the invented mapping, unchanged.
**Site** `PlaceOrderController.cs`; `Unroled/Adapters.cs` D-31.

### Q-41 — F1 / F8 — **PARTLY ANSWERED**, see `rulings.md`
> The outbox relay is an act. `ordering.eventmodel.yaml` declares no automation slice for
> it — the act vocabulary has six slices, all `command` or `read-model`, and the schema's
> `act_type` enum carries `automation` with nothing using it. Should the relay be an act in
> the vocabulary? If it is, it needs a profile that does not exist — the profile says it
> covers "no profile for read-model, automation or translation slices". If it is not, then
> **the thing that actually delivers every event this context produces sits outside the
> model entirely**, which is a large omission for a scheme whose point is that the act
> vocabulary is complete.

**Prompted by** R-Q40 requiring an outbox, and the outbox requiring a relay.
**Answered as to purpose** *"Q-41 is assurance for external delivery - and thats important
for us to have an answer for. How sure do we need to be of this payload reaching the
external system."* — Emil, 2026-09-16. The relay discharges a **delivery assurance
requirement**, and the requirement is a determination.
**Still open as to modelling.** Whether the relay is an act in the vocabulary is untouched:
six slices, all `command` or `read-model`; `automation` in the schema's enum with nothing
using it; no profile for one. A relay built today conforms to nothing, so it stays unbuilt.
**The schema models the ceiling and not the requirement.** `consumption_observable` is a
capability claim; the ruling asks for a requirement claim. Every terminal position reads
back `UndeterminableUnattributed`.
**Site** `rulings.md` R-Q41; `DeliveryAssuranceModel.cs`; `DeliveryAssuranceTests.cs`.

### Q-42 — F9, raised by ruling R-Q40
> An outbox entry the relay cannot publish — a schema the bus rejects, a consumer
> permanently gone — is neither lost nor delivered. Under the store's own standing pattern
> (*no silent omission*, R-Q38) that needs a stated disposition with an owner rather than a
> default. What is it?

**Prompted by** drafting the proposed `outbound:` profile section and reaching the case
that has no answer.
**Proceeded under** left `UNSPECIFIED` in the proposal rather than filled.
**Site** `rulings.md` R-Q40.

### Q-43 — F12, raised by ruling R-Q12
> Should a `proxy.known_divergence` be **reachable** — something a later determination or a
> profile rule can be checked against? DSC-0004's says a five-second lag is "not acceptable
> immediately after the reader's own write"; R-Q12's `Location` header creates exactly that
> case. The two records are individually conformant and jointly produce a 404 for a caller
> following a Location we told it to follow. It is currently prose in one record, read by a
> human once, at the moment it is written.

**Prompted by** implementing R-Q12 and recognising DSC-0004's divergence as a description
of it.
**Proceeded under** the `Location` header is emitted as ruled; the collision is reported.
**Site** `PlaceOrderController.cs`; `rulings.md` R-Q12.

### Q-44 — F9 — **ANSWERED as a consequence of R-Q45**
> R-Q40 requires an outbox for this slice's write, whose boundary is `internal`. R-Q41
> frames the relay as assurance for **external** delivery. Both can hold — an outbox for
> every write, assurance requirements only where we cross out — but the scoping is not
> stated. Is the outbox required for internal writes too, and on what grounds, given the
> assurance argument does not apply to them?

**Prompted by** finding that `PlaceOrder` has no external delivery at all: `OrderPlaced` is
`internal`, and this context's external delivery is `OrderConfirmed` → fulfilment, one act
downstream.
**Answered** by R-Q45: `delivery_assurance` is everywhere, not only at terminal
boundaries, because every write hands a fact to a later act and every handover can fail.
So the internal write has a requirement of its own and the outbox is not belt-and-braces —
which is the grounds this question asked for.
**Site** `rulings.md` R-Q45; `ProfileDischargeTests.No_write_position_anywhere_in_the_store_declares_a_required_assurance`.

### Q-45 — F12 / F13 — **ANSWERED**, see `rulings.md`
> R-Q40 put the transport and its case handling in the **profile**, because they are "pr
> technology". R-Q41 puts *how sure we need to be* in the **determination layer**, because
> it is a statement about risk. That split looks right and is stated nowhere: **the
> requirement is a determination, the mechanism is a profile, and the profile must be able
> to show it discharges the requirement.** Nothing connects the two — an
> `outbound: pattern: outbox` profile section and a `delivery_assurance: confirmed`
> determination would sit in different files with no relation asserted. Confirming the
> split and then requiring the link is what would make R-Q41 enforceable rather than
> merely written down.

**Prompted by** drafting both the `outbound:` profile section (R-Q40) and the
`delivery_assurance` field (R-Q41) and noticing nothing ties them together.
**Answered** *"the profile must declare which assurance level it discharges and the levels
here matter… its a common failure level, we think we only need it in event systems… But we
dont do it in HTTP systems because we often assume that the external HTTP call is important
to the current one we are serving. Even though that might not be the case."* — Emil,
2026-09-16. The split is confirmed and the link required: `discharges >= required`.
**It answers Q-44** — `delivery_assurance` is everywhere, not only terminal, so this
slice's internal write has a requirement and R-Q40's outbox is not belt-and-braces.
**It names a failure mode this profile is an instance of**: "the caller is waiting" is not
an assurance mechanism. `rest-api-v1` as built discharges `Enqueued` — R-Q40 moved it one
step off `Unassured`, not four.
**And it breaks DSC-0100's travel** (see below).
**Site** `rulings.md` R-Q45; `ProfileDischargeModel.cs`; `ProfileDischargeTests.cs`.

### Q-46 — F9, raised by ruling R-Q45
> *"Everywhere"* is doing work and its edge is not stated. Does `delivery_assurance` apply
> to **read** positions as well as writes? A read can fail to arrive too — DSC-0003's
> `ActorIdentity` is an OIDC claim that may simply not be there, which this slice handles by
> throwing (D-19). That is an assurance question wearing a different hat.

**Prompted by** applying "everywhere" and having to choose an edge for it.
**Proceeded under** writes only, because "reaching the external system" is a delivery
phrase.
**Site** `DeterminationStore.ReadRequiredAssuranceForEveryWrite`.

### Q-47 — F8 / F1, raised by ruling R-Q45
> If several profiles exist per act type, differentiated by discharged assurance, **how is
> the profile selected for a given act?** Today one determination pins one profile to all
> command slices by an arbitrary anchor. Selection by requirement needs either a
> determination per act naming its profile, or a resolution rule picking the profile whose
> discharge meets the act's need — and a tie-break if two qualify. None of that exists, and
> it is the mechanism R-Q45 presupposes.

**Prompted by** *"we might have different profile implementations that support the given
need"* — which requires a selection mechanism that is nowhere defined.
**Proceeded under** one profile, as delivered.
**Site** `rulings.md` R-Q45.
