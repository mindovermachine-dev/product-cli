# Gate 3 — builder session

**Party:** builder (CG-R-128). Blind to the design: the four arrived inputs, the session
prompt, and the two rulings that scope the run. No README of the binding, no conformance
manifest, no repository, no web search.

**Result: 36 clarifications against 3 of 13 frame categories settled without invention.**
Four answered by Emil on 2026-09-16 (`rulings.md`), thirty-two open. Read
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
| `questions.md` | 36 questions, verbatim, with what prompted each and the provisional reading the build proceeded under |
| `rulings.md` | Emil's answers, verbatim, with what each changed and what it raised |
| `decisions.md` | 41 points the specification does not settle, each tagged `D-nn` at its site; one withdrawn and one reversed by rulings |
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
4. **Making a rule correct made it less enforceable.** R-Q06's amended rule is no longer
   checkable from the assembly: it turns on the boundary the provider adapts, so a checker
   must read the determination store — and then infer "transport" from `read_provenance`,
   which the schema types as free text. `ProviderTransportCarrierTests` does exactly this
   against the real determinations file and is the first thing here to enforce a profile
   rule by reading them. A `boundary.carrier` enum is proposed.
5. **Exhaustiveness has no stated edge.** Four types still declare no role and every role
   the profile offers rejects them by its own rules — two stores, an id mint that supplies
   no fact, and middleware that calls no handler. Q-33.

## Running it

```bash
cd solution && dotnet test        # 23 tests, .NET 8
```
