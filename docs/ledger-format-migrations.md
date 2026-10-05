# Decision Ledger Format Migrations

Every schema change to the `.decisions/` format is a version bump with a
migration note here, and `ledger verify` checks each entry against the
version it declares — existing entries never break silently. This file is the
migration record. The format itself is specified in `ledger-format-v1.md`.

Two versions move independently; both are recorded here.

| Version | Governs | A bump means |
|---|---|---|
| `format: N` on each file | how a file is read | older files keep working; validation is per declared version |
| `CANONICAL_FORM` in the hash prefix | how a version hash is computed | **every existing acceptance is invalidated** |

A `CANONICAL_FORM` bump is a governed act, not a fix. It is required whenever
a hashed field changes meaning, gains or loses membership in the hashed set,
or is normalised differently. It is *not* required for a `format` bump that
only adds an unhashed field.

---

## A key binding before its namespace's first policy is judged under it (2026-10-05, no format change)

**Ruled 2026-10-05, narrowing D5 (c).** A key binding is never exempt as a
pre-policy act. The pre-policy exemption covers acceptances and revocations
only.

**What the gate did before.** `signing/check.rs` `trust_bindings` skipped
any binding with no policy in force at its position (`let Some(policy) = …
else { continue }`). Such a binding was never trusted and named in no
finding: the verb's gate passed, `verify` passed, and the first act signed
with the key failed `L011`. It arises from a clock behind the policy's in a
later namespace, or by hand.

**What it does now.**
- A binding before its namespace's first policy is judged by D7 and by that
  first policy's requirement. Signed, it is trusted. Unsigned where that
  policy requires a signature, it is `L011` and never trusted. Under a
  `[none]` first policy, D7 alone decides it.
- A binding in a namespace no policy governs at all stays the schema fault
  it already was (`authority/references.rs`), so `init --namespace` refuses
  to govern a namespace holding an unsigned binding that its policy would
  fail. Every filed binding is trusted or named by a finding.
- `init --namespace` in a later namespace binds the genesis holder's live
  key there, in the same change-set and dated with the policy. If every key
  of theirs is closed it refuses, naming them, unless `--without-key`, which
  warns (before, it silently bound and signed nothing).
- D7 is not widened: a key trusted in another namespace vouches only for the
  holder's first binding in a namespace, so a closed key cannot re-enter
  through a namespace where it is still live.

**What can move.** No digest moves, and no file is rewritten. A store
holding a binding before its namespace's first policy now reports:
- that binding as trusted, if it was signed by a key the check accepts;
- `L011`, if it was unsigned under a first policy that requires signing.

This repository's store has no key bindings. Every fixture store's
bindings land after their first policy, and all suites pass unchanged.

---

## A first policy is signed; `init --namespace` binds the genesis key; the `[none]` notice (2026-10-05, #96, no format change)

**What changed.**
- **`init --namespace`** files the genesis holder's self-bound binding in
  the same change-set as the genesis grant and the first policy, whenever
  the genesis holder has no key in the store. The key is the one
  `git config user.signingkey` names. The binding is signed by the key it
  binds, and the same key signs the first policy. **With no usable key it
  refuses** (ruled 2026-10-05), naming what is missing. `--without-key`
  initialises unbound, as before, with a warning.
- **`verify`** says, as a notice, while the genesis holder has no trusted
  key (`genesis_unbound` in `--json`).
- **Callers moved.** 19 test call sites in 13 files bootstrap a namespace
  with no key configured. They now pass `--without-key`. Later-namespace
  calls, made after a key is bound, are unchanged.
- **A namespace's first policy is a signing subject** (`signing/subject.rs`
  `subjects`), judged under its own schemes when its `by` held a live
  trusted key at its position (`signing/check.rs` `judge_first_policy`). In
  a later namespace `init --namespace` signs it with the genesis holder's
  live key.
- **`verify`** prints a notice for every namespace whose policy in force is
  `[none]` (`Report::unsigned`).

**What can move.** No digest moves, and no file is rewritten. A store in
which a later namespace's first policy was filed **unsigned after** its
author already held a trusted key now reports `L011` on that policy. No such
store is known: this repository's own store has no policy, and every test
fixture's first policies predate their keys. The remedy is a sidecar over
the policy's unchanged signed bytes, made with the key that was live at its
`at`. The CLI has no verb for that yet.

**Not gated by format.** The requirement applies to every first policy. A
format gate would let a writer avoid it by declaring the older format.

---

---

## `A006` judges policy authors; roles take effect from their position (2026-10-05, no format change)

**Policy authors — what the gate did before.** No `verify` check looked at
who filed a policy. `verify/acts.rs` `unauthorised` judged acceptances and
revocations only; `signing/check.rs` checked that a policy change was
signed by its `by` under the policy it replaced, and a namespace's first
policy is unsigned by design. Only the verbs (`init --namespace`, `policy
set`) required the genesis holder. So a hand-filed policy change by any
principal with a bound key, signed with that key, passed `verify` and
became the policy in force; under `[none]` the same change needed no
signature at all; and a hand-filed first policy for an ungoverned namespace
by anyone passed.

**What it does now.** `unauthorised` judges every policy, first or change,
through `policy_verdict`: its `under` must name the genesis grant as of the
policy, held by its `by`, live and available at its `at`, through
`authorize_named` (as `Act::SetPolicy`, which needs `grant-role`, carried
by every genesis role). Only the genesis grant authorises a policy: a
holder of `grant-role` over `*` under any other grant does not, signed or
not. A policy that fails is `A006`.

**Roles — what the gate did before.** `Authority::as_of` admitted every
role file whatever its landing, so a role landed after an act still counted
for it. **Now** a role counts only for acts that landed no earlier than
its file — landing alone, because a role file carries no signed `at` and
its `created_at` is in no payload (`role_landing` in `authority/view.rs`).
An act made under a grant whose role landed later fails `A006`; a role and
an act landed in one commit stand together, whatever the role is dated.

No new class, no format change, no digest moves: both are extensions of
`A006` by the `L010` mechanism. This repository's store has no policy and
no role file, and is unaffected. A store holding such a policy or such an
act now fails `A006`; how it recovers is not ruled here.

## `A006` judges old-style revocations after the first policy (2026-10-05, no format change)

**What the gate did before.** `verify/acts.rs` `unauthorised` role-checked
only `rev:` revocations (`.filter(|r| r.is_entity())`), and
`signing/subject.rs` `subjects` made only `rev:` revocations signing
subjects. Yet `verify/view.rs` `View::build` and `Authority::as_of` count
every revocation, either shape, as revoking. So in a governed namespace a
hand-filed format 1–5 file holding an old-style revocation (`acceptance`,
`by`) of another holder's acceptance, unsigned and naming no grant, passed
`verify`; the acceptance counted as revoked, and the export carried the
revocation.

**What it does now.** An old-style revocation is valid only as a pre-policy
act. One that is not before its namespace's first policy (D6: dated *and*
landed before it) fails `A006` (spec §3.10.7). No new class, no format
change, no digest moves: the rule is an extension of `A006` by the `L010`
mechanism.

**For an existing store.** A store whose governed namespace holds an
old-style revocation landed after its first policy now fails `A006`, and
keeps failing: a landed entity is never edited or removed (§3.10.7), so
there is no in-place remedy, and how such a store recovers is not ruled
here. This repository's store has no policy in any namespace and is
unaffected.

## Three format 1 declarations corrected to format 4 (2026-10-05, #81, no format change)

**What was wrong.** Spec §3.7 makes a lower-format file carrying
`revisit_if` a schema fault, but the loader had no row for it
(`ledger-core/src/store.rs` `format_faults` checked formats 2, 3, 5, 6 and
7). Three change-sets in this repository's own store carried `revisit_if`
under `format: 1`:

- `.decisions/log/01KZX70EMPA47TBR0PFKX4M32Z.yml`
- `.decisions/log/01KZX70EQGQCB1B190TS9FZ1A2.yml`
- `.decisions/log/01KZX70ET1GMR2012XKEP5EWDW.yml`

They are the three re-decisions the format 4 note below describes, filed by
`ledger revise` on 2026-08-13 (`ecd1ce2`). At that commit only `add`
stamped `format::needed_for` (`author/decision.rs`). Every verb that files
the next version of a decision — `revise`, `allocate`, `escape`,
`supersede`, through `next_version` in `author/version_ops.rs`, each
inheriting the parent's `revisit_if` — built its change-set from
`Author::shell` at `CURRENT_FORMAT`, and `Author::append` wrote it as-is.
Since #67 (`4800589`) `Author::append` raises every verb's change-set to
`needed_for`, so no verb can write such a file now.

`.decisions/log/01KZX70S86QGXVCA5GW5WSY6XA.yml`, listed beside them in #81,
carries no `revisit_if` field: the word appears only inside its `note` and
`statement` text. It needs format 1 and still declares it.

**The ruling** (principal, 2026-10-05): a landed log file's `format:`
declaration may be corrected — raised only, only to the lowest format the
file's content needs, and nothing else in the file changed (spec
§3.10.7). Recorded in `docs/signing-rulings-2026-10.md`.

**What changed.** The three files' first line, `format: 1` → `format: 4`,
and nothing else. `format` is a `ChangeSet` field, not a `VersionRaw` one,
so no version hash moves (`ledger-cli/tests/digests.rs`); the three
acceptances still sign their versions; the landed-entity rule
(`ledger-core/src/verify/history.rs`) reads `format` as no entity
(`ledger-core/src/landed.rs` `entities`), so it does not fire; and the
graph and the committed exports carry no `format`, so `docs/decisions/*.nt`
is byte-identical. The loader gains the `REVISIT_FORMAT` row, so a
lower-format file carrying `revisit_if` is now a `SCHEMA` fault, as §3.7
always said.

**For another store.** A store that filed a version carrying `revisit_if`
with any verb but `add` before #67 may hold the same fault and will now fail `verify` with
``carries `revisit_if`, a format 4 field — declare `format: 4` ``. The
migration is the same edit: raise that file's declaration to `format: 4`
(or to the higher format its other content needs) and change nothing else.

## Format 7 / Spec v1.8 — signing; `L011`, `L012`, `A006` (2026-10-04, #70)

**New fields.** `under` (the grant an act is made under, D9 (a)) on
acceptances, `rev:` revocations, grants, policies and key bindings — hashed
when present, omitted when absent. `at` joins the policy payload, always
(D8), and a policy becomes a format-7 entry: a policy in a file below
format 7 is a schema fault. No committed store carried a format-6 policy
(namespace policy arrived with format 6 in the same release train, #80), so
none needs rewriting; a store that has one re-files it with `ledger policy
set`. A change-set with `under` anywhere, or with a policy, declares
`format: 7`. The inline acceptance `signature` field is retired:
permanently empty in every format.

**New payload.** `ledger.acceptance.v1` over `{decision, version, actor, at,
scope, expires_at, under}`. Acceptances had no digest before, so none
moves; the digest is computed, never stored.

**No existing digest moves.** `CANONICAL_FORM` is unchanged; every
version digest re-derives unchanged (`ledger-cli/tests/digests.rs`), and the
grant, binding and revocation payloads are pinned against `main` at 88b3de1
(the policy's pin is dropped: its payload now always carries `at`)
(`ledger-core/src/authority/payload_tests.rs`).

**Signatures** are sidecars at `.decisions/sig/<ulid>.<scheme>.sig`
(`ssh`; `dsse` verified only; `none` is no sidecar). The signed bytes are the
digest's input, `prefix || 0x0A || canonical JSON` (§3.10.3).

**Two file-gate classes** take their reserved numbers: `L011` (a required
signature absent or invalid, including one dated or landed after its key's
close) and `L012` (an acceptance under a since-closed key, not re-accepted by
the policy's deadline; before it, a review item). The closed count moves
from twelve to fourteen. `L012` is a release disposition, outside the
readiness gate. The graph stage gains `A006`, the role check over history.
`L007` gains two history rules: a landed role file changed (roles are
write-once), and a namespace's policy removed (opting in is one-way).

**Migration note.** Nothing to migrate mechanically. A namespace with no
policy is unaffected: nothing in it is signature- or role-checked, and
`verify` now names it as unchecked. A namespace already under policy (from
v1.7) now requires its policy's schemes on every act after its first
policy: its key bindings must be signed (the genesis holder's first by the
key it binds, a principal's first by the genesis holder, D7), and acts
must name their grant. A v1.7 store's `allowed_signers` was written with
space-separated options, which `ssh-keygen` cannot read; `ledger identity
sync` rewrites it in the corrected comma form. No committed store in this
repository carried one.

**Out of this format:** signed grants and grant acceptances, and `at` in
the grant payload (D5 (b)) — #82.

---

## Format 6 / Spec v1.7 — the authority records; `A003`, `A005` (2026-10-02)

A **`format` bump without a `CANONICAL_FORM` bump** (#69; the revocation
entity's use for acceptances lands with #66). `format: 6` adds the
authority records (spec §3.9): `roles/<id>.yml`, and as log entries
`grants`, `grant_acceptances`, `unavailabilities`, `availabilities`,
`key_bindings`, `policies`, and the `rev:` revocation entity. Nothing about
a decision version changes, so no version digest moves and no acceptance is
invalidated.

**New hash prefixes**, each over a closed payload under the one
canonical-JSON law (spec §3.9.3): `ledger.authority-grant.v1`,
`ledger.revocation.v1`, `ledger.identity-binding.v1`,
`ledger.namespace-policy.v1`.

**No new file-gate class.** `SCHEMA` gains the structural and referential
rules of §3.9.4; `L006` is extended to every identity an authority record
names; `L007` to every stored authority hash — each by the `L010`
mechanism (stricter, additive). The closed count stays twelve. The graph
stage gains `A003` and `A005` (closed count of graph classes: eight), and
`verify` gains the `[SIGNERS]` stage holding `.decisions/allowed_signers`
byte-identical to the key bindings.

**Migration note.** Nothing to migrate mechanically. A store with no
authority records stays a pre-v2 store: no namespace has a policy, nothing
is role-checked, and every existing acceptance is judged exactly as before.
`ledger init --namespace <ns> --external-ref <mandate>` puts a namespace
under policy; the store's first one bootstraps the genesis (the root role,
the genesis grant, the holder's acceptance of it). As landed in #80 the root
role carried every capability; since #85 (D9 (f)) it carries the four
authority capabilities only, and the policy's accept role is a separate
role (`acceptor` by default) that nobody holds until it is granted. From
then on `accept` and `revoke` in that namespace need a live, accepted,
available grant of the policy's accept role. Legacy revocations stay in
their legacy shape — a log file is never rewritten.

**Revocation as its own entity (#66, ruling 3).** `ledger revoke` now
files the format-6 `rev:` entity — `{id, revokes, actor, at, reason,
hash}` under `ledger.revocation.v1` — for an acceptance as for a grant.
`L006` covers the revoker in both shapes, so a model identity can no longer
revoke a person's acceptance; in a namespace under policy the revoker needs
the accept role. The graph no longer writes `ledger:revokedAt` /
`ledger:revokedBy` / `ledger:revocationReason` on the acceptance IRI: a
revocation is a `ledger:Revocation` node naming the acceptance with
`ledger:revokes` (a legacy revocation at `<urn:rev:legacy-<acc-ulid>>`),
and no triple is ever added to an acceptance after it is filed. This
changes the committed export of any namespace with a revocation — this
repository's store has none, so its exports regenerated byte-identical.
Downstream, Hafeok/decision-driven-analyzers#83 reads both shapes during the
transition (tracked in #76).

**Deferred to Session B** (signing): that every key binding and every policy
change is signed under the policy in force before it, and the `L011`/`L012`
checks against `allowed_signers`. Until then a policy change carries the
hash of the policy it replaces and is the genesis holder's act.

## Format 5 / Spec v1.6 — `key` and `exported`; `L013`, `L014`, `G006` (2026-10-02)

A **`format` bump without a `CANONICAL_FORM` bump**, by the formats 2–4
pattern: `format: 5` adds two optional version fields (#67, PRD
`ledger-cli-prd.md` §4 as amended 2026-10-01, ruling 2).

- **`key`** — the decision's stable human name, `^[A-Z][A-Za-z0-9]{0,63}$`,
  what the analyzers' generator turns into a type name. A malformed key is
  a `SCHEMA` fault at parse.
- **`exported`** — citable from other namespaces. Hashed as the string
  `"true"` when set; absent (and omitted from the canonical object) when
  false, because hashed content is strings only.

Both are hashed when present and omitted when absent (spec §4.2 step 3),
so every version written before them canonicalises to byte-identical
content: no digest moves, no acceptance is invalidated, the prefix stays
`ledger.decision-version.v1`. **Proof:**
`ledger-cli/tests/digests.rs` re-derives every stored digest in every
fixture store and in this repository's own `.decisions/` log under the
current canonical form and asserts none moved; `canon_tests.rs` adds both
fields to the mutation table (each moves the hash when present) and pins
that an absent key and `exported: false` canonicalise like an unwritten
field.

**Two file-gate classes, by the `L010` amendment mechanism** (ruling 2:
the key rules belong to the import surface, never graph-only). `L011` and
`L012` stay reserved for the signing classes (#65, ruling D3), so these
take the next free numbers:

| Code | Fails when |
|---|---|
| `L013` | a version's `key` differs from the key its `parent` or `merged_from` carries |
| `L014` | two live decisions of one namespace carry the same `key` on their latest versions |

The file gate's closed count moves from **ten to twelve** (`finding.rs`
`there_are_exactly_twelve_semantic_classes_plus_the_parse_gate`; the CLI
suite's `fails_only_with` list and the new `l013` / `l014` fixtures).
`L014` gains a graph-stage SPARQL cross-check, **`G006`** — never its only
home. The emitter writes `ledger:key` and `ledger:exported "true"`.

**Migration note.** Nothing to migrate mechanically: existing stores are
format 1–4 and stay valid, and a writer declares `format: 5` only on a
change-set that carries a key or an export flag. **Giving an existing
decision a key is a new version** (`ledger revise <id> --key <Key>`): the
hash moves, so every acceptance of the keyless version becomes history and
the keyed version **needs re-acceptance** — the same rule as any other
edit to hashed content, deliberately not waived for a naming act. Once
given, the key is carried by every later version (the authoring verbs copy
it from the parent) and a rename is refused by `L013`. A superseded
decision's key is free: the successor may carry it (`L014` counts live
decisions only). Carrying the key automatically across `supersede` waits
on one-act supersession (audit C9).

## Format 4 / Spec v1.5 — the `revisit_if` reopen edge (2026-08-13)

A **`format` bump without a `CANONICAL_FORM` bump**, by the same
reasoning as formats 2 and 3: `format: 4` adds exactly one optional
version field, `revisit_if` — a list of pointers to claims whose *death
reopens* the decision. The field is hashed when present; an absent key is
omitted from the canonical object (spec §4.2 step 3), so every version
written before it existed canonicalises to byte-identical content. No
digest moves, no acceptance is invalidated, the prefix stays
`ledger.decision-version.v1`, and the file gate's ten classes are
unchanged.

**What was ruled.** The principal ruled (2026-08-13) that a
watched-not-grounding edge is a **distinct edge type, not a basis**: this
claim's death reopens the decision; it is not the decision's ground. It is
filed as its own edge with its own vocabulary, never inside `based_on`, so
neither the basis-loss scan nor `why` reads it as ground — and a claim's
status movement on a `revisit_if` edge produces a **reopen** finding, not a
basis-loss finding. The two mean different things and must report
differently. This settles the question the 2026-08 basis-quality re-typing
session left open and the ddd M8 migration carried as a provisional
`watched:` marker inside `based_on`.

Rules that arrive with it (spec §3.7):

- A writer declares `format: 4` only on a change-set that actually carries
  a `revisit_if` — a store that never states one remains a pure
  format-1/2/3 store. A lower-format file carrying the field is a schema
  fault (the `merged_from` rule, applied to a field again).
- The two edge lists canonicalise under separate keys, so one token filed
  as ground and the same token filed as a reopen edge are different
  content: an acceptance always names which of the two it signed.
- Resolution — does the named claim exist, has it moved — is not the file
  gate's business, the same posture as every discharge scheme and every
  basis pointer at L0. The reopen finding is a consumer's report
  (`ddd report escapes`), never an eleventh class.

**Migration note.** Nothing to migrate mechanically: existing stores are
format 1–3 and stay valid. What *was* migrated is the three provisional
`watched:` markers the M8 migration filed inside `based_on`
(`DDD-adapter-02` on `dec/ddd/internal-not-surface`, `DDD-gates-01` on
`dec/rust/no-unwrap`, `DDD-adapter-01` on `dec/ddd/m6-proceeds-no-flip`).
Each moved to a real `revisit_if` edge as a **new version filed for the
principal's acceptance** — a re-decision of that entry's edge, not a
silent rewrite. Their prior versions keep signing exactly the historical
content they named; the new versions await a fresh signature. The
`.ddd/` store adopts the same one shape at its own format 7
(`ddd-format-migrations.md`).

**Left blocked, deliberately.** The 2026-08 provenance audit's one
*upstream* watched-not-grounding row — `dec/ddd/workspace-member-delivery`
tracking the What/How vocabulary — is still not expressible: it needs a
cross-repo reference shape (a repo pin plus a revision), which is the
subject of a separate amendment that has **not landed**. `revisit_if` gives
the relation a home; it does not give a cross-repo pointer one. That row
stays unfiled until the cross-repo amendment lands, at which point it
becomes expressible with no further change to this field.

**Renumbering note:** the L6 signing revision, which had renumbered from
`format: 2` to `format: 3` when L3 consumed its slot and to `format: 4`
when M8 consumed that one, renumbers a third time to **spec v1.6 /
`format: 5`**. Nothing else about the L6 plan changes; it remains ruled
and unimplemented.

## Format 3 / Spec v1.4 — the `contract:` discharge scheme (2026-08-12, ddd M8)

A **`format` bump without a `CANONICAL_FORM` bump**, by the same
reasoning as format 2: `format: 3` adds exactly one discharge scheme,
`contract:<boundary>` — the repository-diff contract check as a
discharge kind (M8 ruling 5: the CI contract-check discharge kind is
added through this amendment procedure, at the current version, with
this note). A `contract:` pointer names a declared boundary (a
`seam/...` declaration id or a `file#symbol` contract location) whose
changes are validated in CI by the shared classifier: every
contract-surface change in a revision range must be discharged by a
declaration signing that exact transition.

Rules that arrive with it:

- A writer declares `format: 3` only on a change-set that actually
  carries a `contract:` pointer — a store that never uses the scheme
  remains pure format 1/2. A lower-format file carrying one is a schema
  fault (the `merged_from` rule, applied to a scheme).
- Hashing is unaffected: a discharge pointer was always hashed by its
  string form, so no digest moves, no acceptance is invalidated, and the
  hash prefix stays `ledger.decision-version.v1`.
- The scheme's *resolution* (does the named boundary exist; is the CI
  check actually wired) is not the file gate's business — same posture
  as every other scheme at L0.

**Renumbering note:** the L6 signing revision, which had renumbered from
`format: 2` to `format: 3` when L3 consumed its slot, renumbers a second
time to **spec v1.5 / `format: 4`**. (Renumbered again by the v1.5 reopen
edge — see the format-4 entry above; L6 now holds spec v1.6 / `format: 5`.)
Nothing else about the L6 plan changes; it remains ruled and unimplemented.

**Migration note:** nothing to migrate. Existing stores stay valid; the
first consumers of the scheme are the ddd M8 migration's seam-declaration
entries.

## Format 2 / Spec v1.3 — `merged_from`; `G005` (2026-08-11, L3)

A **`format` bump without a `CANONICAL_FORM` bump**, and the reasoning is
part of the record: `format: 2` adds exactly one optional version field,
`merged_from` — the other tip a merge arbitration closed. The field *is*
hashed when present, but an absent key is omitted from the canonical
object entirely (spec §4.2 step 3), so every version written before the
field existed canonicalises to byte-identical content: no digest moves,
no acceptance is invalidated, and the hash prefix stays
`ledger.decision-version.v1`.

Rules that arrive with it:

- A writer declares `format: 2` only on a change-set that actually carries
  `merged_from` — a store that never merged remains pure format 1. A
  format-1 file carrying the field is a schema fault.
- A version's tip-hood is judged over `parent` **and** `merged_from`: a
  reconciled version closes the tip it names, which is how a `G004` fork
  heals inside the DAG rather than by editing history.
- The graph stage gains `G005` (one decision superseded by two live
  claimants — the write-time fork refusal met across branches) and `G002`
  now also polices a dangling `merged_from`.
- No acceptance survives reconciliation. A reconciled version is a new
  version awaiting a fresh signature; prior acceptances keep signing the
  historical versions they named. Same law as `revise`.

**Migration note:** nothing to migrate. Existing stores are format 1 and
stay valid; they gain `G005` checking, which can newly fail a store that
already carried a silent competing supersession — that is the point.

## Spec v1.2 — latest from the parent DAG; `G004` (2026-08-11)

An **amendment to the specification document**, not a `format` bump and
not a `CANONICAL_FORM` bump: no file schema changes, no hashed field
changes meaning, every existing acceptance stays valid.

The shipped L1 computed a decision's latest version by ULID order of
change-sets — the single-writer leak the L1+L2 report named: ULIDs order
by one clock, and two writers' clocks prove nothing about parenthood. As
of v1.2, §5.2 defines "latest" as **the unique version whose hash no
other version of the same decision names as `parent`** — the parent DAG
(which `G002` already polices) is the authority, and file order is not
consulted. Content-identical filings of one hash are one version.

A chain with more than one tip has **no** latest, and no ordering
heuristic may pick one. The reference implementation's graph stage gains
`G004` (forked version chain) for exactly that state, and its authoring
verbs refuse to extend or sign a forked decision. The remedy is an
arbitration recorded through `ledger merge --resolve` (L3), never a
silent resolution.

**Migration note:** a single-writer store is unaffected — a linear chain's
tip is the same version ULID order found, so no hash moves and no
acceptance is disturbed. A store already carrying interleaved clocks may
change which version `status`/`coverage`/the gate judge as latest; the DAG
reading is the correct one and the ULID reading was the defect. A store
carrying an undetected fork newly fails `G004` — that is the point of the
amendment.

## Spec v1.1 — the tenth class, `L010` (2026-08-10)

An **amendment to the specification document**, not a `format` bump and not
a `CANONICAL_FORM` bump: no file schema changes, no hashed field changes
meaning, every existing acceptance stays valid.

v1.0's §6 stated the gap plainly: the model-identity rule was scoped to
`accepted-by`, and the classes were closed at nine — so a *judgment
allocated to a model actor* passed the gate. Ruled by the principal
(2026-08-10): the gap closes as `L010` — a `judgment`'s `actor` refused by
the §3.2 identity rules fails the gate. The class runs under both gates and
judges latest versions only, like every allocation rule.

**Migration note:** the amendment is additive and stricter. A store that was
conformant under v1.0 may newly fail `L010` — that is the point, not a
regression. The remedy is a new version reallocating the judgment to an
accountable human actor (or to another store); there is nothing to rewrite,
because history is append-only. The "exactly nine" contract is now "exactly
ten" everywhere it is asserted, including the closed-enum count test.

## L2 — the graph stage arrives (2026-08-10, no format change)

Not a `format` bump, not a `CANONICAL_FORM` bump, not part of the file
gate's ten classes. `.decisions/index/` (reserved since format 1) is now
written by `ledger reindex` as byte-deterministic Turtle, and `verify`
gains a distinct graph stage (`G001`–`G003`, cross-entry referential
integrity) with unchanged exit semantics. Specified in
`ledger-format-v1.md` §8, explicitly outside the import surface an
outside implementation must reproduce.

## Format 1 / `ledger.decision-version.v1` (L0)

The baseline. Two file schemas — the set file and the change-set log file —
plus the canonical form and its pinned conformance vector, all specified in
`ledger-format-v1.md`.

Nothing to migrate from.

### Fields reserved but inert at this version

These exist in the schema so their milestone is additive rather than a
migration. Writing them is legal where noted; nothing reads them yet.

| Field | Reserved for | State at format 1 |
|-------|--------------|-------------------|
| `signature` on an acceptance | OD-3's cryptographic upgrade | must be **empty**; a non-empty value is a schema fault |
| `supersedes` on a version | supersession (no command before L1) | may be written; hashed; unresolved |
| `based_on` on a version | §9.4 basis pointers, resolved at L4 | may be written; hashed; vocabulary open |
| `parents` on a change-set | merge (L3) | may be written; still unresolved — L3 shipped on the *version* DAG (`parent`/`merged_from`), not the change-set DAG |
| `scope: class:<ref>` on an acceptance | precommitment (L1) | parses; the acceptance still signs one version hash |
| `.decisions/index/` | the RDF materialized view (L2) | not created; ignored by git |

### Known future migrations

Recorded now so the shape of the change is not a surprise.

- **OD-6 — expiry default.** If `expires_at` becomes mandatory, that is a
  further `format` bump with a migration path for entries that carry none
  (this entry originally said `format: 2`, a number since consumed by L3's
  `merged_from`). Hashing is unaffected: `expires_at` is not hashed.
- **OD-3 — signatures. Ruled 2026-10-02 on #65; Session B.** This entry
  originally read "populating `signature` is a `format: 2`"; that number
  and its successors were consumed by L3's `merged_from`, M8's `contract:`
  scheme, v1.5's `revisit_if` and v1.6's keys (`format: 5`). The rulings
  (`ledger-cli-prd.md` §0 items 7–12) replace the August plan: signing is
  required by **namespace policy**, not tier; the signature lives in a
  **sidecar** `.decisions/sig/<acc-ulid>.<scheme>.sig` and the inline
  `signature` field is retired (required empty, permanently); schemes are
  `ssh`, then `dsse` verification, `none` for pre-v2 stores only. `L011`
  (a required signature absent or invalid) and `L012` (acceptances under a
  since-closed key — a review trigger) **keep their reserved numbers**;
  `L013`/`L014` (format 5) took the next free ones. Hashing is unaffected:
  the signature is over a closed payload, never inside a version hash.
- **§9.4 — upstreams manifest.** A new file schema, not a change to these
  two. Closing the `based_on` vocabulary at that point **is** a hashed-meaning
  change and would require a `CANONICAL_FORM` bump, so the closure should
  arrive as validation over an unchanged canonical form instead. The same
  reasoning applies to `revisit_if`, whose vocabulary is open for the same
  reason and closes by the same route.
- **The cross-repo reference shape.** A basis or reopen pointer that names
  a *repository* and a revision, so an edge can cross a store boundary.
  Not landed as of 2026-08-13; the 2026-08 provenance audit's one upstream
  watched-not-grounding row waits on it (see the format-4 entry). Whatever
  shape it takes applies to both pointer fields — one shape, both edges.
