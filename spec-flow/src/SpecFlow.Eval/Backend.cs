namespace SpecFlow.Eval;

/// <summary>Where a tool's evaluation store lives, chosen by configuration.</summary>
/// <remarks>
/// The spelling is `eval-core`'s, so both runtimes read the same
/// <c>EVAL_STORE</c> value and a repo moves its tools together or not at all.
/// Moving from disk to object storage is an edit to configuration, never to a
/// call site.
/// </remarks>
public abstract record Backend
{
    /// <summary>The variable naming the backend.</summary>
    public const string Environment = "EVAL_STORE";

    /// <summary>Files under a directory in the working tree.</summary>
    public sealed record Disk(string Root) : Backend
    {
        /// <inheritdoc />
        public override IBlobs Open() => new DiskBlobs(Root);

        /// <inheritdoc />
        public override string Describe() => $"disk at {Root}";
    }

    /// <summary>
    /// Blobs in an Azure storage container.
    /// </summary>
    /// <remarks>
    /// Named before it is built, so adopting it is a configuration change and
    /// not a redesign. <see cref="Open"/> says plainly that it is not built
    /// rather than falling back to disk — a tool that silently writes somewhere
    /// other than where it was told is worse than one that stops.
    /// </remarks>
    public sealed record Azure(string Account, string Container, string Prefix) : Backend
    {
        /// <inheritdoc />
        public override IBlobs Open() => throw new InvalidOperationException(
            $"the azure backend is declared but not built — configured for {Account}/{Container}.\n  "
          + "The store's shape does not change with it: the same keys, the same records. "
          + $"Until it is built, set {Environment} to a path.");

        /// <inheritdoc />
        public override string Describe() =>
            Prefix.Length is 0 ? $"azure {Account}/{Container}" : $"azure {Account}/{Container}/{Prefix}";
    }

    /// <summary>Open the backend, or say why it could not be opened.</summary>
    public abstract IBlobs Open();

    /// <summary>What to call this backend in a log line.</summary>
    public abstract string Describe();

    /// <summary>Read the backend from the environment, falling back to a disk default.</summary>
    public static Backend FromEnvironment(string defaultRoot) =>
        System.Environment.GetEnvironmentVariable(Environment) is { Length: > 0 } raw
            ? Parse(raw)
            : new Disk(defaultRoot);

    /// <summary>
    /// Parse the spelling <c>EVAL_STORE</c> uses.
    /// </summary>
    /// <remarks>
    /// Either a path — taken as a disk root — or
    /// <c>azure:&lt;account&gt;/&lt;container&gt;[/&lt;prefix&gt;]</c>.
    /// A half-written azure spelling is refused rather than guessed at.
    /// </remarks>
    public static Backend Parse(string raw)
    {
        const string scheme = "azure:";
        if (!raw.StartsWith(scheme, StringComparison.Ordinal))
        {
            return new Disk(raw);
        }

        var parts = raw[scheme.Length..].Split('/', 3);
        if (parts.Length < 2 || parts[0].Length is 0 || parts[1].Length is 0)
        {
            throw new ArgumentException(
                $"`{raw}` is not a store; use a path, or azure:<account>/<container>[/<prefix>]",
                nameof(raw));
        }
        return new Azure(parts[0], parts[1], parts.Length > 2 ? parts[2] : string.Empty);
    }
}
