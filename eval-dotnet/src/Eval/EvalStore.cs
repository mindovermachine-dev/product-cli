using System.Text.Json;

namespace Eval;

/// <summary>Where runs are kept, beside but never among what judged them.</summary>
/// <remarks>
/// The key layout is the format document's, spelled once here. Disk and object
/// storage see the same keys; only <see cref="IBlobs"/> differs between them,
/// which is what makes moving between them a swap rather than a migration.
/// </remarks>
public sealed class EvalStore(IBlobs blobs)
{
    /// <summary>The key prefix runs are kept under.</summary>
    public const string Runs = "runs";

    /// <summary>The key prefix judgments are kept under.</summary>
    public const string Judgements = "judgements";

    private readonly IBlobs _blobs = blobs;

    /// <summary>The backend beneath, for a caller that needs to say where it wrote.</summary>
    public IBlobs Blobs => _blobs;

    /// <summary>The key one run record is kept at.</summary>
    public static string RunKey(string id) => $"{Runs}/{id}.json";

    /// <summary>
    /// The key prefix one run's judgments are kept under.
    /// </summary>
    /// <remarks>
    /// A prefix per run, because a run is judged more than once: by a second
    /// model, by the same model later, by a bigger one when the question turns
    /// out to matter. A single verdict per run would be one nobody could argue
    /// with.
    /// </remarks>
    public static string JudgementsPrefix(string run) => $"{Judgements}/{run}";

    /// <summary>
    /// The key one judgment is kept at.
    /// </summary>
    /// <remarks>
    /// Named by the judge and the context digest, so re-asking the same judge
    /// the same question replaces in place rather than accreting copies, while
    /// a genuinely new occasion differs in one of them and lands beside the
    /// first.
    /// </remarks>
    public static string JudgementKey(Judgement judgement) =>
        $"{JudgementsPrefix(judgement.JudgesRecord)}/"
      + $"{Slug(judgement.Judge.Model)}.{ShortDigest(judgement.Context.Digest)}.json";

    /// <summary>File one run record, returning a locator.</summary>
    public string WriteRun(RunRecord record) => _blobs.Put(RunKey(record.RecordId), record.ToJson());

    /// <summary>File one judgment, returning a locator.</summary>
    public string WriteJudgement(Judgement judgement) =>
        _blobs.Put(JudgementKey(judgement), judgement.ToJson());

    /// <summary>Every run in the store, oldest first.</summary>
    /// <remarks>
    /// An unreadable entry is skipped rather than thrown on: a store is
    /// measurement, and one corrupt record must not hide the rest.
    /// </remarks>
    public IReadOnlyList<RunRecord> ReadRuns() =>
        [.. ReadUnder(Runs, RunRecord.FromJson).OrderBy(r => r.RanAt)];

    /// <summary>Every judgment of one run, oldest first.</summary>
    public IReadOnlyList<Judgement> ReadJudgements(string run) =>
        [.. ReadUnder(JudgementsPrefix(run), Judgement.FromJson).OrderBy(j => j.JudgedAt)];

    private IEnumerable<T> ReadUnder<T>(string prefix, Func<string, T?> parse) where T : class
    {
        foreach (var key in _blobs.List(prefix).Where(k => k.EndsWith(".json", StringComparison.Ordinal)))
        {
            T? parsed = null;
            try
            {
                if (_blobs.Get(key) is { } body)
                {
                    parsed = parse(body);
                }
            }
            catch (Exception e) when (e is JsonException or IOException)
            {
                // Skipped: one unreadable record must not hide the others.
            }
            if (parsed is not null)
            {
                yield return parsed;
            }
        }
    }

    /// <summary>The digest's first bytes, enough to tell two contexts apart.</summary>
    internal static string ShortDigest(string digest)
    {
        var body = digest.StartsWith("sha256:", StringComparison.Ordinal) ? digest[7..] : digest;
        return body.Length <= 12 ? body : body[..12];
    }

    /// <summary>A model name reduced to something a key accepts.</summary>
    internal static string Slug(string model) =>
        new([.. model.Select(c => char.IsAsciiLetterOrDigit(c) ? char.ToLowerInvariant(c) : '-')]);
}
