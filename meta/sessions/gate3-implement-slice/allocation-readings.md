# Q-04 / Move 2 — the two readings of `allocation.class`, and what each does to the slice

**Asked by CG-R-137:** *"Whether DSC-0005's currency rejection belongs in the slice turns
on its class and on which reading the session took, and I will not rule that from a
summary. State both readings and what each does to the slice, and it is one sentence
back."*

**The principle is ruled and is not in question here:**

> `allocation.class` describes how a determination is **held**, not whether it has been
> acted on. `residual` means nothing in the specification settles it and a named actor
> carries it — recording it as residual does not discharge it. `pinned` means it is
> settled; `checked` means a predicate settles acceptability.

What is in question is what that makes of **DSC-0005**, and the record is:

```yaml
- id: DSC-0005
  statement: >-
    An order placed against a cart whose currency differs from the customer's
    account currency is rejected rather than converted.
  allocation:
    class: residual
    carried_by: human
    principal: { kind: human, identifier: "emil" }
  positions:
    - { fact_type: Cart, role: read, boundary: { kind: internal } }
  provenance:
    made_at: act-time
    made_by: "session:2026-09-02-implement-placeorder"
```

---

## The reading this session took: **A — the statement settles, the class says how surely**

`residual` describes the **assurance**: no pin, no predicate, a named human carries the
risk that the statement holds. The statement itself is authoritative behaviour.

**Recorded at the time as D-20(a)**, with the alternative named and rejected without
authority. Implemented in `PlaceOrderHandler.Handle`.

### What it does to the slice — this is what is built now

- `CurrencyMatchesAccount` rejection is implemented and unverified.
- It requires `ActorIdentity.AccountCurrency` — **invented**, recorded as
  `D-07 — INVENTED AND CONTESTED`, because no fact in the vocabulary carries an account
  currency and **DSC-0005's own `positions` block declares only `Cart: read`**. The
  comparison reads a fact the determination declares no position for.
- It requires an `account_currency` OIDC claim — **invented**, D-16, with no OIDC basis at
  all.
- It requires an invariant name to cite — `CurrencyMatchesAccount`, **coined**, D-20(b),
  appearing in no input.
- It forces a precedence decision against DSC-0001 — D-18, arbitrary.
- Two tests: `Rejects_a_currency_mismatch_rather_than_converting`,
  `Reports_CartNotEmpty_first_when_both_rejections_apply`.

**One determination, five inventions.**

---

## Reading B — the class says nothing settles it, so the statement does not either

`residual` means what the principle says: *nothing in the specification settles it.* The
statement is then **emil's carried proposal about what the rule should be**, filed with an
owner, not an instruction to build.

### What it does to the slice

- The currency rejection **comes out.** So do `ActorIdentity.AccountCurrency`, the
  `account_currency` claim, the coined invariant name, the precedence decision, and both
  tests.
- **D-07, D-16, D-18 and D-20(a)/(b) all retire** — five of this run's contested
  inventions, gone with one ruling.
- The slice is left with exactly **one** rejection: DSC-0001's `CartNotEmpty`, which is
  `pinned` and carries `settled_by: "invariant:CartNotEmpty"`.
- A live residual remains at this address, owned by emil, with nothing built — which is
  Q-17's question (should the code record a live residual, or is silence correct?) now
  bearing on behaviour rather than on documentation.

---

## Three things the record says that bear on which reading is right

Offered because the ruling asked for the readings and their consequences, and these are
consequences. **Not a ruling, and Q-04 stays open.**

**1. DSC-0003 shows what a correct `residual` statement looks like, and DSC-0005 does not
match it.**

| | statement | class |
|---|---|---|
| DSC-0003 | *"Who may place an order is **not settled** at this address."* | residual |
| DSC-0005 | *"An order… **is rejected** rather than converted."* | residual |

DSC-0003's statement **announces its own non-settlement** — exactly as the ruled principle
describes. DSC-0005's statement settles a behaviour. Under the principle the two cannot
both be well-formed `residual`s, so either DSC-0005 is **misclassified** (a settling
statement wants `pinned`, as DSC-0001 has) or its statement is a proposal rather than a
settlement. Reading B is the second; a third possibility is that the record is simply
wrong and wants superseding.

**2. Reading A gives a previous implementation session an authority this one is denied.**

DSC-0005 is `made_at: act-time`, `made_by: "session:2026-09-02-implement-placeorder"`. Under
Reading A, a build session hit the currency question and **authored a settling
determination** mid-build. This session's prompt forbids exactly that: *"Do not author
determinations. If the specification does not settle something, that is the finding, not a
gap to fill."* Under Reading B the same record reads as the permitted thing — a builder hit
a question it could not resolve, filed it with a named human carrying it, and moved on.
**Reading B makes act-time residuals coherent with the prohibition; Reading A makes them an
exception to it.**

**3. Reading A is the reading under which a determination costs five inventions.**

That is not an argument from convenience. It is the observation that a determination whose
statement reaches a fact the vocabulary does not contain, and whose own `positions` block
does not declare that fact, **cannot be implemented without authoring the ground it needs**
— which is R-GROUND, at the level of a single record. Under Reading A this session was
required to model `AccountCurrency` in order to obey DSC-0005, and modelling it was
authoring.

---

## The one sentence back

**This session took Reading A, and on the principle as now ruled it should have taken
Reading B** — under which DSC-0005's currency rejection comes out of the slice along with
five of the run's contested inventions, leaving `CartNotEmpty` as the act's only rejection.

Awaiting the instance ruling before changing any code.
