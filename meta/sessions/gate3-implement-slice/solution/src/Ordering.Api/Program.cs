using Ordering.Api.Slices.PlaceOrder;
using Ordering.Api.Unroled;

// ---------------------------------------------------------------------------
// Composition root.
//
// NO [Slice] ATTRIBUTE APPLIES HERE. The profile constrains three roles and says
// nothing about how one role reaches another, who registers them, or what lifetimes
// they take. D-32 — every line below is invented.
//
// D-33 — WITHDRAWN by ruling R-Q10. This registration used to resolve the controller's
// handler interface to an unroled decorator that performed the persistence the profile
// forbids every role it names — the profile satisfied statically and bypassed
// dynamically by one line here. The ruling gives the write to the provider role, so the
// line is gone and the controller reaches the handler-role type directly.
//
// R-Q16 — "we need the role for the act. We cant have an act without an actor and role
// is part of that." So the unroled claim adapter is gone too: ActorIdentityProvider now
// holds the IHttpContextAccessor reference itself, in the open, in visible breach of the
// provider's must_not. See Providers.cs and Q-06, which that ruling forces.
//
// What survives below is the boundary question R-Q16 leaves open: the stores, the mint
// and the middleware still declare no role, because no role in the profile fits them and
// "participating in the act" has no stated edge. Q-33.
// ---------------------------------------------------------------------------

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddControllers();
builder.Services.AddHttpContextAccessor();

// Profile roles.
builder.Services.AddScoped<CartProvider>();
builder.Services.AddScoped<ActorIdentityProvider>();
builder.Services.AddScoped<OrderPlacedProvider>();   // R-Q10 — the write path.
builder.Services.AddScoped<PlaceOrderHandler>();

// Unroled — outside every profile rule.
builder.Services.AddSingleton<ICartStore, InMemoryCartStore>();
builder.Services.AddSingleton<IOutbox, InMemoryOutbox>();   // R-Q40 — outbox, never straight to a bus.
builder.Services.AddSingleton<IOrderIdentityMint, GuidOrderIdentityMint>();
builder.Services.AddSingleton(TimeProvider.System);

var app = builder.Build();

// D-31 — see Unroled/Adapters.cs.
app.UseMiddleware<ReadPositionUnavailableMiddleware>();

app.MapControllers();

app.Run();

// D-34 — DECIDED. Exposed so the integration tests can drive the slice through HTTP.
// Nothing in the specification requires or forbids a test, which is itself Q-21.
public partial class Program;
