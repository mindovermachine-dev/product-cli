namespace Ordering.Api.Tests;

// ═══════════════════════════════════════════════════════════════════════════════════
// HOW SURE DO WE NEED TO BE THAT THE PAYLOAD REACHED THEM?
//
// Ruling R-Q41: "Q-41 is assurance for external delivery - and thats important for us to
// have an answer for. How sure do we need to be of this payload reaching the external
// system."
//
// So the relay is not plumbing. It is the mechanism that discharges a DELIVERY ASSURANCE
// requirement, and the requirement is a determination — a statement about risk we are
// willing to carry, not about technology.
//
// The question splits in two, and the schema models exactly one half:
//
//   CEILING    — the most assurance obtainable. Already modelled, in
//                `boundary.consumption_observable`. If we cannot observe them consuming
//                it, no amount of relay engineering gets us above "we sent it". This is
//                DERIVED below, from the delivered store, and it is a real constraint.
//
//   REQUIREMENT — how sure we NEED to be. Modelled NOWHERE. There is no field for it in
//                `determination.schema.json`, and under R-GROUND a rule conditioning on
//                it is unrunnable. It reads back UNATTRIBUTED for every terminal position
//                in the store.
//
// `consumption_observable` is a CAPABILITY claim — can we see it? The ruling asks for a
// REQUIREMENT claim — how sure must we be? Those are different, and only the first is
// written down. See rulings.md for the proposed `delivery_assurance` field.
// ═══════════════════════════════════════════════════════════════════════════════════

/// <summary>
/// How far a payload has demonstrably got. Ordered: each level subsumes the one before.
/// </summary>
public enum DeliveryAssurance
{
    /// <summary>Sent and forgotten. A crash anywhere loses it.</summary>
    Unassured = 0,

    /// <summary>Durable on our side. R-Q40's outbox gets us here and no further.</summary>
    Enqueued = 1,

    /// <summary>Handed to the bus and the bus accepted it. We know we sent it.</summary>
    Dispatched = 2,

    /// <summary>Observed being consumed. Requires `consumption_observable: true`.</summary>
    Confirmed = 3,
}

/// <summary>
/// What the determination store says about delivery for one terminal position.
/// </summary>
public sealed record DeliveryGround(
    string FactType,
    string? Consumer,
    bool? ConsumptionObservable,
    DeliveryAssurance? Required)
{
    /// <summary>
    /// The ceiling, derived from `consumption_observable`. **You cannot assure delivery
    /// beyond what the boundary permits observing.** If consumption is not observable,
    /// `Dispatched` is the most that can ever be established, however good the relay.
    /// </summary>
    public DeliveryAssurance? Ceiling => ConsumptionObservable switch
    {
        true => DeliveryAssurance.Confirmed,
        false => DeliveryAssurance.Dispatched,
        null => null,
    };

    /// <summary>R-Q41 — the half the schema does not model.</summary>
    public bool RequirementIsModelled => Required is not null;
}

/// <summary>The rule R-Q41 asks for, over the ground that exists.</summary>
public static class DeliveryAssuranceRule
{
    /// <summary>
    /// Can the stated requirement be met at all? Three outcomes, and the third is the
    /// one the delivered store gives for every position.
    /// </summary>
    public static Verdict Evaluate(DeliveryGround ground)
    {
        if (!ground.RequirementIsModelled || ground.Ceiling is null)
        {
            // R-GROUND. No requirement is modelled anywhere in the schema, so this is
            // where every terminal position in the delivered store lands.
            return Verdict.UndeterminableUnattributed;
        }

        // A requirement above the ceiling is not a demanding requirement — it is an
        // unsatisfiable one, and no relay can close it. The boundary has to change.
        return ground.Required <= ground.Ceiling ? Verdict.Conforms : Verdict.Breaches;
    }
}
