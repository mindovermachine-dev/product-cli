using Microsoft.Extensions.AI.Evaluation;

namespace Eval;

/// <summary>
/// What the flow observed about a run, handed to evaluators that need more
/// than the reply text.
/// </summary>
/// <remarks>
/// The reviewer's amendment is the part no model can supply and no judge has
/// to guess: a person looked at a draft and said what they would file. Carrying
/// it here is what makes the useful metric a measurement rather than an opinion.
/// </remarks>
public sealed class RunContext(
    string slice,
    IReadOnlyList<string> drafted,
    IReadOnlyList<string> reviewed)
    : EvaluationContext(ContextName, Describe(slice, drafted, reviewed))
{
    /// <summary>The name evaluators look this context up by.</summary>
    public const string ContextName = "Specification flow run";

    /// <summary>The slice that was built.</summary>
    public string Slice { get; } = slice;

    /// <summary>What the builder proposed.</summary>
    public IReadOnlyList<string> Drafted { get; } = drafted;

    /// <summary>What the reviewer would file.</summary>
    public IReadOnlyList<string> Reviewed { get; } = reviewed;

    private static string Describe(
        string slice,
        IReadOnlyList<string> drafted,
        IReadOnlyList<string> reviewed)
        => $"slice `{slice}`; drafted [{string.Join(", ", drafted)}]; "
         + $"reviewed [{string.Join(", ", reviewed)}]";
}
