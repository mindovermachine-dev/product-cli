using System.Text;
using ProductCli.Testing;
using Xunit;

namespace ProductCli.Host.Tests;

/// <summary>
/// Every argument vector but `--version` exits 2, with one line on stderr that
/// says where its verb arrives. The lines are for a person, not a format; the
/// test holds them so a change to one is a reviewed change.
/// </summary>
public sealed class VerbTests
{
    /// <summary>Each verb of the definition's chapter 3 names the step of the order of work it arrives in.</summary>
    [Theory]
    [InlineData("init", 4)]
    [InlineData("product", 4)]
    [InlineData("domain", 4)]
    [InlineData("diff", 4)]
    [InlineData("proposal", 5)]
    [InlineData("view", 5)]
    [InlineData("mcp", 5)]
    [InlineData("decider", 6)]
    [InlineData("projector", 6)]
    [InlineData("primitive", 6)]
    [InlineData("scenario", 7)]
    [InlineData("conformance", 7)]
    [InlineData("question", 8)]
    public void AVerbNamesTheStepItArrivesIn(string verb, int step)
    {
        AssertRefused(
            HostProcess.Run(verb, "--product", "shop"),
            $"product: '{verb}' arrives in step {step} of the order of work (D1.25)\n");
    }

    /// <summary>An argument that is no verb is named as such, escaped to one printable line.</summary>
    [Fact]
    public void AnArgumentThatIsNoVerbIsNamed()
    {
        AssertRefused(HostProcess.Run("nodes\nlist"), "product: 'nodes\\u000alist' is not a verb\n");
    }

    /// <summary>No argument at all is refused, and the line says what this build has.</summary>
    [Fact]
    public void NoArgumentIsRefused()
    {
        AssertRefused(HostProcess.Run(), "product: no verb given; this build has only --version (step 0 of the order of work, D1.25)\n");
    }

    /// <summary>The version flag takes nothing after it.</summary>
    [Fact]
    public void VersionWithAnArgumentIsRefused()
    {
        AssertRefused(HostProcess.Run("--version", "--json"), "product: --version takes no arguments\n");
    }

    private static void AssertRefused(CommandResult run, string line)
    {
        Assert.Equal(2, run.ExitCode);
        Assert.Empty(run.Stdout);
        Assert.Equal(Encoding.ASCII.GetBytes(line), run.Stderr);
    }
}
