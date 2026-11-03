using System;
using ProductCli.Testing;
using Xunit;

namespace ProductCli.Session.Tests;

/// <summary>
/// ADR-0002 D3: the layering analyzer refuses a reference from layer 6 to the
/// layer above it. The test reads the refusal in the compiler's output.
/// </summary>
public sealed class LayeringTests
{
    /// <summary>A probe at layer 6 referencing ProductCli.Host, at layer 7, fails with DD0001.</summary>
    [Fact]
    public void AReferenceFromLayer6ToTheLayerAboveIsRefused()
    {
        CommandResult build = Probe.Layering(layer: 6, references: "ProductCli.Host");

        Assert.NotEqual(0, build.ExitCode);
        Assert.Contains(
            "error DD0001: 'ProductCli.Probe' is at layer 6 and references 'ProductCli.Host', which is at layer 7.",
            build.Output,
            StringComparison.Ordinal);
    }
}
