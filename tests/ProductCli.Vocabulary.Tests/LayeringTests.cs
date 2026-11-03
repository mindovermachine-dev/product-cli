using System;
using ProductCli.Testing;
using Xunit;

namespace ProductCli.Vocabulary.Tests;

/// <summary>
/// ADR-0002 D3: the layering analyzer refuses a reference from layer 0 to the
/// layer above it. The test reads the refusal in the compiler's output.
/// </summary>
public sealed class LayeringTests
{
    /// <summary>A probe at layer 0 referencing ProductCli.Canonical, at layer 1, fails with DD0001.</summary>
    [Fact]
    public void AReferenceFromLayer0ToTheLayerAboveIsRefused()
    {
        CommandResult build = Probe.Layering(layer: 0, references: "ProductCli.Canonical");

        Assert.NotEqual(0, build.ExitCode);
        Assert.Contains(
            "error DD0001: 'ProductCli.Probe' is at layer 0 and references 'ProductCli.Canonical', which is at layer 1.",
            build.Output,
            StringComparison.Ordinal);
    }
}
