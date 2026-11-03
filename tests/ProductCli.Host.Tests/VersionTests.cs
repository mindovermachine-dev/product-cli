using System.Text;
using ProductCli.Testing;
using Xunit;

namespace ProductCli.Host.Tests;

/// <summary>`product --version`, the host's one behaviour in slice 0 (D1.25), read as bytes.</summary>
public sealed class VersionTests
{
    /// <summary>
    /// Stdout is exactly <c>product &lt;version&gt; (commit &lt;sha&gt;)</c> and a
    /// line feed; stderr is empty; the exit code is 0. The version is the
    /// build's, and the commit is the checkout's HEAD, read from git.
    /// </summary>
    [Fact]
    public void VersionWritesTheVersionAndTheSourceCommitAsOneLine()
    {
        CommandResult git = Command.Run("git", ["rev-parse", "HEAD"]);
        Assert.Equal(0, git.ExitCode);
        string commit = Encoding.ASCII.GetString(git.Stdout).Trim();

        CommandResult run = HostProcess.Run("--version");

        Assert.Equal(0, run.ExitCode);
        Assert.Equal(Encoding.ASCII.GetBytes($"product {Repository.Version} (commit {commit})\n"), run.Stdout);
        Assert.Empty(run.Stderr);
    }
}
