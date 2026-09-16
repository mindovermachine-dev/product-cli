namespace SpecFlow.Eval;

/// <summary>Somewhere keyed bodies can be put, got and listed.</summary>
/// <remarks>
/// <para>
/// Three methods, matching `eval-core`'s <c>Blobs</c> trait. Everything that
/// knows the format sits above this line, so a second backend implements
/// addressing rather than reimplementing records — and a store swapped by
/// configuration cannot quietly disagree with the one it replaced.
/// </para>
/// <para>
/// A key is a store-relative path with <c>/</c> separators: a file path on
/// disk, a blob name in object storage. The layout was chosen to be key-shaped
/// before there was anything but disk.
/// </para>
/// </remarks>
public interface IBlobs
{
    /// <summary>Store a body under a key, returning a locator a person can follow.</summary>
    string Put(string key, string body);

    /// <summary>Read a body back, or null when the key holds none.</summary>
    string? Get(string key);

    /// <summary>Every key under a prefix, in no guaranteed order.</summary>
    IReadOnlyList<string> List(string prefix);
}

/// <summary>Keyed bodies as files under a root directory.</summary>
public sealed class DiskBlobs(string root) : IBlobs
{
    /// <summary>The root this store writes under.</summary>
    public string Root { get; } = root;

    /// <inheritdoc />
    public string Put(string key, string body)
    {
        var path = PathOf(key);
        Directory.CreateDirectory(Path.GetDirectoryName(path) ?? ".");
        File.WriteAllText(path, body);
        return path;
    }

    /// <inheritdoc />
    public string? Get(string key)
    {
        var path = PathOf(key);
        return File.Exists(path) ? File.ReadAllText(path) : null;
    }

    /// <inheritdoc />
    public IReadOnlyList<string> List(string prefix)
    {
        var directory = PathOf(prefix);
        if (!Directory.Exists(directory))
        {
            return [];
        }
        return
        [
            .. Directory.EnumerateFiles(directory)
                .Select(f => $"{prefix.TrimEnd('/')}/{Path.GetFileName(f)}"),
        ];
    }

    /// <summary>
    /// A key as a path beneath the root.
    /// </summary>
    /// <remarks>
    /// A key that climbs out of the root is refused rather than normalised: a
    /// store that can be talked into writing elsewhere is not a store.
    /// </remarks>
    private string PathOf(string key)
    {
        var segments = key.Split('/');
        if (segments.Any(s => s is ".." or ""))
        {
            throw new ArgumentException($"`{key}` is not a key", nameof(key));
        }
        return Path.Combine([Root, .. segments]);
    }
}
