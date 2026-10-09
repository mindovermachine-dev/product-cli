# Namespaces, layout and authority: issues 1 to 10

Session of 9 October 2026, on `main` at `70395df` after #142 merged. It
builds issues 1 to 10 of `ledger/prd/namespace-independence-prd.md` §6
under rulings 32, 41 to 48 and 61 to 102, one pull request each, after
ruling 102 was written in. It files no decision, accepts nothing, revokes
nothing, edits `.decisions/` only in issue 10's re-layout commit, and does
not touch `docs/decisions/` or `.ddd/`. Issue 11 (#82) and everything after
it are for later sessions: no `after`, `anchor` or `role_hash`, no signing
of grants, no change to D6 beyond reading per-namespace paths.

## 1. Issues and pull requests

Every pull request is stacked on the one before it, so each diff against
`main` includes its predecessors; the last commit of each is the step. The
order differs from the PRD's (§4): issue 10 was built third, because no
pull request can be green on `ledger verify --export` while the loader
refuses flat paths and the repository's own store is still flat.

| Step | Issue | Pull request | Branch (`claude/great-feynman-zcdsdd-…`) | Commit | What landed |
| --- | --- | --- | --- | --- | --- |
| 0 | — | #143 | `step-00-ruling-102` | `63c0cd24` | Ruling 102 in `ledger/rulings/order-from-the-acts-rulings-2026-10-09.md` ("A question the rulings raised"); PRD §3.5.6, issue 15, §7 ("ruled"); a line for 102 in `ledger/sessions/2026-10-namespace-design.md`. |
| 1 | #144 | #154 | `step-01-layout` | `1e019f16` | `ledger-core/src/layout.rs`; every store loads from `ns/<ns>/{sets,roles,log,sig,allowed_signers}`; a flat path at the verified commit is `SCHEMA` (LP-3.34); ids, sets, roles unique per namespace (LP-3.36); a version names its own namespace's set (LP-5.22); the writer homes each change-set by its one namespace (`author/home.rs`); `declare` and `role declare` take `--namespace`; Appendix C "Spec v1.9 — one directory per namespace". |
| 2 | #145 | #155 | `step-02-legacy-history` | `a07559de` | Flat paths detected on the first-parent history; with the legacy capability (on by default) every tracked file is followed along its lineage, flat and namespaced, for `L007`, LP-3.16's `format:` comparison, `L009` and the base overlay; `verify --no-legacy-layout` refuses with exit 2 naming the first flat commit (LP-3.35, LP-8.30, LP-8.32). |
| 10 | #153 | #156 | `step-10-repository-store` | `10eaf151`, `92b8e57b` | One commit moves `.decisions/sets/ddd-governance.yml` and 164 `hafeok.ddd` change-sets to `ns/hafeok.ddd/`, `ledger-design.yml` and 23 to `ns/hafeok.ledger/`; exports regenerated, byte-identical; a second commit holds the repository to AC-82 in the suite. |
| 3 | #146 | #157 | `step-03-belonging` | `61bd97ed` | `verify::belonging`: every entity under `ns/<ns>/` belongs to `<ns>`, revocation targets, grant acceptances, intervals and sidecars included (LP-3.33, LP-8.8). |
| 4 | #147 | #158 | `step-04-supersedes` | `12123c60` | `verify::crossing`: a latest version whose `supersedes` names another namespace is `SCHEMA`; history is not judged (LP-3.31, LP-8.8). |
| 5 | #148 | #159 | `step-05-authority` | `4ebe3658` | `Authority::of` / `as_of(…, ns)`; scopes read inside the namespace, `ns:<other>` a fault; roles per namespace; `A003`/`A005` per namespace; the export keeps a namespace's own authority; the writer opens every namespace on its own genesis (`join_genesis` gone, `--external-ref` required); `grant new --namespace`; the unbound-genesis notice per namespace; Appendix C "Authority is per namespace". |
| 6 | #149 | #160 | `step-06-keys` | `406b17db` | Keys bound, closed and trusted per namespace: `key_close` compares the namespace, D7's store-wide branch and `carried_over` go, the first policy is signed by a key trusted in its own namespace, `init` self-binds the holder's key in every namespace; notices `key_split` and `key_shared`; Appendix C "Keys are per namespace". |
| 7 | #150 | #161 | `step-07-signers` | `9df8605e` | `allowed_signers` derived per namespace from that namespace's trusted bindings; `signers::write` removes a stale file and reports per namespace; Appendix C "A stale `allowed_signers` is removed". |
| 8 | #151 | #162 | `step-08-close-everywhere` | `ccf8a0c3` | `identity revoke\|rotate --everywhere`: one change-set per namespace the holder has the key open in, same `at`, one commit (LP-6.34). |
| 9 | #152 | #163 | `step-09-fixtures` | `011a7218` | The fifteen fixture stores moved under `ns/<namespace>/` by `git mv`, no digest moved; the staging shim removed; Appendix C "Namespaces become independent; a store with shared authority is re-founded". |
| — | — | (this record) | `close-out` | — | This document. |

Issues opened: #144 to #153, one per PRD row, each from its row. No other
issue was opened. #104 and #105, already open, overlap issues 6 and 8 and
are named in those issues' bodies.

### CI, as it stands

- #143 and #156 are green.
- #154 and #155 are red on `ledger verify --export` by design: at those
  commits the loader refuses the repository's own flat store. The pull
  requests say so; the series merges in order.
- #157 to #163 are red on **one step
  only**: the contract-surface gate, `ddd diff-contracts <base>..HEAD`,
  which demands a signed seam binding for every exposed Rust surface
  change (a `pub` item added, removed or re-signed; test helpers in
  `ledger-cli/tests/common/` count). The remedy it names is `ddd bind
  <base>..HEAD --verdict …`, which writes under `.ddd/seams/`. The prompt
  forbids editing `.ddd/`, and a declaration's `verdict_knowledge` is the
  author's judgment, so none was filed. See §4.
- Every other `check` step passes on every pull request: build, `cargo
  t`, clippy with `-D warnings -D clippy::unwrap_used`, the quality
  scripts, `ddd validate`, and `ledger verify --export` printing the same
  output as `main` (284 entries, 93 decisions, 3 awaiting, 2 notices,
  exports matching) from #156 on.

## 2. Per step: tests, verdicts, protocol

Every acceptance criterion has a test that failed before its step and
passes after it. "Verdicts changed" is over this repository's committed
store and the committed fixtures: none changed anywhere, by construction
(no authority record, no sidecar, no `allowed_signers`, no cross-namespace
edge exists in either), and `ledger verify --export` on this repository
prints the same output as `main` at every step from #156 on.

**Step 1 (AC-63).** Added: `store_tests.rs`
`a_file_at_a_flat_path_is_a_schema_fault_and_names_the_path`,
`a_flat_only_store_is_refused_and_nothing_of_it_is_read`,
`a_set_id_is_unique_within_its_namespace_not_across_the_store`,
`a_stray_file_under_ns_and_a_misnamed_namespace_directory_are_faults`;
`layout.rs` unit tests. Rewritten: every test that wrote, read or named a
flat path (§3.11 item 1 — the files its list names, through the helpers
`common/mod.rs`, `common/hand.rs`, `common/export_only.rs`), done here by
necessity rather than in issue 9; `ddd-cli/tests/migrate.rs` for the
`DeclareArgs.namespace` field. A staging shim (`stage_store_into`) laid the
still-flat fixtures and repository store out in a tempdir for the tests
until issues 9 and 10. Protocol: §3.1 layout; LP-3.34, LP-3.35 (marked),
LP-3.36, LP-5.22 new; LP-4.10, §5.2, §5.3, §5.6 paths; Appendix B;
Appendix C v1.9 note.

**Step 2 (AC-82).** Added: `ledger-cli/tests/legacy_layout.rs` (seven
tests: re-laid out in one commit the store verifies with its original
`L009` authors and no `L007`; the pickaxe follows a move; a file edited at
the move is `L007`; a flat path at the verified commit is refused; without
the capability the run refuses with exit 2 naming the first flat commit;
the base overlay reads a flat base; a `format:` correction across the move
stands); `layout` unit tests for `classify_flat`, `moved_to`,
`flat_history`. Rewritten: `blame_tests.rs`, `binding_accounting.rs` call
signatures. Scan: no committed store in this repository's history holds an
authority record or a deletion of a log file (187 files, 164 + 23, none
mixed), so keying landing by entity across both path patterns is sound.
Protocol: LP-3.35 implemented, LP-8.30 (re-layout exception), LP-8.32
(lineage); Appendix B; the v1.9 note's history paragraph.

**Step 10 (AC-82 on this repository).** Added:
`ledger-cli/tests/repository_store.rs` (three tests: the layout holds
with no flat path; `verify --export` with the capability is conformant; without it exit 2 names
`cfff91d61c69e2d32b89f61b9109c2b1a57b4a1b`). Verified: output identical to
`main`'s, every acceptance's `L009` author the same (blame on), both
exports byte-identical (set IRIs unchanged, as issue 18 owns them). One
commit moves the 189 files; `ledger/README.md` and the protocol's v1.9
note name it.

**Step 3 (AC-45).** Added: `verify/belonging_tests.rs` (a decision, its
version and its acceptance of another namespace each named; a binding, a
policy and a revocation target of another namespace named; an unhomed
candidate not judged here); `ledger-cli/tests/belonging.rs` (a decision of
B under A; an acceptance of B under A; a revocation under A of B's
acceptance and the same under B conformant; B's opening change-set under
A). Rewritten: `genesis_role_tests.rs` had homed a `fixture.ledger` policy
in a `hafeok.ledger` change-set; `verify::verify` split into
`file_findings` for the 40-line gate. Protocol: LP-3.33 implemented,
LP-8.8 clause, Appendix B, Appendix C note.

**Step 4 (AC-43).** Added: `verify/crossing_tests.rs` (three);
`ledger-cli/tests/crossing.rs` (the verb refused both ways and writes
nothing; a hand-filed crossing fails the gate until a next version drops
the edge; a supersession at home stands). Protocol: LP-3.31 implemented,
LP-8.8 clause, Appendix B; no Appendix C note (the row says none).

**Step 5 (AC-47 first bullet, AC-48, AC-68).** Added:
`ledger-cli/tests/authority_namespaces.rs` (seven tests, §1 of #159's
description). Rewritten and why: `graph/export_tests.rs` (`*` no longer
reaches every export); `signing/check_tests.rs`, `verify/order_grid.rs`
(one namespace); `ledger-cli/tests/authority.rs` grantor test (`ns:other`
is a grant made elsewhere); `pre_policy_binding.rs` and
`binding_accounting.rs` on the new `hand::found` (a namespace founded by
hand with a genesis older than its policy, which the writer no longer
files), two tests renamed for changed meaning
(`a_skewed_add_dated_before_inits_genesis_is_refused_by_d7`,
`a_binding_in_a_namespace_no_policy_governs_is_a_schema_fault_and_init_is_refused_over_it`
— the latter renamed again in step 6, below); `closed_key.rs`
`the_genesis_holders_first_binding_in_a_second_namespace_vouched_by_the_closed_key`
(binding dated after the policy); `crossing.rs` no longer asserts the
absence of `G001`. Protocol: LP-6.5, LP-6.16, LP-6.28, LP-6.31 implemented;
LP-5.19, LP-8.19 (with the `A005` row), LP-8.23, LP-8.31 (report shape:
`genesis_unbound` one entry per namespace), LP-9.11, LP-8.8; Appendix B;
Appendix C "Authority is per namespace".

**Step 6 (AC-47 second bullet).** Added/rewritten:
`key_across_namespaces.rs` as §3.11 says (bound in A then B or B then A, a
close in A leaves B open and is a notice, a third namespace opens with the
same key self-bound; a rotate in A leaves B's acceptances, review items and
`allowed_signers` byte-identical; four within-namespace tests kept; the
carried test on a key close); `genesis_key.rs`'s four later-namespace
tests (self-bound in the second namespace under its mandate; with every key
closed elsewhere `init` binds it again and `verify` notices the split; with
no usable key `--without-key` initialises unbound); `pre_policy_binding.rs`
(`unsigned_before` self-bound; the signed case signs the first policy with
the key bound before it, LP-4.31); `binding_accounting.rs`
(`self_bound_by_hand`, one namespace per first binding); `trust.rs`
(`…self_binds_once_per_namespace_and_a_further_key_there_is_their_own_add`,
`…beside_inits_in_a_second_namespace_fails`); `key_close_tests.rs` (three
unit tests). Protocol: LP-4.12, LP-4.13, LP-4.31, LP-4.32, LP-4.37,
LP-4.38, LP-4.39 (ruling 101's wording), LP-6.31, LP-6.32 implemented,
LP-8.31 (two notices, report shape); Appendix B; Appendix C "Keys are per
namespace".

**Step 7 (§3.4).** Added: `ledger-cli/tests/signers_per_namespace.rs`
(four tests; a one-second pause added in step 8 after a flaky run).
Protocol: LP-4.10, LP-4.33; Appendix B; Appendix C note.

**Step 8 (AC-47 third bullet).** Added to `key_across_namespaces.rs`: a
revoke everywhere files one change-set per namespace with one `at` in one
commit, no split notice, and a second `--everywhere` is refused; a rotate
everywhere closes the key and binds the new one in each namespace; the
carried test drives `--everywhere` too. Protocol: LP-6.34 new, LP-6.32's
note dropped; Appendix B; no Appendix C note.

**Step 9 (§3.11).** 47 renames under `ledger-cli/tests/fixtures/`;
`UPDATE_FIXTURES=1 cargo test -p ledger-cli --test gate` found nothing to
refresh (no fixture file modified). The staging shim removed
(`fixture_copy`, `copy_fixture_into`, `workspace_copy`, `copy_store_into`
are plain copies); `gate.rs` and `digests.rs` lose their flat fallbacks;
`legacy_layout.rs` lays the `pass` fixture flat itself; `scalars.rs` reads
the fixture's set from its namespace directory. Protocol: the re-founding
note as drafted, headed with no format number; the v1.9 note's two
forward references resolved; Appendix B.

## 3. Test-case candidates

Every store built for a negative test, with the requirement and class it
exercises. All are hand-built past the verbs unless marked (verb).

| Store | Requirement | Class |
| --- | --- | --- |
| A file at `.decisions/log/…` or `.decisions/sets/…` at the verified commit; a stray file under `ns/`; a misnamed namespace directory (`store_tests.rs`) | LP-3.34 | `SCHEMA` |
| A set id declared twice in one namespace (`store_tests.rs`) | LP-3.36 | `SCHEMA` |
| A version naming a set declared in another namespace (`verify/view.rs` tests) | LP-5.22 | `SCHEMA` |
| A flat path at the verified commit after a flat history; a file edited at the re-layout commit (`legacy_layout.rs`) | LP-3.34, LP-8.30 | `SCHEMA`, `L007` |
| A flat history verified without the legacy capability (`legacy_layout.rs`, `repository_store.rs`) | LP-3.35 | exit 2, no verdict |
| A decision, version or acceptance of B under A's directory; a revocation under A of B's acceptance; B's opening change-set under A (`belonging.rs`, `belonging_tests.rs`) | LP-3.33 | `SCHEMA` |
| A latest version whose `supersedes` names another namespace; repaired by a next version (`crossing.rs`, `crossing_tests.rs`) | LP-3.31 | `SCHEMA`, then conformant |
| A grant scoped `ns:<other>` than its directory (`authority_namespaces.rs`) | LP-6.16 | `SCHEMA` |
| A second namespace's policy under the first namespace's genesis, with the holder's `add` signed by a key trusted elsewhere — today's writer's shape (`authority_namespaces.rs`, AC-68) | LP-6.28, LP-4.12 | `A006`, `SCHEMA` (D7) |
| Two genesis grants in one namespace (`authority_tests.rs`); one in each of two (`authority_namespaces.rs`) | LP-6.5 | `A005`; none |
| A `*` grant of A used to accept in B (verb, `authority_namespaces.rs`) | LP-6.16 | refused (`NoGrant`) |
| A self-bound binding filed in a namespace beside `init`'s (`trust.rs`, `genesis_key.rs`) | LP-4.12 | `SCHEMA` (D7) |
| A binding dated before its namespace's genesis, filed before or after opening (`pre_policy_binding.rs`) | LP-4.12 | `SCHEMA` (D7); `init` refused |
| An unsigned self-bound binding before a signed first policy in a hand-founded namespace (`pre_policy_binding.rs`, `binding_accounting.rs`) | LP-4.31, LP-4.36 | `L011` |
| An acceptance in A signed with a key closed in A (`key_across_namespaces.rs`) | LP-4.39 | `L011` |
| A binding in B signed by a key never bound in B (`closed_key.rs`) | LP-4.31 | `L011` |
| A key closed in A and self-bound in B; one key bound to two principals in two namespaces (`key_across_namespaces.rs`, `genesis_key.rs`, `key_close_tests.rs`) | LP-6.32, LP-8.31 | notice, no finding |
| An `allowed_signers` committed in a namespace that binds nothing (`signers_per_namespace.rs`) | LP-4.33 | `[SIGNERS]` |
| `identity revoke --everywhere` on a key closed everywhere (verb) | LP-6.34 | refused, nothing written |
| An agent identity or a piped stdin at `accept` and at `identity revoke [--everywhere]` (`authority.rs`, `authority_namespaces.rs`, `key_across_namespaces.rs`) | LP-8.9 (`L006`), the terminal rule | refused, nothing written |

## 4. Stopped, with options

Nothing in the ten steps was stopped: no stored digest moved, no verdict
of `.decisions/` or a committed fixture changed, and no ruling and the
code disagreed in a way that could not be satisfied. Two things need the
principal, and three decisions I took are stated so they can be reversed.

**The contract-surface gate (needs the principal).** `ddd diff-contracts`
holds every pull request that changes an exposed Rust surface (#154 and
#155 once their verify step passes, #157 to #163), demanding a signed seam
binding per change. The prompt's two instructions — every check in
`CLAUDE.md` passes on every pull request, the contract-surface gate
included; do not edit `.ddd/` — cannot both hold for this series, since
the remedy (`ddd bind <base>..HEAD --verdict …`) writes under
`.ddd/seams/` and the declaration's verdict is the author's. I left the
gate red and said so on each pull request. Options: (a) run `ddd bind` on
each branch yourself, in stack order, and push; (b) tell me to run it,
with the verdict text you want each seam to carry, or with empty verdicts
to be written later (the command warns about those); (c) exempt the
series from the gate. Nothing else on those pull requests waits.

**Merging (the principal's).** Merge in stack order: #143 (independent),
then #154 → #155 → #156 → #157 → #158 → #159 → #160 → #161 → #162 → #163,
then this record. #154 and #155 are red on `ledger verify --export` until
#156 lands with them; the three could also be squashed into one merge.

**Three decisions taken, stated.**
1. *Issue 10 built third*, before issues 3 to 9 (the PRD has it last, after
   9): the only order in which the loader's refusal of flat paths and a
   green `ledger verify --export` can meet. §3.11 item 1's test rewrites
   therefore landed with issue 1, not 9.
2. *Part of issue 8 landed with issues 5 and 6*: the writer opening every
   namespace on its own genesis (5) and self-binding the holder's key there
   (6). With authority and keys per namespace, the old writer's second
   namespace fails `A006` and D7 at the writer's own gate, so no pull
   request between 5 and 8 could open a second namespace. `genesis_key.rs`'s
   later-namespace tests were rewritten in 6 for the same reason.
3. *The decision shapes `G001`–`G006` stay over the store's graph*; only
   `A003`/`A005` run per namespace. PRD §3.9 says the whole stage runs per
   namespace; run that way, a `supersedes` edge in history into another
   namespace would be a permanent `G001`, against ruling 75 (live claims
   only), and `based_on` crosses by design. LP-8.19 records the split.

## 5. Found, not fixed

- `ledger verify`'s `ssh-keygen` windows are read at second granularity:
  an act in the same second as the policy it is judged under, with a close
  of its key in that second, fails to verify. The key tests pause a second
  between acts for this reason; `signers_per_namespace.rs` did not and was
  flaky once. A real store will not file two acts in one second, but the
  reading could be stated in the protocol (LP-4.13) or the writer could
  refuse an act dated within the same second as the policy.
- `Store::namespaces()` collects namespaces from sets, roles, log and
  sidecars, so a namespace that has only a sidecar or only an
  `allowed_signers` counts; `signers::check` and `write` rely on that. A
  namespace directory that is empty is invisible to both.
- The export's set selection keeps the pre-v1.9 rule (the sets a
  namespace's versions name), while PRD §3.9 says "everything in its
  directory"; a declared, unused set is not exported. Unchanged here, since
  issue 18 owns the set IRIs.
- Appendix B's 9 October rows read newest-first within the day, after the
  7 October rows (oldest-first); the first row of the day was appended and
  each later one inserted above it. A reorder is a one-line edit.
- The PRD's `--wait 600` on the contract-surface gate waits ten minutes for
  bindings that no one files in CI; the job spends that time on every
  pull request that touches an exposed surface.
- `GenesisUnbound` lost its `namespaces` list for a single `namespace`;
  any consumer of `verify --json` reading `genesis_unbound.holder` as an
  object now finds an array. LP-8.31's note says so.
- `product` MCP server failed to connect in this environment (`product`
  not on `PATH`); nothing here needed it.
- `cargo t` on this container exhausted the disk once (a 29 GB `target/`);
  `CARGO_PROFILE_DEV_DEBUG=line-tables-only
  CARGO_PROFILE_DEV_INCREMENTAL=false` keeps it near 5 GB.
