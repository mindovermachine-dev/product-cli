# Rulings CG-R-134 … CG-R-137 — Gate 3 answering method

**Issued by Emil, 2026-09-15.**

---

## CG-R-134 — Answer the class, not the instance. Breadth before depth.

Both observations are accepted and both generalise past this session.

**Instance rulings net out flat.** Five of them closed one and raised one to three apiece. Four class rulings closed one and raised one to two — and reframed questions nobody had asked. R-GROUND is the proof: one sentence deleted a check and turned *five rules with undefined terms* into *five rules conditioning on unmodelled ground*, which says what to do next. It came out of a remark about a regex.

**Ruled: a question is answered at the level of the rule that generates it.** Where an instance answer would leave the family intact, the family is what gets ruled. This is the discipline the programme has used for its own rulings without stating it — CG-R-116, CG-R-77 and CG-R-120 are all class rulings extracted from single findings — and it now governs the answering of questions too.

**And breadth before depth.** Eleven rulings on one chain while F2 and F4 took none, and F2 holds the largest gap in the run. **One question from each untouched category before going deep again**, as proposed.

The self-diagnosis is what makes this rulable. A session that reported eleven rulings without noticing they were one chain would have produced the same artefact and no finding.

---

## CG-R-135 — The three-way partition is the headline. The raw count is retired.

**Accepted:** stop when every open question is either (a) a design decision a competent team would make in any notation, or (b) a question about the scheme rather than this slice.

That is a termination criterion and zero was never one. A specification that answered every question a builder could ask would be the implementation.

**The partition is the reportable figure:** of ~35.5 open, roughly **ten are genuine specification gaps**, ~14 ordinary design, ~11 about the scheme.

**Ruled: the raw count does not appear as a headline anywhere.** It has already been reframed once — 29 was question generation, not clarification count (CG-R-132) — and this reframes it again. Both reframings run the same way: **the raw count overstates the problem**, and reporting it would misrepresent the run in the direction of a worse result than was found.

Nine of the ten gaps falling inside moves 1 and 3 is the useful statement, because it is actionable.

---

## CG-R-136 — Q-02: *frame category* was never defined. My omission, and it voids a figure.

The Gate 3 prompt says *per frame category* in three places and defines it nowhere. The session invented a thirteen-category scheme after the build, which is the only thing it could do.

**This is CG-R-63 in its seventh instance**, and the instance count is now the finding: a measurement whose denominator is defined after the measurement is not a measurement, and I have shipped that defect repeatedly.

**Ruled:**

- **The *3 of 13 settled* figure is void.** It is over a denominator invented after the fact and it is not partially salvaged.
- **The Gate C comparison against the notation experiment's category list is deferred, not repaired.** Inventing a frame now to compare against would make a disagreement between the two lists indistinguishable from a disagreement about vocabulary — which the session states exactly. The comparison waits for the notation list to be ratified, which waits on the reconciliation, which is still held.
- The thirteen categories the session invented are **retained as its working scheme**, labelled as invented-after-build, and are not presented as a frame.

---

## CG-R-137 — The four moves, ordered. Move 1 is mine.

**1. Declare the fact type space.** Highest yield available, closes six questions, retires seven invented decisions, and makes the profile's one unsatisfiable rule satisfiable. **No field of any fact is declared anywhere** — which is why DSC-0002 validates inventions against themselves, and it is the largest single gap in the run.

This is mine. The fact vocabulary is the domain model, I authored `ordering.eventmodel.yaml`, and the fields belong with it. I will produce it.

**2. `allocation.class` — discharge or existence.** I rule the principle and need the readings before ruling the instance.

> **`allocation.class` describes how a determination is held, not whether it has been acted on.** `residual` means nothing in the specification settles it and a named actor carries it — recording it as residual does not discharge it. `pinned` means it is settled; `checked` means a predicate settles acceptability.

Whether DSC-0005's currency rejection belongs in the slice turns on its class and on which reading the session took, and I will not rule that from a summary. **State both readings and what each does to the slice**, and it is one sentence back.

**3. Individuation** — how acts, roles and profiles are identified and selected. Accepted as a family, and it carries the DSC-0100 supersession that R-Q10 and R-Q45 both now require. That supersession is mine and lands with the profile repair.

**4. R-GROUND across the whole profile** — correctly named as a modelling exercise rather than a ruling, and the natural point to stop asking and start specifying.

**The nine one-liners go in one sitting**, after move 1 and before move 3.

---

Register debt: CG-R-17 … CG-R-137.
