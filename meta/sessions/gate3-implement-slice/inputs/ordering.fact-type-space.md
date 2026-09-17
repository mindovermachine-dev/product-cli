# Fact type space — ordering context

**Move 1 under CG-R-137.** Declares the fields of every fact in `ordering.eventmodel.yaml`. Supplied as an **input** to the Gate 3 session, not as a ruling.

**Status:** `[PROPOSED]`, authored by Emil. It is the domain model, and it is authored rather than derived — there is no codebase it could be fitted to.

**What it closes:** Q-07, Q-05, Q-03, Q-26, Q-22, Q-09, and it retires the seven invented fields. It makes the profile's unsatisfiable rule satisfiable, since a rule conditioning on a fact's shape now has a shape to condition on.

**What it deliberately does not contain:** `AccountCurrency`, and anything else authored during the build to comply with DSC-0005. That determination is withdrawn under CG-R-138 and its ground goes with it.

---

## Reading rules

**Every field carries a type and a required/optional marking.** A field with neither is not declared.

**A type is either a primitive, another declared fact, or a declared value kind (§5).** Nothing is left to a reader's inference about what a string holds.

**Where a field is deliberately absent, it is listed in §6 with a reason.** Absence and omission must not look alike, which is the rule everywhere else in this scheme.

**This declares shape, not behaviour.** Which fields an act reads or writes is the act's business; that a field exists and what it holds is this file's.

---

## 1. Entities

```yaml
Cart:
  kind: entity
  identity: CartId
  fields:
    - { name: CartId,      type: CartId,          required: true }
    - { name: BuyerId,     type: BuyerId,         required: true }
    - { name: Lines,       type: [CartLine],      required: true, note: "may be empty" }
    - { name: Currency,    type: CurrencyCode,    required: true,
        note: "the currency the cart's prices are expressed in" }

CartLine:
  kind: value
  fields:
    - { name: LineId,       type: CartLineId,  required: true,
        note: "caller-supplied per DSC-0002's counter-conventional reading; two adds of the same catalogue item produce two lines" }
    - { name: CatalogItemId,type: CatalogItemId, required: true }
    - { name: Quantity,     type: Quantity,      required: true }
    - { name: UnitPrice,    type: Money,         required: true,
        note: "captured at add time; the catalogue is not consulted again" }

ActorIdentity:
  kind: entity
  identity: BuyerId
  fields:
    - { name: BuyerId,      type: BuyerId,       required: true }
    - { name: DisplayName,  type: string,        required: false }
    - { name: IsAnonymous,  type: bool,          required: true }
  note: >-
    External ground in this context. Produced by no act in scope and declared
    external on every act that reads it. Its fields are the ones the ordering
    context may rely on, not the ones the identity provider holds.
```

---

## 2. Events

```yaml
ItemAddedToCart:
  kind: event
  fields:
    - { name: CartId,        type: CartId,        required: true }
    - { name: LineId,        type: CartLineId,    required: true }
    - { name: CatalogItemId, type: CatalogItemId, required: true }
    - { name: Quantity,      type: Quantity,      required: true }
    - { name: UnitPrice,     type: Money,         required: true }
    - { name: OccurredAt,    type: Instant,       required: true }

CartEmptied:
  kind: event
  fields:
    - { name: CartId,     type: CartId,  required: true }
    - { name: OccurredAt, type: Instant, required: true }

OrderPlaced:
  kind: event
  fields:
    - { name: OrderId,    type: OrderId,       required: true }
    - { name: CartId,     type: CartId,        required: true }
    - { name: BuyerId,    type: BuyerId,       required: true }
    - { name: Lines,      type: [OrderLine],   required: true, note: "non-empty" }
    - { name: Currency,   type: CurrencyCode,  required: true }
    - { name: Total,      type: Money,         required: true }
    - { name: OccurredAt, type: Instant,       required: true }

OrderLine:
  kind: value
  fields:
    - { name: CatalogItemId, type: CatalogItemId, required: true }
    - { name: Quantity,      type: Quantity,      required: true }
    - { name: UnitPrice,     type: Money,         required: true }
  note: "a snapshot of the cart line at placement; it does not carry CartLineId"

OrderConfirmed:
  kind: event
  fields:
    - { name: OrderId,    type: OrderId, required: true }
    - { name: OccurredAt, type: Instant, required: true }
```

---

## 3. Read models

```yaml
OrderSummary:
  kind: read-model
  identity: OrderId
  fields:
    - { name: OrderId,     type: OrderId,      required: true }
    - { name: BuyerId,     type: BuyerId,      required: true }
    - { name: LineCount,   type: int,          required: true }
    - { name: Total,       type: Money,        required: true }
    - { name: Currency,    type: CurrencyCode, required: true }
    - { name: PlacedAt,    type: Instant,      required: true }
    - { name: ConfirmedAt, type: Instant,      required: false,
        note: "absent until OrderConfirmed is folded" }
```

---

## 4. Value kinds

```yaml
CartId:        { base: uuid }
CartLineId:    { base: string, note: "caller-supplied; opaque to this context" }
BuyerId:       { base: string, note: "opaque; issued by the identity provider" }
CatalogItemId: { base: int }
OrderId:       { base: uuid }
Instant:       { base: timestamp, note: "UTC" }

CurrencyCode:
  base: string
  constraint: "ISO 4217 alpha-3"

Money:
  kind: value
  fields:
    - { name: Amount,   type: decimal,      required: true }
    - { name: Currency, type: CurrencyCode, required: true }

Quantity:
  base: int
  constraint: "1 … 99"
  note: >-
    Bounded, and never clamped — a request outside the bound is rejected rather
    than adjusted. This is the counter-conventional reading carried in the
    determination set and it is stated here so the bound has a home.
```

---

## 5. Cart line cap

The fifty-line cap carried in the determination set is a constraint on `Cart.Lines`:

```yaml
Cart.Lines:
  constraint: "0 … 50"
  note: "the cap is on lines, not on total quantity"
```

Stated here rather than only in the determination, because a rule conditioning on it needs the shape to condition on — which is the R-GROUND finding applied to this file's own reason for existing.

---

## 6. Deliberately absent, with reasons

| Field | Why absent |
|---|---|
| `ActorIdentity.AccountCurrency` | Authored during the build to comply with DSC-0005, which is withdrawn under CG-R-138. The ground goes with the determination. |
| `Cart.CreatedAt`, `Cart.UpdatedAt` | No act in scope reads or writes them. A timestamp nothing uses is persistence shape, not domain shape. |
| `OrderPlaced.ShippingAddress` | Ordering does not settle delivery. It is fulfilment's, and fulfilment declares `OrderConfirmed` external. |
| `OrderPlaced.PaymentMethod` | No act in scope takes payment. Its absence is why no payment determination exists to escape. |
| `OrderSummary.Status` | No act in scope settles what an order's statuses are or what transitions between them. Adding a status field would require inventing that, which is the defect CG-R-139 names. |
| Any soft-delete or tenancy field | Population is not stated for any act (CG-R-115), so tenancy is undetermined. Adding a field would settle by shape what nobody has settled by determination. |

---

## 7. Open

1. **`CurrencyCode` appears on `Cart`, `OrderPlaced` and `OrderSummary` and no determination says what happens when they disagree.** That is a genuine gap, it is now *visible* rather than papered over by an invented field, and it is the residue of DSC-0005 done properly: the question stands, unanswered, with no proposed answer attached.
2. **`OrderLine` does not carry `CartLineId`**, so an order cannot be traced to the cart lines that produced it. Deliberate — nothing in scope needs it — and wrong the moment anything does.
3. **No field is declared nullable versus absent.** `required: false` conflates *may be missing* with *may be null*, and the distinction matters for `OrderSummary.ConfirmedAt`.
4. **This file is authored, not derived, and carries no incidence.** It has no field yet.
