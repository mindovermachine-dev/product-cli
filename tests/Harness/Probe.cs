using System;
using System.Collections.Generic;
using System.IO;

namespace ProductCli.Testing;

/// <summary>
/// Builds a probe under tests/fixtures/ with `dotnet build`, under the
/// repository's real build files (Directory.Build.*, .globalconfig, the
/// analyzers' build props), and returns what the compiler wrote.
/// </summary>
/// <remarks>
/// The probe builds into its own temporary directory, against the outputs the
/// solution build already made (`--no-dependencies`, `RestoreRecursive=false`),
/// so probes run in parallel and write nothing under src/.
/// </remarks>
internal static class Probe
{
    /// <summary>The layering probe, declaring <paramref name="layer"/> (none if null) and referencing the <c>src/</c> project <paramref name="references"/> (none if null).</summary>
    internal static CommandResult Layering(int? layer, string? references)
    {
        Dictionary<string, string> properties = [];
        if (layer is int value)
        {
            properties["ProbeLayer"] = value.ToString(System.Globalization.CultureInfo.InvariantCulture);
        }

        if (references is not null)
        {
            string project = Repository.PathOf($"src/{references}/{references}.csproj");
            string assembly = Repository.PathOf($"src/{references}/bin/{Repository.Configuration}/net10.0/{references}.dll");
            if (!File.Exists(assembly))
            {
                throw new InvalidOperationException($"{assembly} is not built. Build the solution first: the probe builds against its outputs.");
            }

            properties["ProbeReference"] = project;
        }

        return Build("tests/fixtures/layering/ProductCli.Probe.csproj", properties);
    }

    private static CommandResult Build(string probe, Dictionary<string, string> properties)
    {
        string artifacts = Path.Combine(Path.GetTempPath(), "productcli-probe-" + Guid.NewGuid().ToString("N"));
        try
        {
            List<string> arguments =
            [
                "build", Repository.PathOf(probe),
                "-c", Repository.Configuration,
                "--no-dependencies",
                "-nologo",
                "-p:RestoreRecursive=false",
                $"-p:ProbeArtifacts={artifacts}",
            ];
            foreach (KeyValuePair<string, string> property in properties)
            {
                arguments.Add($"-p:{property.Key}={property.Value}");
            }

            return Command.Run("dotnet", arguments);
        }
        finally
        {
            if (Directory.Exists(artifacts))
            {
                Directory.Delete(artifacts, recursive: true);
            }
        }
    }
}
