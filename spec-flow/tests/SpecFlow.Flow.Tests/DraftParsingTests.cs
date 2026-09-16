namespace SpecFlow.Flow.Tests;

/// <summary>Reading a builder's draft out of whatever it actually replied.</summary>
/// <remarks>
/// The prompt asks for one line and models pretty-print. Every shape here was
/// either observed on a live run or is the shape the prompt asked for.
/// </remarks>
public class DraftParsingTests
{
    [Fact]
    public void The_shape_the_prompt_asks_for_parses()
    {
        Assert.Equal(
            ["det/basket-rounding-is-half-even"],
            AgentSliceBuilder.ParseDeterminations(
                "Built it.\n[\"det/basket-rounding-is-half-even\"]"));
    }

    /// <summary>The shape that cost two determinations on a live run.</summary>
    [Fact]
    public void A_pretty_printed_array_parses()
    {
        var reply = """
            I built the slice. Two choices were not settled by the spec.

            [
              "det/basket-item-totals-mapping",
              "det/unmapped-entry-points"
            ]
            """;

        Assert.Equal(
            ["det/basket-item-totals-mapping", "det/unmapped-entry-points"],
            AgentSliceBuilder.ParseDeterminations(reply));
    }

    [Fact]
    public void An_empty_array_means_nothing_arose()
    {
        Assert.Empty(AgentSliceBuilder.ParseDeterminations("Nothing arose.\n[]"));
    }

    /// <summary>The last array wins: prose may quote an example first.</summary>
    [Fact]
    public void The_trailing_array_is_the_draft()
    {
        var reply = """
            For example ["det/an-illustration"] would be the form.
            My answer:
            ["det/the-real-one"]
            """;

        Assert.Equal(["det/the-real-one"], AgentSliceBuilder.ParseDeterminations(reply));
    }

    /// <summary>
    /// An unreadable draft must not stop the record reaching its reviewer.
    /// </summary>
    [Fact]
    public void A_reply_with_no_array_drafts_nothing()
    {
        Assert.Empty(AgentSliceBuilder.ParseDeterminations("I would rather describe it in prose."));
    }

    [Fact]
    public void An_array_of_the_wrong_type_drafts_nothing()
    {
        Assert.Empty(AgentSliceBuilder.ParseDeterminations("Here: [1, 2, 3]"));
    }

    /// <summary>
    /// A nested array is refused, never reduced to the fragment inside it.
    /// </summary>
    /// <remarks>
    /// The scan walks to the array before a failed one, not into it. Returning
    /// `["det/b"]` here would drop `det/a` silently — the same loss the
    /// bracketed scan was written to stop.
    /// </remarks>
    [Fact]
    public void A_nested_array_is_refused_rather_than_reduced()
    {
        Assert.Empty(AgentSliceBuilder.ParseDeterminations("""[["det/a"], ["det/b"]]"""));
    }
}
