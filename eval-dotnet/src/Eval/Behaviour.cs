namespace Eval;

/// <summary>What was read at one address.</summary>
/// <param name="Address">The address these runs share.</param>
/// <param name="Runs">How many runs were seen there.</param>
/// <param name="Arrangements">How many distinct arrangements answered there.</param>
/// <param name="Agreement">
/// Mean pairwise agreement of what was proposed, within one arrangement. Null
/// when a single arrangement never answered twice — which is not a low score
/// but the absence of one, and the two must not be confused.
/// </param>
/// <param name="AcrossArrangements">
/// Mean pairwise agreement <i>between</i> arrangements at this address. Read it
/// with <paramref name="Agreement"/>: the gap is what changed when the
/// arrangement changed.
/// </param>
public sealed record Reading(
    string Address,
    int Runs,
    int Arrangements,
    double? Agreement,
    double? AcrossArrangements)
{
    /// <summary>
    /// How much a changed arrangement moved the answer, where both are known.
    /// </summary>
    /// <remarks>
    /// Positive means an arrangement agrees with itself more than with the
    /// other — behaviour moved when the worker did. This is drift in the
    /// arrangement unless an independent measure of the world moved the same
    /// way in the same window, and nothing here can tell you that.
    /// </remarks>
    public double? Drift =>
        Agreement is { } within && AcrossArrangements is { } between ? within - between : null;
}

/// <summary>Reading behaviour at a fixed address, across runs.</summary>
/// <remarks>
/// <para>
/// Behaviour is prior to verdicts and lives precisely where the acceptance
/// predicate is open: you cannot grade a single act, but you can say whether
/// the same question asked twice was answered the same way, and whether two
/// different questions were answered identically.
/// </para>
/// <para>
/// <b>None of this is a verdict on any single run.</b> A shift at a fixed
/// address means the ground moved or something was resolved that was not
/// declared — it never means the one act was wrong. Reading it the other way is
/// the mistake the whole separation exists to prevent.
/// </para>
/// </remarks>
public static class Behaviour
{
    /// <summary>The label for runs that declared no arrangement.</summary>
    public const string Undeclared = "(undeclared)";

    /// <summary>Group runs by the address they declared, dropping those with none.</summary>
    public static IReadOnlyDictionary<string, IReadOnlyList<RunRecord>> ByAddress(
        IEnumerable<RunRecord> runs)
        => runs
            .Where(r => r.Address is not null)
            .GroupBy(r => r.Address!.Digest, StringComparer.Ordinal)
            .ToDictionary(g => g.Key, g => (IReadOnlyList<RunRecord>)[.. g], StringComparer.Ordinal);

    /// <summary>Read every address a set of runs covers.</summary>
    public static IReadOnlyList<Reading> Read(IEnumerable<RunRecord> runs) =>
        [.. ByAddress(runs).OrderBy(p => p.Key, StringComparer.Ordinal)
            .Select(p => ReadOne(p.Key, p.Value))];

    private static Reading ReadOne(string address, IReadOnlyList<RunRecord> at)
    {
        var groups = at
            .GroupBy(r => r.Arrangement?.Digest ?? Undeclared, StringComparer.Ordinal)
            .Select(g => g.Select(Proposed).ToList())
            .ToList();

        var within = groups.SelectMany(Pairwise).ToList();

        var between = new List<double>();
        for (var i = 0; i < groups.Count; i++)
        {
            foreach (var other in groups.Skip(i + 1))
            {
                between.AddRange(from a in groups[i] from b in other select Similarity(a, b));
            }
        }

        return new Reading(address, at.Count, groups.Count, Mean(within), Mean(between));
    }

    /// <summary>
    /// Addresses whose runs are indistinguishable from another address's.
    /// </summary>
    /// <remarks>
    /// Declared-distinct ground must yield distinct behaviour. Collapse means
    /// the declared ground is not being read — the caller said two questions
    /// differed and the worker answered them identically, so either the
    /// declaration is decorative or the difference never reached the worker.
    /// Reported as pairs, never as a verdict on either address.
    /// </remarks>
    public static IReadOnlyList<(string, string)> Collapsed(IEnumerable<RunRecord> runs)
    {
        var answers = ByAddress(runs).ToDictionary(
            p => p.Key,
            p => p.Value.Select(Proposed).Distinct(SetComparer.Instance).ToList(),
            StringComparer.Ordinal);

        var addresses = answers.Keys.OrderBy(a => a, StringComparer.Ordinal).ToList();
        var found = new List<(string, string)>();
        for (var i = 0; i < addresses.Count; i++)
        {
            foreach (var other in addresses.Skip(i + 1))
            {
                var one = answers[addresses[i]];
                var two = answers[other];
                if (one.Count is 1 && two.Count is 1 && SetComparer.Instance.Equals(one[0], two[0]))
                {
                    found.Add((addresses[i], other));
                }
            }
        }
        return found;
    }

    private static HashSet<string> Proposed(RunRecord run) =>
        new(run.Proposed, StringComparer.Ordinal);

    /// <summary>
    /// Jaccard similarity: 1.0 identical, 0.0 disjoint, 1.0 for two empties.
    /// </summary>
    /// <remarks>
    /// Two runs that both proposed nothing agree. It is a real answer and the
    /// commonest one, so scoring it zero would make silence look like
    /// disagreement.
    /// </remarks>
    public static double Similarity(HashSet<string> a, HashSet<string> b)
    {
        if (a.Count is 0 && b.Count is 0)
        {
            return 1.0;
        }
        var union = new HashSet<string>(a, StringComparer.Ordinal);
        union.UnionWith(b);
        return union.Count is 0 ? 1.0 : (double)a.Intersect(b, StringComparer.Ordinal).Count() / union.Count;
    }

    private static IEnumerable<double> Pairwise(List<HashSet<string>> answers)
    {
        for (var i = 0; i < answers.Count; i++)
        {
            foreach (var other in answers.Skip(i + 1))
            {
                yield return Similarity(answers[i], other);
            }
        }
    }

    private static double? Mean(IReadOnlyList<double> scores) =>
        scores.Count is 0 ? null : scores.Average();

    private sealed class SetComparer : IEqualityComparer<HashSet<string>>
    {
        public static readonly SetComparer Instance = new();

        public bool Equals(HashSet<string>? x, HashSet<string>? y) =>
            x is not null && y is not null && x.SetEquals(y);

        public int GetHashCode(HashSet<string> obj) =>
            obj.Count is 0 ? 0 : obj.Order(StringComparer.Ordinal).Aggregate(
                17, (hash, item) => hash * 31 + StringComparer.Ordinal.GetHashCode(item));
    }
}
