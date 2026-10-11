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
`main` includes its predecessors. Each branch carries the step's commit and,
since 2026-10-11, the commit that files its seam bindings (§6); every branch
above `step-01` was rebased once onto the one below it when those commits
went in, so the commit ids here are the current ones. The order differs
from the PRD's (§4): issue 10 was built third, because no pull request can
be green on `ledger verify --export` while the loader refuses flat paths
and the repository's own store is still flat.

| Step | Issue | Pull request | Branch (`claude/great-feynman-zcdsdd-…`) | Commit | What landed |
| --- | --- | --- | --- | --- | --- |
| 0 | — | #143 | `step-00-ruling-102` | `63c0cd24` | Ruling 102 in `ledger/rulings/order-from-the-acts-rulings-2026-10-09.md` ("A question the rulings raised"); PRD §3.5.6, issue 15, §7 ("ruled"); a line for 102 in `ledger/sessions/2026-10-namespace-design.md`. |
| 1 | #144 | #154 | `step-01-layout` | `1e019f16`, `15c24ade`, `9c64c21a` | `ledger-core/src/layout.rs`; every store loads from `ns/<ns>/{sets,roles,log,sig,allowed_signers}`; a flat path at the verified commit is `SCHEMA` (LP-3.34); ids, sets, roles unique per namespace (LP-3.36); a version names its own namespace's set (LP-5.22); the writer homes each change-set by its one namespace (`author/home.rs`); `declare` and `role declare` take `--namespace`; Appendix C "Spec v1.9 — one directory per namespace". |
| 2 | #145 | #155 | `step-02-legacy-history` | `9e6daf59`, `25ab27ad` | Flat paths detected on the first-parent history; with the legacy capability (on by default) every tracked file is followed along its lineage, flat and namespaced, for `L007`, LP-3.16's `format:` comparison, `L009` and the base overlay; `verify --no-legacy-layout` refuses with exit 2 naming the first flat commit (LP-3.35, LP-8.30, LP-8.32). |
| 10 | #153 | #156 | `step-10-repository-store` | `cb93711f`, `ef2ce1a6` | One commit moves `.decisions/sets/ddd-governance.yml` and 164 `hafeok.ddd` change-sets to `ns/hafeok.ddd/`, `ledger-design.yml` and 23 to `ns/hafeok.ledger/`; exports regenerated, byte-identical; a second commit holds the repository to AC-82 in the suite. |
| 3 | #146 | #157 | `step-03-belonging` | `17854a07`, `eb6bf214` | `verify::belonging`: every entity under `ns/<ns>/` belongs to `<ns>`, revocation targets, grant acceptances, intervals and sidecars included (LP-3.33, LP-8.8). |
| 4 | #147 | #158 | `step-04-supersedes` | `d4d35075`, `deb88ef2` | `verify::crossing`: a latest version whose `supersedes` names another namespace is `SCHEMA`; history is not judged (LP-3.31, LP-8.8). |
| 5 | #148 | #159 | `step-05-authority` | `1d1fb8cf`, `103c1bf1`, `4ee026b7` | `Authority::of` / `as_of(…, ns)`; scopes read inside the namespace, `ns:<other>` a fault; roles per namespace; `A003`/`A005` per namespace; the export keeps a namespace's own authority; the writer opens every namespace on its own genesis (`join_genesis` gone, `--external-ref` required); `grant new --namespace`; the unbound-genesis notice per namespace; Appendix C "Authority is per namespace". |
| 6 | #149 | #160 | `step-06-keys` | `af9e6514`, `53becab1` | Keys bound, closed and trusted per namespace: `key_close` compares the namespace, D7's store-wide branch and `carried_over` go, the first policy is signed by a key trusted in its own namespace, `init` self-binds the holder's key in every namespace; notices `key_split` and `key_shared`; Appendix C "Keys are per namespace". |
| 7 | #150 | #161 | `step-07-signers` | `cffc9a26`, `3cdc1188` | `allowed_signers` derived per namespace from that namespace's trusted bindings; `signers::write` removes a stale file and reports per namespace; Appendix C "A stale `allowed_signers` is removed". |
| 8 | #151 | #162 | `step-08-close-everywhere` | `83eb6639`, `f85d4d53` | `identity revoke\|rotate --everywhere`: one change-set per namespace the holder has the key open in, same `at`, one commit (LP-6.34). |
| 9 | #152 | #163 | `step-09-fixtures` | `a886cf1e`, `7766d21e` | The fifteen fixture stores moved under `ns/<namespace>/` by `git mv`, no digest moved; the staging shim removed; Appendix C "Namespaces become independent; a store with shared authority is re-founded". |
| — | — | (this record) | `close-out` | — | This document. |

Issues opened: #144 to #153, one per PRD row, each from its row. No other
issue was opened. #104 and #105, already open, overlap issues 6 and 8 and
are named in those issues' bodies.

### CI, as it stands

As of the seam-binding commits of 2026-10-11 (§6), on the principal's
ruling that the `.ddd/` prohibition meant the design records and not seam
bindings. The runs on the rebased branches started at 06:22Z and were in
flight when this was committed; the commit after this one records what
they returned.

- #143 and #156 were green before and change nothing the bindings touch.
- #154 and #155 are red on `ledger verify --export` by design: at those
  commits the loader refuses the repository's own flat store. The pull
  requests say so; they pass once #156 lands with them. Their other steps,
  the contract-surface gate included, pass locally.
- #157 to #163: the contract-surface gate, `ddd diff-contracts
  <base>..HEAD`, reports 0 undischarged on each branch locally with the
  bindings of §6, and `ddd validate` is conformant; every other `check`
  step passed before on the same trees — build, `cargo t`, clippy with
  `-D warnings -D clippy::unwrap_used`, the quality scripts, and `ledger
  verify --export` printing the same output as `main` (284 entries, 93
  decisions, 3 awaiting, 2 notices, exports matching) from #156 on.

Before those commits, #157 to #163 were red on the contract-surface gate
alone, which demands a signed seam binding for every exposed Rust surface
change (a `pub` item added, removed or re-signed; test helpers in
`ledger-cli/tests/common/` count); each pull request carries the comment
that said so.

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
code disagreed in a way that could not be satisfied. One thing needed the
principal and was settled on 2026-10-11; merging is theirs; and the three
decisions I took, stated so they could be reversed, were all accepted.

**The contract-surface gate (settled).** `ddd diff-contracts` held every
pull request that changes an exposed Rust surface (#154 and #155 once
their verify step passes, #157 to #163), demanding a signed seam binding
per change. The prompt's two instructions — every check in `CLAUDE.md`
passes on every pull request, the contract-surface gate included; do not
edit `.ddd/` — could not both hold, since the remedy (`ddd bind
<base>..HEAD`) writes under `.ddd/seams/` and a declaration's verdict is
the author's. I left the gate red and said so on each pull request, with
options (a) the principal runs `ddd bind` on each branch, (b) the principal
tells me to, (c) the series is exempted. The principal chose (b) and ruled
that the prohibition was meant for the design records, not for seam
bindings — the prompt's error. `ddd bind <base>..HEAD` then ran on each
branch in stack order, each step's bindings in that step's pull request,
the branches above rebased; every `verdict_knowledge` was written by me,
none left empty, and the two declarations over `ledger-cli/tests/common/`
say the helpers are test-only. Nothing else under `.ddd/` was touched. The
bindings are listed in §6.

**Merging (the principal's).** Merge in stack order: #143 (independent),
then #154 → #155 → #156 → #157 → #158 → #159 → #160 → #161 → #162 → #163,
then this record (#164). #154 and #155 are red on `ledger verify --export`
until #156 lands with them; the three could also be squashed into one
merge.

**Three decisions taken, stated — all accepted by the principal on 2026-10-11.**
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
   Accepted as it stands: the decision-graph shapes stay over the store's
   graph for now; running them per namespace, with historical
   cross-namespace edges skipped, belongs to issue 12.

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

## 6. Seam bindings filed

Filed on 2026-10-11 with `ddd bind <base>..HEAD` on each branch in stack
order (#154, #155, #157 to #163; #143, #156 and #164 change no exposed Rust
surface), each step's bindings committed in that step's pull request and
the branches above it rebased. A binding signs one transition — subject
symbol, the file's content hash before and after, the base revision —
and a declaration (`seam/rust/<file>`) holds the bindings of one file with
one `verdict_knowledge`, so the verdict in a row is its declaration's. 131
bindings discharge 132 events (the two `key` fields of
`verify/key_notices.rs` share one). *new* marks a declaration the bind
created, whose verdict I wrote; *amended* an existing declaration that
received bindings. Ten existing declarations had their verdict rewritten
because the change had made the old sentence false or silent on what the
change teaches — `store.rs`, `signing/mod.rs` and `authority/signers.rs`
(step 1, `signers.rs` again at step 7), `author/authority_ops.rs`,
`authority/view.rs` and `authority/structure.rs` (step 5),
`author/genesis_key.rs` (step 6), `author/identity_ops.rs` (step 8), and
the two over `ledger-cli/tests/common/` (`mod.rs` at step 1, `hand.rs` at
step 5), which now open with "Test-only". Every other amended declaration
keeps the verdict it had. Nothing else under `.ddd/` was touched.

| Pull request | Seam | Surface (change) | Verdict |
| --- | --- | --- | --- |
| #154 | `seam/rust/ledger-cli-src-commands-declare-rs` (new) | `ledger-cli/src/commands/declare.rs#namespace` (added) | CLI users learn declare takes --namespace to name the directory a new set files under, required only when the store holds several namespaces and inferred from the one directory otherwise (LP-3.34). |
| #154 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#stage_fixture` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #154 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#stage_fixture_into` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #154 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#stage_store_into` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #154 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#stage_workspace` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #154 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#declare_in` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #154 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#log_dir` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #154 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#ns_dir` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #154 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#roles_dir` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #154 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#sets_dir` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #154 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#sidecar` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #154 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#sidecar_count` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #154 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#sig_dir` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #154 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#signers` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #154 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#signers_path` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #154 | `seam/rust/ledger-core-src-author-authority-ops-rs` (amended) | `ledger-core/src/author/authority_ops.rs#namespace` (added) | Callers learn the authority verbs (init_namespace, declare_role, grant, accept_grant, revoke_grant) run the role check, then the same delta-gate as every verb, refusing with AuthorError::Unauthorized or the verify finding. |
| #154 | `seam/rust/ledger-core-src-author-declare-rs` (new) | `ledger-core/src/author/declare.rs#namespace` (added) | Callers learn a declared set is filed under the namespace named in DeclareArgs.namespace or, when none is named, the store's single namespace directory, and that a store holding several refuses an unnamed declaration (LP-3.34). |
| #154 | `seam/rust/ledger-core-src-author-home-rs` (new) | `ledger-core/src/author/home.rs#acceptance_namespace` (added) | Callers learn how a change-set finds its home: an act's namespace is read from the entities it carries or names (a decision id, a binding's or policy's namespace, a revocation's target, a grant's under), a new set's from --namespace or the store's single directory, so no verb files a change-set outside .decisions/ns/<ns>/ (LP-3.34, ruling 45). |
| #154 | `seam/rust/ledger-core-src-author-home-rs` (new) | `ledger-core/src/author/home.rs#grant_namespace` (added) | Callers learn how a change-set finds its home: an act's namespace is read from the entities it carries or names (a decision id, a binding's or policy's namespace, a revocation's target, a grant's under), a new set's from --namespace or the store's single directory, so no verb files a change-set outside .decisions/ns/<ns>/ (LP-3.34, ruling 45). |
| #154 | `seam/rust/ledger-core-src-author-home-rs` (new) | `ledger-core/src/author/home.rs#interval_namespace` (added) | Callers learn how a change-set finds its home: an act's namespace is read from the entities it carries or names (a decision id, a binding's or policy's namespace, a revocation's target, a grant's under), a new set's from --namespace or the store's single directory, so no verb files a change-set outside .decisions/ns/<ns>/ (LP-3.34, ruling 45). |
| #154 | `seam/rust/ledger-core-src-author-home-rs` (new) | `ledger-core/src/author/home.rs#namespace_of` (added) | Callers learn how a change-set finds its home: an act's namespace is read from the entities it carries or names (a decision id, a binding's or policy's namespace, a revocation's target, a grant's under), a new set's from --namespace or the store's single directory, so no verb files a change-set outside .decisions/ns/<ns>/ (LP-3.34, ruling 45). |
| #154 | `seam/rust/ledger-core-src-author-home-rs` (new) | `ledger-core/src/author/home.rs#resolve_namespace_dir` (added) | Callers learn how a change-set finds its home: an act's namespace is read from the entities it carries or names (a decision id, a binding's or policy's namespace, a revocation's target, a grant's under), a new set's from --namespace or the store's single directory, so no verb files a change-set outside .decisions/ns/<ns>/ (LP-3.34, ruling 45). |
| #154 | `seam/rust/ledger-core-src-authority-fixture-rs` (amended) | `ledger-core/src/authority/fixture.rs#NS` (added) | Test authors learn the builders for roles, grants, acceptances and intervals used by the authority tests. |
| #154 | `seam/rust/ledger-core-src-authority-role-rs` (amended) | `ledger-core/src/authority/role.rs#namespace` (added) | Consumers learn the role file shape, the closed seven-capability vocabulary, and the Act → capability / limit mapping. |
| #154 | `seam/rust/ledger-core-src-authority-signers-rs` (amended) | `ledger-core/src/authority/signers.rs#derive` (signature-changed) | Consumers learn allowed_signers is derived per namespace from that namespace's key bindings in OpenSSH form, written by the identity verbs at .decisions/ns/<ns>/allowed_signers, and held byte-identical by verify's [SIGNERS] stage. |
| #154 | `seam/rust/ledger-core-src-authority-signers-rs` (amended) | `ledger-core/src/authority/signers.rs#derive_each` (added) | Consumers learn allowed_signers is derived per namespace from that namespace's key bindings in OpenSSH form, written by the identity verbs at .decisions/ns/<ns>/allowed_signers, and held byte-identical by verify's [SIGNERS] stage. |
| #154 | `seam/rust/ledger-core-src-authority-signers-rs` (amended) | `ledger-core/src/authority/signers.rs#path` (signature-changed) | Consumers learn allowed_signers is derived per namespace from that namespace's key bindings in OpenSSH form, written by the identity verbs at .decisions/ns/<ns>/allowed_signers, and held byte-identical by verify's [SIGNERS] stage. |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#INDEX_DIR` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#LOG_DIR` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#NS_DIR` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#ROLES_DIR` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#SETS_DIR` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#SIGNERS_FILE` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#SIG_DIR` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#TRACKED` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#Kind` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#classify` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#is_log` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#is_role` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#log_dir` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#namespace_dir` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#namespaces` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#relative_dir` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#roles_dir` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#sets_dir` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#sig_dir` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#signers_path` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#Log` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#Role` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#Set` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#Sig` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-layout-rs` (new) | `ledger-core/src/layout.rs#Signers` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #154 | `seam/rust/ledger-core-src-lib-rs` (amended) | `ledger-core/src/lib.rs#layout` (added) | A consumer learns which modules are the crate's surface; revisit and show joined it at spec v1.5, and batch and hold join it with the batched acceptance act. None pulls in an ontology — the substrate an outside implementation imports stays ddd-free — and none adds a store: hold is derived from a version's own basis pointers, and batch holds a plan, never state carried between two invocations. |
| #154 | `seam/rust/ledger-core-src-set-rs` (new) | `ledger-core/src/set.rs#namespace` (added) | Consumers learn a loaded set carries the namespace of the directory it was read from, never a field of the file, so a set id is unique within a namespace and a version names a set of its own namespace (LP-3.34, LP-3.36). |
| #154 | `seam/rust/ledger-core-src-signing-mod-rs` (amended) | `ledger-core/src/signing/mod.rs#load` (signature-changed) | Callers learn where signatures live: one sidecar per scheme at .decisions/ns/<ns>/sig/<ulid>.<scheme>.sig over the entity's signed bytes, never inside a log file, and that a sidecar carries the namespace of the directory it was read from (LP-3.34). |
| #154 | `seam/rust/ledger-core-src-signing-mod-rs` (amended) | `ledger-core/src/signing/mod.rs#write` (signature-changed) | Callers learn where signatures live: one sidecar per scheme at .decisions/ns/<ns>/sig/<ulid>.<scheme>.sig over the entity's signed bytes, never inside a log file, and that a sidecar carries the namespace of the directory it was read from (LP-3.34). |
| #154 | `seam/rust/ledger-core-src-signing-mod-rs` (amended) | `ledger-core/src/signing/mod.rs#namespace` (added) | Callers learn where signatures live: one sidecar per scheme at .decisions/ns/<ns>/sig/<ulid>.<scheme>.sig over the entity's signed bytes, never inside a log file, and that a sidecar carries the namespace of the directory it was read from (LP-3.34). |
| #154 | `seam/rust/ledger-core-src-store-rs` (amended) | `ledger-core/src/store.rs#namespace` (added) | Consumers learn the store loads every namespace's sets, roles and log from .decisions/ns/<ns>/, that a loaded set or change-set carries the namespace of the directory it was read from, that a set is looked up by (namespace, id) with set_in, and that every file must declare the format its fields need (LP-3.34). |
| #154 | `seam/rust/ledger-core-src-store-rs` (amended) | `ledger-core/src/store.rs#namespaces` (added) | Consumers learn the store loads every namespace's sets, roles and log from .decisions/ns/<ns>/, that a loaded set or change-set carries the namespace of the directory it was read from, that a set is looked up by (namespace, id) with set_in, and that every file must declare the format its fields need (LP-3.34). |
| #154 | `seam/rust/ledger-core-src-store-rs` (amended) | `ledger-core/src/store.rs#set_in` (added) | Consumers learn the store loads every namespace's sets, roles and log from .decisions/ns/<ns>/, that a loaded set or change-set carries the namespace of the directory it was read from, that a set is looked up by (namespace, id) with set_in, and that every file must declare the format its fields need (LP-3.34). |
| #154 | `seam/rust/ledger-core-src-store-rs` (amended) | `ledger-core/src/store.rs#set` (removed) | Consumers learn the store loads every namespace's sets, roles and log from .decisions/ns/<ns>/, that a loaded set or change-set carries the namespace of the directory it was read from, that a set is looked up by (namespace, id) with set_in, and that every file must declare the format its fields need (LP-3.34). |
| #154 | `seam/rust/ledger-core-src-testkit-rs` (amended) | `ledger-core/src/testkit.rs#NS` (added) | Test authors learn the legacy-shape revocation builder. |
| #155 | `seam/rust/ledger-cli-src-commands-verify-rs` (amended) | `ledger-cli/src/commands/verify.rs#legacy_layout` (added) | CI and users learn `verify --export` adds a third stage to the gate: every committed export must equal the log's byte for byte, and an unrun stage is absent from the report rather than shown as a pass, with exit 1 for findings exactly as the file and graph stages. |
| #155 | `seam/rust/ledger-core-src-blame-rs` (new) | `ledger-core/src/blame.rs#introducing_author` (signature-changed) | Callers learn git blame attribution follows a file along its lineage: with the legacy capability the pickaxe also names the flat path the file had before the re-layout, so the introducing author is the original one and never the re-layout commit's (LP-3.35). |
| #155 | `seam/rust/ledger-core-src-changeset-rs` (amended) | `ledger-core/src/changeset.rs#namespace_hint` (added) | Consumers learn a change-set carries the format-6 authority lists beside decisions, versions, acceptances and revocations, with empty() and entry counts covering every kind. |
| #155 | `seam/rust/ledger-core-src-landing-rs` (amended) | `ledger-core/src/landing.rs#content_at_any` (added) | Callers learn when an entity entered the record (D6): the first first-parent commit whose version of its file holds it, positions ordered by landing then at, computed against a base so a branch reads as its merge would. |
| #155 | `seam/rust/ledger-core-src-landing-rs` (amended) | `ledger-core/src/landing.rs#lineage` (added) | Callers learn when an entity entered the record (D6): the first first-parent commit whose version of its file holds it, positions ordered by landing then at, computed against a base so a branch reads as its merge would. |
| #155 | `seam/rust/ledger-core-src-landing-rs` (amended) | `ledger-core/src/landing.rs#lineage_versions` (added) | Callers learn when an entity entered the record (D6): the first first-parent commit whose version of its file holds it, positions ordered by landing then at, computed against a base so a branch reads as its merge would. |
| #155 | `seam/rust/ledger-core-src-landing-rs` (amended) | `ledger-core/src/landing.rs#touched_after_landing` (signature-changed) | Callers learn when an entity entered the record (D6): the first first-parent commit whose version of its file holds it, positions ordered by landing then at, computed against a base so a branch reads as its merge would. |
| #155 | `seam/rust/ledger-core-src-landing-rs` (amended) | `ledger-core/src/landing.rs#legacy` (added) | Callers learn when an entity entered the record (D6): the first first-parent commit whose version of its file holds it, positions ordered by landing then at, computed against a base so a branch reads as its merge would. |
| #155 | `seam/rust/ledger-core-src-landing-rs` (amended) | `ledger-core/src/landing.rs#compute` (signature-changed) | Callers learn when an entity entered the record (D6): the first first-parent commit whose version of its file holds it, positions ordered by landing then at, computed against a base so a branch reads as its merge would. |
| #155 | `seam/rust/ledger-core-src-layout-rs` (amended) | `ledger-core/src/layout.rs#FLAT_TRACKED` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #155 | `seam/rust/ledger-core-src-layout-rs` (amended) | `ledger-core/src/layout.rs#classify_flat` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #155 | `seam/rust/ledger-core-src-layout-rs` (amended) | `ledger-core/src/layout.rs#flat_counterpart` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #155 | `seam/rust/ledger-core-src-layout-rs` (amended) | `ledger-core/src/layout.rs#flat_history` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #155 | `seam/rust/ledger-core-src-layout-rs` (amended) | `ledger-core/src/layout.rs#moved_to` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #155 | `seam/rust/ledger-core-src-layout-rs` (amended) | `ledger-core/src/layout.rs#moved_to_a_namespace` (added) | Consumers learn the one module that knows the store layout: every namespace lives under .decisions/ns/<ns>/ with its own sets/, roles/, log/, sig/ and allowed_signers, a path's namespace is its directory, and a file at the v1.8 flat paths or anywhere else under .decisions/ outside ns/ and index/ is a schema fault (LP-3.34, rulings 62 and 63). |
| #155 | `seam/rust/ledger-core-src-verify-integrity-rs` (new) | `ledger-core/src/verify/integrity.rs#blame_consistency` (signature-changed) | Callers learn the blame-consistency check takes the legacy flag so that, in a repository whose history holds the flat layout, an entity's introducing author is read across the re-layout rather than from the move (LP-3.35). |
| #155 | `seam/rust/ledger-core-src-verify-mod-rs` (amended) | `ledger-core/src/verify/mod.rs#legacy_layout` (added) | Callers learn a verify report may carry an export stage: present only when asked for, conformant only when empty, so an export check that did not run never reads as one that passed. |
| #157 | `seam/rust/ledger-core-src-verify-belonging-rs` (new) | `ledger-core/src/verify/belonging.rs#findings` (added) | Callers learn the belonging stage: every entity filed under ns/<ns>/ must belong to <ns> (a decision id, a version's or acceptance's decision, a binding's or policy's namespace, a revocation's or grant acceptance's target, a sidecar's entity), each breach a SCHEMA fault and no class added (LP-3.33, ruling 45). |
| #157 | `seam/rust/ledger-core-src-verify-mod-rs` (amended) | `ledger-core/src/verify/mod.rs#belonging` (added) | Callers learn a verify report may carry an export stage: present only when asked for, conformant only when empty, so an export check that did not run never reads as one that passed. |
| #158 | `seam/rust/ledger-core-src-verify-crossing-rs` (new) | `ledger-core/src/verify/crossing.rs#findings` (added) | Callers learn the crossing stage: a decision's latest version may not supersede a decision of another namespace, since only based_on crosses namespaces, judged on the live claim alone so history keeps what it recorded, a SCHEMA fault and no class added (LP-3.31, rulings 43 and 75). |
| #158 | `seam/rust/ledger-core-src-verify-mod-rs` (amended) | `ledger-core/src/verify/mod.rs#crossing` (added) | Callers learn a verify report may carry an export stage: present only when asked for, conformant only when empty, so an export check that did not run never reads as one that passed. |
| #159 | `seam/rust/ledger-cli-tests-common-hand-rs` (amended) | `ledger-cli/tests/common/hand.rs#found` (added) | Test-only: the hand-writing half of the ledger-cli suites' harness, with no consumer outside ledger-cli/tests/ — test suites learn how to write ledger entities by hand (acceptances, bindings, policies with any at, under or signer) and how to found a namespace with its genesis, roles and first policy without the writer, so verify is exercised on files no verb made. |
| #159 | `seam/rust/ledger-core-src-author-authority-ops-rs` (amended) | `ledger-core/src/author/authority_ops.rs#namespace` (added) | Callers learn init_namespace opens every namespace with its own genesis grant, root and accept roles and first policy under .decisions/ns/<ns>/ (a mandate is required), and that declare_role runs the role check over that namespace's authority, then the same delta-gate as every verb, refusing with AuthorError::Unauthorized or the verify finding; the grant verbs live in grant_ops (LP-6.5, ruling 47). |
| #159 | `seam/rust/ledger-core-src-author-authority-ops-rs` (amended) | `ledger-core/src/author/authority_ops.rs#accept_grant` (removed) | Callers learn init_namespace opens every namespace with its own genesis grant, root and accept roles and first policy under .decisions/ns/<ns>/ (a mandate is required), and that declare_role runs the role check over that namespace's authority, then the same delta-gate as every verb, refusing with AuthorError::Unauthorized or the verify finding; the grant verbs live in grant_ops (LP-6.5, ruling 47). |
| #159 | `seam/rust/ledger-core-src-author-authority-ops-rs` (amended) | `ledger-core/src/author/authority_ops.rs#grant` (removed) | Callers learn init_namespace opens every namespace with its own genesis grant, root and accept roles and first policy under .decisions/ns/<ns>/ (a mandate is required), and that declare_role runs the role check over that namespace's authority, then the same delta-gate as every verb, refusing with AuthorError::Unauthorized or the verify finding; the grant verbs live in grant_ops (LP-6.5, ruling 47). |
| #159 | `seam/rust/ledger-core-src-author-authority-ops-rs` (amended) | `ledger-core/src/author/authority_ops.rs#revoke_grant` (removed) | Callers learn init_namespace opens every namespace with its own genesis grant, root and accept roles and first policy under .decisions/ns/<ns>/ (a mandate is required), and that declare_role runs the role check over that namespace's authority, then the same delta-gate as every verb, refusing with AuthorError::Unauthorized or the verify finding; the grant verbs live in grant_ops (LP-6.5, ruling 47). |
| #159 | `seam/rust/ledger-core-src-author-grant-ops-rs` (new) | `ledger-core/src/author/grant_ops.rs#impl Author` (added) | Callers learn the grant verbs (grant, accept_grant, revoke_grant) file into the namespace of the grant they act under or name, with --namespace deciding where a grant is filed when the store holds several, and run the role check over that namespace's authority before the same delta-gate as every verb (LP-6.5, ruling 47). |
| #159 | `seam/rust/ledger-core-src-author-grant-ops-rs` (new) | `ledger-core/src/author/grant_ops.rs#accept_grant` (added) | Callers learn the grant verbs (grant, accept_grant, revoke_grant) file into the namespace of the grant they act under or name, with --namespace deciding where a grant is filed when the store holds several, and run the role check over that namespace's authority before the same delta-gate as every verb (LP-6.5, ruling 47). |
| #159 | `seam/rust/ledger-core-src-author-grant-ops-rs` (new) | `ledger-core/src/author/grant_ops.rs#grant` (added) | Callers learn the grant verbs (grant, accept_grant, revoke_grant) file into the namespace of the grant they act under or name, with --namespace deciding where a grant is filed when the store holds several, and run the role check over that namespace's authority before the same delta-gate as every verb (LP-6.5, ruling 47). |
| #159 | `seam/rust/ledger-core-src-author-grant-ops-rs` (new) | `ledger-core/src/author/grant_ops.rs#revoke_grant` (added) | Callers learn the grant verbs (grant, accept_grant, revoke_grant) file into the namespace of the grant they act under or name, with --namespace deciding where a grant is filed when the store holds several, and run the role check over that namespace's authority before the same delta-gate as every verb (LP-6.5, ruling 47). |
| #159 | `seam/rust/ledger-core-src-authority-references-rs` (amended) | `ledger-core/src/authority/references.rs#faults` (signature-changed) | Consumers learn the cross-record SCHEMA rules: every reference resolves, grant acceptances are the holder's over the grant's hash, nothing is revoked or closed twice, a policy chain has one root and one tip. |
| #159 | `seam/rust/ledger-core-src-authority-structure-rs` (amended) | `ledger-core/src/authority/structure.rs#entry_faults` (signature-changed) | Consumers learn the single-record SCHEMA rules for roles, grants, intervals, bindings and policies, judged in the record's own namespace: a grant's scope is read inside that namespace (* is the whole namespace) and a scope naming another namespace is a schema fault (LP-6.16, ruling 47). |
| #159 | `seam/rust/ledger-core-src-authority-view-rs` (amended) | `ledger-core/src/authority/view.rs#as_of` (signature-changed) | Callers learn the Authority view is built per namespace, of(store, ns) now and as_of(store, landing, pos, ns) at a past act, and answers live, accepted, available, revoked, superseded, the genesis and the policy in force for that namespace alone (LP-6.5, ruling 47). |
| #159 | `seam/rust/ledger-core-src-authority-view-rs` (amended) | `ledger-core/src/authority/view.rs#of` (added) | Callers learn the Authority view is built per namespace, of(store, ns) now and as_of(store, landing, pos, ns) at a past act, and answers live, accepted, available, revoked, superseded, the genesis and the policy in force for that namespace alone (LP-6.5, ruling 47). |
| #159 | `seam/rust/ledger-core-src-store-rs` (amended) | `ledger-core/src/store.rs#role_in` (added) | Consumers learn the store loads every namespace's sets, roles and log from .decisions/ns/<ns>/, that a loaded set, role or change-set carries the namespace of the directory it was read from, that a set or a role is looked up by (namespace, id) with set_in and role_in, and that every file must declare the format its fields need (LP-3.34, ruling 47). |
| #159 | `seam/rust/ledger-core-src-store-rs` (amended) | `ledger-core/src/store.rs#role` (removed) | Consumers learn the store loads every namespace's sets, roles and log from .decisions/ns/<ns>/, that a loaded set, role or change-set carries the namespace of the directory it was read from, that a set or a role is looked up by (namespace, id) with set_in and role_in, and that every file must declare the format its fields need (LP-3.34, ruling 47). |
| #159 | `seam/rust/ledger-core-src-verify-mod-rs` (amended) | `ledger-core/src/verify/mod.rs#namespace` (added) | Callers learn a verify report may carry an export stage: present only when asked for, conformant only when empty, so an export check that did not run never reads as one that passed. |
| #159 | `seam/rust/ledger-core-src-verify-mod-rs` (amended) | `ledger-core/src/verify/mod.rs#genesis_unbound` (signature-changed) | Callers learn a verify report may carry an export stage: present only when asked for, conformant only when empty, so an export check that did not run never reads as one that passed. |
| #159 | `seam/rust/ledger-core-src-verify-mod-rs` (amended) | `ledger-core/src/verify/mod.rs#namespaces` (removed) | Callers learn a verify report may carry an export stage: present only when asked for, conformant only when empty, so an export check that did not run never reads as one that passed. |
| #160 | `seam/rust/ledger-core-src-author-genesis-key-rs` (amended) | `ledger-core/src/author/genesis_key.rs#impl Author` (removed) | Callers learn the genesis holder's key at init: in every namespace init opens, the configured key is self-bound in the same change-set as the first policy, under the genesis mandate, and signs that policy; a later namespace binds the holder's live key there, signed by a key of theirs already trusted, and init is refused with no usable key unless --without-key proceeds unbound (LP-4.31, LP-4.38, ruling 47). |
| #160 | `seam/rust/ledger-core-src-verify-key-notices-rs` (new) | `ledger-core/src/verify/key_notices.rs#shared` (added) | Callers learn the two repository notices about keys across namespaces: a key closed in one namespace and open in another (KeySplit), and one key blob bound to different principals in different namespaces (KeyShared), each reported by verify and never a finding, because a key is per namespace (LP-4.37, LP-4.39, rulings 47 and 69). |
| #160 | `seam/rust/ledger-core-src-verify-key-notices-rs` (new) | `ledger-core/src/verify/key_notices.rs#split` (added) | Callers learn the two repository notices about keys across namespaces: a key closed in one namespace and open in another (KeySplit), and one key blob bound to different principals in different namespaces (KeyShared), each reported by verify and never a finding, because a key is per namespace (LP-4.37, LP-4.39, rulings 47 and 69). |
| #160 | `seam/rust/ledger-core-src-verify-key-notices-rs` (new) | `ledger-core/src/verify/key_notices.rs#KeyShared` (added) | Callers learn the two repository notices about keys across namespaces: a key closed in one namespace and open in another (KeySplit), and one key blob bound to different principals in different namespaces (KeyShared), each reported by verify and never a finding, because a key is per namespace (LP-4.37, LP-4.39, rulings 47 and 69). |
| #160 | `seam/rust/ledger-core-src-verify-key-notices-rs` (new) | `ledger-core/src/verify/key_notices.rs#KeySplit` (added) | Callers learn the two repository notices about keys across namespaces: a key closed in one namespace and open in another (KeySplit), and one key blob bound to different principals in different namespaces (KeyShared), each reported by verify and never a finding, because a key is per namespace (LP-4.37, LP-4.39, rulings 47 and 69). |
| #160 | `seam/rust/ledger-core-src-verify-key-notices-rs` (new) | `ledger-core/src/verify/key_notices.rs#bound_to` (added) | Callers learn the two repository notices about keys across namespaces: a key closed in one namespace and open in another (KeySplit), and one key blob bound to different principals in different namespaces (KeyShared), each reported by verify and never a finding, because a key is per namespace (LP-4.37, LP-4.39, rulings 47 and 69). |
| #160 | `seam/rust/ledger-core-src-verify-key-notices-rs` (new) | `ledger-core/src/verify/key_notices.rs#key` (added) | Callers learn the two repository notices about keys across namespaces: a key closed in one namespace and open in another (KeySplit), and one key blob bound to different principals in different namespaces (KeyShared), each reported by verify and never a finding, because a key is per namespace (LP-4.37, LP-4.39, rulings 47 and 69). |
| #160 | `seam/rust/ledger-core-src-verify-key-notices-rs` (new) | `ledger-core/src/verify/key_notices.rs#closed_in` (added) | Callers learn the two repository notices about keys across namespaces: a key closed in one namespace and open in another (KeySplit), and one key blob bound to different principals in different namespaces (KeyShared), each reported by verify and never a finding, because a key is per namespace (LP-4.37, LP-4.39, rulings 47 and 69). |
| #160 | `seam/rust/ledger-core-src-verify-key-notices-rs` (new) | `ledger-core/src/verify/key_notices.rs#holder` (added) | Callers learn the two repository notices about keys across namespaces: a key closed in one namespace and open in another (KeySplit), and one key blob bound to different principals in different namespaces (KeyShared), each reported by verify and never a finding, because a key is per namespace (LP-4.37, LP-4.39, rulings 47 and 69). |
| #160 | `seam/rust/ledger-core-src-verify-key-notices-rs` (new) | `ledger-core/src/verify/key_notices.rs#open_in` (added) | Callers learn the two repository notices about keys across namespaces: a key closed in one namespace and open in another (KeySplit), and one key blob bound to different principals in different namespaces (KeyShared), each reported by verify and never a finding, because a key is per namespace (LP-4.37, LP-4.39, rulings 47 and 69). |
| #160 | `seam/rust/ledger-core-src-verify-mod-rs` (amended) | `ledger-core/src/verify/mod.rs#key_notices` (added) | Callers learn a verify report may carry an export stage: present only when asked for, conformant only when empty, so an export check that did not run never reads as one that passed. |
| #160 | `seam/rust/ledger-core-src-verify-mod-rs` (amended) | `ledger-core/src/verify/mod.rs#key_shared` (added) | Callers learn a verify report may carry an export stage: present only when asked for, conformant only when empty, so an export check that did not run never reads as one that passed. |
| #160 | `seam/rust/ledger-core-src-verify-mod-rs` (amended) | `ledger-core/src/verify/mod.rs#key_split` (added) | Callers learn a verify report may carry an export stage: present only when asked for, conformant only when empty, so an export check that did not run never reads as one that passed. |
| #161 | `seam/rust/ledger-core-src-authority-signers-rs` (amended) | `ledger-core/src/authority/signers.rs#write` (signature-changed) | Consumers learn allowed_signers is derived per namespace from that namespace's key bindings in OpenSSH form, written by the identity verbs at .decisions/ns/<ns>/allowed_signers, that write reports what it wrote and removed (Synced) and removes a stale file in a namespace with no derived entry, and that verify's [SIGNERS] stage holds every namespace's file byte-identical (LP-4.10, LP-4.33). |
| #161 | `seam/rust/ledger-core-src-authority-signers-rs` (amended) | `ledger-core/src/authority/signers.rs#impl Synced` (added) | Consumers learn allowed_signers is derived per namespace from that namespace's key bindings in OpenSSH form, written by the identity verbs at .decisions/ns/<ns>/allowed_signers, that write reports what it wrote and removed (Synced) and removes a stale file in a namespace with no derived entry, and that verify's [SIGNERS] stage holds every namespace's file byte-identical (LP-4.10, LP-4.33). |
| #161 | `seam/rust/ledger-core-src-authority-signers-rs` (amended) | `ledger-core/src/authority/signers.rs#Synced` (added) | Consumers learn allowed_signers is derived per namespace from that namespace's key bindings in OpenSSH form, written by the identity verbs at .decisions/ns/<ns>/allowed_signers, that write reports what it wrote and removed (Synced) and removes a stale file in a namespace with no derived entry, and that verify's [SIGNERS] stage holds every namespace's file byte-identical (LP-4.10, LP-4.33). |
| #161 | `seam/rust/ledger-core-src-authority-signers-rs` (amended) | `ledger-core/src/authority/signers.rs#removed` (added) | Consumers learn allowed_signers is derived per namespace from that namespace's key bindings in OpenSSH form, written by the identity verbs at .decisions/ns/<ns>/allowed_signers, that write reports what it wrote and removed (Synced) and removes a stale file in a namespace with no derived entry, and that verify's [SIGNERS] stage holds every namespace's file byte-identical (LP-4.10, LP-4.33). |
| #161 | `seam/rust/ledger-core-src-authority-signers-rs` (amended) | `ledger-core/src/authority/signers.rs#written` (added) | Consumers learn allowed_signers is derived per namespace from that namespace's key bindings in OpenSSH form, written by the identity verbs at .decisions/ns/<ns>/allowed_signers, that write reports what it wrote and removed (Synced) and removes a stale file in a namespace with no derived entry, and that verify's [SIGNERS] stage holds every namespace's file byte-identical (LP-4.10, LP-4.33). |
| #161 | `seam/rust/ledger-core-src-authority-signers-rs` (amended) | `ledger-core/src/authority/signers.rs#lines` (added) | Consumers learn allowed_signers is derived per namespace from that namespace's key bindings in OpenSSH form, written by the identity verbs at .decisions/ns/<ns>/allowed_signers, that write reports what it wrote and removed (Synced) and removes a stale file in a namespace with no derived entry, and that verify's [SIGNERS] stage holds every namespace's file byte-identical (LP-4.10, LP-4.33). |
| #162 | `seam/rust/ledger-core-src-author-identity-ops-rs` (amended) | `ledger-core/src/author/identity_ops.rs#KeyArgs` (decorator-changed) | Callers learn identity add\|rotate\|revoke append key bindings in one namespace (the namespace's first is the genesis holder's, self-bound under the mandate), refuse a software key under an -sk policy, rewrite that namespace's allowed_signers, and that rotate and revoke with everywhere file one change-set per namespace the principal has the key open in, to land in one commit (LP-6.34). |
| #162 | `seam/rust/ledger-core-src-author-identity-ops-rs` (amended) | `ledger-core/src/author/identity_ops.rs#identity_revoke` (signature-changed) | Callers learn identity add\|rotate\|revoke append key bindings in one namespace (the namespace's first is the genesis holder's, self-bound under the mandate), refuse a software key under an -sk policy, rewrite that namespace's allowed_signers, and that rotate and revoke with everywhere file one change-set per namespace the principal has the key open in, to land in one commit (LP-6.34). |
| #162 | `seam/rust/ledger-core-src-author-identity-ops-rs` (amended) | `ledger-core/src/author/identity_ops.rs#identity_rotate` (signature-changed) | Callers learn identity add\|rotate\|revoke append key bindings in one namespace (the namespace's first is the genesis holder's, self-bound under the mandate), refuse a software key under an -sk policy, rewrite that namespace's allowed_signers, and that rotate and revoke with everywhere file one change-set per namespace the principal has the key open in, to land in one commit (LP-6.34). |
| #163 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#copy_fixture_into` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #163 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#copy_store_into` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #163 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#fixture_copy` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #163 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#workspace_copy` (added) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #163 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#stage_fixture` (removed) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #163 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#stage_fixture_into` (removed) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #163 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#stage_store_into` (removed) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
| #163 | `seam/rust/ledger-cli-tests-common-mod-rs` (amended) | `ledger-cli/tests/common/mod.rs#stage_workspace` (removed) | Test-only: the shared harness of the ledger-cli integration suites, with no consumer outside ledger-cli/tests/ — test authors learn signing invocations run under a pseudo-terminal through script(1) and everything else under pipes, so the binary's real TTY check is what the suites exercise, and that a repo's per-namespace paths and a fixture store copied in under ns/<namespace>/ come from here. |
