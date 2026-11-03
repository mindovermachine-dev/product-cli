using System;
using ProductCli.Testing;
using Xunit;

namespace ProductCli.Canonical.Tests;

/// <summary>ADR-0002 D2: a reference to a layer below compiles, so the layering tests refuse only what they should.</summary>
public sealed class DownwardReferenceTests
{
    /// <summary>A probe at layer 1 referencing ProductCli.Vocabulary, at layer 0, builds with no DD0001.</summary>
    [Fact]
    public void AReferenceFromLayer1ToLayer0Compiles()
    {
        CommandResult build = Probe.Layering(layer: 1, references: "ProductCli.Vocabulary");

        Assert.True(build.ExitCode == 0, build.Output);
        Assert.DoesNotContain("DD0001", build.Output, StringComparison.Ordinal);
    }
}
