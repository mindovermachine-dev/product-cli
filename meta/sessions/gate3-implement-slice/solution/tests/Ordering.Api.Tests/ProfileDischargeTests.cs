namespace Ordering.Api.Tests;

using Xunit;

/// <summary>
/// R-Q45 applied to `rest-api-v1` as delivered and as built.
/// </summary>
public sealed class ProfileDischargeTests
{
    private static readonly string Delivered =
        Path.Combine(AppContext.BaseDirectory, "place-order.determinations.yaml");

    /// <summary>
    /// `rest-api-v1` as built. R-Q40 put an outbox in; the relay that would take it
    /// further is deliberately unbuilt (Q-41). The profile declares no level, because the
    /// profile as delivered has no field for one — that is what R-Q45 adds.
    /// </summary>
    private static readonly ProfileDischarge RestApiV1 =
        new("rest-api-v1", Declared: null, AssuranceMechanism.Outbox);

    /// <summary>
    /// THE NAMED FAILURE MODE, ASSERTED. "The caller is waiting" is not an assurance
    /// mechanism. A synchronous outbound call inside the request being served discharges
    /// NOTHING — a crash between the call and the commit loses it as completely as a
    /// dropped event, and the assumption that the call matters to the request being served
    /// is often simply false.
    /// </summary>
    [Fact]
    public void A_synchronous_call_in_the_request_discharges_nothing()
    {
        var httpNoOutbox = new ProfileDischarge(
            "rest-api-v1-without-outbox",
            Declared: null,
            AssuranceMechanism.SynchronousCallInRequest);

        Assert.Equal(DeliveryAssurance.Unassured, httpNoOutbox.Actual);
    }

    /// <summary>
    /// And that is exactly where this profile would have sat without R-Q40. The outbox
    /// ruling is what moved an HTTP profile off zero, and it moved it one step, not four.
    /// </summary>
    [Fact]
    public void The_outbox_moves_rest_api_v1_from_unassured_to_enqueued_and_no_further()
    {
        Assert.Equal(DeliveryAssurance.Enqueued, RestApiV1.Actual);
        Assert.True(RestApiV1.Actual < DeliveryAssurance.Dispatched);
    }

    /// <summary>
    /// A profile that claims more than its mechanism delivers breaches whatever the act
    /// needs — and breaches in the most dangerous direction, because it reads as assured.
    /// This is the shape R-Q45's declaration requirement exists to catch.
    /// </summary>
    [Fact]
    public void A_profile_that_overclaims_breaches_regardless_of_what_the_act_needs()
    {
        var overclaiming = new ProfileDischarge(
            "rest-api-v1-as-wished",
            Declared: DeliveryAssurance.Confirmed,
            AssuranceMechanism.SynchronousCallInRequest);

        Assert.True(overclaiming.Overclaims);

        var anyGround = new DeliveryGround("OrderConfirmed", "fulfilment", true, DeliveryAssurance.Unassured);

        Assert.Equal(Verdict.Breaches, AssuranceDischargeRule.Evaluate(anyGround, overclaiming));
    }

    /// <summary>
    /// The join R-Q45 asks for, run against the store as delivered. It cannot decide,
    /// because **both** halves are missing: no determination states a required level, and
    /// the profile declares no discharged one.
    /// </summary>
    [Fact]
    public void The_join_is_undeterminable_from_both_sides_today()
    {
        var ground = DeterminationStore.ReadDeliveryGround(Delivered);

        Assert.NotEmpty(ground);
        Assert.All(ground, g => Assert.False(g.RequirementIsModelled));
        Assert.False(RestApiV1.DeclarationIsModelled);

        Assert.All(
            ground,
            g => Assert.Equal(
                Verdict.UndeterminableUnattributed,
                AssuranceDischargeRule.Evaluate(g, RestApiV1)));
    }

    /// <summary>
    /// With both halves modelled it decides, and the interesting case is the one this
    /// slice would fail: fulfilment needing confirmed delivery against a profile that
    /// only enqueues.
    /// </summary>
    [Theory]
    [InlineData(DeliveryAssurance.Enqueued, DeliveryAssurance.Enqueued, Verdict.Conforms)]
    [InlineData(DeliveryAssurance.Enqueued, DeliveryAssurance.Unassured, Verdict.Conforms)]
    [InlineData(DeliveryAssurance.Enqueued, DeliveryAssurance.Dispatched, Verdict.Breaches)]
    [InlineData(DeliveryAssurance.Enqueued, DeliveryAssurance.Confirmed, Verdict.Breaches)]
    public void The_profile_must_discharge_at_least_what_the_act_requires(
        DeliveryAssurance declared,
        DeliveryAssurance required,
        Verdict expected)
    {
        var profile = RestApiV1 with { Declared = declared };
        var ground = new DeliveryGround("OrderConfirmed", "fulfilment", true, required);

        Assert.Equal(expected, AssuranceDischargeRule.Evaluate(ground, profile));
    }

    /// <summary>
    /// R-Q45 — "we have delivery_assurance every where". Not only at terminal boundaries:
    /// every write hands a fact to a later act and every such handover can fail. So this
    /// slice's INTERNAL write has a requirement too, which answers Q-44: R-Q40's outbox is
    /// not belt-and-braces for it.
    ///
    /// None of the store's write positions declares one — internal or terminal.
    /// </summary>
    [Fact]
    public void No_write_position_anywhere_in_the_store_declares_a_required_assurance()
    {
        var writes = DeterminationStore.ReadEveryPosition(Delivered)
            .Where(p => p.Role == "write")
            .ToArray();

        // Three, not four: DSC-0001 writes OrderPlaced (internal), DSC-0004 writes
        // OrderSummary (terminal), DSC-0006 writes OrderConfirmed (terminal). This
        // session asserted four from memory and the store corrected it.
        Assert.Equal(3, writes.Length);
        Assert.Contains(writes, p => p.Edge == Edge.Internal);   // OrderPlaced
        Assert.Contains(writes, p => p.Edge == Edge.Outbound);   // OrderSummary, OrderConfirmed

        // The reader looks for `delivery_assurance` on every one of them and finds none.
        Assert.All(
            DeterminationStore.ReadRequiredAssuranceForEveryWrite(Delivered),
            required => Assert.Null(required));
    }
}
