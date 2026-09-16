namespace Ordering.Api.Slices.PlaceOrder;

using Ordering.Api.Facts;

/// <summary>
/// The handler's return type. The profile names <c>Accepted</c> and <c>Rejected</c> four
/// times — in the handler's entry-point rule and in the controller's transport rule —
/// and defines neither.
/// </summary>
/// <remarks>
/// D-12 — INVENTED. That <c>Accepted</c> carries the emitted events, that
/// <c>Rejected</c> carries a cited invariant and a reason, and that the pair is a closed
/// hierarchy rather than a generic Result, are all this session's constructions.
///
/// <c>Accepted</c> carrying the event was originally forced rather than chosen: with no
/// realisation for the write position, a handler that may emit but may not write had
/// nowhere to put the event except its own return value. Ruling R-Q10 gave the write to
/// the provider role, so the event is now recorded through <c>OrderPlacedProvider</c>
/// and this field is no longer load-bearing for egress.
///
/// D-12a — DECIDED, POST-RULING. <c>Accepted</c> still carries the event, because the
/// controller derives a <c>Location</c> header from its <c>OrderId</c> (D-21). Whether an
/// accepted outcome should carry its events once a provider records them is unsettled;
/// the alternative is an empty <c>Accepted</c> and a controller that cannot build a
/// Location. Q-30.
///
/// <c>Rejected</c> carrying <c>Invariant</c> is this session's reading of the profile's
/// read-enforced rule 2: "a rejection reason corresponds to the invariant it cites, not
/// merely to a declared one". A rejection that cites an invariant needs a field to cite
/// it in. Q-22.
/// </remarks>
public abstract record PlaceOrderOutcome
{
    private PlaceOrderOutcome() { }

    /// <summary>The act's verdict: the events it wrote.</summary>
    public sealed record Accepted(OrderPlaced Event) : PlaceOrderOutcome;

    /// <summary>
    /// The act declined. <paramref name="Invariant"/> is the invariant identifier the
    /// rejection cites; <paramref name="Reason"/> is the corresponding statement.
    /// </summary>
    public sealed record Rejected(string Invariant, string Reason) : PlaceOrderOutcome;
}
