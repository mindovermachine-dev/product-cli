using System;
using ProductCli.Testing;
using Xunit;

namespace ProductCli.Simulation.Tests;

/// <summary>
/// ADR-0002 D3: the layering analyzer refuses a reference from layer 4 to the
/// layer above it. The test reads the refusal in the compiler's output.
/// </summary>
public sealed class LayeringTests
{
    /// <summary>A probe at layer 4 referencing ProductCli.Runner, at layer 5, fails with DD0001.</summary>
    [Fact]
    public void AReferenceFromLayer4ToTheLayerAboveIsRefused()
    {
        CommandResult build = Probe.Layering(layer: 4, references: "ProductCli.Runner");

        Assert.NotEqual(0, build.ExitCode);
        Assert.Contains(
            "error DD0001: 'ProductCli.Probe' is at layer 4 and references 'ProductCli.Runner', which is at layer 5.",
            build.Output,
            StringComparison.Ordinal);
    }
}
