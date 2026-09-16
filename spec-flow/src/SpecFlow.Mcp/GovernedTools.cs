using Microsoft.Extensions.AI;
using ModelContextProtocol.Client;

namespace SpecFlow.Mcp;

/// <summary>
/// The only tools a slice-building agent gets: the flow's own read surface,
/// consumed over MCP.
/// </summary>
/// <remarks>
/// <para>
/// An agent holding only these can act only through governed surfaces, so
/// escape through un-governed tooling is structurally excluded rather than
/// discouraged. What is not excluded is the planner's own context handling,
/// and that is an honest limit rather than a gap to paper over.
/// </para>
/// <para>
/// The server is the Rust <c>spec-mcp</c> binary, which is the same gate CI
/// runs. An agent reading the store through a second implementation could be
/// told something CI disagrees with, and would then be right to be confused.
/// </para>
/// </remarks>
public static class GovernedTools
{
    /// <summary>
    /// Delegable tools the slice builder still does not receive.
    /// </summary>
    /// <remarks>
    /// <para>
    /// <c>spec_implement</c> opens an act-time record, and by the time a
    /// builder runs the workflow has already opened one — the record opens
    /// first precisely so a crashed run cannot skip it. Offering the tool here
    /// invites a second record for the same slice, which then fails
    /// <c>S001</c> until a person closes something they never asked for.
    /// </para>
    /// <para>
    /// Withheld for a different reason than <see cref="Server.Withheld"/>: not
    /// because it names a principal, but because it is already done. An
    /// external agent calling <c>spec-mcp</c> directly has no workflow above it
    /// and legitimately opens its own record, so the tool stays on the server.
    /// </para>
    /// </remarks>
    public static readonly IReadOnlySet<string> AlreadyPerformed =
        new HashSet<string>(StringComparer.Ordinal) { "spec_implement" };

    /// <summary>The MCP server's file name on this platform.</summary>
    public static string ServerName =>
        OperatingSystem.IsWindows() ? "spec-mcp.exe" : "spec-mcp";

    /// <summary>
    /// Connect to <c>spec-mcp</c> and return its tools, or nothing when the
    /// binary is absent.
    /// </summary>
    /// <remarks>
    /// Absence degrades rather than fails: a repo without the Rust gate built
    /// can still open records, and the record — not the agent's reading — is
    /// what the write-back leg depends on.
    /// </remarks>
    public static async Task<IReadOnlyList<AITool>> ConnectAsync(
        SpecFlowOptions options,
        string? serverBinary = null,
        CancellationToken cancellationToken = default)
    {
        serverBinary ??= Locate();
        try
        {
            var client = await McpClient.CreateAsync(
                new StdioClientTransport(new StdioClientTransportOptions
                {
                    Name = "spec-mcp",
                    Command = serverBinary,
                    Arguments = [options.Root],
                }),
                cancellationToken: cancellationToken).ConfigureAwait(false);

            var tools = await client.ListToolsAsync(cancellationToken: cancellationToken)
                .ConfigureAwait(false);
            return [.. tools.Where(IsPermitted).Cast<AITool>()];
        }
        catch (Exception e) when (e is IOException or InvalidOperationException
                                       or System.ComponentModel.Win32Exception)
        {
            return [];
        }
    }

    /// <summary>
    /// Where the MCP server is: an explicit path, then beside this assembly,
    /// then PATH.
    /// </summary>
    /// <remarks>
    /// A bare name trusted to PATH is how an agent silently ends up with no
    /// tools at all: absence degrades rather than fails, so nothing says so.
    /// One installer puts every binary in one directory, and that is the
    /// directory this assembly was published into.
    /// </remarks>
    public static string Locate() => Locate(Environment.GetEnvironmentVariable("SPEC_MCP_BIN"));

    /// <summary>
    /// The same lookup with the override supplied, so a test need not mutate
    /// the environment of a process running its other tests in parallel.
    /// </summary>
    public static string Locate(string? supplied)
    {
        if (!string.IsNullOrWhiteSpace(supplied))
        {
            return supplied;
        }

        var beside = Path.Combine(AppContext.BaseDirectory, ServerName);
        return File.Exists(beside) ? beside : ServerName;
    }

    /// <summary>
    /// Whether a tool the server offered may reach the agent.
    /// </summary>
    /// <remarks>
    /// The server already withholds every verb that names a principal. This
    /// filter assumes it might not: a surface that trusts what it is handed is
    /// a surface that inherits the other side's next mistake.
    /// </remarks>
    public static bool IsPermitted(McpClientTool tool) =>
        !Server.Withheld.Contains(tool.Name) && !AlreadyPerformed.Contains(tool.Name);
}
