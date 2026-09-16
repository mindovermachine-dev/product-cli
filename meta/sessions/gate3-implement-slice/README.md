# Gate 3 — builder session

**Party:** builder (CG-R-128). Blind to the design: the four arrived inputs, the session
prompt, and the two rulings that scope the run. No README of the binding, no conformance
manifest, no repository, no web search.

**Result: 33 clarifications against 3 of 13 frame categories settled without invention.**
Two answered by Emil on 2026-09-16 (`rulings.md`), thirty-one open. Read
`gate-c-report.md` first; `gate-c-report-baseline.md` is the unrevised version from before
any answer arrived, kept so the before/after is comparable.

**The slice as it now stands knowingly does not conform**, and that is the finding — see
R-Q16.

| File | What it is |
|---|---|
| `prompt.md` | the session prompt, verbatim |
| `inputs/` | the four arrived inputs, verbatim, hashed in `bootstrap.md` before use |
| `bootstrap.md` | first act: hashes, what was read, the reading order, the one standing rule that could not be complied with |
| `gate-a.md` | the expectation list, written with the determinations unopened; the reading of the slice; six contradictions |
| `frame-categories.md` | the 13-category scheme — **invented**, because the bundle never defines "frame category" |
| `questions.md` | 33 questions, verbatim, with what prompted each and the provisional reading the build proceeded under |
| `rulings.md` | Emil's answers, verbatim, with what each changed and what it raised |
| `decisions.md` | 39 points the specification does not settle, each tagged `D-nn` at its site; one withdrawn and one reversed by rulings |
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
3. **Closing the evasion revealed a contradiction rather than producing conformance.**
   R-Q16 made roles exhaustive, which is the right rule and the most enforceable one in
   the profile. With no unroled hop left, DSC-0003 (the fact is an OIDC claim on the
   request), the provider's `must_not` on transport types, and exhaustiveness are
   **jointly unsatisfiable**. `ActorIdentityProvider` breaks the `must_not` in the open,
   asserted by a passing test. **Q-06 is now forced.**
4. **Exhaustiveness has no stated edge.** Four types still declare no role and every role
   the profile offers rejects them by its own rules — two stores, an id mint that supplies
   no fact, and middleware that calls no handler. Q-33.

## Running it

```bash
cd solution && dotnet test        # 15 tests, .NET 8
```
