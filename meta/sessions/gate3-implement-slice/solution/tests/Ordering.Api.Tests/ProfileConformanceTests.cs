namespace Ordering.Api.Tests;

using System.Reflection;
using System.Runtime.CompilerServices;
using Ordering.Api.Profile;
using Ordering.Api.Slices.PlaceOrder;
using Ordering.Api.Unroled;
using Xunit;

/// <summary>
/// The subset of <c>profile-rest-api-v1</c> this session could check mechanically
/// WITHOUT an analyser — reflection over the compiled assembly.
/// </summary>
/// <remarks>
/// This class is the run's direct evidence for PRD §11.4. Per CG-R-127, every profile
/// rule is read-enforced for this run because no analyser exists; what follows is the
/// part that turned out not to need one, which is a smaller set than the profile's
/// <c>enforcement: analyser</c> markings claim.
///
/// The Gate C report states, rule by rule, which of the nine <c>must</c>/<c>must_not</c>
/// rules are checkable by reflection, which need a Roslyn syntax/semantic pass, and
/// which are not mechanically checkable at all under any reading. Two results there are
/// worth carrying: several rules are checkable only in their literal form, and
/// <c>EvadesEveryMustNot_ByIndirection</c> below PASSES — it asserts that this solution
/// performs persistence, transport reads and I/O while every role's <c>must_not</c>
/// holds. Q-16.
/// </remarks>
public sealed class ProfileConformanceTests
{
    private static readonly Assembly Slice = typeof(PlaceOrderHandler).Assembly;

    private static IEnumerable<Type> RoleTypes(SliceRole role) =>
        Slice.GetTypes()
            .Where(t => t.GetCustomAttribute<SliceAttribute>() is { } s
                        && s.ActInstance == "PlaceOrder"
                        && s.Role == role);

    /// <summary>controller must "declares [Slice(&lt;instance&gt;, \"controller\")]". Required: true.</summary>
    [Fact]
    public void A_controller_role_is_declared_for_PlaceOrder() =>
        Assert.Single(RoleTypes(SliceRole.Controller));

    /// <summary>handler must "declares [Slice(&lt;instance&gt;, \"handler\")]". Required: true.</summary>
    [Fact]
    public void A_handler_role_is_declared_for_PlaceOrder() =>
        Assert.Single(RoleTypes(SliceRole.Handler));

    /// <summary>
    /// provider — required: false, but required for THIS act: <c>ActorIdentity</c> is
    /// external per DSC-0003 and the vocabulary's own note.
    /// </summary>
    [Fact]
    public void Provider_roles_are_declared_for_every_position_requiring_storage() =>
        Assert.Equal(3, RoleTypes(SliceRole.Provider).Count());

    /// <summary>
    /// R-Q10 — "providers can supply writes as well as reads. They are the adapters to
    /// the storage options." The write position is roled, so the profile reaches it.
    /// </summary>
    [Fact]
    public void The_write_position_is_carried_by_a_provider_role()
    {
        var writer = Assert.Single(
            RoleTypes(SliceRole.Provider).Where(t => t.Name == nameof(OrderPlacedProvider)));

        var records = Assert.Single(
            writer.GetMethods(BindingFlags.Public | BindingFlags.Instance | BindingFlags.DeclaredOnly));

        Assert.Equal(typeof(Ordering.Api.Facts.OrderPlaced), Assert.Single(records.GetParameters()).ParameterType);
    }

    /// <summary>
    /// R-Q10 withdrew D-33: the controller reaches the handler-role type directly, so
    /// "calls exactly one type declaring the handler role for the same act instance"
    /// holds at run time and not only in source text.
    /// </summary>
    [Fact]
    public void The_controller_depends_on_the_handler_role_type_itself()
    {
        var dependency = Assert.Single(
            Assert.Single(typeof(PlaceOrderController).GetConstructors()).GetParameters());

        Assert.Equal(SliceRole.Handler, dependency.ParameterType.GetCustomAttribute<SliceAttribute>()?.Role);
    }

    /// <summary>
    /// handler must "exposes a single entry point taking the command and returning
    /// Accepted or Rejected". Checkable by reflection — this is one of the few that is.
    /// </summary>
    [Fact]
    public void The_handler_exposes_a_single_entry_point_over_the_command()
    {
        var entryPoints = typeof(PlaceOrderHandler)
            .GetMethods(BindingFlags.Public | BindingFlags.Instance | BindingFlags.DeclaredOnly)
            .Where(m => m.GetParameters().Length == 1
                        && m.GetParameters()[0].ParameterType == typeof(PlaceOrderCommand))
            .ToList();

        var entry = Assert.Single(entryPoints);
        Assert.Equal(typeof(PlaceOrderOutcome), entry.ReturnType);
    }

    /// <summary>
    /// handler must "emits only events the act declares it writes". <c>PlaceOrder</c>
    /// declares <c>writes: [OrderPlaced]</c>.
    /// </summary>
    /// <remarks>
    /// D-36 — this checks the SHAPE of the outcome type, not the emissions. A handler
    /// that constructed a second event type and dropped it would pass. Checking the
    /// real rule needs a Roslyn pass over object-creation expressions, and checking it
    /// against the act vocabulary needs the analyser to read
    /// <c>ordering.eventmodel.yaml</c> — which no input says any tool does.
    /// </remarks>
    [Fact]
    public void The_accepted_outcome_carries_only_the_declared_write()
    {
        var carried = typeof(PlaceOrderOutcome.Accepted)
            .GetProperties(BindingFlags.Public | BindingFlags.Instance | BindingFlags.DeclaredOnly)
            .Select(p => p.PropertyType.Name)
            .ToList();

        Assert.Equal(new[] { nameof(Ordering.Api.Facts.OrderPlaced) }, carried);
    }

    /// <summary>
    /// controller must_not "references a provider role directly" — checkable by
    /// reflection over the constructor signature only. A method-body reference or a
    /// service-locator call would not be caught here.
    /// </summary>
    [Fact]
    public void The_controller_does_not_take_a_provider_role()
    {
        var dependencies = typeof(PlaceOrderController)
            .GetConstructors()
            .SelectMany(c => c.GetParameters())
            .Select(p => p.ParameterType);

        Assert.DoesNotContain(dependencies, t => t.GetCustomAttribute<SliceAttribute>()?.Role == SliceRole.Provider);
    }

    /// <summary>
    /// THE FINDING, AFTER THREE RULINGS: the profile is now satisfiable, and every rule
    /// that survived contact is one a checker can only run by reading the determinations.
    /// </summary>
    /// <remarks>
    /// This test asserted three evasions at Gate C, then one breach after R-Q16. R-Q06
    /// amended the provider's must_not to turn on what the provider adapts, so the breach
    /// is gone and the slice conforms again — but the rule left the code. Whether this
    /// type may hold its transport reference is now a question about DSC-0003, not about
    /// this assembly. See ProviderTransportCarrierTests.
    ///
    /// What remains here is the narrow, still-mechanical part: the provider holds the
    /// reference ITSELF rather than through an unroled hop, which is what R-Q16 bought
    /// and what makes the R-Q06 check meaningful. Hide the reference again and the
    /// carrier check has nothing to look at.
    /// </remarks>
    [Fact]
    public void The_transport_reference_is_held_by_the_roled_provider_itself()
    {
        var referenced = Assert.Single(
            Assert.Single(typeof(ActorIdentityProvider).GetConstructors()).GetParameters());

        Assert.Equal("Microsoft.AspNetCore.Http", referenced.ParameterType.Namespace);
        Assert.Equal(SliceRole.Provider, typeof(ActorIdentityProvider).GetCustomAttribute<SliceAttribute>()?.Role);
    }

    /// <summary>
    /// Q-33 — R-Q16's boundary. Four types still declare no role, and no role in the
    /// profile fits any of them: two stores a provider adapts, an identifier mint that
    /// supplies no fact, and middleware that produces a transport result without being
    /// the controller. Either "participating in the act" stops short of them, or the
    /// profile needs roles it does not have.
    /// </summary>
    [Fact]
    public void Four_participating_types_still_have_no_role_the_profile_can_give_them()
    {
        // D-38 — the compiler generates types too. The async state machine behind the
        // middleware's InvokeAsync is a class in this namespace with no role, and an
        // exhaustiveness rule stated over "every type" catches it. So the rule R-Q16 asks
        // for cannot be written over types as the CLR sees them; it needs "every type
        // declared in source". A small point that only shows up once you try to run it.
        var unroled = Slice.GetTypes()
            .Where(t => t.IsClass && !t.IsAbstract && t.Namespace == "Ordering.Api.Unroled")
            .Where(t => t.GetCustomAttribute<CompilerGeneratedAttribute>() is null)
            .Where(t => t.GetCustomAttribute<SliceAttribute>() is null)
            .Select(t => t.Name)
            .OrderBy(n => n, StringComparer.Ordinal)
            .ToArray();

        Assert.Equal(
            new[]
            {
                nameof(GuidOrderIdentityMint),
                nameof(InMemoryCartStore),
                nameof(InMemoryOrderPlacedStore),
                nameof(ReadPositionUnavailableMiddleware),
            },
            unroled);
    }
}
