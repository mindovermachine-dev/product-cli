namespace Ordering.Api.Tests;

using System.Reflection;
using Ordering.Api.Profile;
using Ordering.Api.Slices.PlaceOrder;
using Xunit;
using YamlDotNet.Serialization;

/// <summary>
/// The amended provider rule from ruling R-Q06, checked against the real determination
/// store — and the first demonstration in this run that a profile rule is enforceable
/// only when the checker can read the determinations.
/// </summary>
/// <remarks>
/// R-Q06 amends the provider's <c>must_not</c>: a provider adapting a transport-borne
/// source MAY reference a transport type; a provider adapting a store MAY NOT. That makes
/// <c>ActorIdentityProvider</c> conforming — it adapts an OIDC token claim, which arrives
/// on the request — and leaves the rule biting on every other provider.
///
/// **The amendment moves the rule, it does not simply relax it.** Before R-Q06 the rule
/// was a flat prohibition: checkable by a namespace test on a type's references, and
/// defeated by one interface. After R-Q06 it is conditional on *what the provider
/// adapts*, which is not in the code at all — it is in the determination that declares
/// the read position's <c>boundary</c>. So the check below does what an analyser would
/// have to do: parse `place-order.determinations.yaml`, find the position for the fact
/// this provider supplies, and decide from its boundary whether a transport reference is
/// permitted here.
///
/// D-40 — THE WEAK LINK, AND IT IS IN THE SCHEMA, NOT THE CODE. Nothing in
/// `determination.schema.json` says a boundary's carrier is transport. `read_provenance`
/// is `{"type": "string"}` — free text. DSC-0003 says "OIDC token claim, validated at the
/// gateway", and the only way to get "transport" out of that is to match on prose, which
/// is what <see cref="IsTransportBorne"/> does and what no analyser should ever ship.
/// A machine-readable `boundary.carrier` enum is proposed in `rulings.md`; until there is
/// one, R-Q06's rule is enforceable in principle and prose-matched in practice.
///
/// D-41 — DECIDED. A provider is matched to the fact it supplies by the return type of
/// its single public method. Nothing states that convention; it is this session's, and an
/// analyser would need it stated. Q-34.
/// </remarks>
public sealed class ProviderTransportCarrierTests
{
    private const string TransportNamespace = "Microsoft.AspNetCore.Http";

    /// <summary>
    /// The rule, as amended by R-Q06, run over every provider role in the slice.
    /// </summary>
    [Fact]
    public void Only_providers_adapting_a_transport_borne_source_reference_transport()
    {
        var positions = ReadPositionsFor("PlaceOrder");

        foreach (var provider in ProviderRoles())
        {
            var fact = FactSuppliedBy(provider);
            var permitted = fact is not null
                            && positions.TryGetValue(fact, out var boundary)
                            && IsTransportBorne(boundary);

            var references = ReferencesTransport(provider);

            Assert.True(
                permitted || !references,
                $"{provider.Name} references a transport type, and the determination for "
                + $"'{fact}' does not declare a transport-borne carrier.");
        }
    }

    /// <summary>
    /// The permission is real and not vacuous: <c>ActorIdentityProvider</c> does reference
    /// transport, and DSC-0003 is what makes that conforming.
    /// </summary>
    [Fact]
    public void ActorIdentityProvider_is_permitted_by_DSC_0003_and_uses_the_permission()
    {
        var positions = ReadPositionsFor("PlaceOrder");

        Assert.True(IsTransportBorne(positions["ActorIdentity"]));
        Assert.True(ReferencesTransport(typeof(ActorIdentityProvider)));
    }

    /// <summary>
    /// And it still bites: <c>Cart</c>'s boundary is plain <c>internal</c>, so
    /// <c>CartProvider</c> gets no permission and takes none.
    /// </summary>
    [Fact]
    public void CartProvider_gets_no_permission_and_references_no_transport()
    {
        var positions = ReadPositionsFor("PlaceOrder");

        Assert.False(IsTransportBorne(positions["Cart"]));
        Assert.False(ReferencesTransport(typeof(CartProvider)));
    }

    // --- what an analyser would have to do -----------------------------------------

    /// <summary>
    /// D-40. Prose-matching, because `read_provenance` is free text and the schema has no
    /// carrier field. This is the method that should not exist.
    /// </summary>
    private static bool IsTransportBorne(IReadOnlyDictionary<string, string> boundary)
    {
        if (!boundary.TryGetValue("read_provenance", out var provenance))
        {
            return false;
        }

        return provenance.Contains("token", StringComparison.OrdinalIgnoreCase)
               || provenance.Contains("claim", StringComparison.OrdinalIgnoreCase)
               || provenance.Contains("request", StringComparison.OrdinalIgnoreCase);
    }

    private static Dictionary<string, IReadOnlyDictionary<string, string>> ReadPositionsFor(string actInstance)
    {
        var path = Path.Combine(AppContext.BaseDirectory, "place-order.determinations.yaml");
        var records = new Deserializer()
            .Deserialize<List<Dictionary<string, object>>>(File.ReadAllText(path));

        var positions = new Dictionary<string, IReadOnlyDictionary<string, string>>(StringComparer.Ordinal);

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

                positions[factType] = boundary.ToDictionary(
                    kv => (string)kv.Key,
                    kv => kv.Value as string ?? string.Empty,
                    StringComparer.Ordinal);
            }
        }

        return positions;
    }

    private static IEnumerable<Type> ProviderRoles() =>
        typeof(PlaceOrderHandler).Assembly.GetTypes()
            .Where(t => t.GetCustomAttribute<SliceAttribute>() is { Role: SliceRole.Provider });

    /// <summary>D-41 — the fact a provider supplies is its method's return type.</summary>
    private static string? FactSuppliedBy(Type provider) =>
        provider.GetMethods(BindingFlags.Public | BindingFlags.Instance | BindingFlags.DeclaredOnly)
            .Select(m => Nullable.GetUnderlyingType(m.ReturnType) ?? m.ReturnType)
            .FirstOrDefault(t => t.Namespace == "Ordering.Api.Facts")
            ?.Name;

    private static bool ReferencesTransport(Type type) =>
        type.GetConstructors()
            .SelectMany(c => c.GetParameters())
            .Any(p => p.ParameterType.Namespace?.StartsWith(TransportNamespace, StringComparison.Ordinal) == true);
}
