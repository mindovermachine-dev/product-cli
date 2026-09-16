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

/// <summary>The three answers a rule over modelled ground can give.</summary>
public enum Verdict
{
    /// <summary>The rule was evaluated and holds.</summary>
    Conforms,

    /// <summary>The rule was evaluated and is broken.</summary>
    Breaches,

    /// <summary>
    /// The rule could not be evaluated, because the ground it determines on is not
    /// modelled. Never collapse this into <see cref="Breaches"/>.
    /// </summary>
    Undeterminable,
}

/// <summary>A read position as the determination store actually declares it.</summary>
public sealed record ModelledPosition(string FactType, string Kind, Carrier? Carrier)
{
    public bool CarrierIsModelled => Carrier is not null;
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
                    ReadCarrier(boundary));
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
        if (adapts is null || !adapts.CarrierIsModelled)
        {
            return Verdict.Undeterminable;
        }

        return adapts.Carrier == Carrier.Transport ? Verdict.Conforms : Verdict.Breaches;
    }
}
