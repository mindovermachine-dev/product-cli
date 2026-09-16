using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Eval;

/// <summary>
/// A model's assessment of one run, on one occasion, over a pinned context.
/// </summary>
/// <remarks>
/// <para>
/// <b>Asking a model to assess a run is itself an act.</b> It happens at a
/// time, by a named model, over a particular context, and it is not
/// reproducible — run the same judge over the same inputs tomorrow and it may
/// say something else. A verdict recorded without saying who gave it and what
/// they saw is a number that reads as fact and cannot be checked, so this
/// record carries all three or it is not written.
/// </para>
/// <para>
/// It is separate from <see cref="RunRecord"/> on purpose. That file holds
/// arithmetic anyone recomputes from the same inputs; this one holds an
/// opinion. Filing them together would let the second borrow the first's
/// standing.
/// </para>
/// <para>
/// <b>Never a ratification.</b> There is no principal field, and a machine
/// could not fill one — <c>S002</c> and <c>L006</c> say so on the other side
/// of the seam. A judgment is evidence a person may read before deciding; it
/// decides nothing.
/// </para>
/// </remarks>
public sealed record Judgement(
    [property: JsonPropertyName("form")] string Form,
    [property: JsonPropertyName("judges")] string JudgesRecord,
    [property: JsonPropertyName("judged_at")] DateTimeOffset JudgedAt,
    [property: JsonPropertyName("judge")] Judge Judge,
    [property: JsonPropertyName("context")] JudgementContext Context,
    [property: JsonPropertyName("verdicts")] IReadOnlyList<RunMetric> Verdicts)
{
    /// <summary>The form this file is written in.</summary>
    public const string FormV1 = "eval.judgement.v1";

    /// <summary>
    /// Always true, and carried rather than implied.
    /// </summary>
    /// <remarks>
    /// A reader who finds this file among the store's other JSON should not
    /// have to know the directory layout to learn that nothing here was
    /// decided by anyone.
    /// </remarks>
    [JsonPropertyName("ratifies_nothing")]
    public bool RatifiesNothing => true;

    private static readonly JsonSerializerOptions Options = new()
    {
        WriteIndented = true,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    };

    /// <summary>Render as the JSON that lands on disk.</summary>
    public string ToJson() => JsonSerializer.Serialize(this, Options) + "\n";

    /// <summary>Read one back.</summary>
    public static Judgement? FromJson(string json) =>
        JsonSerializer.Deserialize<Judgement>(json, Options);
}

/// <summary>Who gave the verdict. A machine, named as one.</summary>
/// <param name="Model">The model asked.</param>
/// <param name="EndpointHost">Where it was asked. Host only; never the key.</param>
public sealed record Judge(
    [property: JsonPropertyName("model")] string Model,
    [property: JsonPropertyName("endpoint_host")] string? EndpointHost)
{
    /// <summary>
    /// How the judge is named, in a shape the identity law reads as a machine.
    /// </summary>
    /// <remarks>
    /// A `model:` prefix rather than an address that could pass for a person's.
    /// The Rust half refuses a machine principal at <c>S002</c> by the same
    /// reading; this side does not get to be vaguer about it.
    /// </remarks>
    [JsonPropertyName("identity")]
    public string Identity => $"model:{Model}";
}

/// <summary>
/// Exactly what the judge was shown, and a digest that pins it.
/// </summary>
/// <remarks>
/// The digest is over the inputs as given, so a verdict can be tied to the
/// state that produced it. Re-observe the run, recompute, and a mismatch says
/// the verdict was about something else — the move the policy's
/// <c>basis_binds</c> makes, for the same reason.
/// </remarks>
public sealed record JudgementContext(
    [property: JsonPropertyName("digest")] string Digest,
    [property: JsonPropertyName("shown")] IReadOnlyDictionary<string, string> Shown)
{
    /// <summary>The digest's prefix, distinguishing it from the store's others.</summary>
    public const string Prefix = "eval.judgement-context.v1";

    /// <summary>The separator between a key and its value in the canonical form.</summary>
    private const char Separator = (char)0x1f;

    /// <summary>
    /// Pin a context by digesting what it shows.
    /// </summary>
    /// <remarks>
    /// The canonical form is stated normatively in §6 of the format document,
    /// and this is one of two implementations of it. Both assert against
    /// `docs/eval-format-v1/context-digest.json`, which belongs to the format
    /// rather than to either of them, so a change made on one side and not the
    /// other fails on both.
    /// </remarks>
    public static JudgementContext Pin(IReadOnlyDictionary<string, string> shown)
    {
        var ordered = new SortedDictionary<string, string>(
            shown.ToDictionary(p => p.Key, p => p.Value), StringComparer.Ordinal);

        var entries = ordered
            .Select(p => (p.Key, Value: Normalise(p.Value)))
            .Where(p => p.Value.Length > 0)
            .Select(p => $"{p.Key}{Separator}{p.Value}");

        var canonical = $"{Prefix}\n{string.Join('\n', entries)}";
        var digest = Convert.ToHexStringLower(SHA256.HashData(Encoding.UTF8.GetBytes(canonical)));
        return new JudgementContext($"sha256:{digest}", ordered);
    }

    /// <summary>
    /// A value as the canonical form takes it: newlines folded, NFC, trimmed.
    /// </summary>
    /// <remarks>
    /// A key whose value normalises to nothing is dropped by the caller, so
    /// absent and present-but-empty cannot pin differently — a distinction no
    /// reader could act on is not one worth hashing.
    /// </remarks>
    public static string Normalise(string value) =>
        value.Replace("\r\n", "\n", StringComparison.Ordinal)
             .Replace('\r', '\n')
             .Normalize(NormalizationForm.FormC)
             .Trim(' ', '\t', '\n', '\v', '\f');

    /// <summary>
    /// Whether the digest still matches what this context says it showed.
    /// </summary>
    /// <remarks>
    /// A record that carries its own check is a record a reader can disbelieve
    /// without reconstructing the run.
    /// </remarks>
    public bool Holds() => Pin(Shown).Digest == Digest;
}
