namespace Ordering.Api.Tests;

using System.Reflection;
using Ordering.Api.Facts;
using Xunit;
using YamlDotNet.Serialization;

/// <summary>
/// CG-R-139's ground test, mechanised against the declared fact type space.
/// </summary>
/// <remarks>
/// > **"If obeying a determination requires authoring ground the determination does not
/// > declare, the determination is not settled."**
///
/// The ruling calls that *"mechanical enough to be useful"* and says *"it would have caught
/// DSC-0005 before the build"*. This class is the mechanism, in the only form available
/// before an analyser exists: compare every field the implementation actually carries
/// against `ordering.fact-type-space.md`, and flag anything the code has that the
/// declaration does not.
///
/// A flagged field is **authored ground**. It is the signal this session had —
/// `ActorIdentity.AccountCurrency` was marked `INVENTED AND CONTESTED` at the moment it
/// was written — with, until now, no rule to read it against.
///
/// D-59 — the comparison is over CONSTRUCTOR PARAMETERS, not properties. A computed
/// member such as `Cart.IsEmpty` or `Cart.Total` is derived from declared fields and is
/// not itself ground; a positional record parameter is a field the type requires someone
/// to supply. Deriving is allowed, requiring is not.
/// </remarks>
public sealed class FactShapeConformanceTests
{
    private static readonly string Declaration =
        Path.Combine(AppContext.BaseDirectory, "ordering.fact-type-space.md");

    /// <summary>
    /// THE GROUND TEST. Nothing this slice carries is ground the declaration does not
    /// supply. Before move 1 this could not be run at all — there was no declaration — and
    /// with the pre-move-1 shapes it would have failed on `AccountCurrency`.
    /// </summary>
    [Fact]
    public void No_implemented_fact_carries_a_field_the_declaration_does_not_declare()
    {
        var declared = DeclaredFacts();
        var authored = new List<string>();

        foreach (var (type, fields) in ImplementedFacts(declared))
        {
            authored.AddRange(
                RequiredFieldsOf(type)
                    .Where(field => !fields.Contains(field))
                    .Select(field => $"{type.Name}.{field}"));
        }

        Assert.Equal(Array.Empty<string>(), authored.OrderBy(a => a, StringComparer.Ordinal).ToArray());
    }

    /// <summary>
    /// And the converse: no declared field is silently dropped. A fact implemented with
    /// fewer fields than declared is under-implemented, which the ground test does not
    /// catch and which matters just as much.
    /// </summary>
    [Fact]
    public void No_implemented_fact_drops_a_declared_field()
    {
        var declared = DeclaredFacts();
        var dropped = new List<string>();

        foreach (var (type, fields) in ImplementedFacts(declared))
        {
            var present = RequiredFieldsOf(type).ToHashSet(StringComparer.Ordinal);
            dropped.AddRange(fields.Where(f => !present.Contains(f)).Select(f => $"{type.Name}.{f}"));
        }

        Assert.Equal(Array.Empty<string>(), dropped.OrderBy(d => d, StringComparer.Ordinal).ToArray());
    }

    /// <summary>
    /// The test proves it can fail. `AccountCurrency` is exactly the field CG-R-138
    /// withdrew and §6 of the declaration strikes by name — so it is checked directly, both
    /// that the declaration does not carry it and that the implementation no longer does.
    /// </summary>
    [Fact]
    public void AccountCurrency_is_absent_from_both_the_declaration_and_the_code()
    {
        Assert.DoesNotContain("AccountCurrency", DeclaredFacts()["ActorIdentity"]);
        Assert.DoesNotContain("AccountCurrency", RequiredFieldsOf(typeof(ActorIdentity)));
    }

    /// <summary>
    /// And that the mechanism itself discriminates — a type carrying an undeclared field is
    /// flagged. Without this the passing test above could be vacuous.
    /// </summary>
    [Fact]
    public void A_type_carrying_an_undeclared_field_is_flagged()
    {
        var declared = DeclaredFacts()["ActorIdentity"];

        var authored = RequiredFieldsOf(typeof(ActorIdentityAsBuiltBeforeMove1))
            .Where(field => !declared.Contains(field))
            .ToArray();

        Assert.Equal(new[] { "AccountCurrency" }, authored);
    }

    /// <summary>
    /// `ActorIdentity` as this session built it before move 1: `BuyerId` was `ActorId` and
    /// there was a second field nobody had declared. Kept only as the ground test's
    /// negative case.
    /// </summary>
    private sealed record ActorIdentityAsBuiltBeforeMove1(string BuyerId, string AccountCurrency);

    /// <summary>
    /// The facts this slice does NOT implement, recorded so their absence reads as scope
    /// rather than omission: they belong to other acts in the vocabulary.
    /// </summary>
    [Fact]
    public void The_facts_this_slice_does_not_implement_belong_to_other_acts()
    {
        var implemented = ImplementedFacts(DeclaredFacts()).Select(x => x.Type.Name).ToHashSet(StringComparer.Ordinal);

        var unimplemented = DeclaredFacts().Keys
            .Where(name => !implemented.Contains(name))
            .OrderBy(name => name, StringComparer.Ordinal)
            .ToArray();

        // ItemAddedToCart + CartEmptied belong to AddToCart / EmptyCart, OrderConfirmed to
        // ConfirmOrder, OrderSummary to the OrderSummary read-model. One slice, one act.
        Assert.Equal(
            new[] { "CartEmptied", "ItemAddedToCart", "OrderConfirmed", "OrderSummary" },
            unimplemented);
    }

    // --- reading the declaration ----------------------------------------------------

    /// <summary>
    /// Every top-level entry in the declaration that has a `fields:` list, as
    /// name → field names. Value kinds declared by `base:` have no fields and are skipped;
    /// §5's `Cart.Lines` constraint entry is skipped by its dotted name.
    /// </summary>
    private static Dictionary<string, HashSet<string>> DeclaredFacts()
    {
        var facts = new Dictionary<string, HashSet<string>>(StringComparer.Ordinal);
        var deserializer = new Deserializer();

        foreach (var block in YamlBlocksIn(File.ReadAllText(Declaration)))
        {
            if (deserializer.Deserialize<Dictionary<string, object>>(block) is not { } entries)
            {
                continue;
            }

            foreach (var (name, body) in entries)
            {
                if (name.Contains('.', StringComparison.Ordinal)
                    || body is not Dictionary<object, object> declaration
                    || declaration.GetValueOrDefault("fields") is not List<object> fields)
                {
                    continue;
                }

                facts[name] = fields
                    .OfType<Dictionary<object, object>>()
                    .Select(f => f.GetValueOrDefault("name") as string)
                    .OfType<string>()
                    .ToHashSet(StringComparer.Ordinal);
            }
        }

        return facts;
    }

    private static IEnumerable<string> YamlBlocksIn(string markdown)
    {
        var lines = markdown.Split('\n');
        var buffer = new List<string>();
        var inside = false;

        foreach (var line in lines)
        {
            if (line.StartsWith("```yaml", StringComparison.Ordinal))
            {
                inside = true;
                buffer.Clear();
                continue;
            }

            if (inside && line.StartsWith("```", StringComparison.Ordinal))
            {
                inside = false;
                yield return string.Join('\n', buffer);
                continue;
            }

            if (inside)
            {
                buffer.Add(line);
            }
        }
    }

    private static IEnumerable<(Type Type, HashSet<string> Fields)> ImplementedFacts(
        Dictionary<string, HashSet<string>> declared) =>
        typeof(Cart).Assembly.GetTypes()
            .Where(t => t.Namespace == "Ordering.Api.Facts" && declared.ContainsKey(t.Name))
            .Select(t => (t, declared[t.Name]));

    /// <summary>D-59 — constructor parameters, not properties. Derived members are not ground.</summary>
    private static IEnumerable<string> RequiredFieldsOf(Type type) =>
        type.GetConstructors()
            .OrderByDescending(c => c.GetParameters().Length)
            .First()
            .GetParameters()
            .Select(p => p.Name)
            .OfType<string>();
}
