namespace Ordering.Api.Slices.PlaceOrder;

using Ordering.Api.Facts;
using Ordering.Api.Profile;

/// <summary>
/// Everything a provider needs that is not itself a profile role.
/// </summary>
/// <remarks>
/// D-13 — INVENTED. No input names a store, a stream, a repository or a claims source.
/// These two abstractions exist so the providers have something to be reached through;
/// their implementations are unroled (see Unroled/).
/// </remarks>
public interface ICartStore
{
    Cart? Find(string cartId);
}

/// <summary>
/// The outbox. R-Q40: "For a event driven system i would expect us to always have an
/// outbox pattern before sending to the eventbus."
/// </summary>
/// <remarks>
/// D-26 — the abstraction is still INVENTED (no input names a store, stream, bus or
/// outbox) but its SHAPE is now settled: the act's write lands in a durable outbox in the
/// same breath as the decision, and something else publishes it onward later.
///
/// D-27 — WHAT THE RULING FIXED, AND WHAT IT MOVED. The earlier version appended
/// straight to a sink, and this session recorded that a failed append after a successful
/// decision loses a placed order. With an outbox the enqueue is the durable act: it
/// either succeeds before the caller is told anything, or the caller is told it failed.
/// The loss window does not vanish — it moves to the relay, where a crash means the event
/// is stored and unpublished, which is recoverable rather than lost. That is the whole
/// point of the pattern and it is why the ruling is right.
/// </remarks>
public interface IOutbox
{
    void Enqueue(OrderPlaced placed);
}

/// <summary>Where an outbox entry has got to. R-Q40.</summary>
public enum OutboxState
{
    /// <summary>Durable here, not yet on the bus.</summary>
    Pending,

    /// <summary>Handed to the event bus by the relay.</summary>
    Dispatched,
}

/// <summary>One durable entry awaiting publication.</summary>
public sealed record OutboxEntry(OrderPlaced Event, OutboxState State);

/// <summary>
/// Supplies the <c>Cart</c> read position.
/// </summary>
/// <remarks>
/// PROFILE / provider — required: false, "where external data is required".
///
/// D-15 — DECIDED. <c>Cart</c> is <c>internal</c>, not external: the <c>Cart</c>
/// read-model slice writes it, and DSC-0001 and DSC-0005 both declare its boundary
/// <c>internal</c>. So the profile's trigger for a provider ("where external data is
/// required") does not fire for this fact. But the handler <c>must_not</c> "perform I/O
/// directly", and the controller <c>must_not</c> "reference a persistence type", so no
/// other role may fetch it either. This session read the provider's trigger clause as a
/// sufficiency condition rather than a restriction on what a provider may supply — the
/// provider's own <c>must</c> rules are satisfied exactly, including "every fact it
/// supplies is declared in a read position on the act". A reading under which internal
/// facts may not go through a provider leaves the slice with no way to read its own
/// ground. Q-11.
///
/// PROFILE / provider must_not "contains a decision": returning null when the cart is
/// absent is a lookup outcome, not a decision. This session believes that holds; it is
/// the kind of line the absent analyser would have to draw.
/// </remarks>
[Slice("PlaceOrder", SliceRole.Provider)]
public sealed class CartProvider
{
    private readonly ICartStore _store;

    public CartProvider(ICartStore store) => _store = store;

    public Cart? Supply(string cartId) => _store.Find(cartId);
}

/// <summary>
/// Supplies the <c>ActorIdentity</c> read position.
/// </summary>
/// <remarks>
/// PROFILE / provider. Required here: <c>ActorIdentity</c> is external by the
/// vocabulary's own note and by DSC-0003, so the profile's "where external data is
/// required, a provider" fires.
///
/// SETTLED by DSC-0003: the fact arrives as a claim on a token already validated at the
/// gateway, so nothing here validates a token. <c>tick_rate: fast</c> is why this reads
/// per act rather than caching.
///
/// D-16 — INVENTED. The claim TYPES are not settled. DSC-0003 says "OIDC token claim"
/// and names no claim. <c>sub</c> for the actor is the OIDC-conventional choice, which
/// is a convention this session imported and the specification did not supply.
/// <c>account_currency</c> is an outright invention with no OIDC basis at all — see
/// Facts.cs D-07 and Q-05. A different reader gets different claim names here and the
/// slice silently reads nothing.
///
/// ═══ D-14 — REVERSED TWICE. NOW CONFORMING, UNDER AN AMENDED RULE. ═══
///
/// This type references <c>IHttpContextAccessor</c>. Its history is the whole argument:
///
///   Gate C   the reference sat behind an `IClaimSource` interface fed by an unroled
///            adapter. The provider's must_not "references a transport type" held in
///            source text while the transport read happened one hop away — satisfied as
///            written, defeated in substance.
///   R-Q16    roles are exhaustive, so the unroled hop had nowhere to live. The reference
///            came home and the rule went from decorative to VIOLATED. Three rules —
///            DSC-0003, the must_not, and exhaustiveness — were jointly unsatisfiable.
///   R-Q06    the must_not is amended: a provider adapting a TRANSPORT-BORNE source may
///            reference a transport type; a provider adapting a store may not. The
///            reference below is now conforming, and the rule still bites on CartProvider
///            and on every provider that adapts a store.
///
/// SETTLED by DSC-0003, and this is what carries the permission: `read_provenance` is
/// "OIDC token claim, validated at the gateway". The fact is on the request, so adapting
/// it means touching the request. Nothing here re-validates the token; the gateway did.
///
/// D-40 — WHAT THE AMENDMENT COSTS, and it is not in this file. The rule is no longer a
/// flat prohibition an analyser can check by looking at this type's references. It is
/// conditional on what the provider adapts, which lives in the determination, not the
/// code — so a checker must read the determination store. And having read it, it must
/// decide that "OIDC token claim, validated at the gateway" means transport, from prose:
/// `read_provenance` is a free-text string and the schema has no carrier field. See
/// ProviderTransportCarrierTests, which does exactly this and says plainly which line
/// should not exist. A machine-readable `boundary.carrier` is proposed in rulings.md.
/// </remarks>
[Slice("PlaceOrder", SliceRole.Provider)]
public sealed class ActorIdentityProvider
{
    private readonly IHttpContextAccessor _requests;

    public ActorIdentityProvider(IHttpContextAccessor requests) => _requests = requests;

    public ActorIdentity? Supply()
    {
        // DSC-0003: the token was validated at the gateway, so nothing here re-validates.
        var principal = _requests.HttpContext?.User;

        var actorId = principal?.FindFirst("sub")?.Value;
        var accountCurrency = principal?.FindFirst("account_currency")?.Value;

        if (string.IsNullOrWhiteSpace(actorId) || string.IsNullOrWhiteSpace(accountCurrency))
        {
            return null;
        }

        return new ActorIdentity(actorId, accountCurrency);
    }
}

/// <summary>
/// Records the <c>OrderPlaced</c> write position — into the outbox, per R-Q40.
/// </summary>
/// <remarks>
/// PROFILE / provider — THE WRITE PATH, per ruling R-Q10:
/// "we need a writer path as we have a reader path, I would argue that providers can
/// supply writes as well as reads. They are the adapters to the storage options."
///
/// Before that ruling this was an unroled decorator, because the profile gave the write
/// position no realisation: the handler may not perform I/O, the controller may not name
/// a persistence type, and every provider rule was about supplying reads. Q-10 is
/// answered; the write now happens inside a type the profile constrains, and it is
/// reached the same way the read providers are, so the handler's
/// must_not "performs I/O directly" holds for the same reason it held for reads.
///
/// PROPOSED, NOT AUTHORED — see rulings.md. The provider role's `must` list has no rule
/// for the write direction. "Every fact it supplies is declared in a read position on
/// the act" has no mirror, so nothing yet stops a provider recording a fact the act does
/// not declare it writes. This type records only `OrderPlaced`, which is the act's sole
/// write position, but it does so by construction rather than by rule.
///
/// R-Q40 — the write goes to an OUTBOX, never straight to a bus. The provider is the
/// adapter to that store, which makes its carrier `store` and keeps it clear of the
/// amended transport `must_not` (R-Q06). **Publication to the bus is not this act.** It
/// is a separate act performed by a relay reading the outbox, and the profile says in so
/// many words that it has "no profile for read-model, automation or translation slices",
/// so the relay has nothing to conform to and is deliberately not built here. Q-41.
///
/// D-27 — the loss window MOVED rather than closed, which is the pattern working. A
/// failure now happens before the caller is told anything. A crash after the enqueue
/// leaves the event durable and unpublished — recoverable, not lost. Q-30, Q-31.
/// </remarks>
[Slice("PlaceOrder", SliceRole.Provider)]
public sealed class OrderPlacedProvider
{
    private readonly IOutbox _outbox;

    public OrderPlacedProvider(IOutbox outbox) => _outbox = outbox;

    public void Record(OrderPlaced placed) => _outbox.Enqueue(placed);
}
