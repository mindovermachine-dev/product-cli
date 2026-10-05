# Session A close-out — format 5, interactivity, authority, revocation

2026-10-02. Order worked: #67 → #71 → #69 → #66. #70 (signing) and #79
are Session B.

## 1. What landed

**One branch, one pull request.** This session could push only to
`claude/nifty-lovelace-ozwww1`, so the four issues land as four commit groups
on that branch and are reviewed as one PR. Each commit leaves the tree green,
so they can be split into four PRs before merge if that is wanted.

| Issue | Commit | Landed |
|---|---|---|
| #67 | `ledger: spec v1.6 / format 5 — key and exported …` | `key` / `exported` on `VersionRaw`; `L013`, `L014`; `G006`; emitter; `add --key --exported`, `revise --key`; `l013`/`l014` fixtures; the digest proof (`tests/digests.rs`); spec v1.6 and migration note |
| #71 | `ledger: accept and revoke refuse a non-interactive invocation` (+ an import fix-up) | stdin TTY check on `accept <dec>`, `accept --set/--group --confirm`, `revoke` (and later `grant revoke`); signing verbs in the tests run under a PTY driver; negative tests in `tests/interactive.rs` |
| #69 | `ledger: spec v1.7 / format 6 — the authority model` (+ `ddd-cli` mapping fix) | `ledger-core/src/authority/`: roles, grants, grant acceptances, unavailability/availability, key bindings, policy, the `rev:` entity; four hash prefixes; `SCHEMA`/`L006`/`L007` extended; `A003`/`A005`; `[SIGNERS]` stage; `authority::authorize`; the verbs; export restriction; spec v1.7 §3.9 |
| #66 | `ledger: revocation is its own entity; acceptances are immutable` | `revoke` files the `rev:` entity; emitter writes every revocation as its own node (legacy ones too); `L006` on the revoker; role check on `revoke`; acceptance-immutability test; exports regenerated |

**Issue text a ruling overrode.** No issue body was edited on GitHub. These
are the places where an issue's text is older than a ruling, and what the
code follows:

- **#67**: "I4 settles which number each takes." Ruled (#65 D3): `L011`/`L012`
  keep their signing meanings, so the key classes are **`L013`, `L014`**.
- **#69**: "`identity add|rotate` (which only add, never edit)". The #65 comment
  adds `identity revoke` (closes a window). It is built, and still append-only.
- **#69**: "Namespace policy … required schemes". Ruled D4: the closed
  `ssh | dsse | none` vocabulary.
- **#66**: "signed like an acceptance (#70)". Signing is Session B. The closed
  payload `{revokes, actor, at, reason}` is hashed under `ledger.revocation.v1`
  now, and the signature over it lands with #70.
- **The authority vocabulary** (`docs/ledger-authority/`) was extended,
  not contradicted:
  - `ledger:KeyBinding` and `ledger:NamespacePolicy` were added; the #65 rulings name them and the draft did not define them.
  - `A003` is tightened to count only a `ledger:GrantAcceptance` as acceptance. The draft's `?ga ledger:grant $this` also matches an `Unavailability`.
  - `A005` also excludes a revoked genesis.

  All three are recorded in the `.ttl` files.

## 2. Format changes

**Formats.** `format: 5` (spec v1.6) adds the key fields. `format: 6` (spec
v1.7) adds the authority records. The `rev:` revocation entity is part of
format 6 because the vocabulary defines one entity for grants and
acceptances. A file declares the format it uses: `Author::append` now stamps
`format::needed_for` for every verb, not only `add`.

**New fields**
- **On versions (format 5):**
  - `key`, matching `^[A-Z][A-Za-z0-9]{0,63}$`. A bad key is a SCHEMA fault at parse.
  - `exported`, hashed as `"true"`, absent when false.
- **New file:** `roles/<id>.yml`, with `format`, `id`, `title?`, `owner`, `may[]`, `created_at` and `notes?`.
- **Log lists (format 6):**
  - `grants`
  - `grant_acceptances`
  - `unavailabilities`
  - `availabilities`
  - `key_bindings`
  - `policies`
- **Revocations (format 6 shape):** `{id: rev:…, revokes: grant:…|acc:…, actor, at, reason, hash}`. The legacy `{acceptance, at, by, reason}` shape stays valid in formats 1–5. Mixing the two shapes, or using the wrong one for the declared format, is a SCHEMA fault.
- **New id schemes:**
  - `grant:`
  - `gacc:`
  - `unav:`
  - `avail:`
  - `rev:`
  - `key:`
  - `pol:`
- **Derived file:** `.decisions/allowed_signers`.

**Class ids**
| Stage | Added | Closed count |
|---|---|---|
| File gate | `L013` key immutability across `parent`/`merged_from`; `L014` key uniqueness among live decisions per namespace. `L011`/`L012` reserved, not variants | 10 → 12 (+ SCHEMA) |
| File gate, extended (L010 mechanism) | `SCHEMA` covers the authority records' structural and referential rules; `L006` covers every identity an authority record names, the revoker included (#66); `L007` covers every stored authority hash | unchanged |
| Graph | `G006` (L014's SPARQL cross-check), `A003`, `A005` | 5 → 8 |
| Verify stages | `[SIGNERS]` (`allowed_signers` byte-identical to the key bindings) | — |

**Hash prefixes** (canonical-JSON law, closed payloads, no-rest destructuring)
- `ledger.authority-grant.v1`: `id`, `role`, `scope`, `holder`, `granted_by`, `order`, `limits`, `genesis`, `external_ref`, `supersedes`.
- `ledger.revocation.v1`: `revokes`, `actor`, `at`, `reason`.
- `ledger.identity-binding.v1`: `id`, `act`, `principal`, `namespace`, `key_type`, `key`, `closes`, `self_bound`, `mandate`, `by`, `at`.
- `ledger.namespace-policy.v1`: `id`, `namespace`, `schemes`, `require_sk`, `accept_role`, `reaccept_within_days`, `replaces`, `by`.

All four are recorded in `ledger-format-v1.md` §3.9.3. `CANONICAL_FORM` stays
`ledger.decision-version.v1`.

**Fixture-digest proof.** `ledger-cli/tests/digests.rs` re-derives every
stored version digest under the current canonical form and asserts that
none moved. It covers every fixture store except the deliberately stale
`l007`, and this repository's own `.decisions/log` (more than 100 versions).
It passes. `canon_tests.rs` adds `key` and `exported` to the mutation table,
and pins that an absent key and `exported: false` canonicalise like an
unwritten field. The format §4.4 conformance vector is unchanged.

**Exports.** They were regenerated byte-identical: this repository's store
carries no revocation, so #66's emitter change moves no committed triple.
Any store that has a revocation will see its export change.
Hafeok/decision-driven-analyzers#83 reads both shapes during the
transition (tracked in #76).

## 3. Questions

None of these stopped a step. Each is a judgement made so the work could
land. Each has a default (the one built) and alternatives for a ruling.
All twelve were ruled on 2 October 2026; each ruling is recorded under its
question (`docs/signing-rulings-2026-10.md`, "D5 to D9").

1. **The selection dry run stays scriptable.** `accept --set/--group`
   without `--confirm` writes nothing and is not TTY-gated; the `--confirm`
   write is.
   (a) as built;
   (b) gate the dry run too. The literal reading of "both forms", but it
   only blocks reading.
   **Ruled 2026-10-02:** (a).
2. **A namespace without a policy is not role-checked.** Without this,
   every existing acceptance flow in this repository (91 acceptances, no
   grants) would refuse.
   (a) as built: pre-v2 until `init --namespace`;
   (b) refuse `accept` outright in an ungoverned namespace, forcing
   migration;
   (c) a store-level switch.
   **Ruled 2026-10-02:** (a), refined now by D5 (c): the position rule replaces the namespace switch (lands in #70). (b) becomes an issue after Session B.
3. **Genesis is per store, not per namespace.** It follows from scope `*`
   and `A005`. A second `init --namespace` files a policy only, as the
   genesis holder.
   (a) as built;
   (b) a genesis per namespace, with `A005` scoped by namespace.
   **Ruled 2026-10-02:** (a).
4. **Namespace scopes match exactly.** `ns:hafeok` does not cover
   `hafeok.ledger`.
   (a) as built, the conservative choice;
   (b) a segment-prefix hierarchy.
   **Ruled 2026-10-02:** (a). A hierarchy, if ever wanted, is a new scope form.
5. **Escalation guard.** Below the genesis, a grantor may grant only the
   role it acts under. Without the guard, `grant-role` could hand out
   `rotate-genesis`.
   (a) as built;
   (b) no guard;
   (c) per-role "may grant" lists, which would be a vocabulary change.
   **Ruled 2026-10-02:** (a), compared on the grant the act names (D9).
6. **Who changes policy.** No capability in the closed vocabulary names
   it. Built: the live, available genesis holder.
   (a) as built;
   (b) add a capability (`set-policy`), which is a vocabulary change;
   (c) reuse `grant-role` over `*`.
   **Ruled 2026-10-02:** (a).
7. **Fallback semantics.** A fallback acts only while no live, available
   grant of the same role and scope at a lower rank exists.
   (a) as built;
   (b) fallbacks act concurrently, with order used only for the `A003`
   uniqueness check.
   **Ruled 2026-10-02:** (a), by covering scope within one role, per D9 (e).
8. **Roles are write-once.** `role declare` needs `grant-role` over `*`,
   and the `no-role-edits` limit withholds declaring. Editing a role (a new
   version of a role file) is not modelled.
   **Ruled 2026-10-02:** Write-once, held by `verify`.
9. **Grant acceptance does not require a TTY.** #71 names only `accept`
   and `revoke`; `grant revoke` was gated too.
   **Ruled 2026-10-02:** Gate it (Session B step 0).
10. **The legacy revocation node.** It is emitted at
    `<urn:rev:legacy-<acc-ulid>>` with its computed hash and **no
    `ledger:id`**, so it fails `RevocationShape`'s `ledger:id minCount 1`.
    (a) as built: honest about the missing id;
    (b) synthesise an id;
    (c) relax the shape for legacy revocations.
    **Ruled 2026-10-02:** (a).
11. **The `no-genesis` limit withholds nothing yet.** No act rotates the
    genesis (`rotate-genesis` is unbuilt).
    **Ruled 2026-10-02:** No ruling; an issue for `rotate-genesis`.
12. **Merge driver for role files.** A both-sides role file is reported
    under the `divergent-floor` conflict class. The merge conflict classes
    are closed, so a new class is a ruling.
    **Ruled 2026-10-02:** No new class.

**Found in passing, out of scope.** The loader never enforces spec §3.7's
"a lower-format file carrying `revisit_if` is a schema fault". This
repository's own store has `revisit_if` in four `format: 1` change-sets,
each holding one version (this paragraph first said `format: 3`, which was
wrong):

- `.decisions/log/01KZX70EMPA47TBR0PFKX4M32Z.yml`
- `.decisions/log/01KZX70EQGQCB1B190TS9FZ1A2.yml`
- `.decisions/log/01KZX70ET1GMR2012XKEP5EWDW.yml`
- `.decisions/log/01KZX70S86QGXVCA5GW5WSY6XA.yml`

Adding the check would fail this repository's gate, so it needs a
migration decision first.

> **Resolved 2026-10-05 (#81).** Three files, not four:
> `01KZX70S86QGXVCA5GW5WSY6XA` names `revisit_if` only inside its `note`
> and `statement` text and needs format 1. The principal ruled that a
> landed file's `format:` may be raised to the lowest format its content
> needs; the other three now declare `format: 4` and the loader checks the
> rule (`docs/ledger-format-migrations.md`).

## 4. Session B TODOs deferred here

- **A policy change is signed under the policy in force before it** (#65).
  Today a change carries the hash of the policy it `replaces` and is
  restricted to the genesis holder. The check that it was signed under the
  old policy is not done.
- **Every later key binding is signed under the policy in force.** The
  genesis self-bound binding is signed by the key it binds.
- `L011` (a required signature is absent or invalid, per namespace policy)
  and `L012` (acceptances under a since-closed key). Ruling 12's
  `reaccept_within_days` deadline is stored and emitted but enforced by
  nothing until `L012`.
- The acceptance hash `ledger.acceptance.v1` over
  `{decision, version, actor, at, scope, expires_at}`. Sidecars at
  `.decisions/sig/<ulid>.<scheme>.sig`. Retiring the inline `signature`
  field with a permanent-empty message.
- Signing and verifying the `rev:` revocation payload (it is hashed now).
- `accept` refusing a software key under a `-sk` policy. `identity add`
  already refuses one.
- `dsse` verification; `ssh-keygen -Y verify` against the derived
  `allowed_signers` with `-Overify-time`.
- `A006`, an orphaned acceptance. It needs the decision-class → role
  mapping.
- The export-time check that an exported decision has an unrevoked
  acceptance.

## 5. Tests

| | Before | After |
|---|---|---|
| `cargo t` (workspace) | 1,833 passed | **1,911 passed, 0 failed** |
| ledger crates (`ledger-core` + `ledger-cli`) | 337 | 415 |

The "before" figures come from a run on the tree at `e33398d`. The workspace
run overlapped this session's first edit, and the two failures it showed came
from that partial edit (a function over the length limit and an updated
class list). The adjusted baseline is 1,833. `eval-dotnet/` and `spec-flow/`
(.NET) are outside `cargo t` and untouched.

New suites:
- `tests/digests.rs`
- `tests/keys.rs`
- `tests/interactive.rs`
- `tests/authority.rs`
- `verify/keys_tests.rs`
- `verify/authority_tests.rs`
- `authority/check_tests.rs`
- `graph/revocation_tests.rs`

**PTY, not an injected predicate.** The `assert_cmd` suites run a signing
invocation through a pseudo-terminal using `script(1)` from util-linux
(`ledger-cli/tests/common/mod.rs` `invoke`). The binary's
`std::io::IsTerminal` check is then the code under test in both
directions:
- the existing `accept` and `revoke` tests pass through it;
- `tests/interactive.rs` proves that piped stdin, even with `yes` typed
  into the pipe, exits non-zero and leaves every byte under `.decisions/`
  unchanged.

An injected predicate would have put a seam into the core that only tests
use. The binary's own check would then go untested, and a seam is the shape
an override grows from. The cost is a test-time dependency on `script(1)`,
which `ubuntu-latest` ships.

## 6. Upstream list: `hafeok/decision-driven-analyzers`

For the analyzers' reader contract. Nothing has been filed upstream.

**Classes that can appear in `ledger export --format ntriples` output and
could not before #80** (`ledger-core/src/graph/authority.rs`):
`ledger:Role`, `ledger:Grant`, `ledger:GrantAcceptance`,
`ledger:Unavailability`, `ledger:Availability`, `ledger:Revocation`,
`ledger:KeyBinding`, `ledger:NamespacePolicy`. The export keeps the
records that reach the exported namespace (`restrict`,
`ledger-core/src/graph/export.rs`), `KeyBinding` and `NamespacePolicy`
nodes included.

**Predicates new since before #80:**
- versions (`ledger-core/src/graph/turtle.rs`): `ledger:key`, `ledger:exported`;
- roles: `ledger:may`;
- grants: `ledger:role`, `ledger:holder`, `ledger:grantedBy`, `ledger:order`,
  `ledger:rank`, `ledger:limit`, `ledger:genesis`, `ledger:externalRef`,
  `ledger:supersedesGrant`;
- grant acceptances: `ledger:grant`, `ledger:signsHash`;
- unavailability and availability: `ledger:from`, `ledger:until`,
  `ledger:basis`, `ledger:reason`, `ledger:ends`, `ledger:availableAt`;
- revocations: `ledger:revokes`;
- key bindings: `ledger:bindingAct`, `ledger:principal`, `ledger:keyType`,
  `ledger:publicKey`, `ledger:closes`, `ledger:selfBound`, `ledger:mandate`;
- namespace policy: `ledger:requiresScheme`, `ledger:requiresSecurityKey`,
  `ledger:acceptRole`, `ledger:reacceptWithinDays`, `ledger:replacesPolicy`.

**Four points for the reader:**
1. Predicates such as `ledger:id`, `ledger:hash`, `ledger:scope` and
   `ledger:namespace` now occur on non-decision nodes, so the reader must
   dispatch on `rdf:type`.
2. `ledger:revokes` can target a grant as well as an acceptance.
3. `ledger:revokedAt` and `ledger:revokedBy` no longer appear on
   acceptances.
4. Legacy revocation nodes (`<urn:rev:legacy-<acc-ulid>>`) carry no
   `ledger:id`.
