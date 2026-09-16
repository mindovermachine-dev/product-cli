namespace Ordering.Api.Tests;

using System.Text.Json;
using Xunit;
using YamlDotNet.Serialization;

/// <summary>
/// An audit of `determination.schema.json`'s closure, and of what ruling R-Q37 costs to
/// apply.
/// </summary>
/// <remarks>
/// R-Q37: *"boundary should be closed, it's an oversight."*
///
/// This class exists because "oversight" is a claim that can be checked, and checking it
/// turned out to matter: the fix is **not** a one-line addition, because closing
/// `boundary` without also declaring `carrier` invalidates the very ground R-GROUND
/// requires. The two changes are coupled and must land together. That is asserted below
/// rather than asserted in prose.
///
/// The schema is an arrived input and is not modified here. The proposed patch is in
/// `rulings.md`.
/// </remarks>
public sealed class SchemaClosureTests
{
    private static readonly string SchemaPath =
        Path.Combine(AppContext.BaseDirectory, "determination.schema.json");

    private static readonly string StorePath =
        Path.Combine(AppContext.BaseDirectory, "place-order.determinations.yaml");

    private static readonly string FixturePath =
        Path.Combine(AppContext.BaseDirectory, "fixtures", "dsc-0003.carrier-modelled.yaml");

    /// <summary>
    /// R-Q37 confirmed: the oversight is a single miss, not a pattern. Every object in the
    /// schema is closed — directly, or by <c>oneOf</c> branches that each close themselves —
    /// except one.
    /// </summary>
    /// <remarks>
    /// `allocation` looks open at its top level and is not: each of its three `oneOf`
    /// branches carries `additionalProperties: false` over its own properties, so an
    /// unexpected key fails every branch and the `oneOf` fails. Closure by branch is still
    /// closure. `boundary` has no such structure and is simply open.
    /// </remarks>
    [Fact]
    public void Boundary_is_the_only_object_the_schema_leaves_open()
    {
        using var schema = JsonDocument.Parse(File.ReadAllText(SchemaPath));

        var open = ObjectsIn(schema.RootElement, string.Empty)
            .Where(o => !o.ClosedDirectly && !o.ClosedByBranches)
            .Select(o => o.Path)
            .OrderBy(p => p, StringComparer.Ordinal)
            .ToArray();

        Assert.Equal(new[] { "/$defs/position/properties/boundary" }, open);
    }

    /// <summary>
    /// A PINNED DEFECT. Delete this test the day R-Q37 is applied; it asserts the state the
    /// ruling says is wrong, so that the fix registers as a change rather than passing
    /// silently.
    /// </summary>
    [Fact]
    public void Boundary_is_currently_open_which_R_Q37_rules_an_oversight()
    {
        using var schema = JsonDocument.Parse(File.ReadAllText(SchemaPath));

        var boundary = schema.RootElement
            .GetProperty("$defs").GetProperty("position")
            .GetProperty("properties").GetProperty("boundary");

        Assert.False(boundary.TryGetProperty("additionalProperties", out _));
    }

    /// <summary>
    /// Closing `boundary` is non-breaking for the store as delivered: every boundary key it
    /// uses is already declared. So the ruling costs nothing in migration.
    /// </summary>
    [Fact]
    public void Closing_boundary_does_not_invalidate_the_delivered_store()
    {
        Assert.Empty(UndeclaredBoundaryKeysIn(StorePath));
    }

    /// <summary>
    /// THE COUPLING, AND THE REASON THIS CLASS EXISTS.
    ///
    /// `carrier` is exactly what closing `boundary` would forbid. Apply R-Q37 alone and the
    /// ground R-GROUND requires becomes unrepresentable; apply it with the `carrier`
    /// declaration and both rulings land. The open `boundary` is currently the only reason
    /// a modelled carrier can be written at all — which is also why it cannot be relied on.
    /// </summary>
    [Fact]
    public void Closing_boundary_without_declaring_carrier_would_forbid_the_modelled_ground()
    {
        var undeclared = UndeclaredBoundaryKeysIn(FixturePath);

        Assert.Equal(new[] { "carrier" }, undeclared);
    }

    // --- helpers -------------------------------------------------------------------

    private static string[] UndeclaredBoundaryKeysIn(string storePath)
    {
        using var schema = JsonDocument.Parse(File.ReadAllText(SchemaPath));

        var declared = schema.RootElement
            .GetProperty("$defs").GetProperty("position")
            .GetProperty("properties").GetProperty("boundary")
            .GetProperty("properties")
            .EnumerateObject()
            .Select(p => p.Name)
            .ToHashSet(StringComparer.Ordinal);

        var records = new Deserializer()
            .Deserialize<List<Dictionary<string, object>>>(File.ReadAllText(storePath));

        return records
            .Select(r => r.GetValueOrDefault("positions"))
            .OfType<List<object>>()
            .SelectMany(positions => positions.OfType<Dictionary<object, object>>())
            .Select(position => position.GetValueOrDefault("boundary"))
            .OfType<Dictionary<object, object>>()
            .SelectMany(boundary => boundary.Keys.OfType<string>())
            .Where(key => !declared.Contains(key))
            .Distinct(StringComparer.Ordinal)
            .OrderBy(key => key, StringComparer.Ordinal)
            .ToArray();
    }

    private sealed record SchemaObject(string Path, bool ClosedDirectly, bool ClosedByBranches);

    private static IEnumerable<SchemaObject> ObjectsIn(JsonElement node, string path)
    {
        if (node.ValueKind == JsonValueKind.Object)
        {
            if (node.TryGetProperty("type", out var type)
                && type.ValueKind == JsonValueKind.String
                && type.GetString() == "object"
                && node.TryGetProperty("properties", out _))
            {
                yield return new SchemaObject(path, ClosedDirectly(node), ClosedByBranches(node));
            }

            foreach (var child in node.EnumerateObject())
            {
                foreach (var found in ObjectsIn(child.Value, $"{path}/{child.Name}"))
                {
                    yield return found;
                }
            }
        }
        else if (node.ValueKind == JsonValueKind.Array)
        {
            var index = 0;
            foreach (var child in node.EnumerateArray())
            {
                foreach (var found in ObjectsIn(child, $"{path}/{index}"))
                {
                    yield return found;
                }

                index++;
            }
        }
    }

    private static bool ClosedDirectly(JsonElement node) =>
        node.TryGetProperty("additionalProperties", out var extra)
        && extra.ValueKind == JsonValueKind.False;

    /// <summary>`allocation`'s shape: every `oneOf` branch closes itself.</summary>
    private static bool ClosedByBranches(JsonElement node) =>
        node.TryGetProperty("oneOf", out var branches)
        && branches.ValueKind == JsonValueKind.Array
        && branches.EnumerateArray().Any()
        && branches.EnumerateArray().All(ClosedDirectly);
}
