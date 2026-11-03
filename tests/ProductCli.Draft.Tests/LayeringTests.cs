using System;
using ProductCli.Testing;
using Xunit;

namespace ProductCli.Draft.Tests;

/// <summary>
/// ADR-0002 D3: the layering analyzer refuses a reference from layer 3 to the
/// layer above it. The test reads the refusal in the compiler's output.
/// </summary>
public sealed class LayeringTests
{
    /// <summary>A probe at layer 3 referencing ProductCli.Simulation, at layer 4, fails with DD0001.</summary>
    [Fact]
    public void AReferenceFromLayer3ToTheLayerAboveIsRefused()
    {
        CommandResult build = Probe.Layering(layer: 3, references: "ProductCli.Simulation");

        Assert.NotEqual(0, build.ExitCode);
        Assert.Contains(
            "error DD0001: 'ProductCli.Probe' is at layer 3 and references 'ProductCli.Simulation', which is at layer 4.",
            build.Output,
            StringComparison.Ordinal);
    }
}
