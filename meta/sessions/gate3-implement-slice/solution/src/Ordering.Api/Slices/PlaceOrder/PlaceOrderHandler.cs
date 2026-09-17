namespace Ordering.Api.Slices.PlaceOrder;

using Ordering.Api.Facts;
using Ordering.Api.Profile;

// D-33 — WITHDRAWN by ruling R-Q10. There was an `IPlaceOrderHandler` interface here so
// that an unroled egress decorator could sit between the controller and the handler,
// because the profile gave the write position no home. The ruling gives the write to the
// provider role, so the decorator is gone and with it the indirection: the controller now
// depends on this concrete type, and the profile's "calls exactly one type declaring the
// handler role for the same act instance" is true at run time and not only in source
// text. The ruling closed an evasion vector as a side effect of closing a hole.

/// <summary>
/// D-17 — INVENTED. Nothing settles that an order has an identifier or who mints it, and
/// a handler that may not perform I/O may not reach for a database sequence. Injecting
/// the mint keeps the decision function deterministic under test. Q-14.
/// </summary>
public interface IOrderIdentityMint
{
    OrderId Next();
}

/// <summary>
/// The <c>PlaceOrder</c> decision.
/// </summary>
/// <remarks>
/// PROFILE / handler — required: true.
///   must     "declares [Slice(&lt;instance&gt;, \"handler\")]"                      — below.
///   must     "exposes a single entry point taking the command and returning
///             Accepted or Rejected"                                                — Handle.
///   must     "emits only events the act declares it writes"                        — OrderPlaced only.
///   must     "rejects only for invariants the fact vocabulary declares"            — SEE Q-03.
///   must_not "references a transport type"                                         — holds.
///   must_not "performs I/O directly"                                               — holds; providers supply, decorator writes.
///
/// Q-03 IS ANSWERED BY MOVE 1, AND THIS FILE IS WHERE IT SHOWS.
/// The profile permits rejection "only for invariants the fact vocabulary declares". For
/// the whole of this run that rule was UNSATISFIABLE BY VACUITY: the vocabulary declared
/// ids, kinds and two prose notes, so no invariant could be stated over anything and
/// `Rejected` was dead. `ordering.fact-type-space.md` declares fields and constraints —
/// `Cart.Lines` with a 0…50 cap, `Quantity` bounded 1…99 and never clamped,
/// `CurrencyCode` as ISO 4217 — so an invariant now has a shape to be stated over.
/// `CartNotEmpty` is a predicate over `Cart.Lines`, which is declared. The rule is
/// satisfiable and this handler satisfies it.
///
/// ═══ DSC-0005 IS WITHDRAWN. CG-R-138 RULES READING B. ═══
/// The currency rejection that stood here is GONE, and with it D-07 (the invented
/// `ActorIdentity.AccountCurrency`), D-16 (the invented `account_currency` claim), D-18
/// (the invented precedence between two rejections), D-20(a) and D-20(b), and two tests.
/// **Five inventions and two tests, from one determination.**
///
/// The ruling is that the record was MALFORMED, not merely misclassified: a residual's
/// allocation says nothing in the specification settles the matter, and DSC-0005's
/// statement settled it. CG-R-139 extracts the general test this session had no rule to
/// read its own signal against:
///
///     "If obeying a determination requires authoring ground the determination does not
///      declare, the determination is not settled."
///
/// This handler had to invent an account currency to obey DSC-0005, and flagged the
/// invention contested at the time. `FactShapeConformanceTests` now runs that test
/// mechanically over this assembly.
///
/// D-18 RETIRES WITH IT. With one rejection there is no precedence to decide.
///
/// NOT SETTLED ANYWHERE, AND NOT DONE (see decisions.md):
///   * the cart is not emptied. `PlaceOrder` writes `OrderPlaced` and nothing else; the
///     act vocabulary gives `CartEmptied` to `EmptyCart`. So a placed order leaves its
///     cart standing. This session believes that is wrong and implements it as
///     specified. Q-15.
///   * no idempotency and no concurrency control. Two calls place two orders. Q-23.
///   * nothing rejects or converts a currency mismatch between the cart, its lines and the
///     order. §7 open item 1 of the fact type space names it as a genuine gap, now
///     visible; DSC-0005 is withdrawn and no determination replaces it. See Cart.Total.
/// </remarks>
[Slice("PlaceOrder", SliceRole.Handler)]
public sealed class PlaceOrderHandler
{
    private readonly CartProvider _carts;
    private readonly ActorIdentityProvider _actors;
    private readonly OrderPlacedProvider _placed;
    private readonly IOrderIdentityMint _mint;
    private readonly TimeProvider _clock;

    public PlaceOrderHandler(
        CartProvider carts,
        ActorIdentityProvider actors,
        OrderPlacedProvider placed,
        IOrderIdentityMint mint,
        TimeProvider clock)
    {
        _carts = carts;
        _actors = actors;
        _placed = placed;
        _mint = mint;
        _clock = clock;
    }

    /// <summary>The single entry point the profile requires.</summary>
    public PlaceOrderOutcome Handle(PlaceOrderCommand command)
    {
        // Read positions, both through provider-role types (Providers.cs).
        var actor = _actors.Supply();
        var cart = _carts.Supply(new CartId(command.CartId));

        // D-19 — DECIDED. A read position that cannot be supplied is not a domain
        // rejection: no determination declares it and the profile forbids rejecting for
        // anything undeclared. So it throws and an unroled middleware maps it. Throwing
        // from a handler the profile describes as returning "Accepted or Rejected" is a
        // third exit the profile does not admit. Q-09, Q-12.
        if (actor is null)
        {
            throw new ReadPositionUnavailableException(nameof(ActorIdentity));
        }

        if (cart is null)
        {
            throw new ReadPositionUnavailableException(nameof(Cart));
        }

        // DSC-0001 — pinned, settled_by "invariant:CartNotEmpty".
        // "An order may not be placed against an empty cart."
        if (cart.IsEmpty)
        {
            return new PlaceOrderOutcome.Rejected(
                "CartNotEmpty",
                "An order may not be placed against an empty cart.");
        }

        // The write position: OrderPlaced, and nothing else. Every field below is a
        // declared field of the declared fact; nothing here is this session's.
        //
        // OrderLine is "a snapshot of the cart line at placement" and "does not carry
        // CartLineId" — so the projection drops the line id deliberately, per §2, and
        // §7 open item 2 records that an order therefore cannot be traced to its cart
        // lines.
        var placed = new OrderPlaced(
            OrderId: _mint.Next(),
            CartId: cart.CartId,
            BuyerId: actor.BuyerId,
            Lines: cart.Lines
                .Select(line => new OrderLine(line.CatalogItemId, line.Quantity, line.UnitPrice))
                .ToArray(),
            Currency: cart.Currency,
            Total: cart.Total,
            OccurredAt: new Instant(_clock.GetUtcNow()));

        // R-Q10 — the write position, recorded through the provider that adapts the store.
        // Q-30: the write happens inside the decision act, before Accepted is returned.
        // Q-31: if this throws, the act has decided and not written, which is neither
        // Accepted nor Rejected. Both open.
        _placed.Record(placed);

        return new PlaceOrderOutcome.Accepted(placed);
    }
}

/// <summary>D-19 — INVENTED. See PlaceOrderHandler.Handle.</summary>
public sealed class ReadPositionUnavailableException : Exception
{
    public ReadPositionUnavailableException(string factType)
        : base($"The read position '{factType}' could not be supplied.")
        => FactType = factType;

    public string FactType { get; }
}
