using System.Text.Json;

namespace ProductCli.PolicyProbe.Plants;

/// <summary>
/// The planted failure of ADR-0003 D2: reflection-based serialisation in a
/// library. The AOT and trim analyzers must refuse it, as errors.
/// </summary>
internal static class ReflectionJson
{
    internal static string Serialise(object value) => JsonSerializer.Serialize(value);
}
