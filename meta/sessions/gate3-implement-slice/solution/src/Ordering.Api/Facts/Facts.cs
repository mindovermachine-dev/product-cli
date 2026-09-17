namespace Ordering.Api.Facts;

// ---------------------------------------------------------------------------
// DECLARED, NOT INVENTED. `inputs/ordering.fact-type-space.md`, [PROPOSED], move 1 under
// CG-R-137.
//
// Every field below is declared there, with a type and a required/optional marking. The
// earlier version of this file said "INVENTED, ENTIRELY" and listed seven fields nobody
// had specified; D-04 … D-09 and D-11 retire with this rewrite.
//
// The file's own reading rules are followed literally:
//   * a field with no type and no required marking is not declared, so none is added;
//   * a type is a primitive, another declared fact, or a declared value kind;
//   * §6 lists what is deliberately absent, so absence here is *stated* absence.
//
// What that changes for this slice, concretely: DSC-0002 — "every command slice validates
// its payload against the domain model's declared types before deciding" — is dischargeable
// for the first time in this run. There are now declared types to validate against.
// ---------------------------------------------------------------------------

// ── §4 Value kinds ─────────────────────────────────────────────────────────

/// <summary>`CartId: { base: uuid }`.</summary>
public readonly record struct CartId(Guid Value);

/// <summary>`CartLineId: { base: string }` — caller-supplied, opaque to this context.</summary>
public readonly record struct CartLineId(string Value);

/// <summary>`BuyerId: { base: string }` — opaque; issued by the identity provider.</summary>
public readonly record struct BuyerId(string Value);

/// <summary>`CatalogItemId: { base: int }`.</summary>
public readonly record struct CatalogItemId(int Value);

/// <summary>`OrderId: { base: uuid }`.</summary>
public readonly record struct OrderId(Guid Value);

/// <summary>`Instant: { base: timestamp }`, UTC.</summary>
public readonly record struct Instant(DateTimeOffset Value);

/// <summary>`CurrencyCode: { base: string, constraint: "ISO 4217 alpha-3" }`.</summary>
public readonly record struct CurrencyCode(string Value);

/// <summary>
/// `Quantity: { base: int, constraint: "1 … 99" }` — **bounded, and never clamped**: a
/// request outside the bound is rejected rather than adjusted.
/// </summary>
/// <remarks>
/// The declaration calls this "the counter-conventional reading carried in the
/// determination set", stated in the fact type space "so the bound has a home". Clamping
/// is therefore forbidden, and this type refuses rather than adjusting.
///
/// D-55 — the bound is enforced at construction. Where it is *checked* for a command is a
/// different question (DSC-0002, at the controller) and the bound belongs to the fact
/// either way. Note this is `AddToCart`'s payload concern, not `PlaceOrder`'s: no
/// `PlaceOrder` payload field carries a Quantity.
/// </remarks>
public readonly record struct Quantity
{
    public const int Min = 1;
    public const int Max = 99;

    public Quantity(int value)
    {
        if (value is < Min or > Max)
        {
            throw new ArgumentOutOfRangeException(
                nameof(value), value, $"Quantity is bounded {Min}…{Max} and is never clamped.");
        }

        Value = value;
    }

    public int Value { get; }
}

/// <summary>`Money: { Amount: decimal, Currency: CurrencyCode }`.</summary>
public readonly record struct Money(decimal Amount, CurrencyCode Currency);

// ── §1 Entities ────────────────────────────────────────────────────────────

/// <summary>`Cart`, identity `CartId`.</summary>
/// <remarks>
/// `Lines` is declared `required: true` with the note "may be empty", and §5 puts a
/// `0 … 50` constraint on it. The cap is on lines, not on total quantity.
/// </remarks>
public sealed record Cart(
    CartId CartId,
    BuyerId BuyerId,
    IReadOnlyList<CartLine> Lines,
    CurrencyCode Currency)
{
    /// <summary>§5 — `Cart.Lines: { constraint: "0 … 50" }`.</summary>
    public const int MaxLines = 50;

    /// <summary>
    /// The predicate DSC-0001 pins: `settled_by: "invariant:CartNotEmpty"`.
    /// </summary>
    /// <remarks>
    /// D-05 RETIRES. "Empty" was this session's invention when `Cart` had no declared
    /// structure; `Lines` is now declared with the note "may be empty", so an empty cart is
    /// a cart with no lines and the reading is the declaration's, not this session's.
    /// </remarks>
    public bool IsEmpty => Lines.Count == 0;

    /// <summary>
    /// D-56 — the total is summed in the CART's currency.
    ///
    /// **This is the residue of DSC-0005, and §7 open item 1 names it:** `CurrencyCode`
    /// appears on `Cart`, `OrderPlaced` and `OrderSummary`, each `CartLine.UnitPrice` is a
    /// `Money` carrying its own, and **no determination says what happens when they
    /// disagree**. DSC-0005 is withdrawn (CG-R-138), so nothing rejects a mismatch and
    /// nothing converts one. This sums line amounts and stamps the cart's currency, which
    /// is arithmetic, not a decision — and it is wrong if the currencies differ.
    ///
    /// Reported rather than papered over, which is the point: the gap is now visible where
    /// an invented `AccountCurrency` used to hide it.
    /// </remarks>
    public Money Total => new(
        Lines.Sum(line => line.UnitPrice.Amount * line.Quantity.Value),
        Currency);
}

/// <summary>`CartLine`, a value kind.</summary>
public sealed record CartLine(
    CartLineId LineId,
    CatalogItemId CatalogItemId,
    Quantity Quantity,
    Money UnitPrice);

/// <summary>
/// `ActorIdentity`, identity `BuyerId`. External ground: produced by no act in scope and
/// declared `external` on every act that reads it, per DSC-0003.
/// </summary>
/// <remarks>
/// **`AccountCurrency` IS GONE.** It was invented during the build to obey DSC-0005
/// (D-07, flagged contested at the time). CG-R-138 withdraws that determination and the
/// fact type space §6 strikes the field by name: *"the ground goes with the
/// determination."* D-07 and D-16 retire.
///
/// The declaration's own note is the governing one: these are "the fields the ordering
/// context may rely on, not the ones the identity provider holds."
/// </remarks>
public sealed record ActorIdentity(
    BuyerId BuyerId,
    string? DisplayName,
    bool IsAnonymous);

// ── §2 Events ──────────────────────────────────────────────────────────────

/// <summary>`OrderPlaced` — the sole write position on `PlaceOrder`.</summary>
/// <remarks>
/// `Lines` is declared `required: true` with the note "non-empty", which is DSC-0001
/// showing up in the fact's own shape: an order cannot be placed against an empty cart, so
/// a placed order cannot carry no lines. The two agree, and that agreement is new — before
/// move 1 neither had a shape to agree with.
/// </remarks>
public sealed record OrderPlaced(
    OrderId OrderId,
    CartId CartId,
    BuyerId BuyerId,
    IReadOnlyList<OrderLine> Lines,
    CurrencyCode Currency,
    Money Total,
    Instant OccurredAt);

/// <summary>
/// `OrderLine` — "a snapshot of the cart line at placement; it does not carry CartLineId".
/// </summary>
/// <remarks>
/// §7 open item 2 records the consequence: an order cannot be traced back to the cart
/// lines that produced it. Declared deliberate, and declared "wrong the moment anything
/// needs it". Implemented as declared.
/// </remarks>
public sealed record OrderLine(
    CatalogItemId CatalogItemId,
    Quantity Quantity,
    Money UnitPrice);
