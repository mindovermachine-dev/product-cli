namespace SpecFlow.Eval;

/// <summary>Where judgments are kept, and how they are filed.</summary>
/// <remarks>
/// <para>
/// One directory per run, because a run is judged more than once: by a second
/// model, by the same model a month later, by a bigger one when the question
/// turns out to matter. Keeping them side by side is the point — a single
/// verdict per run would be a verdict nobody could argue with.
/// </para>
/// <para>
/// Named by the context digest and the judge, so re-running the same judge
/// over the same inputs overwrites nothing and adds nothing: a genuinely new
/// occasion differs in at least one of them, and an accidental re-run does not
/// fill the directory with copies.
/// </para>
/// </remarks>
public static class JudgementStore
{
    /// <summary>The store's directory, relative to the repo root.</summary>
    public const string Directory = ".spec/judgements";

    /// <summary>Where one run's judgments live.</summary>
    public static string DirectoryFor(string root, string recordId) =>
        Path.Combine(root, Directory, recordId);

    /// <summary>Where one judgment lands.</summary>
    public static string PathFor(string root, Judgement judgement) =>
        Path.Combine(
            DirectoryFor(root, judgement.JudgesRecord),
            $"{Slug(judgement.Judge.Model)}.{ShortDigest(judgement.Context.Digest)}.json");

    /// <summary>File one judgment, returning where it landed.</summary>
    public static string Write(string root, Judgement judgement)
    {
        var path = PathFor(root, judgement);
        System.IO.Directory.CreateDirectory(Path.GetDirectoryName(path) ?? ".");
        File.WriteAllText(path, judgement.ToJson());
        return path;
    }

    /// <summary>
    /// Every judgment of one run, oldest first.
    /// </summary>
    /// <remarks>
    /// An unreadable file is skipped rather than thrown on. These are opinions
    /// kept for later reading, and one corrupt entry must not hide the rest.
    /// </remarks>
    public static IReadOnlyList<Judgement> Read(string root, string recordId)
    {
        var directory = DirectoryFor(root, recordId);
        if (!System.IO.Directory.Exists(directory))
        {
            return [];
        }

        var found = new List<Judgement>();
        foreach (var file in System.IO.Directory.EnumerateFiles(directory, "*.json"))
        {
            try
            {
                if (Judgement.FromJson(File.ReadAllText(file)) is { } judgement)
                {
                    found.Add(judgement);
                }
            }
            catch (Exception e) when (e is System.Text.Json.JsonException or IOException)
            {
                // Skipped: one unreadable opinion must not hide the others.
            }
        }
        return [.. found.OrderBy(j => j.JudgedAt)];
    }

    /// <summary>The digest's first bytes, enough to tell two contexts apart.</summary>
    internal static string ShortDigest(string digest)
    {
        var body = digest.StartsWith("sha256:", StringComparison.Ordinal) ? digest[7..] : digest;
        return body.Length <= 12 ? body : body[..12];
    }

    /// <summary>A model name reduced to something a file system accepts.</summary>
    internal static string Slug(string model) =>
        new(model.Select(c => char.IsLetterOrDigit(c) ? char.ToLowerInvariant(c) : '-').ToArray());
}
