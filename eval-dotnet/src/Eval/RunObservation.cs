using System.Diagnostics;
using Microsoft.Extensions.AI;
using Microsoft.Extensions.AI.Evaluation;

namespace Eval;

/// <summary>What one finished run looked like, in this pattern's vocabulary.</summary>
/// <remarks>
/// <para>
/// Deliberately names nothing from any particular tool. A caller maps its own
/// shapes onto this, which is what lets the same store hold runs from an agent
/// workflow, a batch job and a one-shot script without any of them knowing
/// about the others.
/// </para>
/// <para>
/// <paramref name="Proposed"/> and <paramref name="Kept"/> are the pair worth
/// having: what the model put forward, and what a person kept of it. A caller
/// with no human in the loop passes them equal and says so by their being
/// equal, rather than by leaving one out.
/// </para>
/// </remarks>
/// <param name="Id">The run's identity, from whatever the caller already calls this work.</param>
/// <param name="Tool">Which tool ran it, so one store can hold several.</param>
/// <param name="Task">What was attempted. An address, not prose.</param>
/// <param name="Subject">What it was attempted on.</param>
/// <param name="Endpoint">Where the model was asked. Only its host is kept.</param>
public sealed record ObservedRun(
    string Id,
    string Tool,
    string Task,
    string Subject,
    string? Model,
    string? Endpoint,
    TimeSpan Duration,
    IReadOnlyList<string> Proposed,
    IReadOnlyList<string> Kept,
    string Reply);

/// <summary>Turning one finished run into a record in the store.</summary>
/// <remarks>
/// Observation is the last thing a run does and the least important: a failure
/// to write the store must never lose the work, because the work is what the
/// run was for. Every method here is best-effort by design.
/// </remarks>
public static class RunObservation
{
    /// <summary>
    /// Evaluate a finished run and file it.
    /// </summary>
    /// <param name="address">
    /// Where the run happened. Supply it and the run becomes comparable to
    /// others at the same coordinates; leave it out and it stands alone, which
    /// the absence of the field tells a reader plainly.
    /// </param>
    /// <param name="arrangement">What answered, kept apart from the address.</param>
    /// <returns>Where the record landed, or null when nothing was written.</returns>
    public static async Task<string?> ObserveAsync(
        EvalStore store,
        ObservedRun run,
        Pinned? address = null,
        Pinned? arrangement = null,
        CancellationToken cancellationToken = default)
    {
        try
        {
            var context = new RunContext(run.Subject, run.Proposed, run.Kept);
            var result = await Evaluators.Default.EvaluateAsync(
                [new ChatMessage(ChatRole.User, $"Do `{run.Subject}` for `{run.Task}`.")],
                new ChatResponse(new ChatMessage(ChatRole.Assistant, run.Reply)),
                chatConfiguration: null,
                additionalContext: [context],
                cancellationToken).ConfigureAwait(false);

            var record = new RunRecord(
                RunRecord.FormV1,
                run.Id,
                run.Tool,
                run.Subject,
                run.Task,
                DateTimeOffset.UtcNow,
                run.Model,
                RunRecord.HostOf(run.Endpoint),
                (long)run.Duration.TotalMilliseconds,
                run.Proposed,
                run.Kept,
                run.Reply,
                Flatten(result),
                address,
                arrangement);

            return store.WriteRun(record);
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException
                                       or InvalidOperationException or ArgumentException)
        {
            Debug.WriteLine($"the run was not observed: {e.Message}");
            return null;
        }
    }

    /// <summary>Flatten an evaluation result into what the store keeps.</summary>
    public static IReadOnlyList<RunMetric> Flatten(EvaluationResult result) =>
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
