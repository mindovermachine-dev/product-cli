using System;
using System.Collections.Generic;
using System.IO;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using Xunit;

namespace ProductCli.Host.Tests;

/// <summary>
/// The layers the built assemblies declare are ADR-0002 D1's table. Read from
/// each assembly's metadata, where DD0001 reads them, not from a project file.
/// </summary>
public sealed class LayerTableTests
{
    /// <summary>ADR-0002 D1, from the bottom.</summary>
    public static TheoryData<string, int> Table => new()
    {
        { "ProductCli.Vocabulary", 0 },
        { "ProductCli.Canonical", 1 },
        { "ProductCli.Shapes", 2 },
        { "ProductCli.Draft", 3 },
        { "ProductCli.Simulation", 4 },
        { "ProductCli.Runner", 5 },
        { "ProductCli.Session", 6 },
        { "ProductCli.Host", 7 },
    };

    /// <summary>Each assembly's <c>[assembly: ArchLayer(n)]</c> is its row of the table.</summary>
    [Theory]
    [MemberData(nameof(Table))]
    public void TheAssemblyDeclaresItsLayer(string assembly, int layer)
    {
        Assert.Equal([layer], DeclaredLayers(Path.Combine(AppContext.BaseDirectory, assembly + ".dll")));
    }

    private static List<int> DeclaredLayers(string path)
    {
        using FileStream file = File.OpenRead(path);
        using PEReader pe = new(file);
        MetadataReader metadata = pe.GetMetadataReader();

        List<int> layers = [];
        foreach (CustomAttributeHandle handle in metadata.GetAssemblyDefinition().GetCustomAttributes())
        {
            CustomAttribute attribute = metadata.GetCustomAttribute(handle);
            if (attribute.Constructor.Kind != HandleKind.MethodDefinition)
            {
                continue;
            }

            TypeDefinition type = metadata.GetTypeDefinition(
                metadata.GetMethodDefinition((MethodDefinitionHandle)attribute.Constructor).GetDeclaringType());
            if (metadata.GetString(type.Namespace) == "DecisionDriven" && metadata.GetString(type.Name) == "ArchLayerAttribute")
            {
                // The value blob: the prolog 0x0001, then the one int32 argument.
                BlobReader value = metadata.GetBlobReader(attribute.Value);
                Assert.Equal(1, value.ReadUInt16());
                layers.Add(value.ReadInt32());
            }
        }

        return layers;
    }
}
