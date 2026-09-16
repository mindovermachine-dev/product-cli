namespace Ordering.Api.Unroled;

using System.Collections.Concurrent;
using Ordering.Api.Facts;
using Ordering.Api.Slices.PlaceOrder;

/// <summary>
/// D-29 — INVENTED. In-memory, because no input names a store and inventing a schema
/// would be authoring further than the slice requires. The `Cart` read-model slice is
/// what would populate this; it is out of scope for this run, which builds one slice.
/// </summary>
public sealed class InMemoryCartStore : ICartStore
{
    private readonly ConcurrentDictionary<string, Cart> _carts = new(StringComparer.Ordinal);

    public Cart? Find(string cartId) => _carts.TryGetValue(cartId, out var cart) ? cart : null;

    public void Put(Cart cart) => _carts[cart.CartId] = cart;
}

/// <summary>
/// D-29 — INVENTED. In-memory, same reason. Still unroled after ruling R-Q10: the
/// PROVIDER is the adapter and carries the role; the store it adapts is behind it, which
/// is the same shape as ICartStore on the read side.
/// </summary>
public sealed class InMemoryOrderPlacedStore : IOrderPlacedStore
{
    private readonly List<OrderPlaced> _appended = new();

    public IReadOnlyList<OrderPlaced> Appended
    {
        get { lock (_appended) { return _appended.ToArray(); } }
    }

    public void Append(OrderPlaced placed)
    {
        lock (_appended) { _appended.Add(placed); }
    }
}

/// <summary>
/// D-30 — INVENTED. A GUID as the order identifier. Nothing settles that orders have
/// identifiers, let alone their shape, so a human-meaningful order number is equally
/// supported and would change the event, the Location header and the API. Q-14.
/// </summary>
public sealed class GuidOrderIdentityMint : IOrderIdentityMint
{
    public string Next() => Guid.NewGuid().ToString("N");
}

/// <summary>
/// Maps <see cref="ReadPositionUnavailableException"/> to a transport result.
/// </summary>
/// <remarks>
/// NO [Slice] ATTRIBUTE. DELIBERATE.
///
/// D-31 — INVENTED, AND UNSUPPORTED BY ANY INPUT. The profile admits two handler exits,
/// Accepted and Rejected, and the controller derives the transport result from them. A
/// read position that cannot be supplied is neither. The status codes chosen —
/// 401 when `ActorIdentity` is missing, 404 when `Cart` is — encode a security judgement
/// and a REST convention that this session imported wholesale. Q-09, Q-12.
/// </remarks>
public sealed class ReadPositionUnavailableMiddleware
{
    private readonly RequestDelegate _next;

    public ReadPositionUnavailableMiddleware(RequestDelegate next) => _next = next;

    public async Task InvokeAsync(HttpContext context)
    {
        try
        {
            await _next(context);
        }
        catch (ReadPositionUnavailableException ex)
        {
            context.Response.StatusCode = ex.FactType == nameof(ActorIdentity)
                ? StatusCodes.Status401Unauthorized
                : StatusCodes.Status404NotFound;

            await context.Response.WriteAsJsonAsync(new
            {
                title = "A read position could not be supplied.",
                factType = ex.FactType,
            });
        }
    }
}
