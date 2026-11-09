# Step 1: pin sdlc edition 1, the lift ADR and the lift map

> **Recorded contemporaneously**, by the session that does the work, under
> ADR-0007 D2, in the shape of Writ's `docs/traceability/` records (Writ ADR
> 0008). The maintainer's messages are verbatim. The transcript is held by the
> maintainer.
>
> **In progress.** This record is committed early, at the maintainer's request
> (Round 4), so that it is durable before the step-1 work starts. The plan is
> not yet approved and Q1 to Q5 are open, so no step-1 file exists yet. The
> sections for the approved plan, the commits, the checks and any deviations
> are added as the work is done, and the record is completed in the step-1
> pull request.

| | |
|---|---|
| **Issue** | [#169](https://github.com/mindovermachine-dev/product-cli/issues/169) |
| **Pull request** | none yet |
| **Date** | 2026-11-06 to 2026-11-09 |
| **Tool** | Claude Code, a cloud session |
| **Model** | not recorded (ADR-0007 D1) |
| **Session identifier** | `session_01NbWSYPPHBsK3xfFSuALfiQ` |
| **Branch** | `claude/sdlc-edition-1-adr-lift-wfgx7s`, from `main` at `4ce90e9c2a28c3a010c95419a8dbb88dbcfba595` |
| **Identity** | `Claude (Emil) <claude+emil@okkels-klein.dk>`, SSH-signed with key `SHA256:xDu5EyJd05ROum1g+8Aj1Up3MjE6RbHnI70/xNHljDY` |
| **Definition** | decision-driven-design `1ab1de005eb60ce2ea019e9942cd52b790aab0e7` (`v0.6.0`), Draft 4 |

## The prompts

The first message is the body of #169, verbatim:

> This issue is the session brief for step 1 of the order of work (definition, chapter 7). The session posts its plan as a comment here and waits for approval.
>
>
> **Do not start until all of these hold:**
> - decision-driven-design release `v0.6.0` is cut and its tag verifies. It ships `sdlc:` edition 1, with edition hash `sha256:a2006d9da588586bbf7ac19b3eec4861ff5a1aec0006a61784e9e0d842672f29` unless the release's descriptor says otherwise; the descriptor wins. The rulings are DDD-dec-113, from mindovermachine-dev/decision-driven-design#49, mindovermachine-dev/decision-driven-design#50 and mindovermachine-dev/decision-driven-design#59;
> - mindovermachine-dev/decision-driven-design#52 (the definition, Draft 4) is merged;
> - the tag `rust-final` exists at `70395df` in `mindovermachine-dev/product-cli`;
> - the first rewrite pull request has removed the Rust tree from `main` (D1.24);
> - slice 0 is merged (D1.25): the eight layer projects, the layering analyzer, the AOT gate and the package register. Step 1 adds its projects and tests inside that skeleton and changes none of its gates.
>
> If any of these does not hold, stop and say which one.
>
> ## Read first
>
> In this order:
> 1. decision-driven-design `applications/sdlc/product-cli/product-definition.md`, Draft 4: chapters 4, 5 and 7, and D1.1, D1.3, D1.16 to D1.21 and D1.24.
> 2. The accepted ADRs, `ADR-0001` to `ADR-0012` (ADR-0007 for repository discipline).
> 3. decision-driven-design at the tag `v0.6.0`:
>    - `applications/sdlc/what.md`, in full, including §2.3 and the appendix "where the source went";
>    - `what/ontology.ttl` and `what/shapes.ttl`;
>    - `core/decisions/DDD-dec-63.yaml`, `DDD-dec-64.yaml`, `DDD-dec-72.yaml`, `DDD-dec-91.yaml` (which supersedes 72), `DDD-dec-73.yaml` and `DDD-dec-113.yaml` (the edition-1 rulings: the regime, the severities, the five shapes, E-7 and E-8).
> 4. The lift input, read only:
>    - `mindovermachine-dev/product-cli` at the tag `rust-final` (`70395df`): `.product/products/product-cli/`;
>    - `schema/ontology/product-framework.ttl`, for the `pf:` terms.
>
>    Read the Rust crates for behaviour only. Copy no line.
> 5. `AGENTS.md` in this repository.
>
> ## Plan first
>
> Post a plan as a comment on this issue and wait for approval. The plan names:
> - each file;
> - each ADR number;
> - the shape of the lift map;
> - the wire-name rule you will propose;
> - the tests.
>
> Do not write files before approval.
>
> ## Deliverable 0: bring the definition up to date (decision-driven-design, its own PR, first)
>
> Before the product-cli PR, open one PR in decision-driven-design that changes only `applications/sdlc/product-cli/product-definition.md`. Write it as one signed commit with `Refs #49`, `Refs #50` (issues in decision-driven-design) and `Basis: DDD-dec-113; DDD-dec-73; DDD-dec-72`. `python3 scripts/ci.py` must pass. Emil lands it with `canon`, and the product-cli PR cites its landed commit.
>
> The status changes:
> - **Line 174 (D1.19):** `(decision-driven-design #49, to be ruled there)` becomes "(ruled in `DDD-dec-113`, E-1 and E-2; `what.md` §7.2)". After the bullet list, add: "A requirement added to an edition after an example is pinned enters as `sh:Warning`, and may become a violation only in a later edition (`DDD-dec-113`, E-2)."
> - **Line 282 (chapter 5):** `(decision-driven-design #49 and #50)` becomes "(`DDD-dec-113`)".
> - **Line 380:** "at the edition tag" becomes "at `v0.6.0`".
> - **Line 394, order of work step 1:** `This needs decision-driven-design #49, #50 and the edition-1 tag` becomes "Edition 1 shipped in decision-driven-design `v0.6.0` (`DDD-dec-113`)".
> - **Line 443 (chapter 8):** the sentence about §3.7's `[OPEN]` checks becomes: "They are in edition 1 as warning shapes (`DDD-dec-113`, E-4), and are built here only by importing that edition."
>
> The M-12 correction, per `DDD-dec-113` E-3, which keeps `DDD-dec-73`:
> - **Line 194, "Element hashes":** the last sentence becomes "Its vectors over decision-driven-design's examples are offered there as a further cross-check. M-12's second implementation stays Varve's published vectors (`DDD-dec-73`)."
> - **Line 457, the table row:** "product-cli's element-hash vectors, offered as a cross-check beside Varve's (`DDD-dec-73`, `DDD-dec-113` E-3)", with state "to deliver at step 2".
>
> The table:
> - "As of 2026-11-03" becomes "As of 2026-11-06".
> - The `decision-driven-design #49` row: **met**, ruled in `DDD-dec-113` (E-1, E-2), landed by mindovermachine-dev/decision-driven-design#59.
> - The `decision-driven-design #50` row: **met**, `DDD-dec-113` (E-4).
> - The edition row: **met**, `v0.6.0`, tag object `bb47a720832c77f6a53c82b90388cd4a2c5c23bf` on `1ab1de005eb60ce2ea019e9942cd52b790aab0e7`, edition hash `sha256:a2006d9da588586bbf7ac19b3eec4861ff5a1aec0006a61784e9e0d842672f29`.
> - Add a row: D1.25, slice 0, product-cli, **met**, `4ce90e9`.
>
> Change nothing else. If a line has moved, find it by its text and say so in the PR.
>
> ## Deliverables (one PR in product-cli, after deliverable 0 lands)
>
> **1. ADR: pin `sdlc:` edition 1.**
> - **The pin:** `owl:versionIRI https://decision-driven-design.dk/ns/sdlc/1`, the tag and its commit, and the file hash per `DDD-dec-72` as carried by `DDD-dec-91`: sha256 over `ontology.ttl` then `shapes.ttl`, bytes as tagged.
> - **The validation regime, as ruled in DDD-dec-113 (E-1, `what.md` §7.2):** the data graph is `what ∪ ontology`, SHACL Core with SHACL-SPARQL, and no entailment.
> - **The severity mapping:** a violation is refused at write (W1, W5), and a warning is a finding (W3, W4).
> - **The embedding (A5):** the two files are embedded byte for byte, and the build fails if their hash differs from the pin.
> - **Alternatives:** fetch at runtime, and vendor an edited copy, each with why it is refused.
> - **Consequences:** a new edition is a new ADR that shows this edition's exports still import, validate and diff (constraint 8).
>
> **2. ADR: the lift (D1.4, ADR-0009).**
> - **The frozen input:**
>   - repository `mindovermachine-dev/product-cli` at the tag `rust-final` (`70395df`);
>   - command `product domain export --product product-cli`;
>   - RDFC-1.0 graph hash `sha256:ad855b900abd76bdc8a26d91ce995e267856dad8c47d15e1ee509f41c5d07e06`, with 1,412 triples.
>
>   Bytes are not the pin: Rust's Turtle is not byte-stable across builds.
> - **The mapping of old Framework sections to `what.md` sections,** once, taken from `what.md`'s appendix.
> - **The term rules.** Every `pf:` class and predicate in the input is either mapped to an `sdlc:` term or dropped by name with a reason. The How kinds are dropped: `ReificationRule`, `Cio`, style and tokens, `versionedAs`, `targetsPlatform`. `inContext`, `ownsDomain`, `ownsSystem`, `systemOf`, `journeyOf` and `boundedBy` become `sdlc:within` (P-2).
> - **The element rules:**
>   - each IRI subject gets one minted `P/what/<ULID>`, keyed in the map by its old IRI;
>   - the 32 blank nodes become elements (P-3), each keyed in the map by (owning subject IRI, predicate, the node's own description, sorted). That key was checked unique over all 32 at the frozen graph hash, and the ADR states this, so the map needs no blank-node labels:
>     - the 15 `pf:hasAttribute` nodes become `sdlc:Field` elements: `attrName` becomes `rdfs:label`, and `attrType` becomes `sdlc:fieldType`;
>     - the 7 `pf:offers` and the 10 `pf:surfaces` nodes, all on wireframe steps, become `sdlc:Interaction` elements within their UI step. They keep `command` or `projection`, and `typedAs` (`what.md` §3.2.1);
>   - the product IRI P is chosen and stated.
> - **The wire-name rule.** The input carries none. State the rule that proposes them for every kind that requires one (`what.md` §6, R-1, R-2). Review adopts each, and the adopted name is stored in the map with a reference to the review that adopted it. Lifting again therefore reproduces the golden bytes, wire names included.
> - **The two fixtures:**
>   - the self-hosted What, for structure and surface;
>   - decision-driven-design's `acme-shop-5.ttl` and `vectors-5.json` at `v0.6.0`, for behaviour (A6, as moved to `-5` by mindovermachine-dev/decision-driven-design#59). It carries no findings, so the fixture test asserts zero violations and zero warnings.
>
> **3. The lift map**, `lift/map-1.json`, committed and never regenerated:
> - the term table;
> - one row per input element, by the keys above: key, new IRI (the minted ULID, stored rather than minted on each run), label, and wire name with its adoption reference;
> - the drop list.
>
> ULIDs are minted once, here, and stored. The lift reads them from the map and never mints. The map is input to step 3 and is not used by the product.
>
> **4. Tests (C#, observing bytes, ADR-0006):**
> - the embedded edition files hash to the pin;
> - the lift map covers the frozen input: every subject and every `pf:` term is mapped or dropped, and nothing in the map is absent from the input;
> - every minted IRI matches the identity shape's pattern;
> - wire names are unique per kind, and within each owner for fields.
>
> These tests read the input by its frozen hash. A changed input fails them.
>
> **Stubs:** none expected. List any you need in the PR.
>
> ## Rules
>
> - Commits:
>   - signed, as the agent identity, with no model identifier in any commit or record;
>   - `Refs #N` and a DCO sign-off;
>   - a traceability record for the session.
> - No suppression of any analyzer or rule.
> - Package versions come from nuget.org, never from memory. Each new package needs an ADR with an `Adr=` entry in the register.
> - Nothing is written to Varve or Writ. In decision-driven-design, deliverable 0 is the only change. Any other gap found there is an issue in that repository, linked from the PR.
> - ADRs are files under the interim rule (A7) until Hafeok/writ#7 exists.
>
> ## Done when
>
> - The PR is green.
> - Both ADRs are accepted.
> - The map is reviewed, and its wire names are adopted.
> - The release descriptor names the edition pin and the lift input hash.
>
> ## Reconciliation
>
> If another session lands first:
> - merge `main`;
> - re-run the tests;
> - if that session touched the edition pin or the map, stop and ask.
>
>
> ---
> _Generated by [Claude Code](https://claude.ai/code)_
The session read what the brief lists, checked its preconditions (all held),
and posted its plan with questions Q1 to Q5:
[#169 comment 6685760600](https://github.com/mindovermachine-dev/product-cli/issues/169#issuecomment-6685760600).

### Round 1 (2026-11-09)

> check the open PRs and main and see where our work fits in

### Round 2 (2026-11-09)

> Yes, post a short comment on #169: the shift to ADR-0015 (edition pin), ADR-0016 (lift) and
> v0.8.0-preview.4 because #171 takes ADR-0014 and preview.3; that you take whichever numbers are
> free when you write the files; and that you rebase onto #171 and record the reconciliation
> (ADR-0007 D5). Nothing else on #169 until I answer Q1–Q5. Deliverable 0 in decision-driven-design
> can go ahead as its own PR.

### Round 3 (2026-11-09)

> Two things. Record this message verbatim in your rulings record as your next round.
>
> 1. #64 has landed: decision-driven-design main is now a53d02d. #65 is still based on 1ab1de0, so canon cannot fast-forward it. Rebase claude/sdlc-edition-1-adr-lift-wfgx7s onto main. #64 does not touch applications/sdlc/product-cli/product-definition.md, so expect no conflict; if there is one, stop and tell me. Push, wait for green CI, and report the new head. I will approve that head on #65. Record the rebase as ADR-0007 D5 asks.
> 2. A separate, small pull request in decision-driven-design, from a fresh branch off main. Keep it out of #65. It adds two open items to applications/sdlc/sdlc.md §19, as open items only. Neither rules on its substance. You have already read product-cli#171 (ADR-0014), so you are the right session for it.
> O-8. An act that accepts the realisation plan. o1b plans the realisation on the issue, but no act accepts the plan before work starts; o10 accepts only the candidate, after the work. product-cli's issue sessions have one (ADR-0014 D5): a run posts a plan and stops, and works only after the principal approves that plan comment on the issue. It is cheap judgment placed before the expensive acts. Proposed: a plan-acceptance act between o1b and o4, by the How owner or a holder of its grant, whose verdict is in o4's ground reading. Say where it would sit in the record and in the meters. Closes it: Emil.
> O-9. Reconciling product-cli's issue sessions with the SDLC. product-cli specifies issue sessions on its own (ADR-0014, #171): a `claude` label by one hard-coded maintainer, plan approval, landing by a maintainer fast-forward under its ADR-0007, and a traceability record per session. The SDLC uses an allocation label checked against the identity map, `approve <head sha>` on the candidate, the gates App's fast-forward, and a record per act occurrence. product-cli is the natural first adopter. product-cli either adopts the SDLC (ADR-0014 superseded or mapped onto it) or records its departures. Closes it: Emil, then product-cli.
> First read §19 and Rounds 27 to 30 in meta/sessions/2026-11-07-sdlc-and-pdlc/rulings.md (now on main). If either item is already covered, say so and add only what is left. Record this message in that session's rulings.md as its Round 31, since §19 belongs to that record. Basis: DDD-dec-115. Push, wait for green CI, open the pull request, report its number and head, and stop. Do not land.
>
> In the product-cli plan, a separate thing, note that the SDLC is now on main (DDD-dec-115, applications/sdlc/sdlc.md). #171's issue sessions and the SDLC differ (O-9). Do not reconcile them in your PR; just name it as an open item.

### Round 4 (2026-11-09)

> Commit the traceability draft now, on the step-1 branch in product-cli, at the path the
> traceability file for #169 will have. Push the branch, but open no pull request. Then stop.

### Round 5 (2026-11-09)

> #65 has landed: decision-driven-design main is now e037d8e. Rebase #66's branch onto main.
> #65 touched only applications/sdlc/product-cli/product-definition.md, so expect no conflict;
> if there is one, stop and tell me. Push, wait for green CI, and report the new head for my
> approval. Add the rebase to the #169 traceability record on the step-1 branch in product-cli
> (ADR-0007 D5), push that too, and stop. Do not land.

## What was done

- **Round 2.** The numbering comment on #169:
  [comment 6736680629](https://github.com/mindovermachine-dev/product-cli/issues/169#issuecomment-6736680629).
  Deliverable 0 opened as
  [decision-driven-design#65](https://github.com/mindovermachine-dev/decision-driven-design/pull/65):
  one signed commit, changing only `product-definition.md`, with exactly the
  brief's edits; `scripts/ci.py` passed. The first push carried a wrong
  `Claude-Session:` link in the commit message; the commit was amended and the
  branch force-pushed with a lease within a minute, before the pull request
  existed.
- **Round 3, item 1.** #65 rebased onto `a53d02d` (below); new head
  `e037d8ea642b3c353048ece0b1c53a2556cb2f4e`, both `validate` runs green.
- **Round 3, item 2.**
  [decision-driven-design#66](https://github.com/mindovermachine-dev/decision-driven-design/pull/66),
  from `claude/sdlc-open-items-o8-o9` off `a53d02d`, head
  `b79d32641e9564b43fccd7aea2e4305e65438ebd`: O-8 and O-9 in `sdlc.md` §19 as
  open items, and Round 3's message as Round 31 of that session's `rulings.md`.
  The first head, `681704c`, failed CI's `issue-refs` gate (no same-repository
  `Refs #N`); amended with `Refs #61`, the session's issue, and green. Not
  landed.
- **Round 4.** This record, committed on the step-1 branch and pushed. No pull
  request.
- **Round 5.** #65 landed: decision-driven-design `main` is at `e037d8e`, the
  head this session reported. #66 rebased onto it (below); this record updated
  and pushed. Not landed.

## Reconciliations (ADR-0007 D5)

- **2026-11-09, decision-driven-design#65 rebased onto `a53d02d`.** #64
  (`DDD-dec-114`, `-115`) landed while #65 was based on `1ab1de0`.
  `claude/sdlc-edition-1-adr-lift-wfgx7s` was rebased (`577ef5f` →
  `e037d8e`). No conflict: #64 does not touch `product-definition.md`, and the
  diff against `main` is unchanged (one file, 14 lines added, 11 removed).
  `scripts/ci.py` was re-run on the rebased head: all gates passed. Pushed with
  a lease against `577ef5f`.
- **2026-11-09, decision-driven-design#66 rebased onto `e037d8e`.** #65 landed
  while #66 was based on `a53d02d`. `claude/sdlc-open-items-o8-o9` was rebased
  (`b79d326` → `80e90882bffa8ea3dfa67b449fdc6b575cf21b6d`). No conflict: #65
  touched only `product-definition.md`, and #66's diff against `main` is
  unchanged (`sdlc.md` 2 lines added, `rulings.md` 30 lines added). On the
  rebased head, `scripts/ci.py` passed all gates and `canon issue-refs --basis`
  passed. Pushed with a lease against `b79d326`.
- **Pending: #171.** It takes ADR-0014 and `v0.8.0-preview.3`. This step's
  ADRs become ADR-0015 (the edition pin) and ADR-0016 (the lift), and its
  release `v0.8.0-preview.4`, or the next free numbers when the files are
  written. The step-1 pull request rebases onto #171 and records it here.

## Open items for the step-1 plan

- **The SDLC is on decision-driven-design `main`** (`DDD-dec-115`,
  `applications/sdlc/sdlc.md`). #171's issue sessions and the SDLC differ:
  decision-driven-design `sdlc.md` §19, O-9 (#66). The step-1 pull request
  does not reconcile them; it names this as an open item only (Round 3).
- **Q1 to Q5** on #169 are open. Q1 blocks the lift map.
