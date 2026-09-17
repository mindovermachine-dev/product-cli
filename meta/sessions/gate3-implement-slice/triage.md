# Getting to the bottom of the questions

**Asked by Emil, 2026-09-17.** This is an analysis of the question set as it stands, not
new build work. Counts are read off `questions.md` and `rulings.md`.

---

## 1. First, the framing

**The bottom is not zero, and Gate 3 does not need it to be.** CG-R-127 and CG-R-128 scope
this run to finding *where the specification runs out*, and CG-R-128 says in terms that "a
build with many clarifications is a result, not a failure". That finding is already
produced.

*Getting to the bottom of the questions* is a different project — **finishing the
specification** — and it wants a different method from the one this run has been using.
What follows is that method.

---

## 2. The shape of what is left, which is not what it looks like

| | Count |
|---|---|
| Asked at Gate B (Q-01 … Q-29) | 29 |
| Raised by rulings (Q-30 … Q-47, Q-12b) | 20 |
| **Total** | **49** |
| Answered | 13.5 |
| Open | 35.5 |

Now the part that changes the strategy:

> **24 of the 29 original Gate B questions are still open.** Every one of the eleven
> rulings landed on a single chain: Q-10 → Q-16 → Q-06 → R-GROUND → Q-37 → Q-38 → Q-39 →
> Q-40 → Q-41 → Q-45, plus Q-12.

That chain is **carriage, egress and delivery assurance** — F3, F8, F9, F12, F13. It was
the most productive branch in the run and it produced the best findings. But it means the
open set is **not the hard residue after broad clearing**. It is mostly unexplored
breadth, and two whole categories have had no ruling at all:

| Category | Open | Touched by any ruling? |
|---|---|---|
| **F2 Fact shape** — Q-05, Q-07 | 2 | **no** |
| **F4 Invariant & rejection** — Q-03, Q-04, Q-22, Q-26 | 4 | **no** |
| F13 Notation & process — Q-01, Q-02, Q-08, Q-18, Q-19, Q-20 | 6 | no (the F13 questions *raised* by rulings were answered; the original six were not) |
| F5, F6, F7, F10, F11, F12 (originals) | 9 | partly |
| Ruling-raised, still open | 11.5 | n/a |

**The single largest gap in the whole run is in an untouched category.** Q-07: *no field of
any fact is declared anywhere*. That is why `DSC-0002` — a `checked` allocation with an
`operational` closure and a named analyser — currently validates this session's inventions
against themselves.

---

## 3. Why the count has not fallen, precisely

Eleven rulings closed 13.5 questions and raised 20. But the rulings were not all the same
kind, and the difference is the whole lever:

| Kind | Rulings | Effect |
|---|---|---|
| **Instance** — settles one point | R-Q10, R-Q06, R-Q37, R-Q39, R-Q12 | closes 1, raises 1–3. Net roughly flat. |
| **Class** — settles a *rule for making decisions* | **R-GROUND**, R-Q38, R-Q41, R-Q45 | closes 1, raises 1–2 — **and retroactively reframes questions nobody asked about.** |

R-GROUND is the proof. One sentence — *"we cant add decisions to ground we havent
modelled"* — deleted a check, and **reframed five entries in the §11.4 table at once**,
turning "five rules that turn on undefined terms" into "five rules conditioning on
unmodelled ground", which says what to do next. It was asked as a comment on one regex.

R-Q38 did the same in miniature: *absence must be stated and attributed* is now visible as
the same device in four places.

**So the way to the bottom is to stop answering instances and start answering the
generative rule behind a family.** Four such rules would collapse most of what is open.

---

## 4. Four class-level moves, in the order I would take them

### Move 1 — **Declare the fact type space.** Closes ~6 questions, retires ~7 decisions.

The fact vocabulary declares a type *space*, not types. Nothing anywhere says a `Cart` has
lines or an `OrderPlaced` has an id. Declaring them closes or reshapes:

| | |
|---|---|
| **Q-07** | directly — and makes DSC-0002 dischargeable for the first time |
| **Q-05** | the account currency DSC-0005 compares against gets a home |
| **Q-03** | *"invariants the fact vocabulary declares"* becomes satisfiable rather than vacuous — the profile rule stops being unsatisfiable |
| **Q-26** | an invariant can be named where it is stated |
| **Q-22** | the `Rejected` shape follows from what an invariant is |
| **Q-09** | payload validation has something to validate against |
| retires | D-04, D-05, D-06, D-07, D-08, D-09, D-11 — every invented field |

**This is the highest-yield single action available and it has had no ruling.** It also
unblocks the only genuinely unsatisfiable rule in the profile (Q-03).

### Move 2 — **Apply R-GROUND to the whole profile.** Closes ~6.

R-GROUND was applied to one rule. Applied to the profile as a set, the five rules turning
on *domain state*, *persistence type*, *I/O*, *a decision* and *derived from* each need
their ground modelled before "can Roslyn check it?" is well-posed. Reaches Q-25, Q-27,
Q-28, Q-29, Q-21 and Q-43 (*should a `known_divergence` be reachable?* is R-GROUND asked
of divergences).

### Move 3 — **Rule on `allocation.class`: discharge or existence?** Closes ~2, governs many.

Q-04 asks whether `residual` describes how a determination is *discharged* or whether it is
*built at all*. This session read it as discharge and implemented DSC-0005's currency
rejection on that reading. **Three of the six determinations in the store are `residual`.**
If the reading is wrong, behaviour comes out of the slice. It is one sentence and it
governs every residual determination ever filed.

### Move 4 — **Individuation: how are acts, roles and profiles identified and selected?** Closes ~5.

One family, currently scattered: Q-47 (how is a profile selected, now that R-Q45 makes
selection depend on assurance), Q-41 (is the relay an act in the vocabulary), Q-33 (where
does role exhaustiveness stop), Q-34 (how is a provider matched to its fact), Q-32 (one
provider per position or per store). Plus the **DSC-0100 supersession** both R-Q10 and
R-Q45 now require — its `statement` and its `extent` are both wrong, and one supersession
should carry both.

---

## 5. The cheap batch — answer in one sitting, no downstream

Q-13 (route and verb), Q-14 (order identity), Q-23 (idempotency), Q-24 (`[Slice]` supplied
or authored), Q-30 (who calls the write provider), Q-31 (a decided-but-unwritten act),
Q-12b (the transport result for an act that never reached a verdict), Q-17 (should code
record a live residual), Q-46 (does `delivery_assurance` reach reads).

Nine questions. None of them changes anything structural; each is one line.

---

## 6. The ones this run cannot close, and one that blocks the report

**Q-02 first, and cheaply.** *"Frame category"* is undefined, and both Gate B's
instrumentation and Gate C's table are built on it. This session invented a 13-category
scheme after the build, so **the Gate C comparison against the notation experiment's
independent category list is currently unsound** — a disagreement between the two lists
cannot be told apart from a disagreement about vocabulary. This is a deliverable
dependency, not a design question, and it should be answered before the held session tries
the comparison.

The rest are about the scheme rather than the slice and will not close from inside the
build: **Q-01** (`canon-governance` unreadable under the prohibitions), **Q-08**
(`ranges_over` forced into a false statement by C-3), **Q-18** (a pinned `settled_by`
placeholder against a `[PROPOSED]` profile), **Q-19** (read-enforced rule 1 contradicts the
premise of this exercise), **Q-20** (whether extent has any realisation in code).

---

## 7. A termination criterion, since zero is not one

Proposed, because *"the bottom"* needs a definition or the exercise cannot end:

> **Stop when every open question is either (a) a design decision a competent team would
> have to make in any notation, or (b) a question about the scheme rather than about this
> slice.**
>
> What remains at that point is not a specification gap. It is ordinary engineering, plus
> the scheme's own open items.

Applying it to the 35.5 now open:

| | Count | |
|---|---|---|
| **Genuine specification gaps** | **~10** | Q-03, Q-05, Q-07, Q-09, Q-22, Q-26, Q-42, Q-46, Q-47, Q-12b |
| Ordinary design in any notation | ~14 | Q-13, Q-14, Q-15, Q-21, Q-23, Q-24, Q-25, Q-27, Q-30, Q-31, Q-32, Q-33, Q-34, Q-17 |
| About the scheme | ~11 | Q-01, Q-02, Q-04, Q-08, Q-18, Q-19, Q-20, Q-28, Q-29, Q-43, Q-41(half) |

**That is the number worth reporting: 35.5 open, of which ~10 are specification gaps.**
The raw count has been the headline and it overstates the problem. Nine of the ten gaps
fall inside Moves 1 and 4.

---

## 8. What I would actually do

1. **Q-02** — one line, unblocks the Gate C comparison.
2. **Move 1** — declare the fact type space. Highest yield, untouched, unblocks Q-03.
3. **Move 3** — one sentence on `allocation.class`, governs every residual.
4. **The cheap batch** — nine one-liners in one sitting.
5. **Move 4** — individuation, carrying the DSC-0100 supersession.
6. **Move 2** — R-GROUND across the profile, which is a modelling exercise rather than a
   ruling and is the natural point to stop asking and start specifying.

Expected after 1–4: roughly **10–12 open**, nearly all in categories (b) and (c) of the
criterion above. That is the bottom, as defined.

**One caution about method, from this run's own record.** The eleven rulings went
depth-first down one branch and the marginal yield fell as they went: R-Q10 exposed a
structural hole, R-Q45 exposed a narrow one. Meanwhile the branch containing the largest
gap in the run had no ruling at all. **Breadth first, then depth** — the next pass should
take one question from each untouched category before going deep again.
