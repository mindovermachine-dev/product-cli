using System.ClientModel;
using Microsoft.Extensions.AI;
using OpenAI;
using Eval;
using SpecFlow.Flow;

namespace SpecFlow.Cli;

/// <summary>The <c>judge</c> verb: ask a model what it makes of a run.</summary>
/// <remarks>
/// A separate verb because it is a separate act. It happens after a run, on
/// its own occasion, under its own arrangement, and it writes its own record —
/// so a verdict can always be read back with the model that gave it and the
/// context it saw.
/// </remarks>
internal static class JudgeCommand
{
    public static async Task<int> RunAsync(Options options)
    {
        var recordId = options.Get("record");
        if (recordId is null)
        {
            Console.Error.WriteLine("judge needs --record <id>");
            return ExitCodes.CouldNotRun;
        }

        var store = ImplementCommand.EvalStoreOf(options);
        var run = store.ReadRuns().FirstOrDefault(r => r.RecordId == recordId);
        if (run is null)
        {
            Console.Error.WriteLine(
                $"no run record for `{recordId}` in {EvalStore.Runs}/ — "
              + "a run is judged after it is observed, and only a model-backed build is observed");
            return ExitCodes.CouldNotRun;
        }

        var arrangement = Arrangement();
        if (arrangement is null)
        {
            Console.Error.WriteLine(
                $"no judge configured. Set {Judging.Environment.Endpoint} and "
              + $"{Judging.Environment.Model} (and {Judging.Environment.Key} where the endpoint "
              + "wants one).\n  Nothing defaults to the builder's model: a model marking its own "
              + "work is the arrangement least worth recording, so it is not the one you get "
              + "by saying nothing.");
            return ExitCodes.CouldNotRun;
        }

        var act = await ActGround.ReadAsync(options, run.ActRef).ConfigureAwait(false) is { } ground
            ? new Judging.ActUnderJudgement(ground.Name, ground.Settles)
            : null;
        var (judge, client) = arrangement.Value;
        try
        {
            using (client)
            {
                var judgement = await Judging.JudgeAsync(run, judge, client, act).ConfigureAwait(false);
                var path = store.WriteJudgement(judgement);
                Report(judgement, path);
            }
        }
        catch (Exception e) when (e is OperationCanceledException or HttpRequestException
                                       or AggregateException or ClientResultException)
        {
            // A judge that would not answer in time is a judge that did not
            // judge. Nothing is filed: a missing verdict is honest, and an
            // empty one recorded as though it were an opinion is not.
            Console.Error.WriteLine($"the judge did not answer: {Innermost(e).Message}");
            return ExitCodes.CouldNotRun;
        }

        return ExitCodes.Conformant;
    }

    /// <summary>The cause worth printing, out of however many wrapped it.</summary>
    private static Exception Innermost(Exception e) =>
        e is AggregateException aggregate ? aggregate.GetBaseException() : e;

    /// <summary>
    /// The judge's own model and endpoint, or null when none was configured.
    /// </summary>
    /// <remarks>
    /// Read from variables of its own rather than the builder's. Sharing them
    /// would make self-assessment the default, and a default is exactly how an
    /// arrangement nobody chose ends up in the record.
    /// </remarks>
    private static (Judge Judge, IChatClient Client)? Arrangement()
    {
        var endpoint = Environment.GetEnvironmentVariable(Judging.Environment.Endpoint);
        var model = Environment.GetEnvironmentVariable(Judging.Environment.Model);
        if (string.IsNullOrWhiteSpace(endpoint) || string.IsNullOrWhiteSpace(model))
        {
            return null;
        }

        var key = Environment.GetEnvironmentVariable(Judging.Environment.Key) ?? "not-needed";
        var client = new OpenAIClient(
                new ApiKeyCredential(key),
                new OpenAIClientOptions { Endpoint = new Uri(endpoint) })
            .GetChatClient(model)
            .AsIChatClient();

        return (new Judge(model, RunRecord.HostOf(endpoint)), client);
    }

    private static void Report(Judgement judgement, string path)
    {
        Console.WriteLine($"judged {judgement.JudgesRecord} as `{judgement.Judge.Identity}`");
        Console.WriteLine($"  context {judgement.Context.Digest}");
        foreach (var verdict in judgement.Verdicts)
        {
            Console.WriteLine($"  {verdict.Name}: {verdict.Value ?? "(no answer)"} — {verdict.Reason}");
            foreach (var diagnostic in verdict.Diagnostics)
            {
                Console.WriteLine($"    {diagnostic}");
            }
        }
        Console.WriteLine($"  → {path}");
        Console.WriteLine(
            "\nA judgment ratifies nothing. It is evidence a person may read before deciding.");
    }
}
