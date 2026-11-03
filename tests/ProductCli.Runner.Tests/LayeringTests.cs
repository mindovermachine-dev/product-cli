using System;
using ProductCli.Testing;
using Xunit;

namespace ProductCli.Runner.Tests;

/// <summary>
/// ADR-0002 D3: the layering analyzer refuses a reference from layer 5 to the
/// layer above it. The test reads the refusal in the compiler's output.
/// </summary>
public sealed class LayeringTests
{
    /// <summary>A probe at layer 5 referencing ProductCli.Session, at layer 6, fails with DD0001.</summary>
    [Fact]
    public void AReferenceFromLayer5ToTheLayerAboveIsRefused()
    {
        CommandResult build = Probe.Layering(layer: 5, references: "ProductCli.Session");

        Assert.NotEqual(0, build.ExitCode);
        Assert.Contains(
            "error DD0001: 'ProductCli.Probe' is at layer 5 and references 'ProductCli.Session', which is at layer 6.",
            build.Output,
            StringComparison.Ordinal);
    }
}
