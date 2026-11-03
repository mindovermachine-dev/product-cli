using System;
using ProductCli.Testing;
using Xunit;

namespace ProductCli.Host.Tests;

/// <summary>
/// ADR-0002 D3 at the top of the table. Nothing sits above layer 7, so its
/// refusal is a reference within its own layer. And no ProductCli project
/// may stay out of the table (dd_require_layer).
/// </summary>
public sealed class LayeringTests
{
    /// <summary>A probe at layer 7 referencing ProductCli.Host, also at layer 7, fails with DD0001.</summary>
    [Fact]
    public void AReferenceFromLayer7WithinItsLayerIsRefused()
    {
        CommandResult build = Probe.Layering(layer: 7, references: "ProductCli.Host");

        Assert.NotEqual(0, build.ExitCode);
        Assert.Contains(
            "error DD0001: 'ProductCli.Probe' is at layer 7 and references 'ProductCli.Host', which is at layer 7.",
            build.Output,
            StringComparison.Ordinal);
    }

    /// <summary>A ProductCli project that declares no layer fails with DD0001.</summary>
    [Fact]
    public void AnUnplacedProjectIsRefused()
    {
        CommandResult build = Probe.Layering(layer: null, references: null);

        Assert.NotEqual(0, build.ExitCode);
        Assert.Contains(
            "error DD0001: 'ProductCli.Probe' is in family 'ProductCli' and declares no layer",
            build.Output,
            StringComparison.Ordinal);
    }
}
