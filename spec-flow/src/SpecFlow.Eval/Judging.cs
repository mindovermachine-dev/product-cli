using System.Text.Json;
using Microsoft.Extensions.AI;

namespace SpecFlow.Eval;

/// <summary>Asking a model what it makes of a run, and filing what it said.</summary>
/// <remarks>
/// <para>
/// Separate from the build, and separately configured. A judge sharing the
/// builder's model and endpoint by default would make the common case the one
/// where a model marks its own work, which is the arrangement least worth
/// recording.
/// </para>
/// <para>
/// Nothing here defaults. With no judge configured the verb refuses rather
/// than falling back to the builder's model: which model judges is a decision,
/// and a tool that makes it silently has made it badly.
/// </para>
/// </remarks>
public static class Judging
{
    /// <summary>What the judge is asked to assess.</summary>
    public const string Verdict = "Determination warrant";

    /// <summary>Where the judge's own arrangement is read from.</summary>
    public static class Environment
    {
        /// <summary>The judging model. Required; nothing defaults to the builder's.</summary>
        public const string Model = "SPECFLOW_JUDGE_MODEL";

        /// <summary>The judging endpoint. Required.</summary>
        public const string Endpoint = "SPECFLOW_JUDGE_ENDPOINT";

        /// <summary>The judging key, where the endpoint wants one.</summary>
        public const string Key = "SPECFLOW_JUDGE_KEY";
    }

    /// <summary>
    /// What the judge is shown, pinned before it is asked anything.
    /// </summary>
    /// <remarks>
    /// Built from the run record plus what the act settles, so a reader can
    /// reconstruct the inputs from files already on disk. The act's text is
    /// proxied from the Rust binary rather than parsed here — one reading of
    /// what the store means, or a judge and a CI run can be told different
    /// things about the same act.
    /// </para>
    /// <para>
    /// <b>It is the act text that makes the question answerable.</b> Without
    /// it, two different judges independently answered that no determination
    /// could be warranted, which is the correct answer to a question nobody
    /// had given them the means to settle.
    /// </remarks>
    public static JudgementContext ContextOf(RunRecord run, ActUnderJudgement? act = null) =>
        JudgementContext.Pin(
            new Dictionary<string, string>(StringComparer.Ordinal)
            {
                ["record"] = run.RecordId,
                ["slice"] = run.Slice,
                ["act_ref"] = run.ActRef,
                ["act_name"] = act?.Name ?? "(not shown)",
                ["act_settles"] = act?.Settles ?? "(not shown)",
                ["builder_model"] = run.Model ?? "(none)",
                ["drafted"] = string.Join(",", run.Drafted),
                ["reviewed"] = string.Join(",", run.Reviewed),
                ["reply_chars"] = run.ReplyChars.ToString(System.Globalization.CultureInfo.InvariantCulture),
            });

    /// <summary>What the specification settles, as the store reports it.</summary>
    /// <param name="Name">What a principal called the act.</param>
    /// <param name="Settles">The question the act answers.</param>
    public sealed record ActUnderJudgement(string Name, string Settles);

    /// <summary>The question put to the judge.</summary>
    /// <remarks>
    /// It asks about warrant, not quality: whether each drafted determination
    /// is a choice the specification left open, or a restatement of the spec or
    /// a description of the work. That is the reviewer's question, and the one
    /// a second reader could disagree with usefully.
    /// </remarks>
    public static string Question(JudgementContext context)
    {
        var shown = string.Join("\n", context.Shown.Select(p => $"  {p.Key}: {p.Value}"));
        return $$"""
            A slice was built against a specification act, and the builder drafted
            the determination addresses listed below. A determination names a choice
            the specification did not settle — not a restatement of the spec, and not
            a description of the work that was done.

            {{shown}}

            For each drafted address, say whether it is warranted as a determination.
            You are judging warrant, not quality. Where the context does not let
            you settle a case, say so rather than guessing.

            Reply with one line of JSON and nothing else:
            {"warranted": <count>, "unwarranted": <count>, "reason": "<one sentence>"}
            """;
    }

    /// <summary>
    /// Ask the judge, and shape what it said into verdicts.
    /// </summary>
    /// <remarks>
    /// An unparsable reply becomes a verdict with no value and the raw text as
    /// its reason, rather than an error. The judge having been unclear is
    /// itself worth recording — it is the behaviour the journal exists to show
    /// over time.
    /// </remarks>
    public static async Task<Judgement> JudgeAsync(
        RunRecord run,
        Judge judge,
        IChatClient client,
        ActUnderJudgement? act = null,
        CancellationToken cancellationToken = default)
    {
        var context = ContextOf(run, act);
        var response = await client
            .GetResponseAsync(Question(context), cancellationToken: cancellationToken)
            .ConfigureAwait(false);

        return new Judgement(
            Judgement.FormV1,
            run.RecordId,
            DateTimeOffset.UtcNow,
            judge,
            context,
            Read(response.Text ?? string.Empty, run.Drafted.Count));
    }

    /// <summary>Turn the judge's reply into verdicts.</summary>
    public static IReadOnlyList<RunMetric> Read(string reply, int drafted)
    {
        foreach (var line in reply.Split('\n').Reverse())
        {
            var trimmed = line.Trim();
            if (!trimmed.StartsWith('{') || !trimmed.EndsWith('}'))
            {
                continue;
            }
            try
            {
                using var parsed = JsonDocument.Parse(trimmed);
                var root = parsed.RootElement;
                var warranted = root.TryGetProperty("warranted", out var w) ? w.GetInt32() : 0;
                var unwarranted = root.TryGetProperty("unwarranted", out var u) ? u.GetInt32() : 0;
                var reason = root.TryGetProperty("reason", out var r) ? r.GetString() : null;

                return
                [
                    new RunMetric(
                        Verdict,
                        $"{warranted}/{drafted}",
                        reason,
                        Rating(warranted, unwarranted),
                        []),
                ];
            }
            catch (Exception e) when (e is JsonException or FormatException or InvalidOperationException)
            {
                // Not the object we were looking for; keep scanning upward.
            }
        }

        return
        [
            new RunMetric(
                Verdict,
                Value: null,
                Reason: Truncate(reply),
                Rating: "Inconclusive",
                Diagnostics: ["Warning: the judge did not answer in the form it was asked for"]),
        ];
    }

    private static string Rating(int warranted, int unwarranted) => (warranted, unwarranted) switch
    {
        (_, 0) when warranted > 0 => "Good",
        (0, 0) => "Unknown",
        (0, _) => "Poor",
        _ => "Average",
    };

    private static string Truncate(string reply) =>
        reply.Length <= 500 ? reply : reply[..500] + "…";
}
