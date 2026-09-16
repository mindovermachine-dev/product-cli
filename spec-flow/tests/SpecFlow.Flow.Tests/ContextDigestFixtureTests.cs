using System.Text.Json;
using SpecFlow.Eval;

namespace SpecFlow.Flow.Tests;

/// <summary>
/// The digests this runtime shares with `eval-core`.
/// </summary>
/// <remarks>
/// <para>
/// `docs/eval-format-v1.md` §6 states the canonical form normatively and two
/// implementations follow it. A format doc that only one side reads is a doc
/// that drifts, so both assert against the same file: a change made here and
/// not in Rust fails here, and the reverse fails there.
/// </para>
/// <para>
/// The fixture is regenerated from the Rust side
/// (`UPDATE_FIXTURES=1 cargo test -p eval-core --test fixture`), which makes a
/// changed digest visible as a diff to a checked-in file rather than as a test
/// somebody quietly updated.
/// </para>
/// </remarks>
public class ContextDigestFixtureTests
{
    /// <summary>
    /// The shared fixture, found by walking up to the workspace root.
    /// </summary>
    /// <remarks>
    /// Required rather than optional: a test that skips when its subject is
    /// missing passes in exactly the environment where it matters least.
    /// </remarks>
    private static JsonDocument Fixture()
    {
        for (var probe = new DirectoryInfo(AppContext.BaseDirectory); probe is not null; probe = probe.Parent)
        {
            var candidate = Path.Combine(
                probe.FullName, "eval-core", "tests", "fixtures", "context-digest.json");
            if (File.Exists(candidate))
            {
                return JsonDocument.Parse(File.ReadAllText(candidate));
            }
        }
        throw new InvalidOperationException(
            "could not find eval-core/tests/fixtures/context-digest.json");
    }

    public static TheoryData<string> CaseNames()
    {
        var data = new TheoryData<string>();
        using var fixture = Fixture();
        foreach (var testCase in fixture.RootElement.GetProperty("cases").EnumerateArray())
        {
            data.Add(testCase.GetProperty("name").GetString() ?? "(unnamed)");
        }
        return data;
    }

    [Theory]
    [MemberData(nameof(CaseNames))]
    public void This_runtime_pins_the_digest_the_fixture_claims(string name)
    {
        using var fixture = Fixture();
        var testCase = fixture.RootElement.GetProperty("cases").EnumerateArray()
            .First(c => c.GetProperty("name").GetString() == name);

        var shown = new Dictionary<string, string>(StringComparer.Ordinal);
        foreach (var field in testCase.GetProperty("shown").EnumerateObject())
        {
            shown[field.Name] = field.Value.GetString() ?? string.Empty;
        }

        Assert.Equal(
            testCase.GetProperty("digest").GetString(),
            JudgementContext.Pin(shown).Digest);
    }

    /// <summary>Both runtimes agree the store is shared, not merely similar.</summary>
    [Fact]
    public void The_forms_are_the_shared_ones()
    {
        Assert.Equal("eval.judgement.v1", Judgement.FormV1);
        Assert.Equal("eval.run-record.v1", RunRecord.FormV1);
        Assert.Equal("eval.judgement-context.v1", JudgementContext.Prefix);
    }
}
