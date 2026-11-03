using System;
using System.IO;
using System.Security.Cryptography;
using System.Text.Json;
using ProductCli.Testing;
using Varve.Rdf;
using Varve.Turtle;
using Xunit;

namespace ProductCli.Canonical.Tests;

/// <summary>
/// The admission evidence for the Varve preview this repository references
/// (ADR-0004 D2): its Turtle parser and its RDFC-1.0 canonicaliser reproduce
/// decision-driven-design's published graph hash of acme-shop, byte for byte.
/// A test of the package, not of product behaviour; a failure is a Varve
/// issue first (ADR-0002 D5).
/// </summary>
public sealed class VarveAdmissionTests
{
    private static string Fixture(string name) => Path.Combine(Repository.PathOf("tests/fixtures/ddd"), name);

    /// <summary>The vendored files are the bytes SOURCE.md records.</summary>
    [Fact]
    public void TheVendoredFilesAreTheRecordedBytes()
    {
        string source = File.ReadAllText(Fixture("SOURCE.md"));

        Assert.Contains($"`{Sha256(File.ReadAllBytes(Fixture("acme-shop.ttl")))}`", source, StringComparison.Ordinal);
        Assert.Contains($"`{Sha256(File.ReadAllBytes(Fixture("vectors.json")))}`", source, StringComparison.Ordinal);
    }

    /// <summary>The sha256 of acme-shop's canonical N-Quads is vectors.json's graphHash.</summary>
    [Fact]
    public void TheCanonicalNQuadsOfAcmeShopHashToThePublishedGraphHash()
    {
        InMemoryDatasetBuilder builder = new();
        ParseResult parsed = TurtleParser.Parse(
            File.ReadAllBytes(Fixture("acme-shop.ttl")),
            (in QuadView quad) => builder.Add(
                quad.Subject.Materialise(),
                quad.Predicate.Materialise(),
                quad.Object.Materialise(),
                quad.HasGraph ? quad.Graph.Materialise() : null),
            new TurtleOptions { Syntax = RdfSyntax.Turtle });
        Assert.True(parsed.Succeeded, $"acme-shop.ttl does not parse: {parsed.FirstError}");

        CanonicalDataset canonical = RdfCanonicaliser.Canonicalise(builder.ToDataset(), cancellationToken: TestContext.Current.CancellationToken);

        using JsonDocument vectors = JsonDocument.Parse(File.ReadAllBytes(Fixture("vectors.json")));
        string? published = vectors.RootElement.GetProperty("graphHash").GetString();
        Assert.Equal("sha256:10cd90893f7fadb1bdf80064a43795be95e2175684c6d3a93a7736abc7c2156b", published);
        Assert.Equal(published, "sha256:" + Sha256(canonical.NQuads.Span));
    }

    private static string Sha256(ReadOnlySpan<byte> bytes) => Convert.ToHexStringLower(SHA256.HashData(bytes));
}
