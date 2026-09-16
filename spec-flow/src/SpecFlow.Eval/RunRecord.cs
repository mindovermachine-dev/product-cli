using System.Text.Json;
using System.Text.Json.Serialization;

namespace SpecFlow.Eval;

/// <summary>
/// One observed run of the build leg, kept so behaviour can be read over time.
/// </summary>
/// <remarks>
/// <para>
/// <b>Measurement, never a verdict.</b> A run record is not an act-time record:
/// it is not hashed, not signed, not closed, and <c>spec check</c> does not
/// read it. Nothing here can become a determination, and no metric below can
/// fail a build. The flow already has a place where judgment is filed under a
/// name, and this is deliberately not it.
/// </para>
/// <para>
/// The endpoint is kept as a host, and the key is not kept at all. A journal
/// that accumulates credentials is a journal nobody can share, which defeats
/// the reason for keeping one.
/// </para>
/// </remarks>
public sealed record RunRecord(
    [property: JsonPropertyName("form")] string Form,
    [property: JsonPropertyName("record")] string RecordId,
    [property: JsonPropertyName("slice")] string Slice,
    [property: JsonPropertyName("act_ref")] string ActRef,
    [property: JsonPropertyName("ran_at")] DateTimeOffset RanAt,
    [property: JsonPropertyName("model")] string? Model,
    [property: JsonPropertyName("endpoint_host")] string? EndpointHost,
    [property: JsonPropertyName("duration_ms")] long DurationMs,
    [property: JsonPropertyName("drafted")] IReadOnlyList<string> Drafted,
    [property: JsonPropertyName("reviewed")] IReadOnlyList<string> Reviewed,
    [property: JsonPropertyName("reply_chars")] int ReplyChars,
    [property: JsonPropertyName("metrics")] IReadOnlyList<RunMetric> Metrics)
{
    /// <summary>The form this file is written in.</summary>
    public const string FormV1 = "spec.run-record.v1";

    private static readonly JsonSerializerOptions Options = new()
    {
        WriteIndented = true,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
    };

    /// <summary>Render as the JSON that lands on disk.</summary>
    public string ToJson() => JsonSerializer.Serialize(this, Options) + "\n";

    /// <summary>Read one back.</summary>
    public static RunRecord? FromJson(string json) =>
        JsonSerializer.Deserialize<RunRecord>(json, Options);

    /// <summary>The host of an endpoint, or null when there was none.</summary>
    /// <remarks>
    /// Host rather than the whole URL: a query string is where a key ends up
    /// when somebody is in a hurry, and this file is meant to be shareable.
    /// </remarks>
    public static string? HostOf(string? endpoint) =>
        Uri.TryCreate(endpoint, UriKind.Absolute, out var parsed) ? parsed.Host : null;
}

/// <summary>One metric, flattened from an evaluation result for storage.</summary>
/// <param name="Value">
/// The measured value as text. Kept untyped on purpose: a journal that has to
/// be migrated before it can be read is a journal nobody reads.
/// </param>
public sealed record RunMetric(
    [property: JsonPropertyName("name")] string Name,
    [property: JsonPropertyName("value")] string? Value,
    [property: JsonPropertyName("reason")] string? Reason,
    [property: JsonPropertyName("rating")] string? Rating,
    [property: JsonPropertyName("diagnostics")] IReadOnlyList<string> Diagnostics);
