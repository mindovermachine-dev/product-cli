using SpecFlow.Flow;

namespace SpecFlow.Cli;

/// <summary>What an act settles, read back through the Rust binary.</summary>
/// <remarks>
/// <para>
/// The declared ground of a build: the question the specification answers, as
/// the store reports it. Proxied rather than parsed here — one reading of what
/// the store means, or a judge and a CI run can be told different things about
/// the same act.
/// </para>
/// <para>
/// It is read by two verbs for two reasons. `judge` shows it to the judge,
/// without which the question is unanswerable. `build` puts it in the run's
/// address, so that editing what an act settles changes the coordinates and
/// older runs stop being compared to newer ones — which is correct, because
/// after such an edit they were not asked the same question.
/// </para>
/// </remarks>
internal static class ActGround
{
    /// <summary>An act's name, what it settles, and the ground it rests on.</summary>
    /// <param name="Addresses">
    /// The act's own address plus the entry points it is realised at — the
    /// addresses whose value can change how a build of it resolves.
    ///
    /// <b>This is the ground author's declaration, and the author is the
    /// principal who ratified the act.</b> Not the worker's: a worker naming its
    /// own ground is answering from introspection. Not the tool surface's
    /// either — offering a toolbox is not declaring that all of it is
    /// outcome-relevant.
    /// </param>
    internal sealed record Ground(string Name, string Settles, IReadOnlyList<string> Addresses);

    /// <summary>
    /// Read the ground for an act, or null when it could not be read.
    /// </summary>
    /// <remarks>
    /// A read that fails leaves the caller with less rather than stopping the
    /// run, and what was recorded says which: an address pinned without ground
    /// differs from one pinned with it, so the two never silently compare.
    /// </remarks>
    public static async Task<Ground?> ReadAsync(Options options, string actRef)
    {
        try
        {
            var read = await new SpecCli(options.SpecBinary, options.Root)
                .ReadJsonAsync(["acts", "--id", actRef])
                .ConfigureAwait(false);

            if (read.ValueKind is not System.Text.Json.JsonValueKind.Array)
            {
                return null;
            }
            foreach (var act in read.EnumerateArray())
            {
                var name = act.TryGetProperty("name", out var n) ? n.GetString() : null;
                var settles = act.TryGetProperty("settles", out var s) ? s.GetString() : null;
                if (name is null || settles is null)
                {
                    continue;
                }

                var addresses = new List<string> { actRef };
                if (act.TryGetProperty("realised_at", out var at)
                    && at.ValueKind is System.Text.Json.JsonValueKind.Array)
                {
                    addresses.AddRange(at.EnumerateArray()
                        .Select(e => e.GetString())
                        .OfType<string>()
                        .Where(v => v.Length > 0));
                }
                return new Ground(name, settles, addresses);
            }
        }
        catch (Exception e) when (e is InvalidOperationException or System.Text.Json.JsonException)
        {
            Console.Error.WriteLine($"the act's ground could not be read: {e.Message}");
        }
        return null;
    }
}
