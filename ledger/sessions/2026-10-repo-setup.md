# Repository setup close-out: the ledger's own directory

2026-10-06. Run against `main` at `f0bb9ed` (the merge of
mindovermachine-dev/product-cli#107), not the `969c4fa` the prompt was written
against. Since then `main` gained the 2026-10-06 key-close rulings (`ledger-core/src/authority/key_close.rs`,
format document §3.10.5 and §3.10.6). The prompt's claims held at `f0bb9ed`
except where noted below. Where they differed, I followed the repository.

## 1. Pull requests

- **PR 1, the move** (__PR1__): `git mv` of the thirteen files in §2 into `ledger/`.
  Citations of the moved paths were updated in `CLAUDE.md`, the comment on
  the "Ledger gate" step in `.github/workflows/product-ci.yml`, eight
  `ledger-core/src` comments, `docs/ddd-adrs.md`, `docs/ddd-cli-prd.md`,
  `docs/audits/provenance-2026-08.md`, and the moved documents' citations of
  each other. The PR also adds `ledger/README.md` (index and path map). It
  makes no code change.
- **PR 2, the protocol** (__PR2__): the three `incoming/` files placed unedited at
  `ledger/spec/ledger-protocol.md`, `ledger/spec/server-client-protocol.md` and
  `ledger/rulings/ground-and-protocol-rulings-2026-10.md`. Adds
  `ledger/spec/tests/README.md`, the status table in `ledger/README.md`, and
  this close-out. The PR is stacked on PR 1.

**Checks, before and after PR 1.**

- `ledger verify --export` printed byte-identical output on both sides: exit
  `0`, "conformant — 284 entries, 93 decision(s)", 3 awaiting acceptance, the
  two no-policy notices, and "every committed export matches the log byte for
  byte".
- `git hash-object` of each export was unchanged:
  `docs/decisions/hafeok.ddd.nt` = `33218d29376b66de7c241a7e2de976159e5802d9`,
  `docs/decisions/hafeok.ledger.nt` = `5440946bb5fee2ae1aace691d32d791bc8a29f55`.
- `cargo build`, `cargo clippy -- -D warnings -D clippy::unwrap_used` and
  `cargo t`: __GATES__.
- `cargo fmt --all -- --check` fails on `main` already (for example
  `ddd-cli/src/commands/bind.rs`). It is not one of the `CLAUDE.md` gates, and
  this session changed no code it could affect.

**Code that reads a path (step 0.3).** No code reads a moved path. The only
`include_str!` calls under `docs/` read `docs/g-track/registry-template/`
(`product-core/src/registry/template.rs`,
`product-core/src/ground/projection_tests.rs`). The only path constants are
`docs/decisions` (`EXPORT_DIR` in `ledger-core/src/graph/export.rs`, the
`ls-tree` call in `ledger-core/src/inbox/git.rs`, the `git add` in
`ledger-cli/src/commands/inbox_accept.rs`), `docs/eval-format-v1/`
(`eval-core/tests/fixture.rs`), `docs/examples/` (`ddd-cli/tests/render.rs`),
and the `docs/` status check in `product-core/src/fileops.rs`
(`warn_uncommitted_changes`). That check covers `ledger/` no longer, which only
narrows a warning.

**Basis pointers are not paths.** `prd:decision-ledger-prd#…` in
`.decisions/log/`, `docs/decisions/hafeok.ledger.nt`, `docs/ledger-format-v1.md`
§4.4 and the `ledger-core`/`ledger-cli` tests names the document by its base
name. The base name did not change, so none of these was touched. The canonical
form hashes them, so editing them would move digests.

## 2. The path map as landed

| Old path | New path |
| --- | --- |
| `docs/ledger-authority/ledger-authority.ttl` | `ledger/spec/authority/ledger-authority.ttl` |
| `docs/ledger-authority/ledger-authority-shapes.ttl` | `ledger/spec/authority/ledger-authority-shapes.ttl` |
| `docs/ledger-authority/sample-valid.ttl` | `ledger/spec/authority/sample-valid.ttl` |
| `docs/ledger-authority/sample-invalid.ttl` | `ledger/spec/authority/sample-invalid.ttl` |
| `docs/signing-rulings-2026-10.md` | `ledger/rulings/signing-rulings-2026-10.md` |
| `docs/signing-rulings-2026-10-d5-d9.md` | `ledger/rulings/signing-rulings-2026-10-d5-d9.md` |
| `docs/decision-ledger-prd.md` | `ledger/prd/decision-ledger-prd.md` |
| `docs/ledger-cli-prd.md` | `ledger/prd/ledger-cli-prd.md` |
| `docs/decision-registry-prd.md` | `ledger/prd/decision-registry-prd.md` |
| `docs/audit-2026-10.md` | `ledger/audits/audit-2026-10.md` |
| `docs/sessions/2026-10-session-a.md` | `ledger/sessions/2026-10-session-a.md` |
| `docs/sessions/2026-10-session-b.md` | `ledger/sessions/2026-10-session-b.md` |
| `docs/sessions/product-cli-v2-sessions.md` | `ledger/sessions/product-cli-v2-sessions.md` |
| `incoming/ledger-protocol.md` (untracked) | `ledger/spec/ledger-protocol.md` |
| `incoming/server-client-protocol.md` (untracked) | `ledger/spec/server-client-protocol.md` |
| `incoming/ground-and-protocol-rulings-2026-10.md` (untracked) | `ledger/rulings/ground-and-protocol-rulings-2026-10.md` |

`docs/sessions/` is now empty, so git no longer has the directory. Git records
every moved file as a rename: 100% similarity for the files whose text did
not change, 91–99% for those whose citations of other moved files were
updated.

## 3. Citations not updated, and why

- **`docs/ledger-format-v1.md`** (lines 450 and 1379: `docs/ledger-authority/…`)
  and **`docs/ledger-format-migrations.md`** (line 230:
  `docs/signing-rulings-2026-10.md`). The prompt says to leave both files and
  every citation of them untouched. This conflicts with its own check that
  "a search for the old paths returns only the path map and `.ddd/`". I kept to
  the stricter instruction, so these three lines still show up in that search,
  and the path map resolves them. They go away when the absorption session
  deletes the format document.
- **`ledger/rulings/ground-and-protocol-rulings-2026-10.md`** says "until it is
  merged under `docs/`" and cites `docs/ledger-cli-prd.md` §0. These are in the
  text of a placed document, which the prompt says not to edit. The export did
  not break them: no Markdown link was involved.
- **`.ddd/`**: no file in it cites a path that moved. Its ledger citations
  (`.ddd/seams/seam-ledger-{acceptor-identity,canonical-form,disposition-states,graph-stage,verify-classes}.yaml`,
  `.ddd/render.html` line 604) all name `docs/ledger-format-v1.md` or
  `docs/ledger-format-migrations.md`, which did not move.
  `.ddd/seams/seam-rust-ledger-cli-src-commands-export-rs.yaml` names
  `docs/decisions/`, which also did not move. `.ddd/render.html` is generated
  by `ddd render` (`ddd-cli/src/commands/render.rs`, default output
  `.ddd/render.html`). Since nothing it cites moved, it needs no regeneration.
  When the absorption session deletes the format document, the five seams and
  the page will cite a missing file. Regenerating the page then is in scope
  for `ddd`'s own tool, but editing the seams is not in scope for a ledger
  session.
- **`.decisions/` and `docs/decisions/*.nt`**: they cite no moved path, only
  the base-name basis pointers covered in §1.
- **Issue bodies** in mindovermachine-dev/product-cli (for example the ones cited
  by the moved session documents: #65, #66, #67, #69, #70, #71, #79, #82, #85,
  #86, #96) and **other repositories** (`Hafeok/decision-cli`,
  `Hafeok/decision-driven-analyzers`, `Hafeok/Varve`) cannot be edited from
  here. The path map in `ledger/README.md` resolves them.
- **Historical text inside moved documents**, for example
  `ledger/audits/audit-2026-10.md` line 15: "There is no `ledger-authority.ttl`
  … in this repository". It describes the state when the audit was written and
  cites no path, so I left it.

## 4. Documents left in `docs/` that may belong to the ledger

For the principal to decide. None was moved.

| Document | What it is | Reading |
| --- | --- | --- |
| `docs/acceptance-worksheet-2026-08.md` | Grouping and sequence for the August pass over 80 pending ledger entries (`ledger show --group`). | Ledger. Nothing cites it, so moving it to `ledger/sessions/` or `ledger/audits/` is free. |
| `docs/ledger-format-v1.md`, `docs/ledger-format-migrations.md` | The format document and its migrations. | Ledger. Excluded by the prompt; the absorption session takes them. |
| `docs/audits/provenance-2026-08.md` | Cross-repo provenance of decisions across five repositories. | Shared. It reads decisions across both stores. |
| `docs/audits/basis-quality-2026-08.md` | Tests the DDD claim `DDD-method-06` over pre-format-5 decisions. | DDD. Cited by ten `.ddd/` files and by `.decisions/log/01KZTGHF4B5HHKMR0QGBRBFD38.yml` and `01KZX70S86QGXVCA5GW5WSY6XA.yml`, so it cannot move without leaving a cited path behind in an immutable store. |
| `docs/audits/classifier-corpus-2026-08.md` | M8 classifier recall reading. | DDD. Cited by `ddd-cli/tests/corpus.rs`. |
| `docs/way-of-working-decision-allocated-delivery.md` | The Context& way of working; the ledger is its record substrate. | Shared. Cited by `ledger-core/src/tier.rs` and `ledger/prd/decision-ledger-prd.md`. |
| `docs/reviews/ddd-cli-prd-review-2026-08.md` | Review of the DDD PRD. | DDD. |
| `docs/g-track/` | The G-track PRD and its session reports. | Ground (`ground-cli`), not the ledger. See question 3 in §6. |

## 5. Disagreements between the protocol drafts and the format document

I noted these while placing the drafts and fixed none of them.
"LP-" ids are `ledger/spec/ledger-protocol.md`, "SC-" ids are
`ledger/spec/server-client-protocol.md`, and § numbers are
`docs/ledger-format-v1.md`. Until the absorption session lands, the format
document governs.

**Payloads and hashing**

1. **The acceptance payload lacks `under`.** The draft's §4 table gives
   `{decision, version, actor, at, scope, expires_at}` with the prefix "Extraction".
   §3.10.2 gives `ledger.acceptance.v1` with `under` (D9 (a)). *(Known.)*
2. **The revocation payload lacks `under`.** The draft gives `ledger.revocation.v1`
   `{revokes, actor, at, reason}`. §3.10.2 adds `under`. The same holds for the
   grant, key binding and policy payloads, which the draft leaves as
   "Extraction". §3.9.3 and §3.10.2 list their keys, and the policy payload
   includes `at` (D8).
3. **Points the format document already fixes are marked "Extraction".** These are:
   - the timestamp spelling: RFC 3339 UTC, whole seconds, `Z` (§3.10.2);
   - the canonical byte grammar: NFC, whitespace strip, code-point key and set
     order, RFC 8785 escaping, prefix ‖ `0x0A` framing (§4.2, §4.3, §3.10.3);
   - the version prefix `ledger.decision-version.v1` and its closed field set
     (§4.1);
   - the id prefixes `acc:`, `cs:`, `key:`, `pol:` and `avail:` (§3.1, §3.9.2);
   - the DSSE payload type `application/vnd.ledger.signed-bytes.v1` (§3.10.4);
   - the `allowed_signers` line format and order (§3.9.5);
   - the N-Triples form and sort order (§8.1);
   - the sidecar reference `ledger:signature <urn:sig:<file>>` (§3.10.8).

   The draft says it was written before the repository was read, so these are
   gaps rather than conflicts. Still, the draft's claim that "no implementation
   can claim conformance" on them does not hold against the format document.
4. **The draft adds version fields the format does not have.** These are `grounds`,
   `source_prefix`, `source_method` and `source_keys`. §4.1 says "exactly these
   keys, and no others", so in the format today they are schema faults. They come
   from the ground rulings, which are not implemented.
5. **LP-3.4 says "no numbers, booleans or nulls in hashed content".** §4.2 bans
   floats, treats `null` as absent, and renders booleans as `"true"` or omits
   them. The policy payload hashes `reaccept_within_days` (§3.9.3), and the
   format document does not say it is a string.

**Entities**

6. **The entity list.** The draft says "thirteen entity kinds". It has no set
   file (§3.5), availability (`avail:`, §3.9.2), judgment (`L010`) or legacy
   revocation shape (§3.9.2). It has review and ground, which the format does
   not have.
7. **Hashed and signed columns.** The draft marks the change-set, role and
   unavailability "Hashed: Extraction". §4.1 puts change-set fields outside
   the hash, and §3.10.1 says a role file and an unavailability have no hashed
   payload. The draft marks the grant and grant acceptance "Signed:
   Extraction". §3.10.4 lists the signable entities as the acceptance, the
   revocation of an acceptance, the key binding and the policy change. A
   grant's revocation is signed with the grant (#82).
8. **LP-3.3 says every non-version entity "is identified by a ULID with a type
   prefix".** Sets and roles are identified by a set-id-shaped id (§3.5,
   §3.9.1), as the draft's own entity table shows for the role.
9. **Terminology: "Decision: a lineage of versions under one key."** In the
   format a decision is identified by `dec:<ns>/<ulid>` and `key` is optional
   (§3.8). No version in this repository's `.decisions/log/` carries one today.
10. **LP-3.9 says a key is "carried to the successor on supersession".** §3.8
    says a superseded decision's key is free for its successor, which permits
    carrying it but does not require it.

**Authority**

11. **`A006` is described differently.** Draft §8 and LP-6.12: an acceptance's
    actor has no live grant whose role may exercise every claim the version's
    class requires. §3.10.7: every acceptance, `rev:` revocation and policy is
    re-judged as of its landing position against the grant it names (`under`).
    A governed act with no `under` fails. A legacy revocation after the first
    policy fails. A policy not made under the genesis grant fails. The check
    never searches for another grant, while the draft's wording ("has no live
    grant") implies a search. *(Known.)*
12. **Claims and `accept_role`.** The draft's closed vocabulary adds
    `trust-source` (eight claims; §3.9.1 has seven). The draft decides the
    required claim by decision class and lets policy add claims per class
    (LP-6.10 to LP-6.12). The format decides by the policy's `accept_role`:
    only that role's grants count for accepting or revoking, and the genesis
    role carries no decision claim (§3.9.4, D9 (f)). The draft's policy fields
    are "required schemes, key type, class requirements, re-acceptance
    deadline". The format's are `schemes`, `require_sk`, `accept_role`,
    `reaccept_within_days`, `replaces` and `under`/`at`.
13. **The role check is narrower in the draft (LP-6.2).** It leaves out what the format
    has: the named grant (`under`), fewest claims (D9 (c)), the escalation guard
    for `grant new`, and outranking by covering scope (D9 (e)).
14. **Fallback ordering is listed as open.** Draft §12 asks "whether fallback
    ordering uses covering scope or identical scope". §3.9.4 records it as
    ruled: covering scope, D9 (e).
15. **`A003` (LP-6.6 and the draft's §8 table) leaves out "accepted".** The format
    counts only grants that are live, which includes accepted by a
    `ledger:GrantAcceptance`, and `A005` excludes a revoked genesis (§8).
16. **`L006` has a different scope.** The draft says "a model is named as a
    holder or as the actor of a revocation". §5.2 and §3.9.4 cover acceptance
    actors, an escape's `accepted_by`, and every identity an authority record
    names: holder, grantor, grant-acceptance actor, (un)availability declarer,
    revocation actor, binding principal and filer, and policy author.

**Signing and keys**

17. **`L011` and LP-4.13 judge a close by date only.** §3.10.6 also fails an
    entity that *landed* after the close, whatever its `at`, and makes an
    acceptance a review item only when it is both dated and landed before the
    close.
18. **LP-4.9 and LP-4.12 on policies and bindings.** The draft says "every later
    binding is signed under the policy in force". §3.10.5 says a binding before
    the first policy is judged by that first policy's requirement, a principal's
    first key is filed and signed by the genesis holder, and the genesis
    holder's first binding in a later namespace is their own `add`, signed by a
    key trusted elsewhere. §3.10.4 also judges a first policy under its own
    schemes when its author held a trusted key (#96), and §3.10.7 makes every
    policy the genesis holder's act. None of this is in the draft.
19. **The 2026-10-06 key-close rulings are missing.** A close ends the key, not the
    binding. A closed key is never bound again. A key belongs to one principal
    (§3.10.5, §3.10.6). The draft predates them.
20. **`dsse` verification.** The draft says it is verified "against the key
    material the policy names". §3.10.4 says it is an ed25519 signature over
    DSSE's PAE by an `ssh-ed25519` key the signer has bound in the namespace,
    open at the entity's `at`.
21. **`none` is described differently.** The draft says it is "valid only for stores that predate
    signing". §3.10.4 makes `none` governed-and-unsigned (role-checked, not
    signature-checked) with no restriction to stores that predate signing.

**Verification**

22. **Class naming (LP-8.4).** The draft names file-gate classes `L` and
    graph-stage classes `A`. The format also has a `SCHEMA` fault (§5.1),
    graph classes `G001`–`G006` (§8), and the `[EXPORT]` and `[SIGNERS]`
    stages (§8.1, §3.9.5).
23. **`L001`.** The draft says "a decision has no allocation". §5.2 says a
    decision's *latest* version carries no `allocation`.
24. **Derived state.** The draft's states are proposed, accepted, citable,
    superseded, revoked, rejected, needs re-acceptance and trusted. The format's
    disposition vocabulary is undecided, awaiting-acceptance, decided,
    escaped-priced, escape-review-due, expired and superseded (§8, `ledger
    coverage`). The draft has no escape states, and the format has no rejected
    or citable.

**Server-client draft**

25. **SC-3.7: batch rows.** The draft's rows are a repository, a decision and a
    version hash. §3.10.9 rows also carry an optional `branch` and `grant`, and
    the file carries `actor` and an optional `as`. The manifest is
    `ledger.acceptance-manifest.v1` with a `selector` key.
26. **SC-2.5: SSH login against `allowed_signers`.** The draft verifies the
    login signature under the namespace `ledger-auth@<host>` against the union
    of the indexed namespaces' `allowed_signers`. Each line of that file is
    restricted to `namespaces="ledger-accept@<ns>"` (§3.9.5), so
    `ssh-keygen -Y verify -n ledger-auth@<host>` would refuse every line as
    written.

## 6. Questions for the principal

1. **Should the ledger section of the root `CLAUDE.md` become `ledger/CLAUDE.md`?**
   - (a) Keep it in the root. The crates it governs (`ledger-core/`,
     `ledger-cli/`) stay at the root, and Claude Code loads a subdirectory
     `CLAUDE.md` only when files under that directory are read. A session that
     edits only the crates would then never see it.
   - (b) Move the document-facing half (format, rulings, protocol status)
     to `ledger/CLAUDE.md` and keep the crate-facing half (gate, verbs,
     interactivity, acceptance rule) in the root.
   - (c) Move it all, and leave a one-line pointer in the root. You have ruled
     against pointers between documents, though, and this would be one.

   I recommend (a) until the absorption session settles what the protocol
   replaces.
2. **Should this layout be filed as a decision in `hafeok.ledger`?**
   - (a) Yes, in a set such as `ledger-design`, with
     `allocation: constraint` and the path map as its statement's ground. Any
     later move would then be a revision.
   - (b) No. A directory layout is housekeeping. The README's path map is the
     record, and git history carries it.
   - (c) File it only once the absorption session lands, so that it records
     the final layout.

   I filed nothing. If you choose (a), it waits for your acceptance like any
   other decision.
3. **Is the protocol's "ground" the same concept as the G-track's?**
   - The protocol uses "ground" for what a version rests on: another version
     by hash, or external bytes by digest. It is a Merkle edge
     (`ledger:groundedOn`, a sub-property of `prov:wasDerivedFrom`).
   - The G-track (`docs/g-track/prd-ground-as-ontology.md`, `ground-cli`,
     `product-core/src/ground/`) uses "ground" for the organisation's ground
     registry: the ontology across axis layers that the What is expressed
     over.
   - The format document already uses a third sense: a set's
     `ground: characterised | uncharacterised` (§3.5).
   - The eval and spec stores use a fourth: a run's "declared ground", its
     address (`docs/eval-format-v1.md`).

   They are related, since a ledger ground could point into the registry, but
   they are not the same thing. The options are:
   - (a) The G-track keeps "ground", and the protocol renames its edge (for
     example "basis", which the format's `based_on` already uses, or
     "foundation" for the closure it already names).
   - (b) The protocol keeps "ground", and the G-track's becomes "ground
     registry" everywhere, never a bare "ground".
   - (c) Both keep it, with a glossary entry that separates the senses.

   I lean to (a): `based_on` is already the format's word for the open-vocabulary
   predecessor of grounds. But this is a naming ruling, and it belongs to you.

## 7. Issues to raise

I opened none. For the principal to file:

- **Format document:** it still cites `docs/ledger-authority/` (lines 450 and
  1379), and the migrations cite `docs/signing-rulings-2026-10.md` (line 230).
  These will be absorbed or removed in the absorption session.
- **Ledger protocol:** the 26 disagreements in §5, for the absorption session.
- **Server-client protocol:** the `allowed_signers` namespace restriction
  against SSH login (§5, item 26).
- **`.ddd/` seams:** five seams will cite a deleted file once
  `docs/ledger-format-v1.md` is absorbed (§3). This is for the `ddd` owner.
- **`docs/acceptance-worksheet-2026-08.md`:** decide whether it moves (§4).
