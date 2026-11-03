using System;
using System.IO;
using System.Reflection;

namespace ProductCli.Testing;

/// <summary>Where the repository is, and what the build stamped into this test assembly.</summary>
internal static class Repository
{
    /// <summary>The repository root: the nearest directory above the test's output holding global.json and ProductCli.slnx.</summary>
    internal static string Root { get; } = FindRoot();

    /// <summary>The configuration this test assembly was built in, so a probe builds against the same outputs.</summary>
    internal static string Configuration => Stamped("ProductCliConfiguration");

    /// <summary>The product version of this build (Directory.Build.props).</summary>
    internal static string Version => Stamped("ProductCliVersion");

    /// <summary>A path below the root, given with forward slashes.</summary>
    internal static string PathOf(string relative) =>
        Path.Combine(Root, relative.Replace('/', Path.DirectorySeparatorChar));

    private static string Stamped(string key)
    {
        foreach (AssemblyMetadataAttribute metadata in typeof(Repository).Assembly.GetCustomAttributes<AssemblyMetadataAttribute>())
        {
            if (metadata.Key == key && !string.IsNullOrEmpty(metadata.Value))
            {
                return metadata.Value;
            }
        }

        throw new InvalidOperationException($"The build stamped no {key} into this test assembly (Directory.Build.targets).");
    }

    private static string FindRoot()
    {
        for (DirectoryInfo? directory = new(AppContext.BaseDirectory); directory is not null; directory = directory.Parent)
        {
            if (File.Exists(Path.Combine(directory.FullName, "global.json"))
                && File.Exists(Path.Combine(directory.FullName, "ProductCli.slnx")))
            {
                return directory.FullName;
            }
        }

        throw new InvalidOperationException($"No repository root above {AppContext.BaseDirectory}.");
    }
}
