namespace Eval.Tests;

/// <summary>A throwaway directory to put a store in.</summary>
/// <remarks>
/// These tests need somewhere to write and nothing else. Keeping the harness
/// this small is part of the point: a reader copying the library should not
/// have to copy a test fixture that knows about someone else's repo.
/// </remarks>
public sealed class TempStore : IDisposable
{
    /// <summary>The directory, deleted when the test finishes with it.</summary>
    public string Root { get; } =
        Directory.CreateTempSubdirectory("eval-tests-").FullName;

    /// <summary>A disk-backed store rooted there.</summary>
    public EvalStore Store => new(new DiskBlobs(Root));

    public void Dispose()
    {
        try
        {
            Directory.Delete(Root, recursive: true);
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException)
        {
            // A temp directory that outlives the run is not a failed test.
        }
    }
}
