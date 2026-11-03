using System;
using ProductCli.Testing;
using Xunit;

namespace ProductCli.Shapes.Tests;

/// <summary>
/// ADR-0002 D3: the layering analyzer refuses a reference from layer 2 to the
/// layer above it. The test reads the refusal in the compiler's output.
/// </summary>
public sealed class LayeringTests
{
    /// <summary>A probe at layer 2 referencing ProductCli.Draft, at layer 3, fails with DD0001.</summary>
    [Fact]
    public void AReferenceFromLayer2ToTheLayerAboveIsRefused()
    {
        CommandResult build = Probe.Layering(layer: 2, references: "ProductCli.Draft");

        Assert.NotEqual(0, build.ExitCode);
        Assert.Contains(
            "error DD0001: 'ProductCli.Probe' is at layer 2 and references 'ProductCli.Draft', which is at layer 3.",
            build.Output,
            StringComparison.Ordinal);
    }
}
