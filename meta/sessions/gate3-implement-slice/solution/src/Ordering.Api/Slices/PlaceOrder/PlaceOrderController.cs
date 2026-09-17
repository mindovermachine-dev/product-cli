namespace Ordering.Api.Slices.PlaceOrder;

using Microsoft.AspNetCore.Mvc;
using Ordering.Api.Profile;

/// <summary>
/// The transport entry to the <c>PlaceOrder</c> act.
/// </summary>
/// <remarks>
/// PROFILE / controller — required: true.
///   must     "declares [Slice(&lt;instance&gt;, \"controller\")]"                   — below.
///   must     "calls exactly one type declaring the handler role for the same act
///             instance"                                                            — PlaceOrderHandler, one call. R-Q10.
///   must     "returns a transport result derived from the handler's Accepted or
///             Rejected"                                                            — SEE D-21; "derived" is undefined.
///   must_not "contains a conditional on domain state"                              — SEE D-22.
///   must_not "references a provider role directly"                                 — holds.
///   must_not "references a persistence type"                                       — holds.
///
/// D-21 — SETTLED BY RULING R-Q12, HAVING BEEN THE LARGEST HOLE IN THE PROFILE.
///
/// The profile says only that the controller returns "a transport result DERIVED FROM the
/// handler's Accepted or Rejected" — a dependency, not a function. No input states a
/// status, a Location, or a body. This session invented the mapping and reported it as
/// the single largest hole in the run. R-Q12 settles the two branches the rule names:
///     Accepted -> 201 Created, Location: /orders/{OrderId}
///     Rejected -> 422 Unprocessable Entity
/// which is what was invented here. **That the guess matched is not evidence the profile
/// was sufficient** — a second reader guessing 200/400 or 202/409 would have conformed
/// equally well, and the profile still does not say. The ruling fixes it; the gap it
/// fixes was real.
///
/// TWO PATHS THE RULING DOES NOT REACH, still this session's inventions:
///   * invalid payload -> 400 (DSC-0002; see D-23). Not an Accepted and not a Rejected,
///     so the rule R-Q12 settles does not cover it.
///   * a read position that cannot be supplied -> 401 / 404 via unroled middleware
///     (D-19, D-31). The profile admits two handler exits and this is a third.
/// Both remain open. Q-09, Q-12b.
///
/// ═══ R-Q12 COLLIDES WITH DSC-0004, AND NOTHING IN THE SCHEME CONNECTS THEM ═══
///
/// `Location: /orders/{id}` invites the caller to read what it just wrote. The only act
/// that can answer that read is the `OrderSummary` read-model — and DSC-0004 declares it
/// "may lag the event stream by up to five seconds", with a `proxy.known_divergence` that
/// says, in its own words:
///
///     "A five-second window is acceptable for browsing and is not acceptable
///      immediately after the reader's own write, where they expect to see their change."
///
/// That is a description of exactly the case a Location header creates. R-Q40's outbox
/// widens it further: the event is durable at 201 but not yet on the bus, so the
/// projection has not even begun to lag yet.
///
/// **The determination store already contained the warning that this transport decision
/// triggers, and nothing connects the two.** The resolution condition checks that facts
/// resolve; no check notices that a build-time decision on one act lands inside a filed
/// known-divergence on another. Q-43 — reported, not worked around: the Location header
/// is emitted as ruled.
///
/// D-22 — DECIDED. Rejections map to ONE status regardless of which invariant was
/// cited. Branching per invariant would be "a conditional on domain state", which the
/// profile forbids the controller. So the invariant travels in the body and never in
/// the status line. That is this session's reading of a must_not whose boundary — does
/// switching on Accepted vs Rejected already count? — the profile does not draw. The
/// profile requires exactly that switch, so it cannot; where the line falls past it is
/// undrawn.
///
/// D-23 — DECIDED. DSC-0002 ("every command slice validates its payload against the
/// domain model's declared types before deciding") is realised HERE and not in the
/// handler, because the handler may reject "only for invariants the fact vocabulary
/// declares" and a payload type fault is not one. But that puts a transport result on a
/// path that is NOT "derived from the handler's Accepted or Rejected", contradicting
/// the controller's own must. One of the two rules gives; this session chose which.
/// Q-09.
///
/// D-24 — INVENTED. Route, verb, the plural noun, the absence of versioning in the path,
/// and the fact that the resource is /orders rather than /place-order. The profile
/// fixes the stack and names the role; it fixes no URL.
/// </remarks>
[ApiController]
[Route("orders")]
[Slice("PlaceOrder", SliceRole.Controller)]
public sealed class PlaceOrderController : ControllerBase
{
    // R-Q10 withdrew D-33. This is the concrete handler-role type, not an interface a
    // DI registration can redirect to an unroled one.
    private readonly PlaceOrderHandler _handler;

    public PlaceOrderController(PlaceOrderHandler handler) => _handler = handler;

    [HttpPost]
    [ProducesResponseType(StatusCodes.Status201Created)]
    [ProducesResponseType(StatusCodes.Status400BadRequest)]
    [ProducesResponseType(StatusCodes.Status422UnprocessableEntity)]
    public IActionResult Post([FromBody] PlaceOrderCommand command)
    {
        // DSC-0002. [ApiController] already short-circuits an invalid ModelState to 400
        // before this method is entered; the explicit guard states the determination at
        // its site rather than leaving it to framework behaviour. D-23.
        if (!ModelState.IsValid)
        {
            return ValidationProblem(ModelState);
        }

        var outcome = _handler.Handle(command);

        return outcome switch
        {
            PlaceOrderOutcome.Accepted accepted => Created(
                $"/orders/{accepted.Event.OrderId.Value}",
                accepted.Event),

            PlaceOrderOutcome.Rejected rejected => UnprocessableEntity(new ProblemDetails
            {
                Status = StatusCodes.Status422UnprocessableEntity,
                Title = "The command was rejected.",
                Detail = rejected.Reason,
                Extensions = { ["invariant"] = rejected.Invariant },
            }),

            // D-25 — INVENTED. PlaceOrderOutcome is a closed hierarchy, so this arm is
            // unreachable; C# cannot prove it. The profile's "Accepted or Rejected" says
            // the pair is exhaustive and says nothing about how to say so in the stack.
            _ => throw new InvalidOperationException(
                $"Unhandled outcome '{outcome.GetType().Name}'."),
        };
    }
}
