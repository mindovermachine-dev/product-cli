# Gate 3 — builder session

**Party:** builder (CG-R-128). Blind to the design: the four arrived inputs, the session
prompt, and the two rulings that scope the run. No README of the binding, no conformance
manifest, no repository, no web search.

**Result: 40 clarifications against 3 of 13 frame categories settled without invention.**
Nine answered by Emil on 2026-09-16 (`rulings.md`), thirty-one open. Read
`gate-c-report.md` first; `gate-c-report-baseline.md` is the unrevised version from before
any answer arrived, kept so the before/after is comparable.

**The slice conforms against the profile as ruled, and is in breach of the profile as
delivered.** R-Q06's amended rule text and R-Q10's supersession of DSC-0100 are both
proposed in `rulings.md` and neither has been applied to the inputs — per the prohibition
on authoring determinations.

| File | What it is |
|---|---|
| `prompt.md` | the session prompt, verbatim |
| `inputs/` | the four arrived inputs, verbatim, hashed in `bootstrap.md` before use |
| `bootstrap.md` | first act: hashes, what was read, the reading order, the one standing rule that could not be complied with |
| `gate-a.md` | the expectation list, written with the determinations unopened; the reading of the slice; six contradictions |
| `frame-categories.md` | the 13-category scheme — **invented**, because the bundle never defines "frame category" |
| `questions.md` | 40 questions, verbatim, with what prompted each and the provisional reading the build proceeded under |
| `rulings.md` | Emil's answers, verbatim, with what each changed and what it raised |
| `decisions.md` | 48 points the specification does not settle, each tagged `D-nn` at its site; one withdrawn and one reversed by rulings |
| `gate-c-report.md` | the Gate 3 report, revised against the rulings |
| `gate-c-report-baseline.md` | the report as first issued, with nothing answered |
| `solution/` | the slice: `dotnet test` → 15 passing, clean under `TreatWarningsAsErrors` |

## The findings

1. ~~**The write position has no realisation in the profile.**~~ **Answered by R-Q10** —
   the provider role carries the write path. Cost: `DSC-0100` must be superseded, because
   its `statement` prose scopes providers to "where external data is required".
2. **"Returns a transport result derived from Accepted or Rejected" names a dependency,
   not a function.** No status code, no `Location`, no body is stated anywhere. Two
   readers build two incompatible APIs and both conform. Open.
3. **The rules were consistent only while a gap let one be dodged.** Follow one
   reference through three rulings: `ActorIdentityProvider`'s transport dependency was
   **hidden** behind an interface (rule held in source text, defeated in substance) →
   **violated** once R-Q16 made roles exhaustive and left nowhere to hide → **permitted**
   once R-Q06 amended the rule to condition on the carrier. Three states, one reference,
   and the program never changed what it does.
4. **A rule can only condition on ground the determination layer models.** R-Q06's
   amended rule turns on the boundary the provider adapts, so it left the code entirely.
   The first attempt inferred the carrier by matching prose in `read_provenance`; R-GROUND
   deleted it — *"we cant add decisions to ground we havent modelled"* — for a closed
   carrier vocabulary, a reader that never infers, and a **three-valued** verdict. Against
   the store as delivered the rule is `Undeterminable`; against a fixture that models the
   carrier, `Conforms`. **Fully mechanical once the ground is modelled, unrunnable until
   it is.** The same reframing disqualifies four more profile rules that condition on
   *domain state*, *persistence type*, *I/O* and *a decision* — none of which is modelled
   anywhere.
5. **The schema left open exactly the object a rule needed to determine on.** `position`
   is closed with `additionalProperties: false`; `boundary` was not — the only such object
   in the file. R-Q37 rules it an oversight. Applying it is a two-part patch: closing
   `boundary` without declaring `carrier` forbids R-GROUND's ground. `SchemaClosureTests`
   audits the closure and asserts the coupling.
6. **Absence must be stated and attributed — and the schema already knew that twice.**
   R-Q38 makes `carrier` required, with not-supplying it permitted only as a decision a
   named human signs for. That is the third instance of one unnamed device:
   `does_not_cover`'s `asserted-none` sentinel, `residual`'s non-machine principal, and now
   this. Proposed: **name the invariant** — *no silent omission* — so the fourth instance
   is derivable instead of rediscovered.
7. **Carriage belongs to the inbound edge, and the notation already said so.** R-Q39
   scopes the carrier question to positions where an actor acts against us; where we act
   outward it is the consumer's to name. Derived from `role` + `boundary.kind` — the one
   ruling in this run that needed nothing added. It also showed that **bounding a rule's
   scope silently invalidates the examples that justified it**: the breach test had used
   an internal position and quietly stopped demonstrating a breach. The compiler caught
   it; nothing in the specification would have.
8. **Exhaustiveness has no stated edge.** Four types still declare no role and every role
   the profile offers rejects them by its own rules — two stores, an id mint that supplies
   no fact, and middleware that calls no handler. Q-33.

## Running it

```bash
cd solution && dotnet test        # 39 tests, .NET 8
```
