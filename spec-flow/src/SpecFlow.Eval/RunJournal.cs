namespace SpecFlow.Eval;

/// <summary>Where run records are kept, and how they are written.</summary>
/// <remarks>
/// Beside the store rather than inside it: <c>.spec/runs/</c> is not
/// <c>.spec/records/</c>, and the Rust gate reads only the latter. Keeping
/// measurement in the same directory as act-time truth is how measurement
/// eventually gets mistaken for it.
/// </remarks>
public static class RunJournal
{
    /// <summary>The journal's directory, relative to the repo root.</summary>
    public const string Directory = ".spec/runs";

    /// <summary>Where one record lands.</summary>
    /// <remarks>
    /// Named by the act-time record it observed, so the two join without an
    /// index and a run cannot be filed twice for the same record.
    /// </remarks>
    public static string PathFor(string root, string recordId) =>
        Path.Combine(root, Directory, $"{recordId}.json");

    /// <summary>
    /// Write one record, returning where it landed.
    /// </summary>
    public static string Write(string root, RunRecord record)
    {
        var path = PathFor(root, record.RecordId);
        System.IO.Directory.CreateDirectory(Path.GetDirectoryName(path) ?? ".");
        File.WriteAllText(path, record.ToJson());
        return path;
    }

    /// <summary>
    /// Every run record in the journal, oldest first.
    /// </summary>
    /// <remarks>
    /// An unreadable file is skipped rather than thrown on: the journal is
    /// measurement, and one corrupt entry must not stop the rest being read.
    /// </remarks>
    public static IReadOnlyList<RunRecord> Read(string root)
    {
        var directory = Path.Combine(root, Directory);
        if (!System.IO.Directory.Exists(directory))
        {
            return [];
        }

        var records = new List<RunRecord>();
        foreach (var file in System.IO.Directory.EnumerateFiles(directory, "*.json").Order(StringComparer.Ordinal))
        {
            try
            {
                if (RunRecord.FromJson(File.ReadAllText(file)) is { } record)
                {
                    records.Add(record);
                }
            }
            catch (Exception e) when (e is System.Text.Json.JsonException or IOException)
            {
                // Skipped: one unreadable entry must not hide the others.
            }
        }
        return [.. records.OrderBy(r => r.RanAt)];
    }
}
