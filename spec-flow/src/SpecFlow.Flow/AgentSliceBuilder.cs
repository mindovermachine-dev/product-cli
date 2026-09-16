using System.Text.Json;
using Microsoft.Agents.AI;

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
    public async ValueTask<SliceBuilt> BuildAsync(
        ActRecordOpened opened,
        CancellationToken cancellationToken = default)
    {
        var session = await _agent.CreateSessionAsync(cancellationToken).ConfigureAwait(false);
        var response = await _agent
            .RunAsync(Prompt(opened), session, cancellationToken: cancellationToken)
            .ConfigureAwait(false);

        var text = response.Text ?? string.Empty;
        return new SliceBuilt(opened.RecordId, opened.Slice, ParseDeterminations(text), text);
    }

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
