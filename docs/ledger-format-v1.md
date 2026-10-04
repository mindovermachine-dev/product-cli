# Decision Ledger — Entry Format v1

**Status:** normative for `format: 1` through `format: 6`.
Specification revision **v1.7** (2026-10-02): introduces `format: 6`,
the **authority records** (§3.9) — role files, grants and their
acceptances, unavailability and availability, key bindings, namespace
policy, and the `rev:` **revocation entity** that revokes a grant or an
acceptance. No version field changes, so **every existing digest is
unchanged**; four new payload prefixes are added (§3.9.3). No new file-gate
class: `SCHEMA`, `L006` and `L007` are extended to the new records by the
`L010` mechanism; the graph stage gains `A003` and `A005`. Revision
**v1.6** (2026-10-02): introduces `format: 5`,
which adds two optional version fields, `key` and `exported` (§3.8), and
two file-gate classes, `L013` (key immutability) and `L014` (key
uniqueness among live decisions), by the `L010` amendment mechanism —
the file gate's closed count moves from ten to twelve, with `L011`/`L012`
reserved for signing. Both fields are hashed when present and omitted when
absent, so **every existing digest is unchanged** and `CANONICAL_FORM`
does not bump. Revision
**v1.5** (2026-08-13): introduces `format: 4`,
which adds one optional version field, `revisit_if` — the reopen edge,
ruled by the principal a **distinct edge type and never a basis** (§3.7).
The field is hashed when present and omitted when absent, so **every
existing digest is unchanged** and `CANONICAL_FORM` does not bump; the
file gate's ten classes were unchanged. Revision
**v1.4** (2026-08-12, ddd M8): introduces
`format: 3`, which adds one discharge scheme, `contract:` — the
repository-diff contract check as a discharge kind (§3.4). No field
changes, no hashed-meaning changes: **every existing digest is
unchanged** and `CANONICAL_FORM` does not bump. Revision **v1.3**
(2026-08-11, L3) introduced `format: 2`, which adds one
optional version field, `merged_from` — the other tip a merge arbitration
closed. The field is hashed when present and omitted when absent, so
**every format-1 digest is unchanged** and `CANONICAL_FORM` does not bump;
the graph stage gains `G005` (competing supersession). Revision **v1.2**
(2026-08-11) defined "latest" as derived from the version parent DAG
rather than file or ULID order (§5.2) and added `G004`, the forked-chain
shape (§8). Revision v1.1 (2026-08-10) added gate class `L010`. The file
gate's ten classes were unchanged by those four revisions; v1.6 is the
first since v1.1 to add classes — see `ledger-format-migrations.md`.
**Scope:** L0 of `decision-ledger-prd.md` — the file format, the canonical
form, the version hash, and the `verify` gate. No graph, no index, no merge,
no coverage query, no federation. (The L2 graph stage reports through the
same `verify` command but is a distinct stage outside this document's
class set; see §8.)
**Audience:** an implementer building a second implementation of this format,
working from this document alone. Where this document and the reference
implementation (`ledger-core`) disagree, this document wins.

A companion, `ledger-format-migrations.md`, is the migration record: every
schema change is a `format` bump with a note there, and validation is always
against the version an entry declares.

---

## 1. What the format is for

A decision has identity independent of any repository, content-addressed
versions, and an acceptance that signs a **hash**, never an id. That is the
property that makes acceptance mean something: it names an exact state, not
"whatever this decision currently says". Every rule below exists to keep that
property true across machines, platforms, and independent implementations.

Two versions carry the load:

| Version | Governs | Bump when |
|---|---|---|
| `format` | how a *file* is read | a field is added, removed, or re-shaped |
| `CANONICAL_FORM` (§4) | how a *hash* is computed | a hashed field changes meaning |

They are independent. A `format` bump that leaves hashed semantics alone
keeps every existing acceptance valid. A `CANONICAL_FORM` bump does not, and
is therefore a governed act, not a fix.

---

## 2. Storage layout

```
.decisions/
  sets/<set-id>.yml           declared scope: floor, ground, owner
  log/<changeset-ulid>.yml    append-only; the source of truth
  index/                      gitignored; rebuildable cache (not written at L0)
```

- Files are read with either `.yml` or `.yaml`; writers emit `.yml`.
- A log file is written once and **never edited**. A correction is a new
  version; a reversal is a revocation.
- The file stem must equal the id it declares — `<ulid>.yml` for a change-set
  whose `id` is `cs:<ulid>`, `<set-id>.yml` for a set. A disagreement is a
  schema fault.
- `index/` is not created at L0. The ignore line exists so an L2 rebuild
  cache can never be committed by accident.

---

## 3. Schemas

Every file declares its `format` (1 unless it uses a later field — §§3.6–3.8).

### 3.1 Identifiers

```
dec:<namespace>/<ulid>     a decision, stable forever
cs:<ulid>                  a change-set
acc:<ulid>                 an acceptance
sha256:<64 lowercase hex>  a version hash
```

- **ULID**: 26 characters of Crockford base32 (`0123456789ABCDEFGHJKMNPQRSTVWXYZ`
  — `I`, `L`, `O`, `U` excluded). The first character must not exceed `7`;
  above that overflows the 48-bit millisecond timestamp field. Offline
  generatable, lexicographically sortable by creation time.
- **Namespace**: an *owning scope*, not a repo path. Dot-separated segments of
  lowercase alphanumerics and dashes. Repos may reference decisions in
  namespaces they do not own.
- A decision id is permanent. Supersession mints a new id carrying a
  `supersedes` edge; it never mutates or reuses one.
- A decision's namespace is not restated as a separate field: it is inside
  the id, and a second spelling of one fact is a second thing that can
  disagree.

### 3.2 Identity — the actor an acceptance names

An identity is an **email address**, normalised to lowercase at parse.
`§4.4` of the PRD requires `accepted-by` to *resolve* to a human identity; an
address resolves, a display name decorates. Comparing addresses also makes
the blame check (§5, `L009`) robust against the punctuation and whitespace
drift real `user.name` values carry.

Requirements: exactly one `@`, non-empty local part and domain, no
whitespace. A dotless domain is legal.

**Model and CI identities are refused as acceptors** (`L006`). An identity is
refused when any of the following holds:

1. it is a listed vendor no-reply address (`noreply@anthropic.com`,
   `noreply@openai.com`, `noreply@github.com`, `noreply@google.com`);
2. it contains the literal `[bot]`;
3. its domain is `github-actions.*`;
4. its local part is one of `actions, automation, bot, build, cd, ci,
   dependabot, do-not-reply, github-actions, gitlab-ci, jenkins, no-reply,
   noreply, renovate, robot`;
5. its local part contains, as a **whole token** (split on non-alphanumerics,
   trailing digits stripped), one of `agent, ai, aider, chatgpt, claude,
   codex, copilot, cursor, devin, gemini, gpt, llama, llm, mistral, model`.

Whole-token matching is deliberate: `claudia@` and `alain@` are people.
`ai@` is refused and the false positive is accepted knowingly — the person
uses a fuller address; accepting a model is the worse error by a wide margin.

**This list is a floor, not a proof.** It catches the identities a CI system
or an agent harness produces *by default*, which is where the failure
actually occurs. It cannot catch a model configured with a human-looking
address. That gap is closed by review, and by `L009`.

### 3.3 Tolerance

Tiers are ordered `T0 < T1 < T2` (`way-of-working-decision-allocated-delivery.md`
§2.2). A set declares a **floor**. A version pins the floor it was created
under and may carry an up-only `tolerance_override`:

```
effective_tier = tolerance_override, else tolerance_floor_at_creation
```

An override **at or below** the pinned floor is invalid and is rejected at
write, not flagged at review (`L004`). Equality is rejected too: it is not
"above", and it is a no-op that would only add noise to the hash.

The floor is pinned on the version, not read live from the set. Without the
pin the effective tier is not recomputable after a floor raise, so the hash
could not be stable and acceptance-binds-the-tier would be unimplementable.
The consequence is intended: raising a set's floor does not move any existing
hash, so acceptances survive — but every member whose effective tier now
falls below the new floor is **stranded** (`L005`) until a new version pins
the new floor. Entries are never grandfathered.

### 3.4 Discharge pointers

A typed pointer, wire form `scheme:payload`:

| Scheme | Payload | Example |
|---|---|---|
| `analyzer` | rule id | `analyzer:DEC001-no-float-money` |
| `test` | fully-qualified name | `test:ExportIdempotencyTests` |
| `policy` | platform policy id | `policy:deny-public-blob` |
| `whatif` | pre-deployment assertion | `whatif:no-public-network` |
| `otel` | metric name | `otel:dec.004.deadletter` |
| `actor` | an identity (§3.2) | `actor:emk@delegate.dk` |
| `contract` | a declared boundary (seam id or `file#symbol`) | `contract:seam/ledger/verify-classes` |

An unknown scheme is a schema fault: a pointer nothing can ever resolve is
prose, and prose is what this format exists to replace. Nothing *resolves*
these at L0.

`contract` is the **format 3** scheme (spec v1.4, added through this
document's amendment procedure for the ddd M8 integration): the decision
is discharged by the repository-diff contract check — a change to the
named boundary in any revision range must carry a declaration signing
that exact change, validated in CI by the shared classifier. A file
carrying a `contract:` pointer declares `format: 3`; a lower-format file
carrying one is a schema fault, and a store that never uses the scheme
stays a pure format-1/2 store. Hashing is unaffected: a discharge pointer
was always hashed by its string form.

`discharge_stage` is one of `pr | dev | staging | prod`, the ground table's
stages. Discharging later than the ground allowed is waste; earlier is
fiction.

### 3.5 Set file — `.decisions/sets/<set-id>.yml`

```yaml
format: 1
id: ledger-design                       # lowercase alphanumerics, dashes, dots
title: Decision Ledger — the L0 settled design
tolerance_floor: T1                     # T0 | T1 | T2
ground: characterised                   # characterised | uncharacterised
owner: emk@delegate.dk
created_at: 2026-08-10
notes: |                                # optional
  free text
```

**A set does not list its members.** A decision-version names its set;
membership is derived by query. Restating membership here would make every
addition a rewrite of a shared file — fighting append-only and conflicting on
every branch — for a denominator that comes out the same either way.

The honest limit, which every coverage report must state: coverage is
measured against the *enumerated* set, and nothing verifies the set itself.

### 3.6 Change-set file — `.decisions/log/<ulid>.yml`

```yaml
format: 1
id: cs:01K2C4YQJ3F8M0PT5W7NZ9RDXV
created_at: 2026-08-10T09:14:22Z        # RFC 3339
created_by: emk@delegate.dk             # who performed the act, not who signs
parents: [cs:01K2C4M...]                # optional
note: What this act was.                # optional

decisions:                              # identity objects, first appearance only
  - id: dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
    created_at: 2026-08-10T09:14:22Z
    created_by: emk@delegate.dk

versions:
  - decision: dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
    parent: sha256:...                  # optional; absent on the first version
    merged_from: sha256:...             # format 2 only: the tip a merge closed
    hash: sha256:...                    # the stored digest (§4)
    set: ledger-design
    statement: Monetary amounts use decimal, never double.
    allocation: constraint              # constraint|criterion|judgment|escaped
    discharge: [analyzer:DEC001-no-float-money]
    discharge_stage: pr                 # criterion only
    expectation: "…"                    # required when a discharge is otel:
    actor: emk@delegate.dk              # judgment only
    exposure: "…"                       # escaped only
    accepted_by: emk@delegate.dk        # escaped only
    review_by: 2026-09-01               # escaped only
    tolerance_floor_at_creation: T1
    tolerance_override: T2              # optional, strictly above the pin
    based_on: [prd:decision-ledger-prd#4.2.1]
    revisit_if: [claim:DDD-adapter-02@sha256:…]   # format 4 only; not ground
    supersedes: dec:…                   # optional; no command at L0
    key: MoneyIsDecimal                 # format 5 only; §3.8
    exported: true                      # format 5 only; absent means false

acceptances:
  - id: acc:01K2C5…
    decision: dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
    version: sha256:…                   # the hash signed, never the id
    actor: emk@delegate.dk
    at: 2026-08-10T09:20:00Z
    scope: version                      # or class:<discharge-ref>
    expires_at: 2027-08-10              # optional (OD-6 open)
    signature: ""                       # reserved; empty in format 1

revocations:                            # formats 1–5 shape; format 6: §3.9.2
  - acceptance: acc:01K2C5…
    at: 2026-08-11T09:00:00Z
    by: emk@delegate.dk
    reason: filed against the wrong version
```

**Unknown keys are rejected** in every schema. A field nobody reads reads as
governance that is not there.

Per-allocation obligations, enforced at parse:

| Allocation | Requires | May not carry |
|---|---|---|
| `constraint` | — | anything but `discharge` |
| `criterion` | `discharge` (≥1), `discharge_stage`; `expectation` when any pointer is `otel:` | `actor`, `exposure`, `accepted_by`, `review_by` |
| `judgment` | `actor` | everything else |
| `escaped` | `exposure`, `accepted_by`, `review_by` | everything else |

`allocation` may be **absent** — enumerated-but-unallocated is a real
intermediate state, and a file describing it is well-formed. It is simply not
a shippable state, which is what `L001` says.

`signature` is reserved and must be empty under `format: 1`. It exists so a
cryptographic upgrade (OD-3) is additive rather than a migration; its format
is deliberately unspecified. That upgrade is now scheduled — spec v1.6 /
`format: 5`, PRD §4.5 / milestone L6 (renumbered a third time: its
original `format: 2` slot was consumed by L3's `merged_from`, its
`format: 3` slot by the M8 `contract:` scheme, and its `format: 4` slot by
the v1.5 `revisit_if` edge; nothing else about the plan changes) — and
remains **planned, not normative**: under every
format this document specifies, a non-empty `signature` is still a
schema fault (§6, last bullet).

`merged_from` (spec v1.3) exists only at `format: 2` — a format-1 file
carrying it is a schema fault. It names the *other tip* a merge
arbitration closed: a reconciled version extends its `parent` chain and
closes the divergent chain it settled against, which is how a fork
(`G004`) heals inside the DAG. A file declares the format it actually
needs, so a store that never merged remains a pure format-1 store. The
reconciled version is a new version: no acceptance survives reconciliation
— prior acceptances keep signing exactly the historical versions they
named, and the reconciled content awaits a fresh signature.

`based_on` is a list of single-token basis pointers. Its vocabulary is
**open** at L0 — nothing dereferences a basis pointer yet, and closing the
vocabulary now would reject adopters' existing reference schemes for no gain.
§9.4 of the PRD closes it at L4.

`revisit_if` is specified in §3.7. It is **not** part of `based_on` and
never appears inside it.

### 3.7 The reopen edge — `revisit_if` (format 4)

Ruled by the principal (2026-08-13), settling the watched-edge question the
2026-08 basis-quality re-typing session left open and the ddd M8 migration
carried as a provisional `watched:` marker *inside* `based_on`.

A `revisit_if` pointer names a claim whose **death reopens the decision**.
That is the converse of ground, not a weaker form of it: the decision does
not rest on the claim, so falsifying the claim does not undermine the
decision — it obliges someone to look at it again.

```yaml
format: 4
versions:
  - decision: dec:hafeok.ddd/01KZ…
    based_on: [mandate:dec/ddd/internal-not-surface]
    revisit_if: [claim:DDD-adapter-02@sha256:b333063d…]
```

Rules:

- **A reopen edge is never a basis.** It lives in its own field with its
  own vocabulary. Writing one inside `based_on` — as a `watched:` token or
  under any other marker — is not the way to say this, and a consumer must
  not read `revisit_if` as ground. In the reference implementation the two
  are distinct *types* (`RevisitRef`, `BasisRef`), so the separation is not
  a convention anyone can forget.
- **The two report differently.** A claim on a `based_on` edge moving
  produces a **basis-loss** finding — the ground shifted under a standing
  decision. A claim on a `revisit_if` edge moving produces a **reopen**
  finding — the tripwire fired and the decision is due a fresh look. These
  are different facts about a decision and a report that merges them tells
  the reader neither. Neither finding is a gate class (see below).
- **Same declare-what-you-need rule as formats 2 and 3.** A change-set
  declares `format: 4` only when one of its versions actually carries a
  `revisit_if`; a store that never states one stays a pure format-1/2/3
  store, and a lower-format file carrying the field is a schema fault.
- **The vocabulary is open**, exactly as `based_on`'s is: L0 dereferences
  no pointer. Open is not shared — the pointer types stay distinct.
- **Hashing.** `revisit_if` joins the hashed field set as a list (a *set*,
  like `discharge` and `based_on`: deduplicated, code-point sorted,
  reordering is formatting). An absent key is omitted from the canonical
  object (§4.2 step 3), so every version written before the field existed
  canonicalises to byte-identical content: no digest moves, no acceptance
  is invalidated, and the prefix stays `ledger.decision-version.v1`. The
  two lists canonicalise under **separate keys**, so one token filed as
  ground and the same token filed as a reopen edge are different content —
  an acceptance always names which of the two it signed.
- **Not a gate class.** The file gate's ten classes are unchanged: nothing
  here fails `verify`. Resolving a reopen pointer — does the claim exist,
  has it moved — is a consumer's business at L0, the same posture every
  discharge scheme and every basis pointer already has. An eleventh class
  would be a further format-spec change, by the `L010` mechanism.

### 3.8 The decision key and the export flag (format 5)

Spec v1.6 (2026-10-02; `ledger-cli-prd.md` §4 as amended 2026-10-01,
ruling 2). Two optional version fields:

- **`key`** — the decision's stable human name, the string the analyzers'
  generator turns into a type name. It matches
  `^[A-Z][A-Za-z0-9]{0,63}$`; anything else is a `SCHEMA` fault at parse.
- **`exported`** — `true` makes the decision citable from other
  namespaces. Absent means `false`; an explicit `false` reads as absent.

Rules:

- **Immutable once given** (`L013`). A version must carry the key its
  `parent` carries, and the key its `merged_from` carries, whenever that
  predecessor has one. Giving a keyless decision a key is legal — it is a
  new version, so the hash moves and the version needs re-acceptance.
- **Unique among live decisions per namespace** (`L014`). Take each
  decision's latest version (§5.2); drop decisions some other decision's
  latest version `supersedes`; no two of the rest in one namespace (the
  namespace inside the id, §3.1) may carry the same key. A superseded
  decision's key is therefore free for its successor.
- **Both rules are file-gate classes**, not graph-only: a generated type
  name depends on each, so an importer of this format must enforce them.
  `L014` also has a graph-stage cross-check (`G006`, §8), which is never
  its only home.
- **Same declare-what-you-need rule as formats 2–4.** A change-set
  declares `format: 5` only when one of its versions carries a `key` or
  `exported: true`; a lower-format file carrying either is a schema fault.
- **Hashing.** `key` joins the hashed field set as a string; `exported`
  as the string `"true"` when set and nothing when false (§4.2 step 7:
  hashed content is strings only). Absent keys are omitted (§4.2 step 3),
  so every version written before format 5 canonicalises to the same
  bytes and the prefix stays `ledger.decision-version.v1`.

### 3.9 Authority records (format 6)

Spec v1.7 (2026-10-02; #69, #66). The file schema the authority
vocabulary projects (`docs/ledger-authority/ledger-authority.ttl`,
draft-2026-09-22 as amended for ruling 3). Nothing here changes how a
decision version is read or hashed.

#### 3.9.1 Files

```
.decisions/
  roles/<role-id>.yml        declared scope, like a set file (written once)
  allowed_signers            derived from key bindings; never edited (§3.9.5)
```

```yaml
# roles/steward.yml
format: 6
id: steward                    # set-id rule
title: Genesis steward         # optional
owner: emk@delegate.dk
may: [accept-decision, grant-role]   # closed vocabulary, ≥ 1
created_at: 2026-10-02
notes: …                       # optional
```

The capability vocabulary is closed: `accept-decision`,
`sign-off-pattern`, `waive-invalidation`, `grant-role`, `revoke-grant`,
`declare-unavailability`, `rotate-genesis`.

#### 3.9.2 Log entries

A change-set carrying any of these declares `format: 6`.

```yaml
grants:
  - id: grant:<ulid>
    role: steward
    scope: "*"                 # * | ns:<namespace> | set:<set-id> | pattern:<id>
    holder: emk@delegate.dk
    granted_by: emk@delegate.dk
    order: primary             # primary | fallback-N (N ≥ 1)
    limits: [no-grants]        # fallback only: no-grants | no-grant-revocations | no-genesis | no-role-edits
    genesis: true              # the genesis grant only
    external_ref: contract 2026/117   # the genesis grant only, required there
    supersedes: grant:<ulid>   # optional
    at: 2026-10-02T09:00:00Z
    hash: sha256:…             # ledger.authority-grant.v1
grant_acceptances:
  - id: gacc:<ulid>
    grant: grant:<ulid>
    signs: sha256:…            # the grant's hash
    actor: emk@delegate.dk     # must be the holder
    at: …
unavailabilities:
  - id: unav:<ulid>
    grant: grant:<ulid>
    from: …
    until: …                   # optional; absent is open-ended; after `from`
    basis: self                # self | grantor | fallback-of-genesis
    reason: …                  # optional
    by: …
    at: …
availabilities:
  - id: avail:<ulid>
    ends: unav:<ulid>
    available_at: …            # after the interval's `from`
    by: …                      # the holder of the unavailable grant
    at: …
revocations:                   # the format 6 shape
  - id: rev:<ulid>
    revokes: grant:<ulid>      # or acc:<ulid>
    actor: …
    at: …
    reason: …                  # non-empty
    hash: sha256:…             # ledger.revocation.v1
key_bindings:
  - id: key:<ulid>
    act: add                   # add | rotate | revoke
    principal: emk@delegate.dk
    namespace: hafeok.ledger
    key_type: ssh-ed25519      # add | rotate only
    key: AAAA…                 # add | rotate only (base64)
    closes: key:<ulid>         # rotate | revoke only: the window it closes
    self_bound: true           # the namespace's first binding, by the genesis holder
    mandate: contract 2026/117 # with self_bound only: the genesis external_ref
    by: …
    at: …                      # opens (or closes) the window
    hash: sha256:…             # ledger.identity-binding.v1
policies:
  - id: pol:<ulid>
    namespace: hafeok.ledger
    schemes: [ssh]             # ssh | dsse | none; ≥ 1
    require_sk: true           # optional; absent is false
    accept_role: steward       # the role whose grants carry accept-decision here
    reaccept_within_days: 30   # optional (ruling 12)
    replaces: sha256:…         # absent on the namespace's first policy
    by: …
    at: …
    hash: sha256:…             # ledger.namespace-policy.v1
```

**Two revocation shapes.** Formats 1–5 carry the legacy shape
`{acceptance, at, by, reason}`; format 6 carries the entity shape above,
which revokes a grant or an acceptance. A file carries the shape its
declared format defines; the other, or a mixture, is a schema fault. Both
shapes are read forever — a log file is never rewritten.

#### 3.9.3 Hashing

Each hashed record is a **closed payload** canonicalised by §4.2's law
(strings normalised, absent keys omitted, lists as sets, keys code-point
sorted, compact) and digested exactly as §4.3, under its own prefix:

| Prefix | Payload keys |
|---|---|
| `ledger.authority-grant.v1` | `id`, `role`, `scope`, `holder`, `granted_by`, `order`, `limits` (set), `genesis` (`"true"` or absent), `external_ref`, `supersedes` |
| `ledger.revocation.v1` | `revokes`, `actor`, `at`, `reason` (the PRD §7 closed payload; `at` as RFC 3339 UTC seconds) |
| `ledger.identity-binding.v1` | `id`, `act`, `principal`, `namespace`, `key_type`, `key`, `closes`, `self_bound` (`"true"` or absent), `mandate`, `by`, `at` |
| `ledger.namespace-policy.v1` | `id`, `namespace`, `schemes` (set), `require_sk` (`"true"` or absent), `accept_role`, `reaccept_within_days`, `replaces`, `by` |

The stored `hash` is never inside its own payload. A legacy revocation
has no stored hash; its payload is still computable from
`{acceptance, by, at, reason}` read as `{revokes, actor, at, reason}`.

#### 3.9.4 What the gate checks

No new file-gate class. The records are policed by the classes that
already mean what is wrong:

- **`SCHEMA`** — every rule of §3.9.2 a single record states (a primary
  grant with limits; a genesis grant not self-granted, `*`, primary and
  carrying `external_ref`; `external_ref` off the genesis; `until` not after
  `from`; a binding carrying fields its `act` does not define; a policy
  with no scheme), and every cross-record rule: a grant naming an
  undeclared role or superseding no filed grant; a grant acceptance not by
  the holder or not signing the grant's hash; an unavailability whose
  declarer does not stand in its `basis`; an availability not by the
  holder, not after `from`, or ending an interval twice; a revocation
  naming no filed grant or acceptance, or a second revocation of one
  record; a binding in a namespace with no policy, closing what opens no
  window, another principal's window, or one already closed, or a
  self-bound binding whose mandate is no genesis `external_ref`; a
  namespace with two root policies or a forked `replaces` chain; a policy
  whose `accept_role` is no declared role that may `accept-decision`; an id
  filed twice; a role file that is not format 6, misnamed, duplicated, or
  may do nothing.
- **`L006`** (extended, stricter, additive) — every identity an authority
  record attributes an act to or gives authority to: a grant's holder and
  grantor, a grant acceptance's actor, an unavailability's and an
  availability's declarer, a revocation's actor, a binding's principal and
  filer, a policy's author. A model is never a holder.
- **`L007`** (extended) — a stored grant, revocation, binding or policy
  hash that does not equal its recomputed payload digest.

**Liveness.** A grant is *live* when unrevoked, unsuperseded, and accepted
by its holder (a grant acceptance signing its current hash); *available*
at an instant when no unavailability covers it (`[from, until)`, unless an
availability ended it at or before the instant). The namespace's policy *in
force* is the tip of its `replaces` chain.

**The role check is verb-time.** An authoring verb asks whether the actor
holds a live, accepted, available grant of a role that `may` the act, over
a scope covering it (`*`; a namespace scope its own namespace; a set scope
its own set; namespaces match exactly), and — for a fallback — one not
limited from the act while no live, available grant of the same role at a
lower rank covers the act's target (fallback order by covering scope, D9
(e): a `fallback-1` over a set waits on a primary over `*` in its role;
another role never outranks; equal rank acts concurrently). In a namespace
with a policy, accepting and revoking an acceptance count only grants of
the policy's `accept_role`, which is never the genesis role: the genesis
(root) role carries `grant-role`, `revoke-grant`, `declare-unavailability`
and `rotate-genesis` and none of the decision capabilities (D9 (f)). A
namespace without a policy is a pre-v2 namespace: nothing is role-checked
there. The gate-time counterpart over history — an acceptance whose actor
held no such grant (`A006`) — waits on the decision-class → role mapping.

#### 3.9.5 `allowed_signers`

Derived, one line per key window, in OpenSSH's `allowed_signers` form so
`ssh-keygen -Y verify -f .decisions/allowed_signers` reads it:

```
<principal> namespaces="ledger-accept@<ns>" valid-after="<YYYYMMDDhhmmssZ>"[ valid-before="<…>"] <key_type> <key>
```

`valid-after` is the opening binding's `at`; `valid-before` the `at` of the
`rotate` or `revoke` that closed it. Lines are sorted by code point, under a
two-line `#` header. `verify` re-derives the file and fails a
**`[SIGNERS]`** stage when the committed bytes differ, when the log binds
keys and no file is committed, or when a file is committed and the log
binds none. Outside the file gate's classes, like the export stage.

---

## 4. Canonicalisation and hashing

**YAML is the file format; canonical JSON is the hash form.** Routing through
a second, restricted serialisation is what makes "a formatting-only edit
leaves the hash unchanged" structural rather than a rule someone must
remember: quoting style, key order, indentation, comments, line endings and
`null`-versus-absent all disappear in the parse, before anything is hashed.
It also removes YAML's ambiguity — anchors, tags, four multi-line scalar
styles — from the surface a second implementation has to reproduce.

### 4.1 The hashed field set

Exactly these keys, and no others:

```
decision · parent · merged_from · set · statement · allocation ·
discharge · discharge_stage · actor · expectation · exposure ·
accepted_by · review_by · tolerance_floor_at_creation ·
tolerance_override · based_on · revisit_if · supersedes · key · exported
```

`merged_from` joined the set at spec v1.3 (`format: 2`), `revisit_if` at
spec v1.5 (`format: 4`), and `key` and `exported` at spec v1.6
(`format: 5`; `exported` canonicalises as `"true"` or is omitted). Because an absent key is omitted from the canonical
object (§4.2 step 3), every version written before either field existed
canonicalises to the same bytes as before: no digest moved, no acceptance
was invalidated, and `CANONICAL_FORM` stays `v1`. The reference
implementation proves it over every stored digest it holds
(`ledger-cli/tests/digests.rs`).

Outside the hash: the `hash` field itself (including it would be circular),
everything at change-set level (`format`, `id`, `created_at`, `created_by`,
`parents`, `note`), and all acceptances and revocations.

Note that **both tolerance inputs are hashed, not the resolved tier**. `T0`
floor with a `T2` override and a native `T2` floor both resolve to an
effective `T2`, but they are different provenance and must not collide —
which is what keeps override-rate-per-set (PRD §10) computable from hashed
content.

### 4.2 The algorithm

1. Parse the file. Take only the hashed field set.
2. **Normalise every string**, in this order:
   a. replace `\r\n` and lone `\r` with `\n`;
   b. normalise to Unicode NFC;
   c. strip leading and trailing ASCII whitespace
      (`\t \n \v \f \r` and space).
3. **Treat as absent**: a missing key, an explicit `null`, an empty
   collection, and any string that step 2 reduces to the empty string. Absent
   keys are omitted from the object; there is no `null` in the canonical form.
4. **List fields are sets.** `discharge`, `based_on` and `revisit_if` are rendered as their
   members' canonical string forms, deduplicated, then sorted ascending by
   Unicode code point. Reordering a list in a file is formatting.
5. Emit a JSON object with keys sorted ascending by Unicode code point, with
   no insignificant whitespace and no trailing newline. Keys in this format
   are ASCII, where code-point order coincides with RFC 8785's UTF-16
   code-unit order.
6. Strings are escaped per RFC 8785: the two-character forms `\" \\ \b \f \n
   \r \t` where they exist, `\u00XX` for other control characters, and every
   other character emitted literally as UTF-8.
7. **No floating-point value may appear in hashed content.** A float is a
   schema fault. This removes RFC 8785's entire number-serialisation problem;
   the only numeric field in the format (`format`) is not hashed.
8. Dates are `YYYY-MM-DD`. No timestamp is hashed, so time-zone
   normalisation never arises.

### 4.3 The digest

```
version_hash = "sha256:" + lowercase_hex(
    SHA-256( "ledger.decision-version.v1" ‖ 0x0A ‖ canonical_json_utf8 )
)
```

The prefix is domain separation **and** a version pin. A future entity type
gets its own prefix and can never collide. A `format` bump that changes what
a hashed field *means* must bump the prefix to `.v2`, with a migration note —
otherwise acceptances signed under the old reading would silently re-point.

A short form (first 12 hex characters) exists for display only and is never
compared.

### 4.4 Conformance vector

A second implementation must reproduce both of these exactly.

Input (a version with `parent`, `discharge_stage`, `actor`, `expectation`,
`exposure`, `accepted_by`, `review_by`, `tolerance_override` and `supersedes`
all absent):

```yaml
decision: dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
set: ledger-design
statement: Monetary amounts use decimal, never double.
allocation: constraint
discharge: [analyzer:DEC001-no-float-money]
tolerance_floor_at_creation: T1
based_on: [prd:decision-ledger-prd#4.2.1]
```

Canonical JSON (one line, shown wrapped):

```
{"allocation":"constraint","based_on":["prd:decision-ledger-prd#4.2.1"],
"decision":"dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV",
"discharge":["analyzer:DEC001-no-float-money"],"set":"ledger-design",
"statement":"Monetary amounts use decimal, never double.",
"tolerance_floor_at_creation":"T1"}
```

Digest:

```
sha256:ac2a68023018391550b542b1f093104f1f32115d3603e2fda9c37805875437cc
```

The eight fixture stores under `ledger-cli/tests/fixtures/` are further
vectors: their stored hashes are real, and a canonicalisation change makes
every one of them fail.

---

## 5. The gate

`ledger verify` fails for a **schema fault** or one of **twelve classes**,
and for nothing else. A new reason is a change to this document — `L010`
arrived that way, as the spec v1.1 amendment, and `L013`/`L014` as spec
v1.6. `L011` and `L012` are **reserved** for the signing classes (#65,
ruling D3) and are not classes until a revision specifies them.

### 5.1 The parse gate

`SCHEMA` covers: a file that does not parse against the format it declares;
an unknown `format`; an unknown key; an unknown discharge scheme; a file stem
disagreeing with its declared id; a duplicate id; a per-allocation obligation
from §3.6 that is not met (except the escape's, which is `L002`); a
non-empty `signature`; a version naming an undeclared set; a revocation
naming an acceptance nobody filed; a `key` not matching
`^[A-Z][A-Za-z0-9]{0,63}$`.

### 5.2 The twelve

| Code | Fails when |
|---|---|
| `L001` | a decision's latest version carries no `allocation` |
| `L002` | an `escaped` version is missing `exposure`, `accepted_by`, or `review_by` |
| `L003` | a live acceptance of a decision's current version has `expires_at` before today |
| `L004` | `tolerance_override` is at or below `tolerance_floor_at_creation` |
| `L005` | a decision's latest effective tier is below its set's current floor |
| `L006` | an acceptance actor, or an escape's `accepted_by`, is refused by §3.2 |
| `L007` | a stored `hash` does not equal the recomputed canonical hash |
| `L008` | an acceptance's `(decision, version)` pair matches no filed version |
| `L009` | an acceptance's actor is not the author of the commit that introduced it |
| `L010` | a `judgment`'s `actor` is refused by §3.2 (spec v1.1) |
| `L013` | a version's `key` differs from the key its `parent` or `merged_from` carries (spec v1.6, §3.8) |
| `L014` | two live decisions of one namespace carry the same `key` on their latest versions (spec v1.6, §3.8) |

Notes that are part of the specification, not implementation detail:

- **Only the latest version of a decision is judged** by `L001`, `L003`,
  `L005`, `L010` and `L014`. (`L013` judges every version against its
  predecessors: a rename anywhere in the chain is a rename.) An acceptance of a superseded version was already
  invalidated when the hash moved; reporting it again is noise on a resolved
  fact.
- **"Latest" derives from the parent DAG, never from file or ULID order**
  (spec v1.2). The latest version of a decision is the unique version whose
  hash no other version of the same decision names as `parent`.
  Content-identical filings (one hash filed more than once) are one version.
  ULIDs order by one clock, and two writers' clocks prove nothing about
  parenthood — deriving latest from change-set order was the single-writer
  leak the L1 report named, retired here. A chain that cannot name one tip —
  two versions unclaimed as parents, the store two divergent writers leave
  behind — has no latest: an implementation must not resolve the ambiguity
  by any ordering heuristic. The reference implementation reports it as
  `G004` (§8) and refuses authoring verbs against the forked decision until
  a recorded arbitration (`ledger merge --resolve`) settles the chain.
- **A revoked acceptance is not judged** for expiry.
- **`L008` checks the pair.** An acceptance naming one decision while signing
  another's hash is signing nothing about the decision it claims to accept.
- **`L009` skips, never fails, when there is no introducing commit.** An
  uncommitted acceptance is the state every acceptance passes through; failing
  it would make an acceptance impossible to commit in the first place. The
  check lands on the next run over committed history, which in practice is
  CI. A skipped check is always reported — a silently unrun rule reads as a
  passing one.
- **Allocated-awaiting-acceptance is status, not a failure.** The gate
  polices violations, not pendency. A gate that fires on ordinary work in
  progress is a gate people learn to ignore.

### 5.3 Gates and exit codes

`--gate readiness` blocks produce and runs every class except `L002` and
`L003`, which are dispositions that must hold at release rather than
preconditions for starting. `--gate completeness` blocks release and runs
all twelve. With no flag, all twelve run.

| Exit | Meaning |
|---|---|
| `0` | conformant |
| `1` | findings |
| `2` | the gate could not run (no store, unreadable file, bad flag) |

CI has to tell "the gate said no" apart from "the gate broke". This differs
from `ddd validate`, which returns `1` for both.

---

## 6. Open edges

Stated so an adopter meets them in this document rather than in production.

- **`L009` cannot see uncommitted work**, by construction (§5.2). A repository
  with no git history has the check skipped entirely.
- **`actor:` discharge pointers are not covered by `L006` or `L010`.** The
  judgment-actor half of the gap v1.0 flagged here closed as `L010` in spec
  v1.1; a *discharge pointer* naming a model identity remains representable.
  A pointer is a reference to where discharge happens, not an allocation of
  accountability, so extending the rule there is a separate decision.
- **The model-identity list is a floor** (§3.2), not a proof.
- **`constraint` carries no discharge requirement.** §4.4 does not impose one,
  so neither does this format — a constraint with no encoder is possible and
  is not a finding.
- **`expires_at` is optional** pending OD-6. An acceptance without one never
  goes stale, which is precisely the risk OD-6 has to settle.
- **A class-scoped acceptance still signs a version hash.** The scope widens
  what the acceptance covers; it does not loosen what it names. L1 owns the
  operational semantics of `accept --class`.
- **Nothing verifies the set.** Coverage is measured against the enumerated
  set (PRD §8), and enumeration completeness has no mechanical check at any
  milestone. Late-discovery rate is the lagging proxy.
- **ULID generation is not specified here** because L0 mints no ids. L1's
  `add` needs it.
- **Planned, not yet normative — signing (Session B; ruled 2026-10-02 on
  #65).** The August plan for this slot (tier-gated `gpg.format`
  signatures as `format: 5`) is replaced by `ledger-cli-prd.md` §0 items
  7–12: namespace policy decides when a signature is required; signatures
  are sidecars under `.decisions/sig/`; schemes `ssh`, `dsse`
  verification, and `none` for pre-v2 stores. `format: 5` went to the key
  revision (§3.8). `L011` (a required signature absent or invalid) and
  `L012` (acceptances under a since-closed key, a review trigger) keep
  their reserved numbers. Until that revision ships, a non-empty
  `signature` stays a schema fault.

---

## 7. What L1 needs from L0

The authoring operations reuse these library surfaces rather than growing a
second copy of the rules:

| L1 operation | Calls |
|---|---|
| `add` | ULID generation (new), `DecisionSet::validate_id`, `version_hash` |
| `allocate` / `escape` | `Allocation::assemble` (the §3.6 obligations, including `L002`) |
| any write | `Tolerance::new` (the `L004` rejection at write) |
| `accept` | `Identity` parse plus `model_or_bot_reason` (`L006`), `version_hash` to sign |
| `revoke` | the acceptance/revocation schemas |
| `status` | `verify::view::View` — latest versions, live acceptances, pendency |
| `blame` | `blame::introducing_author` |
| `diff` | `canon::canonical_json`, field by field |

L1 shipped against this table (2026-08-10); every verb routes through the
listed surface via a delta-gate — the verb refuses at write exactly the
findings `verify` would report afterwards. Semantic `diff <ref>..<ref>`
alone is deferred: it needs store-at-revision loading, which arrives with
L3's per-decision merge base.

---

## 8. The graph stage (L2)

**Outside the import surface.** Sections 1–5 are what an outside
implementation of the *format* reproduces; this section describes the
reference implementation's L2 graph stage, which an outside implementation
may skip without losing format conformance. The file gate's closed
classes (§5) are unchanged by it.

The `.decisions/index/` cache holds the RDF materialisation of the log
(`index/ledger.ttl`), rebuilt by `ledger reindex`. The log is the source of
truth; the emission is byte-deterministic, so deleting the index and
rebuilding reproduces it byte-identically — the PRD §5 correctness test,
run in CI. Acceptance provenance is PROV-O (an acceptance
`prov:wasAttributedTo` its actor; a version `prov:wasRevisionOf` its
parent; both `prov:wasGeneratedBy` their change-set). **No triple is ever
added to a node after the record that creates it is filed** (ruling 3,
spec v1.7): a revocation is its own `ledger:Revocation, prov:Entity` node
— `ledger:id`, `ledger:hash`, `ledger:revokes <urn:acc:…>` (or
`<urn:grant:…>`), `ledger:revocationReason`, `prov:wasAttributedTo`,
`prov:generatedAtTime` — so an acceptance's triples are fixed at filing.
A legacy-shape revocation (formats 1–5, no id) is emitted the same way at
`<urn:rev:legacy-<acc-ulid>>`, with its computed payload hash and no
`ledger:id`. The retired shape (`ledger:revokedAt`, `ledger:revokedBy`
on the acceptance IRI) is no longer emitted; readers tolerate it during
the transition. `based_on` tokens
become `ledger:basedOn` literals exactly as written — the vocabulary stays
open at this milestone; the graph exposes it and does not police it.

`ledger verify` additionally runs SPARQL shape checks over the emitted
graph — cross-entry referential integrity the per-file schema cannot
name. They report as a **distinct stage** of the same command, with
unchanged exit semantics (findings exit `1`):

| Code | Fails when |
|---|---|
| `G001` | a `supersedes` edge targets a decision no change-set filed |
| `G002` | a version's `parent` (or `merged_from`) hash matches no filed version of its decision |
| `G003` | a version names a decision no change-set introduced |
| `G004` | a decision's version chain forks into more than one tip (spec v1.2) |
| `G005` | one decision is superseded by two live claimants (spec v1.3) |
| `G006` | two live decisions of one namespace whose tips share a `key` — the cross-check of `L014` (spec v1.6) |
| `A003` | two live grants (unrevoked, unsuperseded, accepted) share role, scope and order (spec v1.7) |
| `A005` | more than one live (unsuperseded, unrevoked) genesis grant (spec v1.7) |

`A003` and `A005` are the authority shapes' gate classes
(`docs/ledger-authority/ledger-authority-shapes.ttl`), with two
tightenings recorded there: `A003` counts only a `ledger:GrantAcceptance`
as acceptance, and `A005` excludes a revoked genesis. `A006` (an orphaned
acceptance) is deferred until the decision-class → role mapping exists.

`G004` is the state two divergent writers leave behind — a plain git merge
of two branches' logs, each having revised the same decision from the same
parent. No file is malformed; the *store* cannot name a latest version, so
it is non-conformant until a recorded arbitration (`ledger merge
--resolve`) extends one chain past the fork, closing the other tip via
`merged_from`. A tip, for both `G004` and `G005`, is a version no other
version of the decision claims by `parent` *or* `merged_from`.

`G005` is the write-time one-superseder-per-decision refusal met across
branches, where it cannot refuse retroactively: each side's claim was
legal alone. Only live claims count — a claimant whose next version drops
the edge has withdrawn, which is exactly the arbitration act `ledger merge
--resolve` records for the losing side. The graph classes are closed the
same way the file classes are: `G006` arrived as a change to this
section (spec v1.6), and a `G007` would be another. Coverage (`ledger coverage`) reports the
seven-state disposition vocabulary — `undecided`, `awaiting-acceptance`,
`decided`, `escaped-priced`, `escape-review-due`, `expired`, `superseded`
— per set and per namespace, with supersession chains walked to their
tips, and always states §8-of-the-PRD's honest limit: coverage is measured
against the enumerated set, and nothing verifies the set itself.

### 8.1 The committed export and the export stage

**Outside the import surface**, like the rest of this section. `ledger
export --format ntriples [--namespace <ns>] [--out <path>|-]` writes one
namespace of the log to `docs/decisions/<ns>.nt` (every namespace the log
speaks when `--namespace` is absent). It is the read model the analyzers'
generator consumes.

- **Triples.** They are exactly the index's triples (§8), restricted to one namespace:
  - the namespace's decisions and their versions;
  - acceptances of those decisions, and revocations of those acceptances;
  - the sets those versions name;
  - the change-sets that filed any of these, each holding only what belongs to the namespace.

  `ledger:set` is the set IRI `<urn:ledger-set:<id>>`; a reader takes the set id from its local part.
- **Form.** RDF 1.2 canonical N-Triples:
  - every term written in full: no prefixes, and `rdf:type` instead of `a`;
  - literals with `ECHAR` for BS, HT, LF, FF, CR, `"` and `\`;
  - uppercase-hex `UCHAR` for the other C0 controls, DEL, U+FFFE and U+FFFF;
  - one space after each term, one LF per line;
  - lines sorted by code point.

  The bytes are therefore a function of the log alone.
- **Export stage.** `ledger verify --export` re-derives every committed export and compares bytes. It reports a distinct `[EXPORT]` stage with unchanged exit semantics (findings exit `1`). It fails when:
  - an export differs from the log's;
  - an export names a namespace the log does not speak;
  - a namespace the log speaks has no committed export;
  - there is no committed export at all.

  Without `--export` the stage does not run, and the JSON report omits its key rather than reporting an empty pass. `*.citations.nt` files (the analyzers' citation projection) share the directory and are never compared.
