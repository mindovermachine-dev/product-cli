using System.ClientModel;
using Microsoft.Agents.AI;
using OpenAI;
using OpenAI.Chat;
using Eval;
using SpecFlow.Flow;
using SpecFlow.Mcp;

namespace SpecFlow.Cli;

/// <summary>The <c>implement</c> verb: build a slice, hand the closure over.</summary>
internal static class ImplementCommand
{
    public static async Task<int> RunAsync(Options options)
    {
        var slice = options.Get("slice");
        var actRef = options.Get("act");
        if (slice is null || actRef is null)
        {
            Console.Error.WriteLine("implement needs --slice and --act");
            return ExitCodes.CouldNotRun;
        }

        var cli = new SpecCli(options.SpecBinary, options.Root);
        var tools = await GovernedTools
            .ConnectAsync(new SpecFlowOptions(options.Root, options.SpecBinary))
            .ConfigureAwait(false);

        // What was built, kept so the run can be observed after the hand-off.
        // Observing is the last thing the run does, and the least important.
        SliceBuilt? built = null;
        var build = BuildStrategy(options, tools);
        var request = new ImplementRequest(slice, actRef, options.Get("by") ?? "agent@example.invalid");

        var started = System.Diagnostics.Stopwatch.StartNew();
        var driver = new ImplementDriver(
            cli,
            async (opened, token) => built = await build(opened, token).ConfigureAwait(false),
            ReviewAtTheTerminal);
        var outcome = await driver.RunAsync(request).ConfigureAwait(false);
        started.Stop();

        Report(outcome);

        if (built is not null)
        {
            var ground = await ActGround.ReadAsync(options, actRef).ConfigureAwait(false);
            var journalled = await RunObservation.ObserveAsync(
                EvalStoreOf(options),
                Observed(request, built, outcome, started.Elapsed),
                Address(actRef, ground),
                Arrangement(options),
                Bounds(ground),
                ground?.Addresses ?? []).ConfigureAwait(false);
            if (journalled is not null)
            {
                Console.WriteLine($"observed: {journalled}");
            }
        }

        return ExitCodes.PendingClosure;
    }

    /// <summary>
    /// This flow's shapes, mapped onto the pattern's vocabulary.
    /// </summary>
    /// <remarks>
    /// The mapping lives here rather than in the library: `Eval` names nothing
    /// from this flow, which is what lets the same store hold runs from tools
    /// that know nothing about each other.
    /// </remarks>
    private static ObservedRun Observed(
        ImplementRequest request, SliceBuilt built, ImplementOutcome outcome, TimeSpan elapsed)
        => new(
            outcome.RecordId,
            Tool,
            request.ActRef,
            built.Slice,
            Environment.GetEnvironmentVariable("SPECFLOW_MODEL"),
            Environment.GetEnvironmentVariable("SPECFLOW_MODEL_ENDPOINT"),
            elapsed,
            built.DraftDeterminations,
            outcome.ReviewedDeterminations,
            built.Notes,
            built.Declared,
            built.GroundRead ?? []);

    /// <summary>Which tool these runs came from, so one store can hold several.</summary>
    internal const string Tool = "spec-flow";

    /// <summary>
    /// Where a build happened: the act, and the ground it declares.
    /// </summary>
    /// <remarks>
    /// <para>
    /// Deliberately <i>not</i> the slice. Two attempts at one act are the same
    /// question asked twice, and naming them `settle-totals-v2` and `-v3` does
    /// not make them different ones. Including the slice would give every run
    /// its own address and nothing would ever be comparable — which is the
    /// failure mode worth avoiding, because it looks like success.
    /// </para>
    /// <para>
    /// The ground is in the address, so editing what an act settles moves the
    /// coordinates and stops older runs being read against newer ones. That is
    /// correct: after such an edit they were not asked the same question.
    /// </para>
    /// </remarks>
    private static Pinned Address(string actRef, ActGround.Ground? ground) => Pinned.Of(
        ("task", actRef),
        ("ground", ground?.Settles ?? "(unread)"));

    /// <summary>
    /// What the arrangement fixed, as against what the worker declared.
    /// </summary>
    /// <remarks>
    /// Tolerance and assurance are the arrangement's to state, never the
    /// worker's: a worker setting its own tolerance decides how wrong it may be,
    /// and one declaring its own assurance prices a consequence it does not
    /// carry. This flow states them plainly and modestly — the act's own text is
    /// the bound, and a build is drafted for review rather than accepted.
    /// </remarks>
    private static (string Tolerance, string Assurance) Bounds(ActGround.Ground? ground) => (
        ground is null
            ? "the act's text, which could not be read"
            : $"what `{ground.Name}` settles: {ground.Settles}",
        "drafted for review; no verdict is accepted from this run");

    /// <summary>
    /// What answered: the worker, and where it was reached.
    /// </summary>
    /// <remarks>
    /// Separate from the address so the two vary independently. A model bump at
    /// a fixed address is the reading worth having, and it is only visible if
    /// the worker is not part of the coordinates.
    /// </remarks>
    private static Pinned Arrangement(Options options) => Pinned.Of(
        ("model", Environment.GetEnvironmentVariable("SPECFLOW_MODEL") ?? "(none)"),
        ("endpoint_host",
            RunRecord.HostOf(Environment.GetEnvironmentVariable("SPECFLOW_MODEL_ENDPOINT")) ?? "(none)"),
        ("instructions", options.Get("instructions") ?? ""));

    /// <summary>
    /// The store runs are observed into, as configuration names it.
    /// </summary>
    /// <remarks>
    /// Defaults to `.spec/` beside the act store, which is where a repo with no
    /// opinion wants it. `EVAL_STORE` moves it, including to a backend this
    /// build does not have — which refuses rather than silently writing to disk.
    /// </remarks>
    internal static EvalStore EvalStoreOf(Options options) =>
        new(Backend.FromEnvironment(Path.Combine(options.Root, ".spec")).Open());

    /// <summary>
    /// How the slice gets built. With no model endpoint configured the run
    /// still happens and still opens a record — the drafting is what is
    /// missing, not the write-back leg.
    /// </summary>
    private static Func<ActRecordOpened, CancellationToken, ValueTask<SliceBuilt>> BuildStrategy(
        Options options,
        IReadOnlyList<Microsoft.Extensions.AI.AITool> tools)
    {
        var endpoint = Environment.GetEnvironmentVariable("SPECFLOW_MODEL_ENDPOINT");
        if (string.IsNullOrWhiteSpace(endpoint))
        {
            return (opened, _) => new ValueTask<SliceBuilt>(
                new SliceBuilt(opened.RecordId, opened.Slice, [], "no model endpoint configured; nothing drafted"));
        }

        var model = Environment.GetEnvironmentVariable("SPECFLOW_MODEL") ?? "gpt-4o-mini";
        var key = Environment.GetEnvironmentVariable("SPECFLOW_MODEL_KEY") ?? "not-needed";
        var client = new OpenAIClient(
            new ApiKeyCredential(key),
            new OpenAIClientOptions { Endpoint = new Uri(endpoint) });
        AIAgent agent = client.GetChatClient(model).AsAIAgent(new ChatClientAgentOptions
        {
            // Stable logical-role id: a checkpoint can only resume into a graph
            // whose executor identities match, and a random id per run makes
            // every checkpoint its own unresumable lineage.
            Id = "slice-builder",
            Name = "SliceBuilder",
            // The flow's own read surface, and nothing else. An agent holding
            // only these can act only through governed surfaces.
            ChatOptions = new() { Tools = [.. tools] },
        });
        return new AgentSliceBuilder(agent, options.Get("instructions")).BuildAsync;
    }

    /// <summary>
    /// The human-in-the-loop port, answered at a terminal.
    /// </summary>
    /// <remarks>
    /// The reviewer amends a draft here. They do not close the record here,
    /// and the prompt says so: the closure is a separate act, under their own
    /// identity, at the other binary.
    /// </remarks>
    private static ValueTask<DraftReview> ReviewAtTheTerminal(ClosureDraft draft, CancellationToken cancellationToken)
    {
        Console.WriteLine($"\nrecord {draft.RecordId}  slice `{draft.Slice}`");
        Console.WriteLine(draft.Notes);
        Console.WriteLine("\ndrafted determinations:");
        foreach (var determination in draft.DraftDeterminations)
        {
            Console.WriteLine($"  {determination}");
        }
        if (draft.DraftDeterminations.Count is 0)
        {
            Console.WriteLine("  (none)");
        }

        Console.Write("\namend (comma-separated addresses, blank to keep, `-` for none): ");
        var typed = Console.ReadLine();
        IReadOnlyList<string> determinations = typed switch
        {
            null or "" => draft.DraftDeterminations,
            "-" => [],
            _ => typed.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries),
        };
        return new ValueTask<DraftReview>(new DraftReview(draft.RecordId, determinations));
    }

    private static void Report(ImplementOutcome outcome)
    {
        Console.WriteLine($"\nrecord {outcome.RecordId} is open. Closure is pending and is not this process's to do.");
        Console.WriteLine("Run this yourself, under your own identity:\n");
        Console.WriteLine($"  {outcome.HandOffCommand}\n");
    }
}
