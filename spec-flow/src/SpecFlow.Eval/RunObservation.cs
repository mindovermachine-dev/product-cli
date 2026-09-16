using System.Diagnostics;
using Microsoft.Extensions.AI;
using Microsoft.Extensions.AI.Evaluation;
using SpecFlow.Flow;

namespace SpecFlow.Eval;

/// <summary>Turning one finished run into a record in the journal.</summary>
/// <remarks>
/// Observation is the last thing a run does and the least important: a failure
/// to write the journal must never lose the hand-off, because the hand-off is
/// what the flow exists for. Every method here is best-effort by design.
/// </remarks>
public static class RunObservation
{
    /// <summary>What was configured for the run, as the journal keeps it.</summary>
    /// <param name="Model">The model asked, or null when none was.</param>
    /// <param name="Endpoint">The endpoint URL; only its host is kept.</param>
    public sealed record Arrangement(string? Model, string? Endpoint)
    {
        /// <summary>Read the arrangement the process was started with.</summary>
        /// <remarks>
        /// The key is deliberately not read. A journal that accumulates
        /// credentials is one nobody can share.
        /// </remarks>
        public static Arrangement FromEnvironment() => new(
            Environment.GetEnvironmentVariable("SPECFLOW_MODEL"),
            Environment.GetEnvironmentVariable("SPECFLOW_MODEL_ENDPOINT"));
    }

    /// <summary>
    /// Evaluate a finished run and write it to the journal.
    /// </summary>
    /// <returns>Where the record landed, or null when nothing was written.</returns>
    public static async Task<string?> ObserveAsync(
        string root,
        ImplementRequest request,
        SliceBuilt built,
        ImplementOutcome outcome,
        Arrangement arrangement,
        TimeSpan duration,
        CancellationToken cancellationToken = default)
    {
        try
        {
            var context = new FlowRunContext(built.Slice, built.DraftDeterminations, outcome.ReviewedDeterminations);
            var result = await Evaluators.Default.EvaluateAsync(
                [new ChatMessage(ChatRole.User, $"Build the slice `{built.Slice}` against `{request.ActRef}`.")],
                new ChatResponse(new ChatMessage(ChatRole.Assistant, built.Notes)),
                chatConfiguration: null,
                additionalContext: [context],
                cancellationToken).ConfigureAwait(false);

            var record = new RunRecord(
                RunRecord.FormV1,
                outcome.RecordId,
                built.Slice,
                request.ActRef,
                DateTimeOffset.UtcNow,
                arrangement.Model,
                RunRecord.HostOf(arrangement.Endpoint),
                (long)duration.TotalMilliseconds,
                built.DraftDeterminations,
                outcome.ReviewedDeterminations,
                built.Notes.Length,
                built.Notes,
                Flatten(result));

            return RunJournal.Write(root, record);
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException)
        {
            Debug.WriteLine($"the run journal was not written: {e.Message}");
            return null;
        }
    }

    /// <summary>Flatten an evaluation result into what the journal stores.</summary>
    internal static IReadOnlyList<RunMetric> Flatten(EvaluationResult result) =>
    [
        .. result.Metrics.Values.Select(m => new RunMetric(
            m.Name,
            ValueOf(m),
            m.Reason,
            m.Interpretation?.Rating.ToString(),
            [.. (m.Diagnostics ?? []).Select(d => $"{d.Severity}: {d.Message}")])),
    ];

    /// <summary>A metric's value as text, whatever type it carries.</summary>
    private static string? ValueOf(EvaluationMetric metric) => metric switch
    {
        BooleanMetric b => b.Value?.ToString(),
        NumericMetric n => n.Value?.ToString(System.Globalization.CultureInfo.InvariantCulture),
        StringMetric s => s.Value,
        _ => null,
    };
}
