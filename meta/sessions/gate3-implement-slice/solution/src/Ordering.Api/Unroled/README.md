# `Unroled/` — the types the profile does not reach

Every type in this folder is part of the `PlaceOrder` slice and **declares no
`[Slice]` role**, so not one rule in `profile-rest-api-v1` applies to it.

This folder is a **finding**, not a layer.

## What ruling R-Q10 closed

It originally held three exhibits. Emil's ruling on Q-10 — *"providers can supply
writes as well as reads. They are the adapters to the storage options"* — removed
two of them:

* `EventAppendingPlaceOrderHandler` **is gone.** The write position now belongs to
  `OrderPlacedProvider`, a roled type. The event the act declares it writes is
  recorded inside the profile.
* The `IPlaceOrderHandler` indirection **is gone with it** (D-33 withdrawn). The
  controller depends on `PlaceOrderHandler` directly, so *"calls exactly one type
  declaring the handler role for the same act instance"* is now true at run time
  and not only in source text.

## What survives, and it is enough

`HttpContextClaimSource` still declares no role and still holds the transport
reference that a provider `must_not` have. DSC-0003 settles that `ActorIdentity`
arrives as an OIDC token claim on the request; the provider is the only permitted
supplier; the provider may not touch the only thing that carries it. The ruling
**sharpens** this rather than closing it: if a provider is the adapter to a
source, then an adapter to a request-borne claim must reference the request.

So the general point stands undiminished:

**A `must_not` about references is a rule about source text, not about programs.**
One interface moves the forbidden reference one hop out of the analyser's reach,
and nothing in the profile forbids a slice from containing types that declare no
role. Closing it needs a rule that a slice's roles are **exhaustive** — every type
participating in an act declares one — which the profile does not have.

`ProfileConformanceTests.EvadesAMustNot_ByIndirection` asserts the surviving case
and passes. See Q-06 and Q-16, both open.

## The rest of this folder

`InMemoryCartStore`, `InMemoryOrderPlacedStore`, `GuidOrderIdentityMint` and
`ReadPositionUnavailableMiddleware` are unroled and unremarkable: the stores sit
*behind* provider-role adapters on both the read and the write side, which is the
shape R-Q10 describes. The middleware is D-31 — the third handler exit the profile
does not admit.
