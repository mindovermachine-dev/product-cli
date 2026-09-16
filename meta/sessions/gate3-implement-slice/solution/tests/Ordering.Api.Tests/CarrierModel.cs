namespace Ordering.Api.Tests;

using YamlDotNet.Serialization;

// ═══════════════════════════════════════════════════════════════════════════════════
// A MODEL OF THE GROUND A PROFILE RULE DETERMINES ON.
//
// Ruling R-GROUND: "We cant add decisions to ground we havent modelled. Instead of
// using a regex, we need to build a proper model of what we want to determine on."
//
// R-Q06's amended rule — a provider may reference a transport type only where the
// position it adapts declares a transport-borne carrier — determines on the CARRIER of
// a fact. The first implementation inferred the carrier by matching prose in
// `read_provenance`. That was not enforcement; it was a guess dressed as a check, and it
// would have passed or failed on a determination author's choice of words.
//
// What follows is the carrier modelled instead of inferred:
//   * a closed vocabulary (Carrier),
//   * a reader that reads the modelled field and NEVER infers,
//   * and a THREE-valued verdict, because "the ground is not modelled" is not the same
//     answer as "the rule is broken".
//
// That third value is the whole point, and it is the schema's own argument one level up.
// `determination.schema.json` keeps `silent` as a distinct extent state because
// collapsing it into `does-not-travel` "makes the revisit computation unsound the first
// time an axis is added" (DP-1). The same holds for enforcement: collapsing
// UNDETERMINABLE into BREACHES makes a conformance report unsound the first time a
// carrier is modelled. A checker that cannot say "I have no ground for this" will lie.
//
// ─── and then R-Q38 split that third value in two ─────────────────────────────────
//
// Ruling R-Q38: "if we dont supply carrier that needs to be an explicit decision made by
// a human. Because its vital for the systems design."
//
// So "no carrier" is not one state. It is two, and they demand different actions:
//   * NOBODY DECIDED — the field is simply absent. Under the closed schema of R-Q37 this
//     is a validation failure, and that failure is the MECHANISM: it is how a human gets
//     asked. Remedy: amend the determination.
//   * SOMEONE DECIDED NOT TO SUPPLY IT — absence stated, with a named principal carrying
//     it. Remedy: none. It is a filed risk with an owner, and the checker's job is to
//     report whose it is.
//
// The argument for splitting them is again the schema's own, and by now it has made it
// twice: `does_not_cover` demands the `asserted-none` sentinel "because an omitted
// uncovered set is indistinguishable from an unconsidered one" (DP-3), and a `residual`
// allocation requires an accountable principal who "cannot be a machine" (PR-2/DP-4).
// R-Q38 is the same device a third time. See rulings.md — the schema appears to have an
// unnamed recurring pattern: NO SILENT OMISSION; absence is stated and attributed.
// ═══════════════════════════════════════════════════════════════════════════════════

/// <summary>
/// What physically carries a fact to the act. A closed vocabulary — which is the point:
/// an open one cannot be determined on, because two authors spell it differently.
/// </summary>
/// <remarks>
/// PROPOSED, NOT AUTHORED. This vocabulary does not exist in
/// `determination.schema.json`; the proposal is in `rulings.md`. Note that the schema
/// closes `position` with `additionalProperties: false` and leaves `boundary` OPEN, so a
/// `carrier` key is already schema-valid today — it is simply not modelled. The proposal
/// is therefore a CLOSURE, not a relaxation: the field can already be written, and until
/// the vocabulary is closed nothing can rely on what it says.
/// </remarks>
public enum Carrier
{
    Transport,
    Store,
    Computed,
    ExternalCall,
}

/// <summary>The four answers a rule over modelled ground can give.</summary>
public enum Verdict
{
    /// <summary>The rule was evaluated and holds.</summary>
    Conforms,

    /// <summary>The rule was evaluated and is broken.</summary>
    Breaches,

    /// <summary>
    /// The rule could not be evaluated and NOBODY DECIDED THAT. The ground is simply
    /// absent. Under R-Q37's closed schema this is a validation failure, and the failure
    /// is how a human gets asked. Remedy: amend the determination.
    /// </summary>
    UndeterminableUnattributed,

    /// <summary>
    /// The rule could not be evaluated and A NAMED PRINCIPAL DECIDED SO. The absence is
    /// stated and carried. Remedy: none — it is a filed risk with an owner. R-Q38.
    /// </summary>
    UndeterminableCarried,
}

/// <summary>
/// An explicit, attributed decision not to supply a carrier. R-Q38.
/// </summary>
/// <remarks>
/// Modelled on the schema's own `allocation.residual`: a carrying principal is required
/// and <see cref="PrincipalKind"/> excludes `machine`, because — in the schema's words —
/// "a model identity cannot be an accepting principal". An agent may not decide that the
/// carrier of a fact does not matter.
/// </remarks>
public sealed record CarrierNotSupplied(string PrincipalKind, string PrincipalIdentifier, string Reason)
{
    public static readonly string[] AcceptableKinds = { "human", "team", "external-party" };

    public bool HasAccountablePrincipal =>
        AcceptableKinds.Contains(PrincipalKind, StringComparer.Ordinal)
        && !string.IsNullOrWhiteSpace(PrincipalIdentifier);
}

/// <summary>A read position as the determination store actually declares it.</summary>
public sealed record ModelledPosition(
    string FactType,
    string Kind,
    Carrier? Carrier,
    CarrierNotSupplied? CarrierWithheld = null)
{
    public bool CarrierIsModelled => Carrier is not null;

    /// <summary>R-Q38 — absence stated by a named principal, rather than absence.</summary>
    public bool CarrierIsWithheldDeliberately => CarrierWithheld is not null;
}

/// <summary>
/// Reads read positions out of a determination store. It reads the modelled carrier and
/// nothing else — no prose, no inference, no fallback.
/// </summary>
public static class DeterminationStore
{
    public static IReadOnlyDictionary<string, ModelledPosition> ReadPositionsFor(
        string path,
        string actInstance)
    {
        var records = new Deserializer()
            .Deserialize<List<Dictionary<string, object>>>(File.ReadAllText(path));

        var positions = new Dictionary<string, ModelledPosition>(StringComparer.Ordinal);

        foreach (var record in records)
        {
            if (record.GetValueOrDefault("address") is not Dictionary<object, object> address
                || address.GetValueOrDefault("act_instance") as string != actInstance
                || record.GetValueOrDefault("positions") is not List<object> declared)
            {
                continue;
            }

            foreach (var entry in declared.OfType<Dictionary<object, object>>())
            {
                if (entry.GetValueOrDefault("role") as string != "read"
                    || entry.GetValueOrDefault("fact_type") is not string factType)
                {
                    continue;
                }

                var boundary = entry.GetValueOrDefault("boundary") as Dictionary<object, object>
                               ?? new Dictionary<object, object>();

                positions[factType] = new ModelledPosition(
                    factType,
                    boundary.GetValueOrDefault("kind") as string ?? string.Empty,
                    ReadCarrier(boundary),
                    ReadWithheldCarrier(boundary));
            }
        }

        return positions;
    }

    /// <summary>
    /// The carrier is read, never inferred. An absent field means UNMODELLED, which is an
    /// answer. A present field outside the closed vocabulary is also unmodelled — an
    /// unrecognised value is not a licence to guess.
    /// </summary>
    private static Carrier? ReadCarrier(Dictionary<object, object> boundary) =>
        (boundary.GetValueOrDefault("carrier") as string) switch
        {
            "transport" => Carrier.Transport,
            "store" => Carrier.Store,
            "computed" => Carrier.Computed,
            "external-call" => Carrier.ExternalCall,
            _ => null,
        };

    /// <summary>
    /// R-Q38 — the object form of `carrier`: a stated, attributed decision not to supply
    /// one. Read, never inferred, and never accepted without a principal.
    /// </summary>
    private static CarrierNotSupplied? ReadWithheldCarrier(Dictionary<object, object> boundary)
    {
        if (boundary.GetValueOrDefault("carrier") is not Dictionary<object, object> stated
            || stated.GetValueOrDefault("state") as string != "not-supplied")
        {
            return null;
        }

        var principal = stated.GetValueOrDefault("principal") as Dictionary<object, object>
                        ?? new Dictionary<object, object>();

        var withheld = new CarrierNotSupplied(
            principal.GetValueOrDefault("kind") as string ?? string.Empty,
            principal.GetValueOrDefault("identifier") as string ?? string.Empty,
            stated.GetValueOrDefault("reason") as string ?? string.Empty);

        // No principal, no attribution — so it is not a decision, it is an omission
        // wearing one's clothes. The schema's own rule: the principal cannot be a machine.
        return withheld.HasAccountablePrincipal ? withheld : null;
    }
}

/// <summary>
/// The provider transport rule as amended by R-Q06, evaluated over modelled ground.
/// </summary>
public static class ProviderTransportRule
{
    public static Verdict Evaluate(bool referencesTransport, ModelledPosition? adapts)
    {
        // The prohibition is not engaged, so no ground is needed to answer.
        if (!referencesTransport)
        {
            return Verdict.Conforms;
        }

        // The rule turns on the carrier of the position adapted. No position, no ground.
        if (adapts is null)
        {
            return Verdict.UndeterminableUnattributed;
        }

        if (adapts.CarrierIsModelled)
        {
            return adapts.Carrier == Carrier.Transport ? Verdict.Conforms : Verdict.Breaches;
        }

        // R-Q38 — still no ground, but now it matters enormously WHY. An absence someone
        // signed for is a filed risk with an owner; an absence nobody signed for is a
        // question that has not been asked.
        return adapts.CarrierIsWithheldDeliberately
            ? Verdict.UndeterminableCarried
            : Verdict.UndeterminableUnattributed;
    }
}
