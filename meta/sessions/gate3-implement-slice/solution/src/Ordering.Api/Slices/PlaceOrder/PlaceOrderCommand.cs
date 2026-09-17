using System.ComponentModel.DataAnnotations;

namespace Ordering.Api.Slices.PlaceOrder;

/// <summary>
/// The command message <c>PlaceOrder</c> takes.
/// </summary>
/// <remarks>
/// D-09 — INVENTED. <c>PlaceOrder</c> is a name in the act vocabulary. No input declares
/// a command payload, its fields, or its relation to the HTTP request body. The single
/// field below is this session's minimum: the handler reads <c>Cart</c>, so it must be
/// told which cart, and <c>ActorIdentity</c> is external and arrives by a different
/// route (DSC-0003), so it is deliberately NOT a payload field.
///
/// D-10 — The payload doubles as the HTTP request body; there is no separate transport
/// DTO. Nothing settles whether the command and the request body are one type.
///
/// DSC-0002 — "Every command slice validates its payload against the domain model's
/// declared types before deciding", allocation <c>checked</c>, closure
/// <c>operational</c>, <c>runnable_by: "roslyn-analyzer:PayloadTypeConformance"</c>.
///
/// **DISCHARGEABLE FOR THE FIRST TIME IN THIS RUN.** For the whole build this validated
/// the payload against types THIS SESSION had invented, because the domain model declared
/// none — the check was self-referential and said so. Move 1 supplies the declared types:
/// `CartId: { base: uuid }`. So the annotation below checks the payload against a domain
/// type someone else declared, which is what the determination asked for. Q-07 closed.
///
/// The analyser still does not exist, so per CG-R-127 the rule remains read-enforced for
/// this run — but it is now a rule that *could* be run, which it was not before.
///
/// D-11 — DSC-0002's acceptance <c>does_not_cover</c> lists
/// <c>cross-field-consistency</c>. There is only one field, so the exclusion is
/// vacuous here and nothing is done about it.
///
/// D-58 — NO `Quantity` OR LINE-CAP CHECK HERE, AND THAT IS NOT AN OMISSION. The fact
/// type space bounds `Quantity` at 1…99 "never clamped" and caps `Cart.Lines` at 0…50.
/// Neither is a `PlaceOrder` payload field: this command carries a `CartId` and nothing
/// else, so DSC-0002's payload check over it has exactly one field to check. Both
/// constraints belong to `AddToCart`'s payload and to `Cart`'s construction, which are
/// other acts. The bound is enforced on the `Quantity` type itself (Facts.cs D-55), so a
/// cart cannot hold an out-of-bound quantity however it was built.
/// </remarks>
public sealed record PlaceOrderCommand
{
    /// <summary>Declared type: `CartId: { base: uuid }`.</summary>
    [Required(AllowEmptyStrings = false)]
    public Guid CartId { get; init; }
}
