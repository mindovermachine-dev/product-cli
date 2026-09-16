using Microsoft.Extensions.AI;
using Microsoft.Extensions.AI.Evaluation;
using Eval;

namespace Eval.Tests;

/// <summary>What the flow observes about its own runs, and what it refuses to.</summary>
public class EvaluationTests
{
    private static RunContext Run(string slice, string[] drafted, string[] reviewed) =>
        new(slice, drafted, reviewed);

    private static async Task<EvaluationResult> Evaluate(IEvaluator evaluator, string reply, RunContext flow) =>
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
        using var repo = new TempStore();
        var record = new RunRecord(
            RunRecord.FormV1, "01ABC", "spec-flow", "slice", "act/a", DateTimeOffset.UtcNow,
            "qwen3.6-35b-a3b", "api.scaleway.ai", 1234, ["det/a"], [], "built it", []);

        var store = new EvalStore(new DiskBlobs(repo.Root));
        var path = store.WriteRun(record);

        Assert.True(File.Exists(path));
        Assert.Contains("01ABC", Path.GetFileName(path), StringComparison.Ordinal);
        Assert.Collection(store.ReadRuns(), only => Assert.Equal("qwen3.6-35b-a3b", only.Model));
    }

    /// <summary>The journal is beside the act store, never inside it.</summary>
    [Fact]
    public void The_journal_is_not_the_record_store()
    {
        Assert.Equal("runs/01ABC.json", EvalStore.RunKey("01ABC"));
        Assert.DoesNotContain("records", EvalStore.Runs, StringComparison.Ordinal);
    }

    /// <summary>
    /// The keys are the same whatever holds them, which is what makes a swap a swap.
    /// </summary>
    [Fact]
    public void The_layout_does_not_depend_on_the_backend()
    {
        Assert.Equal("runs/01ABC.json", EvalStore.RunKey("01ABC"));
        Assert.Equal("judgements/01ABC", EvalStore.JudgementsPrefix("01ABC"));
    }

    /// <summary>A path configures disk; the azure spelling configures azure.</summary>
    [Fact]
    public void The_backend_is_named_by_configuration()
    {
        Assert.IsType<Backend.Disk>(Backend.Parse(".spec"));
        var azure = Assert.IsType<Backend.Azure>(Backend.Parse("azure:evalstore/runs/spec-flow"));
        Assert.Equal("evalstore", azure.Account);
        Assert.Equal("spec-flow", azure.Prefix);
    }

    /// <summary>A tool told to write to Azure must not quietly write to disk.</summary>
    [Fact]
    public void Azure_says_it_is_not_built_rather_than_falling_back()
    {
        var refusal = Assert.Throws<InvalidOperationException>(
            () => Backend.Parse("azure:evalstore/runs").Open());
        Assert.Contains("not built", refusal.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void A_half_written_azure_spelling_is_refused_rather_than_guessed_at()
    {
        Assert.Throws<ArgumentException>(() => Backend.Parse("azure:evalstore"));
    }

    /// <summary>A store that can be talked into writing elsewhere is not a store.</summary>
    [Fact]
    public void A_key_cannot_climb_out_of_the_root()
    {
        using var repo = new TempStore();
        var blobs = new DiskBlobs(repo.Root);
        Assert.Throws<ArgumentException>(() => blobs.Put("../escaped.json", "{}"));
    }
}

/// <summary>A judgment is an act: who judged, over what, on what occasion.</summary>
public class JudgementTests
{
    private static RunRecord Run(params string[] drafted) => new(
        RunRecord.FormV1, "01ABC", "spec-flow", "settle-totals", "act/settle-a-basket",
        DateTimeOffset.UnixEpoch, "builder-model", "api.scaleway.ai", 100,
        drafted, [], "built it", []);

    [Fact]
    public void A_context_pins_what_the_judge_was_shown()
    {
        var context = Judging.ContextOf(Run("det/a"));

        Assert.StartsWith("sha256:", context.Digest, StringComparison.Ordinal);
        Assert.True(context.Holds());
        Assert.Equal("det/a", context.Shown["proposed"]);
        Assert.Equal("(not shown)", context.Shown["act_settles"]);
        Assert.Equal("01ABC", context.Shown["run"]);
    }

    /// <summary>A verdict is about a state, and says which one.</summary>
    [Fact]
    public void A_different_run_pins_a_different_context()
    {
        Assert.NotEqual(Judging.ContextOf(Run("det/a")).Digest, Judging.ContextOf(Run("det/b")).Digest);
    }

    [Fact]
    public void The_same_run_pins_the_same_context()
    {
        Assert.Equal(Judging.ContextOf(Run("det/a")).Digest, Judging.ContextOf(Run("det/a")).Digest);
    }

    /// <summary>A context edited after the fact no longer holds its digest.</summary>
    [Fact]
    public void An_altered_context_does_not_hold()
    {
        var pinned = Judging.ContextOf(Run("det/a"));
        var altered = pinned with
        {
            Shown = new Dictionary<string, string>(pinned.Shown, StringComparer.Ordinal)
            {
                ["drafted"] = "det/something-else",
            },
        };

        Assert.False(altered.Holds());
    }

    /// <summary>What the judge was shown of the act is part of what it saw.</summary>
    [Fact]
    public void An_act_shown_pins_a_different_context_than_one_withheld()
    {
        var withheld = Judging.ContextOf(Run("det/a"));
        var shown = Judging.ContextOf(
            Run("det/a"),
            new Judging.ActUnderJudgement("Settle a basket", "What the customer owes."));

        Assert.NotEqual(withheld.Digest, shown.Digest);
        Assert.Equal("What the customer owes.", shown.Shown["act_settles"]);
        Assert.True(shown.Holds());
    }

    [Fact]
    public void A_judge_is_named_as_a_machine()
    {
        Assert.Equal("model:glm-5.2", new Judge("glm-5.2", "api.scaleway.ai").Identity);
    }

    /// <summary>
    /// Nothing a judge says ratifies anything.
    /// </summary>
    /// <remarks>
    /// Carried on the record rather than implied by where the file sits: a
    /// reader should not have to know the directory layout to learn that
    /// nothing here was decided by anyone.
    /// </remarks>
    [Fact]
    public void A_judgement_ratifies_nothing()
    {
        var judgement = new Judgement(
            Judgement.FormV1, "01ABC", DateTimeOffset.UtcNow,
            new Judge("glm-5.2", "api.scaleway.ai"), Judging.ContextOf(Run("det/a")), []);

        Assert.True(judgement.RatifiesNothing);
        Assert.Contains("\"ratifies_nothing\": true", judgement.ToJson(), StringComparison.Ordinal);
        Assert.DoesNotContain("principal", judgement.ToJson(), StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void A_judge_answering_in_form_produces_a_verdict()
    {
        var verdicts = Judging.Read("""{"warranted": 1, "unwarranted": 0, "reason": "it names a choice"}""", 1);

        Assert.Collection(verdicts, only =>
        {
            Assert.Equal(Judging.Verdict, only.Name);
            Assert.Equal("1/1", only.Value);
            Assert.Equal("Good", only.Rating);
        });
    }

    /// <summary>A judge that would not answer is itself worth recording.</summary>
    [Fact]
    public void A_judge_answering_out_of_form_is_inconclusive_not_an_error()
    {
        var verdicts = Judging.Read("I would rather not say.", 1);

        Assert.Collection(verdicts, only =>
        {
            Assert.Null(only.Value);
            Assert.Equal("Inconclusive", only.Rating);
            Assert.Contains(only.Diagnostics, d => d.Contains("did not answer", StringComparison.Ordinal));
        });
    }

    /// <summary>One run, judged more than once, keeps both opinions.</summary>
    [Fact]
    public void Two_judges_of_one_run_are_both_kept()
    {
        using var repo = new TempStore();
        var store = new EvalStore(new DiskBlobs(repo.Root));
        var context = Judging.ContextOf(Run("det/a"));
        foreach (var model in new[] { "glm-5.2", "qwen3.6-35b-a3b" })
        {
            store.WriteJudgement(new Judgement(
                Judgement.FormV1, "01ABC", DateTimeOffset.UtcNow,
                new Judge(model, "api.scaleway.ai"), context, []));
        }

        var kept = store.ReadJudgements("01ABC");
        Assert.Equal(2, kept.Count);
        Assert.Contains(kept, j => j.Judge.Model is "glm-5.2");
        Assert.Contains(kept, j => j.Judge.Model is "qwen3.6-35b-a3b");
    }

    /// <summary>Re-asking the same judge the same question overwrites, never accretes.</summary>
    [Fact]
    public void The_same_judge_over_the_same_context_files_once()
    {
        using var repo = new TempStore();
        var judgement = new Judgement(
            Judgement.FormV1, "01ABC", DateTimeOffset.UtcNow,
            new Judge("glm-5.2", "api.scaleway.ai"), Judging.ContextOf(Run("det/a")), []);

        var store = new EvalStore(new DiskBlobs(repo.Root));
        Assert.Equal(store.WriteJudgement(judgement), store.WriteJudgement(judgement));
        Assert.Single(store.ReadJudgements("01ABC"));
    }

    /// <summary>Judgments are kept apart from the run they judge.</summary>
    [Fact]
    public void The_judgement_store_is_not_the_run_journal()
    {
        Assert.NotEqual(EvalStore.Runs, EvalStore.Judgements);
    }
}
