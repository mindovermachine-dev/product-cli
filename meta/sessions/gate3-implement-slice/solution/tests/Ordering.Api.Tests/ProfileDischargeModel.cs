namespace Ordering.Api.Tests;

// ═══════════════════════════════════════════════════════════════════════════════════
// THE LINK BETWEEN A REQUIREMENT AND A MECHANISM.
//
// Ruling R-Q45: "the profile must declare which assurance level it discharges and the
// levels here matter - we have delivery_assurance every where and then we might have
// different profile implementations that support the given need. And its a common failure
// level, we think we only need it in event systems where we need to ensure that the event
// is delivered. But we dont do it in HTTP systems because we often assume that the
// external HTTP call is important to the current one we are serving. Even though that
// might not be the case."
//
// Three things, and the third is the one that bites this solution.
//
// 1. THE LINK IS CHECKABLE. The determination states a required level; the profile states
//    the level it discharges; conformance is `discharges >= required`. Q-45 asked whether
//    the split was right and whether anything joined the two halves. It is, and this is
//    the join.
//
// 2. DELIVERY_ASSURANCE IS EVERYWHERE, not only at terminal boundaries. Every write
//    delivers a fact to a later act, and every such delivery can fail. That answers Q-44
//    as a consequence: this slice's `internal` write has an assurance requirement too, so
//    R-Q40's outbox is not belt-and-braces for it.
//
// 3. "THE CALLER IS WAITING" IS NOT AN ASSURANCE MECHANISM. The named failure is that we
//    reach for outboxes in event systems, where we know delivery is a separate concern,
//    and skip them in HTTP systems because a synchronous call *feels* assured — the
//    request is right there, the response comes back. But a synchronous outbound call
//    discharges NOTHING: a crash between the call and the commit loses it just as
//    completely as a dropped event, and the assumption that the call matters to the
//    request being served is often simply false.
//
//    `rest-api-v1` is an HTTP profile. It declares no assurance level at all. This file
//    computes what it ACTUALLY discharges as built, and the answer is ENQUEUED and no
//    more — because R-Q40 put an outbox in, and because the relay that would take it to
//    DISPATCHED is deliberately unbuilt (Q-41).
// ═══════════════════════════════════════════════════════════════════════════════════

/// <summary>How a write is actually got to its consumer.</summary>
public enum AssuranceMechanism
{
    /// <summary>
    /// An outbound call inside the request being served. Discharges NOTHING — see the
    /// failure mode above. This is the entry that makes the model worth having.
    /// </summary>
    SynchronousCallInRequest,

    /// <summary>Durable on our side before the caller is answered. R-Q40.</summary>
    Outbox,

    /// <summary>An outbox plus a relay that drains it and confirms the bus accepted it.</summary>
    OutboxWithRelay,

    /// <summary>A relay plus observed consumption. Needs `consumption_observable: true`.</summary>
    OutboxWithRelayAndAcknowledgement,
}

/// <summary>What a profile declares, and what it actually does.</summary>
public sealed record ProfileDischarge(
    string Profile,
    DeliveryAssurance? Declared,
    AssuranceMechanism Mechanism)
{
    /// <summary>
    /// The level a mechanism actually reaches. Derived from the mechanism, never from
    /// intent: a profile that declares `confirmed` and implements a synchronous call
    /// discharges `Unassured`, and the declaration is the thing that is wrong.
    /// </summary>
    public DeliveryAssurance Actual => Mechanism switch
    {
        AssuranceMechanism.SynchronousCallInRequest => DeliveryAssurance.Unassured,
        AssuranceMechanism.Outbox => DeliveryAssurance.Enqueued,
        AssuranceMechanism.OutboxWithRelay => DeliveryAssurance.Dispatched,
        AssuranceMechanism.OutboxWithRelayAndAcknowledgement => DeliveryAssurance.Confirmed,
        _ => DeliveryAssurance.Unassured,
    };

    public bool DeclarationIsModelled => Declared is not null;

    /// <summary>A profile that claims more than its mechanism delivers. R-Q45.</summary>
    public bool Overclaims => Declared is not null && Declared > Actual;
}

/// <summary>The join R-Q45 requires: does this profile discharge what this act needs?</summary>
public static class AssuranceDischargeRule
{
    public static Verdict Evaluate(DeliveryGround ground, ProfileDischarge profile)
    {
        // A profile that overclaims is wrong whatever the act needs, and it is wrong in
        // the most dangerous direction: it reads as assured and is not.
        if (profile.Overclaims)
        {
            return Verdict.Breaches;
        }

        // R-GROUND, twice over: the requirement side is unmodelled in the schema, and the
        // discharge side is undeclared in the profile. Either absence stops the check.
        if (!ground.RequirementIsModelled || !profile.DeclarationIsModelled)
        {
            return Verdict.UndeterminableUnattributed;
        }

        return profile.Declared >= ground.Required ? Verdict.Conforms : Verdict.Breaches;
    }
}
