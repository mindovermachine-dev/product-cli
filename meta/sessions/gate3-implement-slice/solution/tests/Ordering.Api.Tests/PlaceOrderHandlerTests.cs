namespace Ordering.Api.Tests;

using System.Security.Claims;
using Microsoft.AspNetCore.Http;
using Ordering.Api.Facts;
using Ordering.Api.Slices.PlaceOrder;
using Ordering.Api.Unroled;
using Xunit;

/// <summary>
/// One test per determination addressed to <c>command PlaceOrder</c>, plus the profile
/// rules that can be checked from outside an analyser.
/// </summary>
/// <remarks>
/// D-35 — DECIDED. Nothing in the four inputs requires a test, forbids one, or says
/// whether a slice with no test conforms. These exist because a determination nobody
/// can see fail is indistinguishable from one nobody implemented. Q-21.
///
/// These are NOT the acceptances. DSC-0002's acceptance names
/// <c>runnable_by: "roslyn-analyzer:PayloadTypeConformance"</c>, which does not exist;
/// per CG-R-127 every rule is read-enforced for this run and these tests are evidence
/// about the implementation, not discharge of a checked allocation.
/// </remarks>
public sealed class PlaceOrderHandlerTests
{
    // CG-R-138 rules Reading B: DSC-0005 is withdrawn, so
    // `Rejects_a_currency_mismatch_rather_than_converting` and
    // `Reports_CartNotEmpty_first_when_both_rejections_apply` are DELETED. There is one
    // rejection now and no precedence to decide. D-18, D-20 retire with them.

    private static readonly CurrencyCode Gbp = new("GBP");
    private static readonly CartId TheCart = new(Guid.Parse("11111111-1111-1111-1111-111111111111"));
    private static readonly OrderId TheOrder = new(Guid.Parse("22222222-2222-2222-2222-222222222222"));

    private static ActorIdentity Actor(string buyerId) =>
        new(new BuyerId(buyerId), DisplayName: null, IsAnonymous: false);

    private static PlaceOrderHandler Handler(Cart? cart, ActorIdentity? actor) =>
        Handler(cart, actor, new InMemoryOutbox());

    private static PlaceOrderHandler Handler(Cart? cart, ActorIdentity? actor, InMemoryOutbox outbox)
    {
        var store = new InMemoryCartStore();
        if (cart is not null)
        {
            store.Put(cart);
        }

        return new PlaceOrderHandler(
            new CartProvider(store),
            new ActorIdentityProvider(RequestFor(actor)),
            new OrderPlacedProvider(outbox),
            new StubMint(TheOrder),
            TimeProvider.System);
    }

    private static Cart CartWith(params CartLine[] lines) =>
        new(TheCart, new BuyerId("buyer-1"), lines, Gbp);

    private static CartLine Line(int quantity, decimal unitPrice) =>
        new(new CartLineId("line-1"), new CatalogItemId(7), new Quantity(quantity), new Money(unitPrice, Gbp));

    private static PlaceOrderCommand Command() => new() { CartId = TheCart.Value };

    /// <summary>DSC-0001 — "An order may not be placed against an empty cart."</summary>
    [Fact]
    public void Rejects_an_empty_cart_citing_CartNotEmpty()
    {
        var outcome = Handler(CartWith(), Actor("actor-1"))
            .Handle(Command());

        var rejected = Assert.IsType<PlaceOrderOutcome.Rejected>(outcome);
        Assert.Equal("CartNotEmpty", rejected.Invariant);
    }



    /// <summary>The write position: <c>OrderPlaced</c> and nothing else.</summary>
    [Fact]
    public void Emits_OrderPlaced_when_the_cart_is_placeable()
    {
        var outcome = Handler(CartWith(Line(2, 500m)), Actor("actor-1"))
            .Handle(Command());

        var accepted = Assert.IsType<PlaceOrderOutcome.Accepted>(outcome);
        Assert.Equal(TheOrder, accepted.Event.OrderId);
        Assert.Equal(new BuyerId("actor-1"), accepted.Event.BuyerId);
        Assert.Equal(new Money(1000m, Gbp), accepted.Event.Total);

        // OrderLine is "a snapshot of the cart line at placement" and does not carry
        // CartLineId — §2 of the declaration, and §7 open item 2 records the consequence.
        var line = Assert.Single(accepted.Event.Lines);
        Assert.Equal(new CatalogItemId(7), line.CatalogItemId);
    }

    /// <summary>
    /// DSC-0003 — "Who may place an order is not settled at this address", residual,
    /// carried by team platform-security. The slice must therefore NOT decide it. This
    /// asserts the absence: an arbitrary actor is not turned away on authority grounds.
    /// </summary>
    [Fact]
    public void Does_not_decide_who_may_place_an_order()
    {
        var outcome = Handler(CartWith(Line(1, 100m)), Actor("some-stranger"))
            .Handle(Command());

        Assert.IsType<PlaceOrderOutcome.Accepted>(outcome);
    }

    /// <summary>
    /// Q-15 — the cart survives its own order, because the act vocabulary gives
    /// <c>CartEmptied</c> to <c>EmptyCart</c> and <c>PlaceOrder</c> writes only
    /// <c>OrderPlaced</c>. This session believes that is wrong and pins the specified
    /// behaviour here so the belief cannot quietly become a fix.
    /// </summary>
    [Fact]
    public void Leaves_the_cart_standing_after_the_order_is_placed()
    {
        var store = new InMemoryCartStore();
        store.Put(CartWith(Line(1, 100m)));

        var handler = new PlaceOrderHandler(
            new CartProvider(store),
            new ActorIdentityProvider(RequestFor(Actor("actor-1"))),
            new OrderPlacedProvider(new InMemoryOutbox()),
            new StubMint(TheOrder),
            TimeProvider.System);

        handler.Handle(Command());

        Assert.NotNull(store.Find(TheCart));
        Assert.False(store.Find(TheCart)!.IsEmpty);
    }

    /// <summary>Q-23 — no idempotency is specified and none is implemented.</summary>
    [Fact]
    public void Places_a_second_order_from_the_same_cart()
    {
        var handler = Handler(CartWith(Line(1, 100m)), Actor("actor-1"));
        var command = Command();

        Assert.IsType<PlaceOrderOutcome.Accepted>(handler.Handle(command));
        Assert.IsType<PlaceOrderOutcome.Accepted>(handler.Handle(command));
    }

    /// <summary>D-19 — an unsuppliable read position is a third exit the profile does not admit.</summary>
    [Fact]
    public void Throws_when_a_read_position_cannot_be_supplied()
    {
        var handler = Handler(CartWith(Line(1, 100m)), actor: null);

        var ex = Assert.Throws<ReadPositionUnavailableException>(
            () => handler.Handle(Command()));

        Assert.Equal(nameof(ActorIdentity), ex.FactType);
    }

    /// <summary>
    /// R-Q10 — the write position is recorded through a provider-role type. Before the
    /// ruling nothing in the profile wrote the event at all.
    /// </summary>
    [Fact]
    public void Records_the_write_position_through_the_provider()
    {
        var outbox = new InMemoryOutbox();
        var handler = Handler(CartWith(Line(2, 500m)), Actor("actor-1"), outbox);

        handler.Handle(Command());

        var recorded = Assert.Single(outbox.Entries);
        Assert.Equal(TheOrder, recorded.Event.OrderId);
    }

    /// <summary>R-Q10 — a rejected act writes nothing.</summary>
    [Fact]
    public void Records_nothing_when_the_act_is_rejected()
    {
        var outbox = new InMemoryOutbox();
        var handler = Handler(CartWith(), Actor("actor-1"), outbox);

        handler.Handle(Command());

        Assert.Empty(outbox.Entries);
    }

    /// <summary>
    /// R-Q40 — "For a event driven system i would expect us to always have an outbox
    /// pattern before sending to the eventbus." The entry is durable and PENDING; nothing
    /// in this slice puts it on a bus.
    /// </summary>
    [Fact]
    public void The_write_lands_in_the_outbox_pending_publication()
    {
        var outbox = new InMemoryOutbox();
        var handler = Handler(CartWith(Line(1, 100m)), Actor("actor-1"), outbox);

        handler.Handle(Command());

        Assert.Equal(OutboxState.Pending, Assert.Single(outbox.Entries).State);
    }

    /// <summary>
    /// And nothing here dispatches it. The relay that drains the outbox onto the bus is a
    /// SEPARATE ACT, and the profile states it covers "no profile for read-model,
    /// automation or translation slices" — so there is nothing for such a relay to conform
    /// to and it is deliberately not built. Asserted so the absence reads as a boundary
    /// rather than an omission. Q-41.
    /// </summary>
    [Fact]
    public void Nothing_in_this_slice_dispatches_the_outbox()
    {
        var outbox = new InMemoryOutbox();
        var handler = Handler(CartWith(Line(1, 100m)), Actor("actor-1"), outbox);

        handler.Handle(Command());
        handler.Handle(Command());

        Assert.All(outbox.Entries, e => Assert.Equal(OutboxState.Pending, e.State));
    }

    /// <summary>
    /// R-Q16 — the provider reads the claim off the request itself now, so a test must
    /// supply a request. The awkwardness is the point: it is what a transport reference
    /// inside a provider costs, and it was hidden while an unroled adapter carried it.
    /// </summary>
    private static IHttpContextAccessor RequestFor(ActorIdentity? actor)
    {
        var context = new DefaultHttpContext();

        if (actor is not null)
        {
            context.User = new ClaimsPrincipal(new ClaimsIdentity(new[]
            {
                new Claim("sub", actor.BuyerId.Value),
            }));
        }

        return new HttpContextAccessor { HttpContext = context };
    }

    private sealed class StubMint : IOrderIdentityMint
    {
        private readonly OrderId _id;

        public StubMint(OrderId id) => _id = id;

        public OrderId Next() => _id;
    }
}
