using System;
using System.Globalization;
using System.IO;
using System.Reflection;
using System.Text;

namespace ProductCli.Host;

/// <summary>
/// The entry point of `product`. In slice 0 (D1.25) its only behaviour is
/// `product --version`; every other argument vector exits 2 and names the step
/// of the order of work its verb arrives in.
/// </summary>
internal static class Program
{
    private const int Success = 0;
    private const int NotBuilt = 2;

    private static int Main(string[] args)
    {
        if (args.Length == 1 && args[0] == "--version")
        {
            Write(Console.OpenStandardOutput(), VersionLine());
            return Success;
        }

        Write(Console.OpenStandardError(), Refusal(args));
        return NotBuilt;
    }

    /// <summary>
    /// <c>product &lt;version&gt; (commit &lt;sha&gt;)</c> and a line feed, read
    /// from the metadata the build stamps into this assembly.
    /// </summary>
    private static string VersionLine()
    {
        Assembly assembly = typeof(Program).Assembly;
        string version = assembly.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion
            ?? throw new InvalidOperationException("The build stamped no informational version.");

        string? commit = null;
        foreach (AssemblyMetadataAttribute metadata in assembly.GetCustomAttributes<AssemblyMetadataAttribute>())
        {
            if (metadata.Key == "SourceCommit")
            {
                commit = metadata.Value;
            }
        }

        return string.IsNullOrEmpty(commit)
            ? throw new InvalidOperationException("The build stamped no source commit.")
            : "product " + version + " (commit " + commit + ")\n";
    }

    /// <summary>The one line, with its line feed, for an argument vector this build does not serve.</summary>
    private static string Refusal(string[] args)
    {
        if (args.Length == 0)
        {
            return "product: no verb given; this build has only --version (step 0 of the order of work, D1.25)\n";
        }

        string first = args[0];
        if (first == "--version")
        {
            return "product: --version takes no arguments\n";
        }

        int step = Verbs.StepOf(first);
        return step == 0
            ? "product: '" + Printable(first) + "' is not a verb\n"
            : "product: '" + first + "' arrives in step " + step.ToString(CultureInfo.InvariantCulture) + " of the order of work (D1.25)\n";
    }

    /// <summary>
    /// The argument as printable ASCII: every other character is written as
    /// <c>\uXXXX</c>, so the message stays one line whatever was typed.
    /// </summary>
    private static string Printable(string argument)
    {
        StringBuilder text = new(argument.Length);
        foreach (char c in argument)
        {
            if (c is >= ' ' and <= '~')
            {
                text.Append(c);
            }
            else
            {
                text.Append("\\u").Append(((int)c).ToString("x4", CultureInfo.InvariantCulture));
            }
        }

        return text.ToString();
    }

    /// <summary>Writes the text as ASCII bytes, so the bytes are the same on every platform and console.</summary>
    private static void Write(Stream stream, string text)
    {
        using (stream)
        {
            stream.Write(Encoding.ASCII.GetBytes(text));
            stream.Flush();
        }
    }
}
