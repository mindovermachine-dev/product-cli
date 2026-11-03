using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Text;
using System.Threading.Tasks;

namespace ProductCli.Testing;

/// <summary>What a process wrote and how it exited: the bytes, not an interpretation of them (ADR-0006).</summary>
internal sealed record CommandResult(int ExitCode, byte[] Stdout, byte[] Stderr)
{
    /// <summary>Stdout and stderr, decoded as UTF-8, for assertions on a compiler's messages.</summary>
    public string Output => Encoding.UTF8.GetString(Stdout) + Encoding.UTF8.GetString(Stderr);
}

/// <summary>Runs a process to completion and keeps its output bytes.</summary>
internal static class Command
{
    internal static CommandResult Run(string fileName, IEnumerable<string> arguments, string? workingDirectory = null)
    {
        ProcessStartInfo start = new(fileName)
        {
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            RedirectStandardInput = true,
            UseShellExecute = false,
            WorkingDirectory = workingDirectory ?? Repository.Root,
        };
        foreach (string argument in arguments)
        {
            start.ArgumentList.Add(argument);
        }

        // The test runs under `dotnet test`, which sets MSBuild's own variables
        // for its child processes. A probe build is a fresh build: none of
        // them may reach it.
        foreach (DictionaryEntry variable in Environment.GetEnvironmentVariables())
        {
            string name = (string)variable.Key;
            if (name.StartsWith("MSBUILD", StringComparison.OrdinalIgnoreCase)
                || name.StartsWith("MSBuild", StringComparison.Ordinal)
                || name.Equals("DOTNET_HOST_PATH", StringComparison.OrdinalIgnoreCase))
            {
                start.Environment.Remove(name);
            }
        }

        using Process process = Process.Start(start)
            ?? throw new InvalidOperationException($"{fileName} did not start.");
        process.StandardInput.Close();

        using MemoryStream stdout = new();
        using MemoryStream stderr = new();
        Task copyOut = process.StandardOutput.BaseStream.CopyToAsync(stdout);
        Task copyErr = process.StandardError.BaseStream.CopyToAsync(stderr);
        process.WaitForExit();
        Task.WaitAll(copyOut, copyErr);

        return new CommandResult(process.ExitCode, stdout.ToArray(), stderr.ToArray());
    }
}
