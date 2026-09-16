using System.Text.Json.Serialization;

namespace Eval;

/// <summary>What a run said it was doing, before it did it.</summary>
/// <remarks>
/// <para>
/// A declaration is a closed object even where the acceptance predicate is
/// open: whether it is <i>complete</i> is mechanically checkable, whatever the
/// quality of the work that follows. Prediction before operation, applied to
/// the worker.
/// </para>
/// <para>
/// Made before the act or not at all. A declaration written afterwards
/// describes what happened, which is a summary and not a prediction — and a
/// summary cannot be contradicted by the run it summarises.
/// </para>
/// </remarks>
/// <param name="Decision">The decision being resolved, in the worker's own words.</param>
/// <param name="Ground">The ground it says it needs. Named elements, not prose.</param>
/// <param name="Tolerance">
/// The declared bound on outcome-relevant variation. The arrangement's, not the
/// worker's — a worker setting its own tolerance is deciding how wrong it is
/// allowed to be.
/// </param>
/// <param name="Assurance">
/// The assurance level the act is performed at. Also the arrangement's: it is
/// derived from the worst credible outcome by whoever bears that outcome, and a
/// worker declaring it would be pricing a consequence it does not carry.
/// </param>
public sealed record Declaration(
    [property: JsonPropertyName("decision")] string Decision,
    [property: JsonPropertyName("ground")] IReadOnlyList<string> Ground,
    [property: JsonPropertyName("tolerance")] string? Tolerance = null,
    [property: JsonPropertyName("assurance")] string? Assurance = null)
{
    /// <summary>The fields this declaration is missing to be complete.</summary>
    public IReadOnlyList<string> Missing()
    {
        var missing = new List<string>();
        if (string.IsNullOrWhiteSpace(Decision))
        {
            missing.Add("decision");
        }
        if (Ground.Count is 0)
        {
            missing.Add("ground");
        }
        if (string.IsNullOrWhiteSpace(Tolerance))
        {
            missing.Add("tolerance");
        }
        if (string.IsNullOrWhiteSpace(Assurance))
        {
            missing.Add("assurance");
        }
        return missing;
    }

    /// <summary>Add what the arrangement — not the worker — fixed.</summary>
    public Declaration BoundedBy(string tolerance, string assurance) =>
        this with { Tolerance = tolerance, Assurance = assurance };
}

/// <summary>A claim the act made, and the declared ground it rests on.</summary>
/// <remarks>
/// A claim with no ground is an escape candidate: the worker resolved something
/// on a basis it did not declare, or on none.
///
/// <b>An attribution is the worker's own account of itself</b> and can be
/// confabulated. It is worth checking because it is cheap and because its
/// falsifier is sharp — attribution passing on acts where behaviour shows
/// dependence on undeclared ground means the attribution is decorative.
/// </remarks>
public sealed record Attribution(
    [property: JsonPropertyName("claim")] string Claim,
    [property: JsonPropertyName("ground")] string? Ground = null);
