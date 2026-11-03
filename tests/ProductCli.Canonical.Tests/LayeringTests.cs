using System;
using ProductCli.Testing;
using Xunit;

namespace ProductCli.Canonical.Tests;

/// <summary>
/// ADR-0002 D3: the layering analyzer refuses a reference from layer 1 to the
/// layer above it. The test reads the refusal in the compiler's output.
/// </summary>
public sealed class LayeringTests
{
    /// <summary>A probe at layer 1 referencing ProductCli.Shapes, at layer 2, fails with DD0001.</summary>
    [Fact]
    public void AReferenceFromLayer1ToTheLayerAboveIsRefused()
    {
        CommandResult build = Probe.Layering(layer: 1, references: "ProductCli.Shapes");

        Assert.NotEqual(0, build.ExitCode);
        Assert.Contains(
            "error DD0001: 'ProductCli.Probe' is at layer 1 and references 'ProductCli.Shapes', which is at layer 2.",
            build.Output,
            StringComparison.Ordinal);
    }
}
