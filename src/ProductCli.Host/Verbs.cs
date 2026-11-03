namespace ProductCli.Host;

/// <summary>
/// Where each verb of the definition's chapter 3 arrives in the order of work
/// (D1.25). The message that uses this is not a format: it is for a person,
/// and it changes as verbs are built.
/// </summary>
internal static class Verbs
{
    /// <summary>The step a verb arrives in, or 0 for an argument that is not a verb.</summary>
    internal static int StepOf(string verb) => verb switch
    {
        "init" or "product" or "domain" or "diff" => 4,
        "proposal" or "view" or "mcp" => 5,
        "decider" or "projector" or "primitive" => 6,
        "scenario" or "conformance" => 7,
        "question" => 8,
        _ => 0,
    };
}
