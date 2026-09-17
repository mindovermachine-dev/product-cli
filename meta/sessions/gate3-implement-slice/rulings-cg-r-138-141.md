# Rulings CG-R-138 … CG-R-141

**Issued by Emil, 2026-09-15.**

---

## CG-R-138 — Reading B. And DSC-0005 is malformed, not merely misclassified.

**Ruled: Reading B.** The rejection comes out; D-07, D-16, D-18, D-20 and the two tests retire with it; `CartNotEmpty` remains as the act's only rejection.

**But the choice was not between two readings of a well-formed record.** Compare the two residuals:

| | statement | |
|---|---|---|
| DSC-0003 | *who may place an order is not settled at this address* | names **what is unsettled** |
| DSC-0005 | *an order … is rejected rather than converted* | asserts **an answer** |

A residual's allocation says nothing in the specification settles the matter. DSC-0005's statement settles it. **The record contradicts itself**, and neither reading rescues that — Reading A resolves the contradiction in favour of the statement, Reading B in favour of the allocation, and the record should not have permitted the contradiction to exist.

**The demonstrated consequence is the finding.** A residual whose statement reads as an answer **will be implemented as one**: it was, at a cost of five inventions and two tests. The record produced exactly the failure its class exists to prevent, on real work, which is better evidence than any argument I could have made for the class.

**The two consequences from the record are decisive independent of the principle.**

Under Reading A, `made_by: session:2026-09-02-implement-placeorder` means **a build session authored a settling determination mid-build** — which the Gate 3 prompt forbids in terms. Reading A makes the shipped corpus contain a violation of my own rule; Reading B makes the same record the permitted thing, which is what an act-time residual is for.

And Reading A required modelling `AccountCurrency` to obey a determination that declares no position for it.

---

## CG-R-139 — Two general tests, both extracted from this instance

**A residual's statement names what is unsettled, never what the answer would be.** A proposed answer in a residual is advice wearing a determination's clothes, and a builder will implement it. If a resolution is worth recording, it is worth pinning with a `settled_by` and a principal; if nobody will pin it, it does not belong in the statement.

Read-enforced — no schema can tell an assertion from a description. **But the shipped example carried a malformed record**, which is worse than a schema gap: it is the reference doing the thing the class forbids, and every reader of the binding has seen it.

**And the ground test, which is R-GROUND at the level of one record:**

> **If obeying a determination requires authoring ground the determination does not declare, the determination is not settled.**

That is mechanical enough to be useful and it would have caught DSC-0005 before the build. The session had to invent `ActorIdentity.AccountCurrency` in order to comply, and the invention was flagged contested at the time — the signal was there and there was no rule to read it against.

---

## CG-R-140 — The baseline stands uncorrected

**Ratified as judged.** `gate-c-report-baseline.md` is the historical record of what was reported before any answer arrived, and correcting it would destroy the evidence that the defect was shipped. A note saying the figures are void and why they are not being fixed is the right disposition.

This is supersession-never-rewrites applied to a report rather than to a claim, and it is the correct reading of it.

---

## CG-R-141 — A reporting bias, and it runs opposite to CG-R-133's

**Self-reported and worth ruling because it is the mirror of one already on the record.**

29, then 49, and both reframings ran the same way: **the count overstated the problem.** Meanwhile CG-R-133 found expectation lists blind to defects of commission — under-reporting what is wrong with the artefact.

| | direction |
|---|---|
| expectation lists (CG-R-133) | **under**-report defects in the artefact |
| count headlines (CG-R-141) | **over**-report how badly the run went |

**The reporting bias is the conservative one** — a session overstating the difficulty of its own work is the opposite of flattery, and it is the direction to prefer if a bias must exist. It still distorts, and the partition exists to correct it.

**One qualification the session made itself:** the chain observation of CG-R-134 was prompted, not volunteered. **A self-diagnosis that required prompting is weaker evidence of self-correction than a volunteered one**, and recording which is which is what keeps the distinction usable.

---

Register debt: CG-R-17 … CG-R-141.
