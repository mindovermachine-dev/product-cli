namespace Ordering.Api.Tests;

using System.Reflection;
using Ordering.Api.Profile;
using Ordering.Api.Slices.PlaceOrder;
using Xunit;

/// <summary>
/// R-Q06's amended provider rule, evaluated over modelled ground per ruling R-GROUND.
/// </summary>
/// <remarks>
/// **The earlier version of this class inferred the carrier by matching prose in
/// `read_provenance`. That was wrong and is deleted.** R-GROUND: "We cant add decisions
/// to ground we havent modelled. Instead of using a regex, we need to build a proper
/// model of what we want to determine on." The model is in `CarrierModel.cs`.
///
/// The result of doing it properly is the finding:
///
/// * Against the determination store **as delivered**, the rule is
///   <see cref="Verdict.Undeterminable"/> for `ActorIdentity`. Not conforming, not
///   breaching — unevaluable, because no determination says what carries the fact.
/// * Against a fixture in which `carrier: transport` is modelled, the same rule and the
///   same code return <see cref="Verdict.Conforms"/>.
///
/// So the amended rule is **fully mechanical once the ground is modelled, and cannot be
/// run at all until it is.** That is a better answer for PRD §11.4 than either "checkable"
/// or "not checkable": the rule is not the problem, the missing model is.
///
/// D-42 — DECIDED, AND IT IS THE LOAD-BEARING ONE. UNDETERMINABLE is kept distinct from
/// BREACHES. Collapsing them would report this slice as non-conforming, which is false:
/// nothing here is known to be wrong. It is unknown. The schema makes exactly this
/// argument for `silent` as a distinct extent state (DP-1); the same holds for
/// enforcement, and a checker without the third value will lie in whichever direction its
/// author defaulted.
/// </remarks>
public sealed class ProviderTransportCarrierTests
{
    private static readonly string Delivered =
        Path.Combine(AppContext.BaseDirectory, "place-order.determinations.yaml");

    private static readonly string CarrierModelled =
        Path.Combine(AppContext.BaseDirectory, "fixtures", "dsc-0003.carrier-modelled.yaml");

    private static readonly string CarrierWithheld =
        Path.Combine(AppContext.BaseDirectory, "fixtures", "dsc-0003.carrier-withheld.yaml");

    /// <summary>
    /// THE FINDING. On the store as delivered, the rule cannot be evaluated for the one
    /// provider it matters for.
    /// </summary>
    [Fact]
    public void The_rule_is_undeterminable_on_the_delivered_store()
    {
        var positions = DeterminationStore.ReadPositionsFor(Delivered, "PlaceOrder");

        Assert.False(positions["ActorIdentity"].CarrierIsModelled);

        var verdict = ProviderTransportRule.Evaluate(
            ReferencesTransport(typeof(ActorIdentityProvider)),
            positions["ActorIdentity"]);

        Assert.Equal(Verdict.UndeterminableUnattributed, verdict);
    }

    /// <summary>
    /// R-Q38 — the same absence, decided and signed for, is a different verdict. Not
    /// because the checker learned anything about the carrier, but because it learned
    /// whose problem it is.
    /// </summary>
    [Fact]
    public void A_withheld_carrier_is_carried_not_unattributed()
    {
        var positions = DeterminationStore.ReadPositionsFor(CarrierWithheld, "PlaceOrder");
        var position = positions["ActorIdentity"];

        Assert.False(position.CarrierIsModelled);
        Assert.True(position.CarrierIsWithheldDeliberately);

        var verdict = ProviderTransportRule.Evaluate(
            ReferencesTransport(typeof(ActorIdentityProvider)),
            position);

        Assert.Equal(Verdict.UndeterminableCarried, verdict);
    }

    /// <summary>
    /// And the decision names who carries it, so the report can say so. A conformance
    /// report that ends "undeterminable" is a shrug; one that ends "undeterminable, and
    /// emil decided that on 2026-08-14, because the identity provider is being replaced"
    /// is a position someone can act on.
    /// </summary>
    [Fact]
    public void The_withheld_carrier_names_an_accountable_human_principal()
    {
        var withheld = DeterminationStore
            .ReadPositionsFor(CarrierWithheld, "PlaceOrder")["ActorIdentity"]
            .CarrierWithheld;

        Assert.NotNull(withheld);
        Assert.Equal("human", withheld.PrincipalKind);
        Assert.Equal("emil", withheld.PrincipalIdentifier);
        Assert.NotEmpty(withheld.Reason);
        Assert.True(withheld.HasAccountablePrincipal);
    }

    /// <summary>
    /// R-Q38's hard edge, taken from the schema's own: "a model identity cannot be an
    /// accepting principal". A machine may not decide the carrier does not matter, and a
    /// withholding without an acceptable principal is not a decision at all — it reads
    /// back as UNATTRIBUTED, which is the safe direction.
    /// </summary>
    [Theory]
    [InlineData("machine", "gate3-builder")]
    [InlineData("human", "")]
    [InlineData("", "emil")]
    public void A_withholding_without_an_accountable_principal_is_not_a_decision(
        string kind,
        string identifier)
    {
        var withheld = new CarrierNotSupplied(kind, identifier, "because");

        Assert.False(withheld.HasAccountablePrincipal);
    }

    /// <summary>
    /// And once the ground is modelled, the same rule and the same code decide it — with
    /// no inference anywhere.
    /// </summary>
    [Fact]
    public void The_same_rule_conforms_once_the_carrier_is_modelled()
    {
        var positions = DeterminationStore.ReadPositionsFor(CarrierModelled, "PlaceOrder");

        Assert.Equal(Carrier.Transport, positions["ActorIdentity"].Carrier);

        var verdict = ProviderTransportRule.Evaluate(
            ReferencesTransport(typeof(ActorIdentityProvider)),
            positions["ActorIdentity"]);

        Assert.Equal(Verdict.Conforms, verdict);
    }

    /// <summary>
    /// The rule still bites, and it bites without needing ground: a provider that
    /// references no transport type conforms whatever the carrier is, so the prohibition
    /// is never vacuous.
    /// </summary>
    [Fact]
    public void A_provider_that_references_no_transport_conforms_without_needing_ground()
    {
        var positions = DeterminationStore.ReadPositionsFor(Delivered, "PlaceOrder");

        Assert.False(positions["Cart"].CarrierIsModelled);
        Assert.False(ReferencesTransport(typeof(CartProvider)));

        Assert.Equal(
            Verdict.Conforms,
            ProviderTransportRule.Evaluate(false, positions["Cart"]));
    }

    /// <summary>
    /// A modelled carrier that is not transport is a real breach — the rule can say no,
    /// which is what distinguishes it from a permission that always grants.
    /// </summary>
    [Fact]
    public void A_store_carrier_breaches_when_the_provider_references_transport()
    {
        var store = new ModelledPosition("Cart", "internal", Carrier.Store);

        Assert.Equal(Verdict.Breaches, ProviderTransportRule.Evaluate(true, store));
    }

    /// <summary>
    /// An unrecognised carrier value is UNMODELLED, not a licence to guess. This is the
    /// regex's grave: any string outside the closed vocabulary yields no ground.
    /// </summary>
    [Fact]
    public void An_unrecognised_carrier_is_unmodelled_rather_than_inferred()
    {
        var positions = DeterminationStore.ReadPositionsFor(Delivered, "PlaceOrder");

        // `read_provenance` on ActorIdentity says "OIDC token claim, validated at the
        // gateway" — which the deleted regex read as transport. The model reads nothing
        // from it at all, which is correct.
        Assert.Null(positions["ActorIdentity"].Carrier);
    }

    /// <summary>
    /// R-Q16 still holds and still matters: the transport reference is on the roled
    /// provider itself. Hide it behind an unroled hop again and there is nothing for the
    /// carrier rule to be about.
    /// </summary>
    [Fact]
    public void The_transport_reference_is_held_by_the_roled_provider_itself()
    {
        Assert.True(ReferencesTransport(typeof(ActorIdentityProvider)));
        Assert.Equal(
            SliceRole.Provider,
            typeof(ActorIdentityProvider).GetCustomAttribute<SliceAttribute>()?.Role);
    }

    private static bool ReferencesTransport(Type type) =>
        type.GetConstructors()
            .SelectMany(c => c.GetParameters())
            .Any(p => p.ParameterType.Namespace?.StartsWith(
                "Microsoft.AspNetCore.Http", StringComparison.Ordinal) == true);
}
