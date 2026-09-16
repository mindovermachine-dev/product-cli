using System.Text.Json;
using Eval;
using Microsoft.Agents.AI;
using Microsoft.Extensions.AI;

namespace SpecFlow.Flow;

/// <summary>
/// Builds a slice by asking an <see cref="AIAgent"/>, and drafts what arose.
/// </summary>
/// <remarks>
/// <para>
/// This is the delegable half in one class. Everything it produces is a
/// draft: the determinations come back as proposals a reviewer amends and a
/// principal later files. The agent is never told it is closing anything,
/// because it never is.
/// </para>
/// <para>
/// The agent is injected rather than constructed here. Which model, which
/// endpoint, which instructions are arrangement parameters — the thing under
/// test when a small model is asked to hold the same flow a large one does —
/// and baking one in would turn the experiment into a configuration.
/// </para>
/// </remarks>
public sealed class AgentSliceBuilder(AIAgent agent, string? extraInstructions = null)
{
    private readonly AIAgent _agent = agent;
    private readonly string? _extraInstructions = extraInstructions;

    /// <summary>The build step, shaped for the workflow's executor.</summary>
    /// <remarks>
    /// Two turns in one session, and the order is the point. The worker says
    /// what it is about to resolve and what it needs before it can see what it
    /// produced; only then does it act. A declaration asked for afterwards is a
    /// summary, and a summary cannot be contradicted by the run it summarises.
    /// </remarks>
    public async ValueTask<SliceBuilt> BuildAsync(
        ActRecordOpened opened,
        CancellationToken cancellationToken = default)
    {
        var session = await _agent.CreateSessionAsync(cancellationToken).ConfigureAwait(false);

        var declaring = await _agent
            .RunAsync(DeclarationPrompt(opened), session, cancellationToken: cancellationToken)
            .ConfigureAwait(false);
        var declared = ReadDeclaration(declaring.Text ?? string.Empty);

        var response = await _agent
            .RunAsync(Prompt(opened), session, cancellationToken: cancellationToken)
            .ConfigureAwait(false);

        var text = response.Text ?? string.Empty;
        return new SliceBuilt(
            opened.RecordId,
            opened.Slice,
            ParseDeterminations(text),
            text,
            declared,
            [.. GroundRead(declaring).Concat(GroundRead(response)).Distinct(StringComparer.Ordinal)]);
    }

    /// <summary>
    /// What the worker is asked before it is allowed to act.
    /// </summary>
    /// <remarks>
    /// It is asked for the decision and nothing else. A worker asked to name its
    /// own ground answers from introspection — and answers in whatever
    /// vocabulary it likes, so the comparison against what it actually read
    /// finds differences that were only ever differences in naming. The ground
    /// is what the tools were used at, and that is observed rather than asked
    /// for.
    /// </remarks>
    private string DeclarationPrompt(ActRecordOpened opened) => $$"""
        Before you do anything: say in one sentence what you are about to
        resolve.

        You are about to build the slice `{{opened.Slice}}` against `{{opened.ActRef}}`.

        Do not list the tools you will use or the things you will read. What you
        consult is observed as you work; it is not yours to describe.

        Reply with one line of JSON and nothing else:
        {"decision": "<one sentence>"}
        """;

    /// <summary>
    /// Read the declaration, or record that none was made.
    /// </summary>
    /// <remarks>
    /// A worker that would not declare has not declared — the run is filed
    /// without one and the check says so. Inventing a declaration on its behalf
    /// would be manufacturing the very thing the check exists to look for.
    /// </remarks>
    public static Declaration? ReadDeclaration(string reply)
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
                if (!root.TryGetProperty("decision", out var decision))
                {
                    continue;
                }
                // Ground is not read from the reply even if the worker offered
                // some: it is the ground author's to declare, and accepting the
                // worker's version would let that decision escape into the
                // thing it is supposed to constrain.
                return new Declaration(decision.GetString() ?? "", []);
            }
            catch (JsonException)
            {
                // Not the object we were looking for; keep scanning upward.
            }
        }
        return null;
    }

    /// <summary>
    /// The ground the worker actually read: the addresses its tools were used at.
    /// </summary>
    /// <remarks>
    /// <para>
    /// A tool used at an address reads the ground at that address — so the
    /// address is what is recorded, not the tool's name. `spec_acts` called for
    /// `act/settle-a-basket` read `act/settle-a-basket`, and that is a name the
    /// ground author also uses, which is what makes the comparison mean
    /// anything.
    /// </para>
    /// <para>
    /// A call carrying no address reads no ground, and records none. Listing
    /// every act is not reading the act this build rests on: it touches the
    /// channel without naming what in it mattered, and the run then shows the
    /// declared ground as unread — which is what happened.
    /// </para>
    /// <para>
    /// Observed, never asked for. A worker's account of what it consulted is its
    /// own introspection; the call is a fact about the arrangement.
    /// </para>
    /// </remarks>
    public static IReadOnlyList<string> GroundRead(AgentResponse response) =>
    [
        .. response.Messages
            .SelectMany(m => m.Contents)
            .OfType<FunctionCallContent>()
            .SelectMany(AddressesOf)
            .Distinct(StringComparer.Ordinal),
    ];

    /// <summary>The addresses one call was made at. None, where it named none.</summary>
    private static IEnumerable<string> AddressesOf(FunctionCallContent call) =>
        (call.Arguments ?? new Dictionary<string, object?>())
            .Values
            .OfType<string>()
            .Where(v => !string.IsNullOrWhiteSpace(v));

    private string Prompt(ActRecordOpened opened) => $"""
        Build the slice `{opened.Slice}` against the specification act `{opened.ActRef}`.

        While acting, note any determination you had to make that the specification
        did not already settle — a choice a reader of the spec could not have
        predicted from it.

        You are not closing anything. Your determinations are drafts: a person
        reviews them and files them under their own name. Do not claim to have
        filed, accepted, or closed anything.

        End your reply with a JSON array of determination addresses on its own
        line, for example:
        ["det/basket-rounding-is-half-even"]
        An empty array is a legitimate answer and means nothing arose.
        {_extraInstructions}
        """;

    /// <summary>
    /// Pull the trailing JSON array off the reply.
    /// </summary>
    /// <remarks>
    /// <para>
    /// The prompt asks for the array on one line, and models pretty-print it
    /// anyway. Taking the instruction literally cost two real determinations on
    /// a live run — the builder drafted them, the reviewer never saw them, and
    /// the run was journalled as having drafted none. So the scan brackets the
    /// last array in the reply rather than reading lines.
    /// </para>
    /// <para>
    /// A reply with no parsable array yields an empty draft rather than an
    /// error: an unreadable draft must not be able to stop the record from
    /// reaching its reviewer. The reviewer sees the raw notes either way.
    /// </para>
    /// </remarks>
    public static IReadOnlyList<string> ParseDeterminations(string reply)
    {
        var close = reply.LastIndexOf(']');
        while (close >= 0)
        {
            var open = MatchingOpen(reply, close);
            if (open < 0)
            {
                break;
            }
            try
            {
                if (JsonSerializer.Deserialize<string[]>(reply[open..(close + 1)]) is { } parsed)
                {
                    return parsed;
                }
            }
            catch (JsonException)
            {
                // Not the array we were looking for; try the one before it.
            }

            // Before this array, never inside it. Descending into one that
            // failed to parse would return a fragment of a draft as though it
            // were the whole of it, which is the loss this scan exists to stop.
            close = open > 0 ? reply.LastIndexOf(']', open - 1) : -1;
        }
        return [];
    }

    /// <summary>
    /// The `[` that opens the array closing at <paramref name="close"/>, or -1.
    /// </summary>
    /// <remarks>
    /// Bracket depth only. A `]` inside a string would throw the count off, but
    /// a determination address containing one is not an address, and the parse
    /// that follows is what actually decides.
    /// </remarks>
    private static int MatchingOpen(string reply, int close)
    {
        var depth = 0;
        for (var at = close; at >= 0; at--)
        {
            depth += reply[at] switch { ']' => 1, '[' => -1, _ => 0 };
            if (depth is 0)
            {
                return at;
            }
        }
        return -1;
    }
}
