# Fixes for the verification findings

Session of 7 October 2026. It carries out rulings 49 to 60 (`ledger/rulings/verification-rulings-2026-10-07.md`), the principal's answers to section 14 of `ledger/sessions/2026-10-verification.md`. That is ten fixes to the code and one amendment to the text, each in its own pull request.

The repository had moved only by #114: `main` was at `e20fadc`, the merge of #114 onto `55c3bcf`, as the prompt assumed. Nothing was stopped. Every pull request is green locally and branches from `main`, so they can merge in any order. Merging is the principal's.

## 1. Issues and pull requests

One issue per code step, from the matching row of section 15 of the verification report. Steps 2 and 3 each match two rows, and each has one issue covering both. No issue was opened for the export-only verifier.

| Step | Ruling | Issue | Pull request | What it lands |
| --- | --- | --- | --- | --- |
| 0 | 49 to 60, 51 | — | #115 | This file, the rulings file, and the amendment of LP-9.14 and LP-9.15 |
| 1 | 49 | #116 | #126 | A duplicate acceptance id, and a decision identity object filed more than once, are `SCHEMA` |
| 2 | 50 | #117 | #127 | A policy whose accept role is the genesis role, and a genesis role with a decision capability, are `SCHEMA`; `init --namespace` refuses such a root role |
| 3 | 55, 56 | #118 | #129 | A plain float in hashed content, and an explicit null in a required string field, are `SCHEMA`; one Appendix C note |
| 4 | 52 | #119 | #130 | A sidecar on an act no policy governs is `SCHEMA` |
| 5 | 53 | #120 | #131 | A `rotate` verifies only against the key it closes; the writer signs it with that key |
| 6 | 54 | #121 | #136 | A forked decision has no latest version; `G004` is its one finding |
| 7 | 57 | #122 | #132 | `L006` judges an escape's acceptor on every version |
| 8 | 58 | #123 | #133 | A landed `format:` declaration is compared across history under `L007` |
| 9 | 59 | #124 | #134 | A `set:` grant scope accepts dots |
| 10 | 60 | #125 | #135 | `parents` is part of the header entity |

#128 is not part of this session. Another session opened it, the namespace-independence design, while this one ran.

## 2. Each step

Every code step was checked in three ways:

- **Negative check.** The new test was run with the fix removed (the call commented out, or the old line restored) and failed. It passes with the fix.
- **Gates.** With the git identity variables unset: `cargo clippy -- -D warnings -D clippy::unwrap_used` clean, `cargo t` green, and `ledger verify --export` on the repository byte-identical to `main`.
- **Verdicts.** No committed store or fixture changed verdict.

`cargo t` on `main` passes 2,083 tests with 2 ignored. Each step's total is below.

### Step 1: duplicate ids (ruling 49), #126

- **Change.** `authority::references::duplicate_ids` also counts `acc:` ids, and counts decision identity objects separately ("filed twice — a decision is introduced in one change-set only").
- **Tests.**
  - `ledger-core/src/verify/duplicate_tests.rs`: the report's cases 3a (each half) and 3b, plus a conformant control.
  - `ledger-cli/tests/legacy_revocation.rs::an_acceptance_id_reused_in_an_ungoverned_namespace_is_a_schema_fault`: case 3c, the authority case. A legacy revocation by an ungranted actor, under a reused id in `open.ns`, reached a signed acceptance in a governed namespace. It was conformant and is now `SCHEMA`.
- **Fixture changed.** `testkit::changeset` filed one decision identity object per version, so two versions of one decision filed it twice. It now files one per decision.
- **Verdicts.** None of the committed stores changed.
- **Protocol.** LP-5.11 and LP-8.8, plus an Appendix B row.
- **`cargo t`:** 2,088 passed.

### Step 2: the genesis role (ruling 50), #127

- **Change.**
  - The new `verify/genesis_role.rs` reports two `SCHEMA` faults. One is a policy whose `accept_role` is the role of the genesis grant live as of the policy (`Authority::as_of` at the policy's position, the grant `A006` judges the policy's author against). The other is a role a genesis grant names that carries any of `Capability::DECISION`.
  - Bootstrap refuses an existing root role that carries a decision capability. The check moved to `Role::genesis_refusal`, which kept `author/authority_ops.rs` under 400 lines.
- **Tests.**
  - `verify/genesis_role_tests.rs`: section 2, section 13.2 for each decision capability, and a control.
  - `ledger-cli/tests/authority.rs`: the f02 store, the f02b writer refusal, and the test that no agent identity and no non-interactive session produce an acceptance.
- **Test changed.** `verify/authority_tests.rs::steward()` built a genesis role with every capability. It now uses `Capability::ROOT`, and its emission test looks for `grant-role` instead of `accept-decision`.
- **Verdicts.** None of the committed stores changed: none holds a role or a policy.
- **Protocol.** LP-6.17, with the definition of the genesis role at a position and D9 (f) extended to the writer; LP-8.16; an Appendix B row.
- **`cargo t`:** 2,089 passed.

### Step 3: floats and nulls (rulings 55, 56), #129

- **Approach.** The new `scalars.rs` reads each change-set a second time as an untyped `serde_yaml::Value`. There a quoted scalar is always a string, and a plain one is resolved: floats come back as `Number` and nulls as `Null`.
  - No dependency was added; serde_yaml is already the loader.
  - A custom deserializer for every hashed string field was rejected as wider than one module.
  - The unit tests pin serde_yaml to the YAML 1.2 core schema for the spellings that matter: `+1.5`, `.Inf`, `.NaN` and `1.` are floats; `inf`, `nan` and `42` are not.
- **What is refused.**
  - A float anywhere in an item of a hashed entity list, which is every list but `decisions`.
  - A null in a required `String` field: a version's `set` and `statement`, a revocation's `reason`, a grant's `role`, a key binding's `namespace`, a policy's `namespace` and `accept_role`.
- **Tests.** `scalars_tests.rs`, and `ledger-cli/tests/scalars.rs`, the report's 9b and 13.1 stores.
- **Verdicts.** `ledger verify` over all 16 committed stores prints identical output with and without the check.
- **Protocol.** LP-3.4, a paragraph after LP-4.18, and **one Appendix C note**. The note says that no digest moves and `CANONICAL_FORM` stays `v1`, and that a file which verified may now be refused. Plus an Appendix B row.
- **`cargo t`:** 2,092 passed.

### Step 4: ungoverned sidecars (ruling 52), #130

- **Change.** In `signing::check::check`, a non-binding entity with no governing policy now reports each of its sidecars as `SCHEMA` on `sig/<file>`.
- **Tests.** `ledger-cli/tests/ungoverned_sidecar.rs`: cases 1a and 1b, plus a control.
- **Verdicts.** None of the committed stores changed: none has a `sig/` directory.
- **Protocol.** LP-4.7, LP-4.30 and LP-8.8, plus an Appendix B row.
- **`cargo t`:** 2,086 passed.

### Step 5: rotate (ruling 53), #131

- **Change.**
  - Verification: a `rotate` is judged against `closed_key_only(…)`, the principal's opening bindings of the closed key only. Closes stay in that set, so a key already closed elsewhere is still caught.
  - Writer: `identity rotate` passes the closed key as the one key that must sign, the slot a self-bound binding uses, and refuses any other configured key.
- **Tests.** `ledger-cli/tests/rotate_key.rs`: the writer refusal, a control, and the prompt's test that a rotate signed by another live key of the same principal fails `L011`.
- **Verdicts.** None of the committed stores changed: none holds a key binding.
- **Protocol.** LP-4.12, plus an Appendix B row.
- **`cargo t`:** 2,086 passed.

### Step 6: forked decisions (ruling 54), #136

- **Change.** A forked decision is absent from `View::latest`. Every reader of it has a stated behaviour, now in the protocol as LP-8.34:
  - the gate classes skip it;
  - it is never awaiting acceptance;
  - `status` and `coverage` list it apart from the seven states, with its tips;
  - `show` gives each tip a screen, marked as one of the tips, with the state that tip's own content and acceptances give it;
  - a batch holds it as one forked member with no version;
  - `show` and `blame` read an acceptance of a tip as live;
  - `diff` reports a settled fork as a tip move from the tips;
  - a merge plan notes a side already forked.
- **Tests.**
  - **Rewritten, as the prompt requires:** `a_forked_chain_is_recorded_and_the_representative_is_order_independent` is now `a_forked_chain_is_recorded_and_has_no_latest_in_any_order`.
  - **New:** `latest_only_classes_skip_a_forked_decision_whichever_tip_is_defective`, cases 5b and 5c.
  - `graph.rs::g004_…` now asserts that the `forked` fixture has exactly one finding, `G004`. It also checks the new `status`, `show` and `coverage` output; its old status assertion was replaced. `show_tests.rs` wraps `state` in `Some`.
- **Verdicts.** The `forked` fixture still exits 1 on `G004` alone. Its report no longer lists the decision as awaiting acceptance.
- **Protocol.** LP-8.11, the new LP-8.34 (the next free id in section 8), and an Appendix B row.
- **`cargo t`:** 2,084 passed.

### Step 7: escape acceptor (ruling 57), #132

- **Change.** `disposition::model_acceptor` iterates every version for an escape's acceptor.
- **Tests.** `verify/mod_tests.rs::l006_judges_an_escape_acceptor_on_a_version_that_is_no_longer_latest` covers case 8a. The existing escape test is the 8b control.
- **Verdicts.** None of the committed stores changed.
- **Protocol.** LP-8.9's `L006` row and LP-8.10, plus an Appendix B row.
- **`cargo t`:** 2,084 passed.

### Step 8: format history (ruling 58), #133

- **Change.** `verify/history.rs::format_changes` walks a log file's declaration along its first-parent versions, the working tree last. Each change must raise the declaration to exactly `format::needed_for` with `landed::entities` unchanged, or it is `L007` on the file.
- **Tests.**
  - `ledger-cli/tests/format_history.rs`: lowered 3 to 1, raised 1 to 5, changed in the working tree, and unchanged.
  - `revisit_format.rs::a_raise_beside_another_edit_is_not_a_correction`.
  - The #81 replay `a_landed_declaration_may_be_raised_but_an_entity_beside_it_may_not_change` still passes. The repository's own three corrections (`01KZX70EMPA47TBR0PFKX4M32Z`, `01KZX70EQGQCB1B190TS9FZ1A2`, `01KZX70ET1GMR2012XKEP5EWDW`) stay green: `verify --export` is identical to `main`.
- **Test changed.** `immutability.rs::an_acceptance_appended_to_a_landed_file_after_the_close_fails_l011` asserted no `L007`. Its helper `hand::append_into` raises the landed file from 6 to 7 in the same step as the append, which ruling 58 makes `L007` on the declaration. The test now asserts that, and still asserts no changed landed entity and the `L011` it is about. The helper's doc comment says the same.
- **Verdicts.** None of the committed stores changed.
- **Protocol.** LP-3.16, LP-8.30 and `L007`'s row, plus an Appendix B row.
- **`cargo t`:** 2,088 passed.

### Step 9: dots in a set scope (ruling 59), #134

- **Change.** `GrantScope::from_str` validates a `set:` payload with `DecisionSet::validate_id`.
- **Tests.** A unit test in `authority/grant.rs`, and `authority.rs::a_set_scope_with_a_dot_is_granted_and_verifies` (the f11 store).
- **Verdicts.** None of the committed stores changed.
- **Protocol.** LP-3.3, plus an Appendix B row. Section 3.3 already said `set:<set-id>`.
- **`cargo t`:** 2,085 passed.

### Step 10: parents (ruling 60), #135

- **Change.** `landed::entities` puts `parents` in the header entity.
- **Tests.** `landed_tests.rs::parents_are_keyed_with_the_header`, and `ledger-cli/tests/landed_header.rs`: the parent case and the `note` control.
- **Verdicts.** None of the committed stores changed.
- **Protocol.** LP-8.25, plus an Appendix B row.
- **`cargo t`:** 2,086 passed.

## 3. Test-case candidates

Every store built for a negative test, with the requirement it exercises and the class it draws. No manifest is written: approving test cases is the principal's.

| Store (where it is built) | Requirement | Class |
| --- | --- | --- |
| A decision identity object filed again in a second change-set (`duplicate_tests.rs`) | LP-5.11 | `SCHEMA` |
| An acceptance id filed again by another actor (`duplicate_tests.rs`, report 3a) | LP-8.8 | `SCHEMA` |
| 3a plus a legacy revocation of the shared id (`duplicate_tests.rs`, report 3b) | LP-8.8 | `SCHEMA` |
| A governed signed acceptance, its id reused in `open.ns` with an ungranted legacy revocation (`legacy_revocation.rs`, report 3c) | LP-8.8, LP-6.17 | `SCHEMA` |
| A policy naming the genesis role, the genesis role widened to `accept-decision` (`genesis_role_tests.rs`; `authority.rs`, report f02) | LP-6.17, LP-8.16 | `SCHEMA` (two findings) |
| The root role plus each one decision capability, accept role separate (`genesis_role_tests.rs`, report 13.2) | LP-6.17 | `SCHEMA` |
| A root role file carrying `accept-decision` before `init --namespace` (`authority.rs`, report f02b) | LP-6.17 (W) | writer refusal |
| An acceptance attempted piped, and as three agent identities at a terminal (`authority.rs`) | LP-3.7, #71 | writer refusal |
| `statement:` as plain `1.5`, `1.50`, `1e3`, `.inf`, committed with the loader's digest (`scalars.rs`, report 9b) | LP-3.4, LP-4.18 step 7 | `SCHEMA` |
| `statement:` as plain `null`, `~` (`scalars.rs`, report 13.1) | LP-3.4, LP-4.18 step 3 | `SCHEMA` |
| A sidecar that is not a signature, on an acceptance in an ungoverned namespace (`ungoverned_sidecar.rs`, report 1a) | LP-4.7, LP-8.8 | `SCHEMA` |
| The same, the acceptance before the namespace's first policy (`ungoverned_sidecar.rs`, report 1b) | LP-4.30, LP-8.8 | `SCHEMA` |
| A rotate of K1 to K3, hand-filed, signed by the principal's other live key K2 (`rotate_key.rs`, report 6) | LP-4.12 | `L011` |
| A rotate attempted with K2 configured (`rotate_key.rs`) | LP-4.12 (W) | writer refusal |
| A forked chain whose unallocated tip sorts first, then last, by hash (`mod_tests.rs`, report 5b, 5c) | LP-8.10, LP-8.11 | `G004` only |
| An escape priced by `claude@example.com`, then re-allocated by a child version (`mod_tests.rs`, report 8a) | LP-8.9 | `L006` |
| `pass` landed at format 3, then lowered to 1 (`format_history.rs`, report 7) | LP-3.16, LP-8.30 | `L007` |
| `pass` landed at format 1, then raised to 5 (`format_history.rs`, report 7) | LP-3.16, LP-8.30 | `L007` |
| `pass` landed at format 1, raised to 2 in the working tree (`format_history.rs`) | LP-3.16, LP-8.30 | `L007` |
| A reopen edge landed under format 1, then raised to 4 with the header's note edited in the same commit (`revisit_format.rs`) | LP-3.16, LP-8.30 | `L007` (declaration and header) |
| An acceptance by a closed key appended to a landed format-6 file, raised to 7 in the same commit (`immutability.rs`) | LP-3.16, LP-4.13 | `L007`, `L011` |
| A grant scoped to `set:money.rules` (`authority.rs`, report 11) | LP-3.3 | conformant (a positive case) |
| `pass` with a parent appended to the landed header (`landed_header.rs`, report 12) | LP-8.25 | `L007` |

## 4. Anything stopped

Nothing. No fix changed the verdict of `.decisions/` or of a committed fixture, other than the report text of the `forked` fixture (its verdict is unchanged). No stored digest moved. No ruling met code that could not satisfy it as written.

Two choices were made inside the rulings and are stated here for review:

- **Step 2: the genesis role at a policy's position.** I took this as the role of the genesis grant live as of that policy. A policy filed with no live genesis grant as of it (already `A006`) gets no genesis-role finding, since there is nothing to compare against.
- **Step 6: the readers' behaviours.** None of these is ruled beyond "a forked decision has no latest version", so they are my proposals, now stated as LP-8.34. The one to look at:
  - **Acceptance standing.** `show` and `blame` read an acceptance of a tip as `live`, while the gate (`signs_latest`) treats no acceptance of a forked decision as live.
  - **The alternative.** Reading them as `historical` would call an acceptance of a standing tip history.
  - **A tip's screen.** Each tip's `show` screen carries the state its own content and acceptances give it. I first made `Screen.state` optional and left it empty on a tip, but that changes the signature of a public field. The `ddd diff-contracts` gate in CI refuses that without a signed binding, which is the principal's to file, so the field kept its type.

## 5. Anything else found

Listed, not fixed.

1. **Appendix B conflicts.** Every pull request appends a row to the same table at the end of Appendix B. Whichever merges second conflicts there textually; the resolution is to keep both rows.
2. **Appending a later-format entity to a landed file is now always `L007`.** Under ruling 58, a raise must change no entity, and a raise past the current need is refused. So no sequence of commits can append an entity that needs a higher format to a landed file. LP-8.24 still describes appending to a landed file as legitimate, which now holds only when the file already declares a high enough format. The principal may want to say whether that is intended.
3. **Step 3 scans change-sets only.** Set files and role files are not hashed, so they are not scanned. A float in a role's `title` or a set's `title` is still read as text.
4. **Step 1's placement.** The duplicate check for acceptance ids and decision identity objects lives in `authority/references.rs` beside the authority-record check, though neither is an authority record. Its module doc says so. A later tidy-up might move all duplicate checks to one place; `store.rs` holds the set and change-set ones.
5. **The contract-surface gate.** CI runs `ddd diff-contracts` and fails on any new `pub` item in `ledger-core` without a signed binding. Steps 2, 3 and 6 first added public items and failed it on #127 and #129. Their new items are now `pub(crate)`, since they are used inside the crate only, and the gate reports no change, checked locally on each branch before pushing.
6. **A shared `CARGO_TARGET_DIR` across git worktrees reuses stale artifacts.** Cargo decided a workspace crate was fresh by the dep-info of another worktree's build, so one run tested another branch's code. Every result above comes from a run in a target directory of the step's own worktree, with clippy re-run that way for steps 2 to 5. This is a property of the environment, not of the repository, but anyone running several branches side by side should know it.
7. **`ledger verify` notes no `origin/HEAD`** in a fresh clone or worktree ("landing computed on HEAD's own first-parent line"). It did not affect any result here.
8. **The GitHub API reports #114 as closed and not merged**, though `main` carries its merge commit `e20fadc`. The repository was taken as authoritative.
9. **Disk.** The session ran out of its disk allowance once, building many worktrees. Clearing `target/` recovered it.
