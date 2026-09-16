using Microsoft.Extensions.AI;
using Microsoft.Extensions.AI.Evaluation;

namespace Eval;

/// <summary>
/// The deterministic evaluators the flow runs over its own builds.
/// </summary>
/// <remarks>
/// <para>
/// None of these asks a model anything. A judge scoring the output of the
/// thing it is judging inherits that thing's blind spots, costs a call per
/// run, and produces a number nobody can reproduce — so the default set is
/// observations a second reader could check by hand.
/// </para>
/// <para>
/// They are <see cref="IEvaluator"/>s so that a judged evaluator can be added
/// beside them without changing what reads the journal. Adding one is a
/// decision about cost and trust, which is why it is not made here.
/// </para>
/// </remarks>
public static class Evaluators
{
    /// <summary>The set that runs after every build.</summary>
    public static IEvaluator Default { get; } = new CompositeEvaluator(
        new DraftCoherenceEvaluator(),
        new AmendmentEvaluator(),
        new DeterminationShapeEvaluator());

    /// <summary>Find the flow's own context among what was handed over.</summary>
    internal static RunContext? FlowContext(IEnumerable<EvaluationContext>? context) =>
        context?.OfType<RunContext>().FirstOrDefault();

    /// <summary>A metric nothing can fail, carrying a rating and a reason.</summary>
    /// <remarks>
    /// <c>Failed</c> is always false. These metrics report; the gate is
    /// `spec check`, and it does not read this journal. A metric that could
    /// fail a build would make a measurement into a verdict nobody signed.
    /// </remarks>
    internal static T Reporting<T>(this T metric, EvaluationRating rating, string reason)
        where T : EvaluationMetric
    {
        metric.Interpretation = new EvaluationMetricInterpretation(rating, failed: false, reason);
        return metric;
    }
}

/// <summary>Whether the reply's prose agrees with the array it ended on.</summary>
/// <remarks>
/// A builder that writes "I made no determinations" and then emits one has
/// contradicted itself, and the draft carries the contradiction to a reviewer
/// as if it were a proposal. Observed against a live model on the first run
/// that had one.
/// </remarks>
public sealed class DraftCoherenceEvaluator : IEvaluator
{
    /// <summary>What this evaluator reports.</summary>
    public const string Metric = "Draft coherence";

    /// <summary>
    /// Phrases that claim nothing arose.
    /// </summary>
    /// <remarks>
    /// A small, literal list rather than a judge. It answers "unknown" when
    /// none matches, which is the honest answer to a reply phrased some other
    /// way — a heuristic that guesses when it cannot tell is worse than one
    /// that says so.
    /// </remarks>
    private static readonly string[] ClaimsNone =
    [
        "did not need to make any determinations",
        "no determinations",
        "nothing arose",
        "made no determinations",
    ];

    /// <inheritdoc />
    public IReadOnlyCollection<string> EvaluationMetricNames => [Metric];

    /// <inheritdoc />
    public ValueTask<EvaluationResult> EvaluateAsync(
        IEnumerable<ChatMessage> messages,
        ChatResponse modelResponse,
        ChatConfiguration? chatConfiguration = null,
        IEnumerable<EvaluationContext>? additionalContext = null,
        CancellationToken cancellationToken = default)
    {
        var flow = Evaluators.FlowContext(additionalContext);
        var reply = modelResponse.Text ?? string.Empty;
        var claimed = ClaimsNone.FirstOrDefault(
            phrase => reply.Contains(phrase, StringComparison.OrdinalIgnoreCase));
        var drafted = flow?.Drafted.Count ?? 0;

        var metric = (claimed, drafted) switch
        {
            (null, _) => new BooleanMetric(Metric, value: null,
                    reason: "the reply states no claim this evaluator recognises")
                .Reporting(EvaluationRating.Inconclusive, "no recognised claim to compare"),

            (not null, 0) => new BooleanMetric(Metric, value: true,
                    reason: $"claimed none (\"{claimed}\") and drafted none")
                .Reporting(EvaluationRating.Good, "prose and draft agree"),

            _ => new BooleanMetric(Metric, value: false,
                    reason: $"claimed none (\"{claimed}\") yet drafted {drafted}")
                .Reporting(EvaluationRating.Poor,
                    "the reviewer was shown a determination the reply says was not made"),
        };

        return new ValueTask<EvaluationResult>(new EvaluationResult(metric));
    }
}

/// <summary>How far the reviewer moved the draft.</summary>
/// <remarks>
/// The signal worth having: a person looked at what a model proposed and said
/// what they would actually file. Zero means the draft stood. It is not a
/// score of the model — a reviewer may amend a good draft — but over many runs
/// it is the only measure here grounded in a human judgment.
/// </remarks>
public sealed class AmendmentEvaluator : IEvaluator
{
    /// <summary>What this evaluator reports.</summary>
    public const string Metric = "Reviewer amendment";

    /// <inheritdoc />
    public IReadOnlyCollection<string> EvaluationMetricNames => [Metric];

    /// <inheritdoc />
    public ValueTask<EvaluationResult> EvaluateAsync(
        IEnumerable<ChatMessage> messages,
        ChatResponse modelResponse,
        ChatConfiguration? chatConfiguration = null,
        IEnumerable<EvaluationContext>? additionalContext = null,
        CancellationToken cancellationToken = default)
    {
        var flow = Evaluators.FlowContext(additionalContext);
        var metric = flow is null
            ? new NumericMetric(Metric, value: null, reason: "the run carried no review")
                .Reporting(EvaluationRating.Unknown, "nothing to compare")
            : Measure(flow);

        return new ValueTask<EvaluationResult>(new EvaluationResult(metric));
    }

    /// <summary>Jaccard distance between what was drafted and what was kept.</summary>
    public static NumericMetric Measure(RunContext flow)
    {
        var drafted = flow.Drafted.ToHashSet(StringComparer.Ordinal);
        var reviewed = flow.Reviewed.ToHashSet(StringComparer.Ordinal);
        if (drafted.Count is 0 && reviewed.Count is 0)
        {
            return new NumericMetric(Metric, value: 0d, reason: "nothing drafted, nothing filed")
                .Reporting(EvaluationRating.Good, "the draft stood");
        }

        var union = new HashSet<string>(drafted, StringComparer.Ordinal);
        union.UnionWith(reviewed);
        var kept = drafted.Intersect(reviewed, StringComparer.Ordinal).Count();
        var distance = 1d - ((double)kept / union.Count);

        return new NumericMetric(Metric, Math.Round(distance, 4),
                reason: $"{kept} of {union.Count} address(es) survived review")
            .Reporting(
                distance is 0d ? EvaluationRating.Good : EvaluationRating.Average,
                distance is 0d ? "the draft stood" : "the reviewer amended the draft");
    }
}

/// <summary>Whether the drafted addresses are shaped like determinations.</summary>
/// <remarks>
/// An address is a pointer to something the specification did not settle. One
/// that echoes the slice name usually points at the work instead, which reads
/// as governed and is not — reported as a diagnostic rather than a failure,
/// because it is a smell and not a rule.
/// </remarks>
public sealed class DeterminationShapeEvaluator : IEvaluator
{
    /// <summary>What this evaluator reports.</summary>
    public const string Metric = "Determination shape";

    /// <inheritdoc />
    public IReadOnlyCollection<string> EvaluationMetricNames => [Metric];

    /// <inheritdoc />
    public ValueTask<EvaluationResult> EvaluateAsync(
        IEnumerable<ChatMessage> messages,
        ChatResponse modelResponse,
        ChatConfiguration? chatConfiguration = null,
        IEnumerable<EvaluationContext>? additionalContext = null,
        CancellationToken cancellationToken = default)
    {
        var flow = Evaluators.FlowContext(additionalContext);
        var metric = flow is null
            ? new BooleanMetric(Metric, value: null, reason: "the run drafted nothing to shape")
                .Reporting(EvaluationRating.Unknown, "nothing to inspect")
            : Inspect(flow);

        return new ValueTask<EvaluationResult>(new EvaluationResult(metric));
    }

    /// <summary>Judge the shape of every drafted address.</summary>
    public static BooleanMetric Inspect(RunContext flow)
    {
        var malformed = flow.Drafted.Where(a => !IsWellFormed(a)).ToList();
        var echoes = flow.Drafted
            .Where(a => IsWellFormed(a) && a.Contains(flow.Slice, StringComparison.OrdinalIgnoreCase))
            .ToList();

        var metric = new BooleanMetric(Metric, malformed.Count is 0,
            reason: malformed.Count is 0
                ? $"{flow.Drafted.Count} address(es), all shaped `det/…`"
                : $"not an address: {string.Join(", ", malformed)}");

        foreach (var echo in echoes)
        {
            metric.AddDiagnostics(EvaluationDiagnostic.Warning(
                $"`{echo}` echoes the slice name — a determination addresses what was "
              + "settled, not what was built"));
        }

        return metric.Reporting(
            malformed.Count is 0 && echoes.Count is 0 ? EvaluationRating.Good : EvaluationRating.Average,
            malformed.Count is 0 ? "addresses are well formed" : "an address is not one");
    }

    /// <summary>A determination address: `det/` then a slug.</summary>
    public static bool IsWellFormed(string address) =>
        address.StartsWith("det/", StringComparison.Ordinal)
        && address.Length > 4
        && !address.Any(char.IsWhiteSpace);
}
