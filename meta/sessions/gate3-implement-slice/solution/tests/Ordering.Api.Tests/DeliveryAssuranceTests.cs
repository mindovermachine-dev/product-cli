namespace Ordering.Api.Tests;

using Ordering.Api.Slices.PlaceOrder;
using Xunit;

/// <summary>
/// R-Q41 read back against the delivered determination store.
/// </summary>
/// <remarks>
/// *"Q-41 is assurance for external delivery — and thats important for us to have an
/// answer for. How sure do we need to be of this payload reaching the external system."*
///
/// The store answers half the question and is silent on the half the ruling asks for.
/// Both halves are asserted below rather than described.
/// </remarks>
public sealed class DeliveryAssuranceTests
{
    private static readonly string Delivered =
        Path.Combine(AppContext.BaseDirectory, "place-order.determinations.yaml");

    private static IReadOnlyList<DeliveryGround> Ground() =>
        DeterminationStore.ReadDeliveryGround(Delivered);

    /// <summary>
    /// THE HALF THE SCHEMA MODELS. `consumption_observable` sets a ceiling on how sure we
    /// can ever be, and the two terminal positions in the store sit at different ceilings.
    /// </summary>
    [Fact]
    public void Consumption_observability_sets_the_ceiling_on_assurance()
    {
        var byFact = Ground().ToDictionary(g => g.FactType, StringComparer.Ordinal);

        // DSC-0006: consumer "fulfilment", consumption_observable: true.
        Assert.Equal(DeliveryAssurance.Confirmed, byFact["OrderConfirmed"].Ceiling);

        // DSC-0004: consumer "the order history screen", consumption_observable: false.
        // No relay, however good, can establish more than "we sent it".
        Assert.Equal(DeliveryAssurance.Dispatched, byFact["OrderSummary"].Ceiling);
    }

    /// <summary>
    /// THE HALF THE SCHEMA DOES NOT MODEL, AND THE RULING ASKS FOR. No terminal position
    /// in the store states how sure we need to be — there is no field for it.
    /// </summary>
    [Fact]
    public void No_position_in_the_store_states_a_required_assurance()
    {
        Assert.NotEmpty(Ground());
        Assert.All(Ground(), g => Assert.False(g.RequirementIsModelled));
    }

    /// <summary>
    /// So the rule the ruling asks for is unrunnable, per R-GROUND — and unattributed,
    /// per R-Q38: nobody decided this, which under R-Q37's closed schema is the state that
    /// should be impossible to reach silently.
    /// </summary>
    [Fact]
    public void The_assurance_rule_is_undeterminable_on_the_delivered_store()
    {
        Assert.All(
            Ground(),
            g => Assert.Equal(Verdict.UndeterminableUnattributed, DeliveryAssuranceRule.Evaluate(g)));
    }

    /// <summary>
    /// Once a requirement IS modelled, the rule decides — and the interesting case is a
    /// requirement that no relay can meet. Demanding `Confirmed` where consumption is not
    /// observable is not an ambitious requirement; it is an unsatisfiable one, and the
    /// boundary has to change rather than the code.
    /// </summary>
    [Theory]
    [InlineData(true, DeliveryAssurance.Confirmed, Verdict.Conforms)]
    [InlineData(true, DeliveryAssurance.Dispatched, Verdict.Conforms)]
    [InlineData(false, DeliveryAssurance.Dispatched, Verdict.Conforms)]
    [InlineData(false, DeliveryAssurance.Confirmed, Verdict.Breaches)]
    public void A_requirement_above_the_ceiling_cannot_be_met_by_any_relay(
        bool observable,
        DeliveryAssurance required,
        Verdict expected)
    {
        var ground = new DeliveryGround("OrderConfirmed", "fulfilment", observable, required);

        Assert.Equal(expected, DeliveryAssuranceRule.Evaluate(ground));
    }

    /// <summary>
    /// WHERE THE QUESTION ACTUALLY BITES, AND IT IS NOT THIS SLICE.
    ///
    /// `PlaceOrder` writes `OrderPlaced` with boundary `internal` — read in scope by
    /// `ConfirmOrder` and `OrderSummary`. **There is no external system in this slice at
    /// all.** The external delivery in this context is `OrderConfirmed` → fulfilment, one
    /// act downstream, at `ConfirmOrder`.
    ///
    /// So R-Q40 requires an outbox for a write whose delivery is internal, while R-Q41
    /// frames the relay as assurance for *external* delivery. Both can be right — an
    /// outbox for every write, assurance requirements only where we cross out — but the
    /// scoping is not stated. Q-44.
    /// </summary>
    [Fact]
    public void This_slice_has_no_external_delivery_at_all()
    {
        var everyPosition = DeterminationStore.ReadEveryPosition(Delivered);

        var placeOrderWrites = everyPosition
            .Where(p => p.Role == "write" && p.FactType == nameof(Ordering.Api.Facts.OrderPlaced))
            .ToArray();

        Assert.NotEmpty(placeOrderWrites);
        Assert.All(placeOrderWrites, p => Assert.Equal(Edge.Internal, p.Edge));

        // And the outbox this slice writes to is therefore feeding internal consumers.
        var outbox = new Ordering.Api.Unroled.InMemoryOutbox();
        outbox.Enqueue(new Ordering.Api.Facts.OrderPlaced(
            "order-1", "cart-1", "actor-1", "GBP",
            Array.Empty<Ordering.Api.Facts.CartLine>(), 0, DateTimeOffset.UnixEpoch));

        Assert.Equal(OutboxState.Pending, Assert.Single(outbox.Entries).State);
    }
}
