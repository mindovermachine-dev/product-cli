namespace Eval;

/// <summary>What kind of escape was found.</summary>
public enum EscapeKind
{
    /// <summary>The run declared nothing, so nothing about it can be contradicted.</summary>
    Undeclared,

    /// <summary>A declaration is missing a field it must carry to be complete.</summary>
    IncompleteDeclaration,

    /// <summary>Ground was declared as needed and never read.</summary>
    DeclaredButUnread,

    /// <summary>Ground was read that the declaration never named.</summary>
    ReadButUndeclared,

    /// <summary>A claim rests on nothing the worker declared.</summary>
    UnattributedClaim,
}

/// <summary>One escape candidate.</summary>
/// <param name="Subject">What the finding is about: a field, a ground element, a claim.</param>
/// <param name="Message">Why it was raised, in terms a reader can act on.</param>
public sealed record EscapeFinding(EscapeKind Kind, string Subject, string Message)
{
    /// <summary>What this kind is called in a report.</summary>
    public string KindName => Kind switch
    {
        EscapeKind.Undeclared => "undeclared",
        EscapeKind.IncompleteDeclaration => "incomplete-declaration",
        EscapeKind.DeclaredButUnread => "declared-but-unread",
        EscapeKind.ReadButUndeclared => "read-but-undeclared",
        _ => "unattributed-claim",
    };
}

/// <summary>
/// Detecting the forbidden state: a resolution nobody can be shown to have held.
/// </summary>
/// <remarks>
/// <para>
/// Three checks, each closed even though the acceptance predicate is not.
/// Completeness asks whether the declaration says enough to be contradicted.
/// Ground reads ask whether what was consulted matches what was declared.
/// Attribution asks whether every claim rests on something declared.
/// </para>
/// <para>
/// <b>These are findings, not verdicts, and they do not gate.</b> Whether a
/// particular escape is tolerable is a judgement, and a judgement needs an
/// owner. What the checks give that owner is a list they did not have to
/// assemble by reading transcripts.
/// </para>
/// </remarks>
public static class Escape
{
    /// <summary>Every escape candidate in one run.</summary>
    public static IReadOnlyList<EscapeFinding> Check(RunRecord run)
    {
        if (run.Declared is not { } declared)
        {
            return
            [
                new EscapeFinding(EscapeKind.Undeclared, run.RecordId,
                    "the run declared nothing before acting, so nothing it did can be "
                  + "contradicted by what it said it would do"),
            ];
        }

        var found = declared.Missing()
            .Select(field => new EscapeFinding(
                EscapeKind.IncompleteDeclaration, field, $"the declaration carries no `{field}`"))
            .ToList();

        found.AddRange(GroundFindings(run, declared));
        found.AddRange(AttributionFindings(run, declared));
        return found;
    }

    /// <summary>Every escape candidate across runs, paired with the run it is in.</summary>
    public static IReadOnlyList<(string Run, EscapeFinding Finding)> CheckAll(
        IEnumerable<RunRecord> runs)
        => [.. runs.SelectMany(run => Check(run).Select(f => (run.RecordId, f)))];

    /// <summary>
    /// Declared against read, both directions.
    /// </summary>
    /// <remarks>
    /// <b>Only two of the three ground findings.</b> The third — a fast-ticking
    /// axis with no read at act time — needs a tick rate per ground element, and
    /// nothing in this format carries one. Reporting two and naming the third as
    /// absent is honest; reporting two as though they were all of them is not.
    /// </remarks>
    private static IEnumerable<EscapeFinding> GroundFindings(RunRecord run, Declaration declared)
    {
        foreach (var element in declared.Ground.Where(g => !run.GroundRead.Contains(g, StringComparer.Ordinal)))
        {
            yield return new EscapeFinding(EscapeKind.DeclaredButUnread, element,
                $"`{element}` was declared as needed and never read");
        }
        foreach (var element in run.GroundRead.Where(g => !declared.Ground.Contains(g, StringComparer.Ordinal)))
        {
            yield return new EscapeFinding(EscapeKind.ReadButUndeclared, element,
                $"`{element}` was read but the declaration does not name it");
        }
    }

    /// <summary>
    /// Claims resting on nothing declared.
    /// </summary>
    /// <remarks>
    /// A claim attributed to ground the declaration never named is <i>also</i> an
    /// escape, reported against that ground rather than as a second bad claim —
    /// the defect is the undeclared ground, and saying it twice would make one
    /// problem look like two.
    /// </remarks>
    private static IEnumerable<EscapeFinding> AttributionFindings(
        RunRecord run, Declaration declared)
    {
        foreach (var attribution in run.Attributions)
        {
            if (attribution.Ground is null)
            {
                yield return new EscapeFinding(EscapeKind.UnattributedClaim, attribution.Claim,
                    "the claim rests on nothing the worker declared");
            }
            else if (!declared.Ground.Contains(attribution.Ground, StringComparer.Ordinal))
            {
                yield return new EscapeFinding(EscapeKind.ReadButUndeclared, attribution.Ground,
                    $"a claim rests on `{attribution.Ground}`, which the declaration does not name");
            }
        }
    }
}
