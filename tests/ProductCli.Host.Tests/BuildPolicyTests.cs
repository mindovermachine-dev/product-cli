using System;
using ProductCli.Testing;
using Xunit;

namespace ProductCli.Host.Tests;

/// <summary>
/// The build policy's failing cases, built on every run (ADR-0003 D2,
/// ADR-0013 D4). Each plant is a property of the probe under
/// tests/fixtures/build-policy/; the test reads the error in the build's
/// output. The probe stands in for a library under src/.
/// </summary>
public sealed class BuildPolicyTests
{
    /// <summary>With no plant, the probe builds: each failure below is the plant's, not the probe's.</summary>
    [Fact]
    public void TheProbeBuildsWithNoPlant()
    {
        CommandResult build = Probe.BuildPolicy(plant: null);

        Assert.True(build.ExitCode == 0, build.Output);
    }

    /// <summary>A register entry with no Adr= fails the build with PCLI0001.</summary>
    [Fact]
    public void AnEntryWithoutAdrFails()
    {
        AssertFails(
            Probe.BuildPolicy("no-adr"),
            "error PCLI0001: PackageVersion 'ProductCli.PlantedPackage' carries no Adr=");
    }

    /// <summary>A register entry citing an ADR that does not exist fails the build with PCLI0002.</summary>
    [Fact]
    public void AnEntryCitingNoAdrFileFails()
    {
        AssertFails(
            Probe.BuildPolicy("dangling-adr"),
            "error PCLI0002: PackageVersion 'ProductCli.PlantedPackage' cites Adr=\"9999\", and docs/decisions/ holds no ADR-9999-*.md");
    }

    /// <summary>A test-only package referenced from product code fails the build with PCLI0003.</summary>
    [Fact]
    public void ATestOnlyPackageInProductCodeFails()
    {
        AssertFails(
            Probe.BuildPolicy("out-of-scope"),
            "error PCLI0003: ProductCli.PolicyProbe (src/ProductCli.PolicyProbe/) references 'CsCheck', whose register entry is scoped to tests/");
    }

    /// <summary>Reflection-based System.Text.Json in a library fails the build with the trim and AOT analyzers' errors.</summary>
    [Fact]
    public void ReflectionBasedJsonInALibraryFails()
    {
        CommandResult build = Probe.BuildPolicy("reflection-json");

        AssertFails(build, "error IL2026: Using member 'System.Text.Json.JsonSerializer.Serialize<TValue>(TValue, JsonSerializerOptions)'");
        AssertFails(build, "error IL3050: Using member 'System.Text.Json.JsonSerializer.Serialize<TValue>(TValue, JsonSerializerOptions)'");
    }

    private static void AssertFails(CommandResult build, string error)
    {
        Assert.NotEqual(0, build.ExitCode);
        Assert.Contains(error, build.Output, StringComparison.Ordinal);
    }
}
