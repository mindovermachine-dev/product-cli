using System;
using System.Collections.Generic;
using System.IO;
using ProductCli.Testing;

namespace ProductCli.Host.Tests;

/// <summary>Runs the built host, `product`, as a process, and keeps what it wrote (ADR-0006).</summary>
internal static class HostProcess
{
    private static readonly string Assembly = Path.Combine(AppContext.BaseDirectory, "ProductCli.Host.dll");

    internal static CommandResult Run(params string[] arguments)
    {
        List<string> command = ["exec", Assembly];
        command.AddRange(arguments);
        return Command.Run("dotnet", command);
    }
}
