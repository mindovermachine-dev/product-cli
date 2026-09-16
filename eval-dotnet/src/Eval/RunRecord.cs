using System.Text.Json;
using System.Text.Json.Serialization;

namespace Eval;

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
/// <para>
/// The reply is kept whole. A run that proposed nothing is the interesting
/// case, and a length cannot tell a model that answered `[]` from one whose
/// answer could not be read.
/// </para>
/// <para>
/// <b>Address and arrangement are what make runs comparable.</b> The address is
/// where a run happened — the task instance together with the ground the caller
/// declared — so two runs at the same address were asked the same question. The
/// arrangement is what answered. Kept apart so they vary independently: same
/// address and same arrangement isolates run-to-run variance, while same
/// address and a changed arrangement is drift in the arrangement rather than in
/// the world.
/// </para>
/// <para>
/// Both are optional. A caller that declares no coordinates can still record
/// what it did; its runs simply cannot be read against each other, and the
/// absence of the field is how a reader learns that rather than guessing.
/// </para>
/// <para>
/// The shape is the format document's, so one store can hold runs from tools
/// that know nothing about each other — including tools in another runtime.
/// </para>
/// </remarks>
public sealed record RunRecord(
    [property: JsonPropertyName("form")] string Form,
    [property: JsonPropertyName("id")] string RecordId,
    [property: JsonPropertyName("tool")] string Tool,
    [property: JsonPropertyName("subject")] string Slice,
    [property: JsonPropertyName("task")] string ActRef,
    [property: JsonPropertyName("ran_at")] DateTimeOffset RanAt,
    [property: JsonPropertyName("model")] string? Model,
    [property: JsonPropertyName("endpoint_host")] string? EndpointHost,
    [property: JsonPropertyName("duration_ms")] long DurationMs,
    [property: JsonPropertyName("proposed")] IReadOnlyList<string> Proposed,
    [property: JsonPropertyName("kept")] IReadOnlyList<string> Kept,
    [property: JsonPropertyName("reply")] string? Reply,
    [property: JsonPropertyName("metrics")] IReadOnlyList<RunMetric> Metrics,
    [property: JsonPropertyName("address")] Pinned? Address = null,
    [property: JsonPropertyName("arrangement")] Pinned? Arrangement = null,
    [property: JsonPropertyName("declared")] Declaration? Declared = null,
    [property: JsonPropertyName("ground_read")] IReadOnlyList<string>? GroundReadOrNull = null,
    [property: JsonPropertyName("attributions")] IReadOnlyList<Attribution>? AttributionsOrNull = null)
{
    /// <summary>The form this file is written in.</summary>
    public const string FormV1 = "eval.run-record.v1";

    /// <summary>The ground actually consulted.</summary>
    /// <remarks>
    /// Read against <see cref="Declaration.Ground"/>: declared and never read,
    /// or read and never declared, are different faults with different fixes.
    /// </remarks>
    [JsonIgnore]
    public IReadOnlyList<string> GroundRead => GroundReadOrNull ?? [];

    /// <summary>What the act claimed, and what each claim rests on.</summary>
    [JsonIgnore]
    public IReadOnlyList<Attribution> Attributions => AttributionsOrNull ?? [];

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
