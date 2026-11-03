using System;
using System.Linq;
using CsCheck;
using ProductCli.Testing;
using Xunit;

namespace ProductCli.Host.Tests;

/// <summary>The host's refusal, as a property over argument vectors (CsCheck, ADR-0013 D1).</summary>
public sealed class ArgumentPropertyTests
{
    /// <summary>
    /// For any argument vector other than exactly <c>["--version"]</c>, the
    /// empty one included: exit 2, nothing on stdout, and exactly one line on
    /// stderr, beginning <c>product: </c>.
    /// </summary>
    [Fact]
    public void EveryArgumentVectorButVersionExitsTwoWithOneLine()
    {
        // Any character but NUL, which no operating system passes in an
        // argument, and the surrogates, which are not characters alone.
        Gen<string> anyText = Gen.Char['\u0001', '퟿'].Array[0, 12].Select(chars => new string(chars));
        Gen<string> nearMiss = Gen.OneOf("--version", "--VERSION", "-v", "--help", "domain", "init", "", " ");
        Gen<string[]> vectors = Gen.OneOf(anyText, nearMiss).Array[0, 4]
            .Where(arguments => !(arguments.Length == 1 && arguments[0] == "--version"));

        vectors.Sample(
            arguments =>
            {
                CommandResult run = HostProcess.Run(arguments);

                Assert.Equal(2, run.ExitCode);
                Assert.Empty(run.Stdout);
                Assert.True(run.Stderr.Length > 0 && run.Stderr[^1] == (byte)'\n', "stderr does not end with a line feed");
                Assert.Equal(1, run.Stderr.Count(b => b == (byte)'\n'));
                Assert.True(run.Stderr.AsSpan().StartsWith("product: "u8), "stderr does not begin with 'product: '");
            },
            iter: 100,
            print: arguments => "[" + string.Join(", ", arguments.Select(a => "\"" + a + "\"")) + "]");
    }
}
