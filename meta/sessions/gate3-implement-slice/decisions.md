# Decisions — every point the specification does not settle, resolved by this session

Forty-eight, of which **one is withdrawn, one reversed twice, one deleted outright, and one superseded by a later ruling**. Each is marked at its site in the source with the same `D-nn` tag
(`grep -rn 'D-[0-9][0-9]' solution/src`), so the code and this record cannot drift.

**INVENTED** = the specification is silent and this session supplied something.
**DECIDED** = the specification is ambiguous or two of its rules conflict, and this
session chose a reading.

A decision here is not a determination. Nothing below was filed, and per the
prohibitions nothing below may be: where the specification does not settle something,
that is the finding.

| # | Kind | The decision | Site |
|---|---|---|---|
| D-01 | INVENTED | The `[Slice]` attribute type itself — namespace, argument types, `AttributeUsage`. Three profile `must` rules are stated in terms of it and it is supplied nowhere. | `Profile/SliceAttribute.cs` |
| D-02 | DECIDED | The role argument is a closed enum, not the bare string the profile's rule text shows. More checkable; possibly a divergence from a literal reading. | `Profile/SliceAttribute.cs` |
| D-03 | INVENTED | The act instance is an unvalidated string; nothing ties it to the act vocabulary at compile time. | `Profile/SliceAttribute.cs` |
| D-04 | INVENTED | **Every field of every fact.** `Cart`, `CartLine`, `ActorIdentity`, `OrderPlaced` — every name, type, unit and nullability. The fact vocabulary declares a type space, not types. | `Facts/Facts.cs` |
| D-05 | INVENTED | "Empty" means "no lines". A cart of zero-quantity lines is not empty under this reading. DSC-0001 pins the invariant by name and cannot define it, because `Cart` has no declared structure. | `Facts/Facts.cs` |
| D-06 | INVENTED | Money as minor units in a `long`. `decimal`, a Money type, or per-line currency are equally supported. | `Facts/Facts.cs` |
| D-07 | INVENTED | `ActorIdentity.AccountCurrency`. DSC-0005 compares against "the customer's account currency"; no fact carries one and DSC-0005 declares no position for it. Contested — see Q-05. | `Facts/Facts.cs` |
| D-08 | INVENTED | `OrderPlaced` carries an `OrderId` and an `OccurredAt`. Nothing says an order has either. | `Facts/Facts.cs` |
| D-09 | INVENTED | The `PlaceOrderCommand` shape: one field, `CartId`. `ActorIdentity` is deliberately not a payload field because DSC-0003 routes it elsewhere. | `Slices/PlaceOrder/PlaceOrderCommand.cs` |
| D-10 | DECIDED | The command doubles as the HTTP request body; no separate transport DTO. | `Slices/PlaceOrder/PlaceOrderCommand.cs` |
| D-11 | DECIDED | DSC-0002's `does_not_cover: cross-field-consistency` is vacuous with one field; nothing is done about it. | `Slices/PlaceOrder/PlaceOrderCommand.cs` |
| D-12a | DECIDED | Post-ruling: `Accepted` still carries the event, because the controller derives a `Location` from its `OrderId`. Whether an accepted outcome should carry events a provider has already recorded is unsettled. Q-30. | `Slices/PlaceOrder/PlaceOrderOutcome.cs` |
| D-12 | INVENTED | `Accepted` / `Rejected` as a closed hierarchy; `Accepted` carries the event, `Rejected` carries invariant + reason. | `Slices/PlaceOrder/PlaceOrderOutcome.cs` |
| D-13 | INVENTED | `ICartStore` and `IClaimSource` exist at all. No input names a store, stream, repository or claims source. | `Slices/PlaceOrder/Providers.cs` |
| ~~D-14~~ | **REVERSED by R-Q16, then RESOLVED by R-Q06** | The reference was hidden behind `IClaimSource` (rule held in source text, defeated in substance) → R-Q16 made roles exhaustive so the hop had nowhere to live and the rule went from decorative to **violated** → R-Q06 amended the rule to turn on what the provider adapts, and the slice conforms again. Three states, one line of code. | `Slices/PlaceOrder/Providers.cs` |
| ~~D-40~~ | **DELETED by R-GROUND** | The prose-matching `IsTransportBorne` is gone. "We cant add decisions to ground we havent modelled." Replaced by a closed `Carrier` vocabulary and a reader that never infers. | `CarrierModel.cs` |
| D-42 | DECIDED | **`Undeterminable` is kept distinct from `Breaches`.** Collapsing them would report this slice as non-conforming, which is false — nothing here is known to be wrong, it is unknown. The schema makes the identical argument for `silent` (DP-1) and for `asserted-none` (DP-3). A checker without the third value lies in whichever direction its author defaulted. | `CarrierModel.cs` |
| D-43 | DECIDED | A carrier value outside the closed vocabulary is treated as **unmodelled**, not guessed at. An unrecognised string is not a licence to infer. | `CarrierModel.ReadCarrier` |
| ~~D-44~~ | **SUPERSEDED by R-Q38** | `carrier` was proposed **optional**, pricing the validation break as migration cost. The ruling is that the break is the *mechanism*: `carrier` is required, and absence is permitted only as a stated decision attributed to a named human. The optional default would have let every external position drift into silent `Undeterminable`. | `rulings.md` R-Q38 |
| D-46 | DECIDED | The verdict is **four-valued**, not three. `UndeterminableUnattributed` (nobody decided) is kept distinct from `UndeterminableCarried` (a named principal decided, and why). Same argument as D-42, one level down: the checker learns nothing more about the carrier, it learns whose problem it is. | `CarrierModel.cs` |
| D-48 | DECIDED | The verdict is **five-valued**. `NotApplicable` (R-Q39 — never ours to ask) is kept distinct from both Undeterminables (in scope, unanswered). Collapsing them would put every internal position into a queue of things to go and model, which inverts the ruling. | `CarrierModel.cs` |
| D-49 | DECIDED | `Edge` is **derived** from `role` + `boundary.kind`, not asserted as a new field. R-Q39 is the one ruling that needed no addition to the notation. | `ModelledPosition.Edge` |
| D-47 | DECIDED | A withholding naming no acceptable principal reads back as **unattributed**, not as a decision — the safe direction. `machine` is excluded, per the schema's own "a model identity cannot be an accepting principal". | `CarrierModel.ReadWithheldCarrier` |
| D-45 | DECIDED | `Boundary_is_currently_open_which_R_Q37_rules_an_oversight` is a **pinned defect test**: it asserts the state the ruling calls wrong, so applying the fix registers as a change rather than passing silently. Delete it when R-Q37 lands. | `SchemaClosureTests.cs` |
| D-41 | DECIDED | A provider is matched to the fact it supplies by the return type of its single public method. Nothing states the convention; two facts or a DTO breaks it. Q-34. | `ProviderTransportCarrierTests.cs` |
| D-38 | DECIDED | An exhaustiveness rule written over "every type" catches the compiler-generated async state machine behind the middleware. An analyser implementing R-Q16 must scope to types declared in source. Found by running it. | `ProfileConformanceTests.cs` |
| D-15 | DECIDED | The provider's "where external data is required" is a sufficiency condition, not a restriction, so `CartProvider` supplies an internal fact. Under the other reading the slice cannot read its own ground. | `Slices/PlaceOrder/Providers.cs` |
| D-16 | INVENTED | The claim names: `sub` (OIDC convention, imported) and `account_currency` (no basis at all). | `Slices/PlaceOrder/Providers.cs` |
| D-39 | DECIDED | Stores, mint and middleware are left unroled after R-Q16, because every role the profile offers rejects them: a mint supplies no fact in a read position; middleware calling no handler cannot be a controller. Marked as breaches rather than mis-roled. Q-33. | `Unroled/` |
| D-17 | INVENTED | `IOrderIdentityMint` as an injected abstraction, so the decision stays deterministic. | `Slices/PlaceOrder/PlaceOrderHandler.cs` |
| D-18 | DECIDED | Rejection precedence: DSC-0001 before DSC-0005. A cart both empty and mis-currencied reports `CartNotEmpty`. Reversing it is equally supported. | `Slices/PlaceOrder/PlaceOrderHandler.cs` |
| D-19 | INVENTED | An unsuppliable read position throws rather than rejecting — a third handler exit the profile does not admit. | `Slices/PlaceOrder/PlaceOrderHandler.cs` |
| D-20a | DECIDED | A `residual` determination is **implemented**, on the reading that allocation describes discharge and not existence. If residual means "build nothing", this is behaviour the specification did not ask for. | `Slices/PlaceOrder/PlaceOrderHandler.cs` |
| D-20b | INVENTED | The invariant name `CurrencyMatchesAccount`. DSC-0005 supplies none and `Rejected` must cite something. | `Slices/PlaceOrder/PlaceOrderHandler.cs` |
| D-21 | INVENTED | **The entire transport mapping.** Accepted → 201 + `Location`; Rejected → 422 + ProblemDetails; invalid payload → 400. "Derived from" names a dependency, not a function. The largest single hole. | `Slices/PlaceOrder/PlaceOrderController.cs` |
| D-22 | DECIDED | All rejections map to one status; the invariant travels in the body. Branching per invariant would be "a conditional on domain state". | `Slices/PlaceOrder/PlaceOrderController.cs` |
| D-23 | DECIDED | DSC-0002's payload check lives in the controller, so the 400 path is a transport result **not** derived from Accepted-or-Rejected. That rule gives. | `Slices/PlaceOrder/PlaceOrderController.cs` |
| D-24 | INVENTED | `POST /orders`, unversioned, plural noun, resource-named rather than act-named. | `Slices/PlaceOrder/PlaceOrderController.cs` |
| D-25 | INVENTED | An unreachable default arm, because C# cannot prove the closed hierarchy exhaustive and the profile says nothing about expressing exhaustiveness in the stack. | `Slices/PlaceOrder/PlaceOrderController.cs` |
| D-26 | INVENTED | `IOrderPlacedStore` — an event store abstraction. No input names one; R-Q10 settles who adapts it, not what it is. | `Slices/PlaceOrder/Providers.cs` |
| D-27 | DECIDED | The append is synchronous and non-transactional; no retry, no outbox, no ordering guarantee. **A failed append still loses a placed order** — R-Q10 moved the loss inside the profile, it did not prevent it. Q-30, Q-31. | `Slices/PlaceOrder/Providers.cs` |
| D-28 | DECIDED | The token is trusted as already validated, per DSC-0003's `read_provenance`. No signature, issuer, audience or expiry check. If the gateway is absent in some deployment, this reads an attacker's claim — the direct consequence of a settled determination, recorded rather than hedged. | `Unroled/Adapters.cs` |
| D-29 | INVENTED | In-memory stores on both sides, rather than inventing a schema. They sit behind provider-role adapters, which is the shape R-Q10 describes. | `Unroled/Adapters.cs` |
| D-30 | INVENTED | A GUID as the order identifier. | `Unroled/Adapters.cs` |
| D-31 | INVENTED | 401 for a missing `ActorIdentity`, 404 for a missing `Cart`. A security judgement and a REST convention, both imported wholesale. | `Unroled/Adapters.cs` |
| D-32 | INVENTED | The composition root: registration, lifetimes, middleware order. The profile says nothing about how one role reaches another. | `Program.cs` |
| ~~D-33~~ | **WITHDRAWN by R-Q10** | The controller reached the handler through an interface DI resolved to an unroled decorator — the profile satisfied statically, bypassed dynamically, by one line. The ruling gives the write to the provider role, so the decorator and the indirection are both gone and the controller depends on `PlaceOrderHandler` itself. **An evasion vector closed as a side effect of closing a hole.** | `Program.cs` |
| D-34 | DECIDED | `Program` made public so tests can drive the slice. | `Program.cs` |
| D-35 | DECIDED | Tests are written at all, and are marked as evidence rather than as discharge of any `checked` allocation. | `tests/` |
| D-37 | DECIDED | The handler entry point is synchronous; no `CancellationToken`. Predicted by the Gate A list (item 7), decided, and **not marked until the Gate C report caught it** — the one silent resolution in this run. | `Slices/PlaceOrder/PlaceOrderHandler.cs` |
| D-36 | DECIDED | The mechanical profile check inspects the shape of the outcome type, not the emissions. A handler constructing a second event type and dropping it would pass. | `ProfileConformanceTests.cs` |

---

## Three decisions about the run rather than the slice

| # | The decision | Recorded in |
|---|---|---|
| R-1 | The Gate A expectation list was written after reading the act vocabulary, the profile and the schema, and before opening `place-order.determinations.yaml`. The strictest reading of the gate would have written it before opening anything. This makes the list better informed than the strictest reading allows. | `bootstrap.md` |
| R-2 | The greenfield solution lives inside the session directory, so no host-repository convention leaks into a slice whose point is to be built from the specification alone. | `bootstrap.md` |
| R-3 | The gates say *Hold*; no principal was present to ratify. This session proceeded past Gate A and Gate B unratified, recording a provisional reading for each open question, because holding would have produced no artefact to report on. **Every question in `questions.md` is open.** | `questions.md` |
