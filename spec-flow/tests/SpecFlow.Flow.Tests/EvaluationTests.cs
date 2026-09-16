using Microsoft.Extensions.AI;
using Microsoft.Extensions.AI.Evaluation;
using SpecFlow.Eval;

namespace SpecFlow.Flow.Tests;

/// <summary>What the flow observes about its own runs, and what it refuses to.</summary>
public class EvaluationTests
{
    private static FlowRunContext Run(string slice, string[] drafted, string[] reviewed) =>
        new(slice, drafted, reviewed);

    private static async Task<EvaluationResult> Evaluate(IEvaluator evaluator, string reply, FlowRunContext flow) =>
        await evaluator.EvaluateAsync(
            [new ChatMessage(ChatRole.User, "build it")],
            new ChatResponse(new ChatMessage(ChatRole.Assistant, reply)),
            chatConfiguration: null,
            additionalContext: [flow]);

    [Fact]
    public async Task A_reply_claiming_none_while_drafting_one_is_incoherent()
    {
        var result = await Evaluate(
            new DraftCoherenceEvaluator(),
            "I did not need to make any determinations during this action.\n[\"det/a\"]",
            Run("s", ["det/a"], ["det/a"]));

        Assert.False(result.Get<BooleanMetric>(DraftCoherenceEvaluator.Metric).Value);
    }

    [Fact]
    public async Task A_reply_claiming_none_and_drafting_none_agrees_with_itself()
    {
        var result = await Evaluate(
            new DraftCoherenceEvaluator(), "Nothing arose.\n[]", Run("s", [], []));

        Assert.True(result.Get<BooleanMetric>(DraftCoherenceEvaluator.Metric).Value);
    }

    /// <summary>A heuristic that cannot tell says so, rather than guessing.</summary>
    [Fact]
    public async Task An_unrecognised_claim_is_inconclusive_rather_than_true()
    {
        var result = await Evaluate(
            new DraftCoherenceEvaluator(), "Built it. Seemed fine.\n[]", Run("s", [], []));

        var metric = result.Get<BooleanMetric>(DraftCoherenceEvaluator.Metric);
        Assert.Null(metric.Value);
        Assert.Equal(EvaluationRating.Inconclusive, metric.Interpretation?.Rating);
    }

    [Fact]
    public void An_untouched_draft_measures_no_amendment()
    {
        Assert.Equal(0d, AmendmentEvaluator.Measure(Run("s", ["det/a", "det/b"], ["det/a", "det/b"])).Value);
    }

    [Fact]
    public void A_draft_the_reviewer_discarded_measures_total_amendment()
    {
        Assert.Equal(1d, AmendmentEvaluator.Measure(Run("s", ["det/a"], [])).Value);
    }

    [Fact]
    public void A_half_kept_draft_measures_between()
    {
        var measured = AmendmentEvaluator.Measure(Run("s", ["det/a", "det/b"], ["det/a"])).Value;
        Assert.NotNull(measured);
        Assert.InRange(measured.Value, 0.4, 0.6);
    }

    [Fact]
    public void A_malformed_address_is_not_a_determination()
    {
        Assert.False(DeterminationShapeEvaluator.Inspect(Run("s", ["settle totals"], [])).Value);
    }

    /// <summary>The smell we saw live: an address naming the work, not the choice.</summary>
    [Fact]
    public void An_address_echoing_the_slice_is_a_diagnostic_not_a_failure()
    {
        var metric = DeterminationShapeEvaluator.Inspect(
            Run("settle-totals-v2", ["det/settle-totals-v2-builtin"], []));

        Assert.True(metric.Value, "it is well formed, so it is not a failure");
        Assert.NotNull(metric.Diagnostics);
        Assert.Contains(metric.Diagnostics, d => d.Message.Contains("echoes the slice name", StringComparison.Ordinal));
    }

    /// <summary>
    /// No metric this journal keeps can fail anything.
    /// </summary>
    /// <remarks>
    /// The gate is `spec check`, and it does not read the journal. A metric
    /// that could fail a build would make a measurement into a verdict that
    /// no principal signed.
    /// </remarks>
    [Fact]
    public async Task No_metric_can_fail_a_build()
    {
        var result = await Evaluate(
            Evaluators.Default,
            "I did not need to make any determinations.\n[\"not an address\"]",
            Run("s", ["not an address"], []));

        Assert.NotEmpty(result.Metrics);
        foreach (var metric in result.Metrics.Values)
        {
            Assert.False(metric.Interpretation?.Failed, $"`{metric.Name}` reports; it must not gate");
        }
    }

    [Fact]
    public void The_journal_keeps_the_endpoint_host_and_never_the_key()
    {
        Assert.Equal("api.scaleway.ai", RunRecord.HostOf("https://api.scaleway.ai/v1"));
        Assert.Null(RunRecord.HostOf(null));
    }

    [Fact]
    public void A_written_record_reads_back()
    {
        using var repo = new SpecRepo();
        var record = new RunRecord(
            RunRecord.FormV1, "01ABC", "slice", "act/a", DateTimeOffset.UtcNow,
            "qwen3.6-35b-a3b", "api.scaleway.ai", 1234, ["det/a"], [], 42, []);

        var path = RunJournal.Write(repo.Root, record);

        Assert.True(File.Exists(path));
        Assert.Contains("01ABC", Path.GetFileName(path), StringComparison.Ordinal);
        var read = RunJournal.Read(repo.Root);
        Assert.Collection(read, only => Assert.Equal("qwen3.6-35b-a3b", only.Model));
    }

    /// <summary>The journal is beside the store, never inside it.</summary>
    [Fact]
    public void The_journal_is_not_the_record_store()
    {
        Assert.Equal(".spec/runs", RunJournal.Directory);
        Assert.DoesNotContain("records", RunJournal.Directory, StringComparison.Ordinal);
    }
}
