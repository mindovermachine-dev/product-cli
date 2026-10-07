# Protocol absorption close-out: one normative text for the ledger

2026-10-06. Run against `main` at `275638c`, the commit the prompt (revision 4) was written against. The repository had not moved.

## Summary (10 lines)

1. **Path filter:** [mindovermachine-dev/product-cli#110](https://github.com/mindovermachine-dev/product-cli/pull/110) adds `'ledger/**'` to both `paths:` lists of `.github/workflows/product-ci.yml`, and nothing else.
2. **Absorption:** `docs/ledger-format-v1.md` is merged into `ledger/spec/ledger-protocol.md`, and both format files are deleted with no stub. The protocol is now the normative format. Unimplemented requirements are marked **Not implemented**, and unruled names **Open**.
3. **Ids:** all 76 draft ids keep their numbers, and absorbed requirements take the next free number in their section, with no gaps or duplicates. Top-level section numbers 1–14 did not move; subsections are now numbered.
4. **Exact copies:** the algorithm, the digest, the §4.4 vector, every YAML example and the signed-bytes example are byte-identical to the format document. The migration notes are Appendix C.1, verbatim apart from heading levels. The format document's status paragraph is Appendix B.1, verbatim.
5. **Renames and rulings:** the renames of rulings 23 and 25 are applied, and rulings 22–26 are filed at `ledger/rulings/basis-and-absorption-rulings-2026-10-06.md`.
6. **Server-client protocol:** SC-2.5 and step 5 of §3.3 changed (ruling 26), three "ground" renames (ruling 23), and one Appendix B row.
7. **Checks:** build and clippy pass. `cargo t` gives 2083 passed, 0 failed with the container's git-identity variables unset (§11). `ledger verify --export` is identical to `main`, both exports hash the same, and `ddd validate` passes.
8. **Scope:** no code change beyond comments; no fixture, test or digest change; nothing filed, accepted, revised or revoked; no issue opened.
9. **Disagreements:** 36 recorded (§3), the 26 known plus 10 found. Four are left for the principal because a ruling and the implemented format point different ways.
10. **Open:** thirteen places where the code differs from its text (§4), and fourteen questions (§10). The weightiest: `\v` in canonicalisation, a forked decision's stand-in "latest", and the `ledger:basis` name collision.

## 1. Section map

Format document § → protocol section and requirement ids, one row per normative statement. "Note" means a non-normative note in the protocol; "12.x" rows are non-normative sections. The section-level map is in `ledger/README.md` and in the protocol's Appendix C.0.

| Format § | Statement | Protocol § | Id |
| --- | --- | --- | --- |
| Header | normative for formats 1–7, revision v1.8, revision history | Status; B.1 (verbatim) | — |
| Header | scope: L0, graph stage a distinct stage | 1 (format conformance); 8.1 | LP-8.6 |
| Header | audience; this document wins over `ledger-core` | 1 (Audience, Authority) | — |
| Header | migrations companion; validation per declared format | Status; C | LP-3.14 |
| §1 | acceptance signs a hash, never an id; purpose | 2 | — |
| §1 | two versions, independent; canonical bump is a governed act | 4.1 | LP-4.15 |
| §2 | layout `sets/`, `log/`, `index/` | 3.1 | — |
| §2 | `.yml` or `.yaml` read, `.yml` written | 3.1 | LP-3.10 |
| §2 | log file written once, never edited | 3.1 | LP-3.11 |
| §2 | stem equals declared id, else schema fault | 3.1 | LP-3.12 |
| §2 | `index/` not created at L0; ignore line | 3.1 | LP-3.13 (updated: L2 writes it, LP-9.18) |
| §3 | every file declares `format` | 3.2 | LP-3.14 |
| §3.1 | id forms `dec:`, `cs:`, `acc:`, `sha256:` | 3.3 | LP-3.3 |
| §3.1 | ULID alphabet, first char ≤ 7 | 3.3 | LP-3.17 |
| §3.1 | namespace: owning scope, segment grammar; foreign namespaces referenceable | 3.3 | LP-3.18 |
| §3.1 | decision id permanent; supersession mints new id | 3.3 | LP-3.19 |
| §3.1 | namespace not restated | 3.3 | LP-3.20 |
| §3.2 | identity is email, lowercased | 3.4 | LP-3.6, LP-3.21 |
| §3.2 | one `@`, non-empty parts, no whitespace, dotless domain legal | 3.4 | LP-3.21 |
| §3.2 | five refusal rules (`L006`) | 3.4 | LP-3.22 |
| §3.2 | whole-token rationale | 3.4 (rationale) | — |
| §3.2 | list is a floor, not a proof | 3.4 | LP-3.23 |
| §3.3 | tier order, floor, effective tier | 5.1 | — |
| §3.3 | override at or below floor rejected at write (`L004`) | 5.1 | LP-5.8 |
| §3.3 | floor pinned on version; stranded `L005`; never grandfathered | 5.1 | LP-5.9 |
| §3.4 | scheme table | 5.4 | — |
| §3.4 | unknown scheme schema fault; nothing resolves | 5.4 | LP-5.16 |
| §3.4 | `contract:` is format 3; lower format is a fault; hashing unaffected | 5.4 | LP-5.17 |
| §3.4 | `discharge_stage` values | 5.4 | LP-5.18 |
| §3.5 | set file schema | 5.2 | — |
| §3.5 | set does not list members | 5.2 | LP-5.10 |
| §3.5 | honest limit | 5.2; 8.10 | LP-8.33 |
| §3.6 | change-set schema (example verbatim) | 5.3 | — |
| §3.6 | unknown keys rejected | 5 | LP-5.7 |
| §3.6 | `created_by` is who acted; identity objects first appearance only | 5.3 | LP-5.11 |
| §3.6 | per-allocation obligations | 5.3 | LP-5.12 |
| §3.6 | allocation may be absent | 5.3 | LP-5.13 |
| §3.6 | `signature` reserved, empty | 5.3; 4.8 | LP-4.8 |
| §3.6 | `merged_from` format 2; reconciliation; no acceptance survives | 5.3 | LP-5.14 |
| §3.6 | `scope: version` or `class:<ref>` | 5.3 | LP-5.15 |
| §3.6 | `based_on` single tokens, open vocabulary | 7.1 | LP-7.19, LP-7.20 |
| §3.6 | `revisit_if` not part of `based_on` | 7.2 | LP-7.22 |
| §3.7 | reopen edge semantics | 7.2 | — |
| §3.7 | never a basis; distinct types | 7.2 | LP-7.22 |
| §3.7 | basis-loss vs reopen findings; neither a gate class | 7.2; 7.7 | LP-7.23 |
| §3.7 | declare-a-format-all-content-is-valid-in; two faults | 3.2 | LP-3.15 |
| §3.7 | #81 enforcement history | 7.2 (paragraph) | — |
| §3.7 | vocabulary open, not shared | 7.2 | — |
| §3.7 | hashing as a separate set | 7.2 | LP-7.25 |
| §3.7 | not a gate class | 7.2 | LP-7.26 |
| §3.8 | `key` grammar, `exported` | 3.5 | LP-3.9 |
| §3.8 | immutable once given (`L013`) | 3.5 | LP-3.25 |
| §3.8 | unique among live decisions (`L014`) | 3.5 | LP-3.26 |
| §3.8 | both are file-gate classes; `G006` never the only home | 3.5 | LP-3.27, LP-8.5 |
| §3.8 | format 5 declare rule | 3.5 | LP-3.28 |
| §3.8 | hashing | 3.5 | LP-3.29 |
| §3.9 | authority records, schema the vocabulary projects | 5.6 | — |
| §3.9.1 | `roles/`, `allowed_signers` files; role schema | 5.6 | LP-5.19 |
| §3.9.1 | closed capability vocabulary | 6.1 | LP-6.1 |
| §3.9.2 | format 6 for these entries | 5.6 | LP-5.20 |
| §3.9.2 | log-entry schema (verbatim) | 5.6 | — |
| §3.9.2 | two revocation shapes; legacy only pre-policy | 5.6 | LP-5.21 |
| §3.9.3 | closed payloads under one law | 4.6 | LP-4.22 |
| §3.9.3 | payload table | 4.6 | — |
| §3.9.3 | stored hash never in its payload; legacy computable | 4.6 | LP-4.22, LP-4.23 |
| §3.9.4 | no new class | 8.4 | LP-8.16 |
| §3.9.4 | `SCHEMA` rules for authority records | 8.4 | LP-8.16 |
| §3.9.4 | `L006` extension | 8.4 | LP-8.16, LP-3.7 |
| §3.9.4 | `L007` extension | 8.4 | LP-8.16 |
| §3.9.4 | liveness, availability, policy in force | 6.1 | LP-6.15 |
| §3.9.4 | role check is verb-time; scope matching; fallback by covering scope | 6.1 | LP-6.16 |
| §3.9.4 | accept role, never the genesis role; genesis capabilities | 6.1 | LP-6.17 |
| §3.9.4 | no policy: nothing role-checked | 6.1 | LP-6.18 |
| §3.9.4 | check also at verify (`A006`) | 6.1; 6.6 | LP-6.16, LP-6.27 |
| §3.9.5 | derived, one line per window; line form | 4.9 | LP-4.32 |
| §3.9.5 | comma options; v1.7 correction | 4.9 | LP-4.32 (+ paragraph) |
| §3.9.5 | `valid-after`, `valid-before` from the key's earliest close | 4.9 | LP-4.32 |
| §3.9.5 | only trusted bindings | 4.9 | LP-4.32 |
| §3.9.5 | sorted, two-line header | 4.9 | LP-4.32 (header bytes filled from code) |
| §3.9.5 | `[SIGNERS]` stage | 4.9 | LP-4.33 |
| §3.10 | D1, D2, D4 | 4.8 | LP-4.6, LP-4.7 |
| §3.10.1 | `under` table | 6.2 | LP-6.19 |
| §3.10.1 | only five kinds carry `under`; role declare and unavailable checked without it; grant acceptance none | 6.2 | LP-6.19, LP-6.20 |
| §3.10.1 | which grants authorise accept; narrowest scope, lowest rank | 6.2 | LP-6.21 |
| §3.10.1 | fewest claims | 6.2 | LP-6.22 |
| §3.10.1 | escalation guard | 6.2 | LP-6.23 |
| §3.10.1 | verbs print `under <grant> (<role>)` | 6.2 (note) | — |
| §3.10.1 | `under` → format 7; lower format fault | 6.2 | LP-6.24 |
| §3.10.1 | inline `signature` retired | 4.8 | LP-4.8 |
| §3.10.2 | payload table with `under`, `at` | 4.6 | — |
| §3.10.2 | instants RFC 3339 UTC seconds `Z` | 3.3; 4.6 | LP-3.24 |
| §3.10.2 | acceptance digest computed, never stored | 4.6 | LP-4.24 |
| §3.10.3 | signed bytes = digest input; example | 4.7 | LP-4.5 |
| §3.10.3 | `ledger_core::hash::signed_bytes` | report §4 only | — |
| §3.10.4 | sidecar name; misnamed is a fault | 4.8 | LP-4.7 |
| §3.10.4 | signable entities and signers | 4.8 | LP-4.25 |
| §3.10.4 | `<ns>` is the store's namespace | 4.8 | LP-4.26 |
| §3.10.4 | `ssh` | 4.8 | LP-4.27 (+ example) |
| §3.10.4 | `dsse` | 4.8 | LP-4.28 |
| §3.10.4 | `none` exclusive | 4.8 | LP-4.29 |
| §3.10.4 | requirement; pre-policy exemption; bindings never exempt | 4.8 | LP-4.30, LP-4.9 |
| §3.10.4 | first policy (#96) | 4.8 | LP-4.31 |
| §3.10.5 | trusted binding; judged first | 4.10 | LP-4.34 |
| §3.10.5 | before the first policy | 4.10 | LP-4.35 |
| §3.10.5 | `init --namespace` binds the genesis key; `--without-key`; later namespace | 4.10 | LP-4.38 (+ note) |
| §3.10.5 | D7 filers (four bullets) | 4.10 | LP-4.12 |
| §3.10.5 | anyone else: schema fault; bad signature `L011` | 4.10 | LP-4.36 |
| §3.10.5 | which keys may be bound (three bullets) | 4.10 | LP-4.37 |
| §3.10.6 | landing commit; position | 8.7 | LP-8.24 |
| §3.10.6 | entity of a change-set file; `format:` not an entity | 8.7 | LP-8.25 |
| §3.10.6 | before, enabling, terminating | 8.7 | LP-8.26 |
| §3.10.6 | role file placed by landing | 8.7 | LP-8.27 |
| §3.10.6 | policy governs every act not before it | 8.7 | LP-8.28 |
| §3.10.6 | branches and the base | 8.7 | LP-8.29 |
| §3.10.6 | closed keys: a close ends the key; all bindings; finding names three things | 4.11 | LP-4.39 |
| §3.10.6 | closed-key verdicts; review item; `L012` | 4.11 | LP-4.13 |
| §3.10.6 | reader takes the latest valid acceptance | 4.11 | LP-4.14 |
| §3.10.7 | `A006` | 6.6 | LP-6.27 |
| §3.10.7 | a policy is the genesis holder's act | 6.6 | LP-6.28 (+ note) |
| §3.10.7 | a role takes effect from its landing | 6.6 | LP-6.30 |
| §3.10.7 | old-style revocation is a pre-policy act | 6.6 | LP-6.29 |
| §3.10.7 | unchecked, unsigned, unbound-genesis notices | 8.8 | LP-8.31 (+ note for the JSON keys) |
| §3.10.7 | landed entities immutable; three cases | 8.7 | LP-8.30 |
| §3.10.7 | `format:` correction | 3.2 | LP-3.5, LP-3.16 |
| §3.10.8 | export carries signable payloads and sidecar nodes | 9.3 | LP-9.3 |
| §3.10.8 | reconstruction rules | 9.3 | LP-9.14 |
| §3.10.8 | rebuilds `allowed_signers`; verifies | 9.3 | LP-9.14 |
| §3.10.8 | its limit: no landing order | 9.3 | LP-9.15 |
| §3.10.9 | batch file is a hand-off, never committed, unsigned | 10.1 | — |
| §3.10.9 | shape | 10.1 | LP-10.1 |
| §3.10.9 | manifest | 10.1 | LP-10.2 |
| §3.10.9 | signing in a clone; drift; whole-batch refusal; confirmation | 10.1 | LP-10.3 |
| §3.10.9 | one confirmation, one signature per acceptance | 10.1 | LP-10.4 |
| §3.10.9 | the actor | 10.1 | LP-10.5 |
| §4 | YAML file form, canonical JSON hash form | 4 (intro) | — |
| §4.1 | hashed field set, exactly | 4.2 | LP-4.16 |
| §4.1 | what is outside the hash | 4.2 | LP-4.16 |
| §4.1 | `digests.rs` proves it | report §4 only | — |
| §4.1 | both tolerance inputs hashed | 4.2 | LP-4.17 |
| §4.2 | the algorithm, steps 1–8 (verbatim) | 4.3 | LP-4.18 |
| §4.2 step 7 | no floats | 4.3; 3 | LP-4.18, LP-3.4 |
| §4.3 | digest formula (verbatim) | 4.4 | — |
| §4.3 | prefix is domain separation and version pin; `.v2` on meaning change | 4.4 | LP-4.19, LP-4.3 |
| §4.3 | short form never compared | 4.4 | LP-4.20 |
| §4.4 | the conformance vector (verbatim) | 4.5 | LP-4.21 |
| §4.4 | the eight fixture stores are further vectors | 4.5 (paragraph, without count or path) | — |
| §5 | fails for schema fault or fourteen classes, nothing else; amendment | 8.2 | LP-8.7 |
| §5.1 | `SCHEMA` coverage | 8.2 | LP-8.8 |
| §5.2 | the fourteen | 8.3 | LP-8.9 |
| §5.2 | only latest judged | 8.3 | LP-8.10 |
| §5.2 | latest from the parent DAG | 8.3 | LP-8.11 |
| §5.2 | revoked acceptance not judged for expiry | 8.3 | LP-8.12 |
| §5.2 | `L008` checks the pair | 8.3 | LP-8.13 |
| §5.2 | `L009` skips, reported | 8.3 | LP-8.14 |
| §5.2 | pendency is status | 8.3 | LP-8.15 |
| §5.3 | readiness and completeness gates | 8.5 | LP-8.17 |
| §5.3 | exit codes | 8.5 | LP-8.18 |
| §6 | open edges (seven of eight bullets) | 12.4 | — |
| §6 | "planned, not yet normative: signing" | none (see §2) | — |
| §7 | L1 operations → library surfaces | 12.5 (behaviour); report §4 (symbols) | — |
| §8 | outside the import surface | 1; 8.1 | LP-8.6 |
| §8 | index, rebuild byte-identical | 9.1 | LP-9.18 |
| §8 | PROV-O provenance | 9.1 | LP-9.19 |
| §8 | no triple added after filing; revocation node | 9.1 | LP-9.4 |
| §8 | legacy revocation node; retired shape | 9.1 | LP-9.17 |
| §8 | `based_on` as `ledger:basedOn` literals | 9.1 | LP-9.16 |
| §8 | SPARQL shapes, distinct stage, exit semantics | 8.6; 8.5 | LP-8.19, LP-8.18 |
| §8 | G/A table | 8.6 | LP-8.19 |
| §8 | `A003`/`A005` tightenings; `A006` not SPARQL | 8.6 | LP-8.23 |
| §8 | `G004` | 8.6 | LP-8.20 |
| §8 | `G005` | 8.6 | LP-8.21 |
| §8 | graph classes closed | 8.6 | LP-8.22 |
| §8 | coverage dispositions; honest limit | 8.10 | LP-8.33 |
| §8.1 | export of one namespace at `docs/decisions/<ns>.nt` | 9.2 | LP-9.1 |
| §8.1 | read model the generator consumes | 9.2 | — |
| §8.1 | triples | 9.2 | LP-9.11 |
| §8.1 | form | 9.2 | LP-9.12 |
| §8.1 | export stage | 9.2 | LP-9.13 |
| §8.1 | `*.citations.nt` never compared | 9.2 | LP-9.2 |
| Migrations | every note | Appendix C.1 (verbatim, headings two levels down) | — |

## 2. Statements with no destination, and what I did with each

| Format § | Statement | What I did |
| --- | --- | --- |
| §6, last bullet | "Planned, not yet normative — signing … Until that revision ships, a non-empty `signature` stays a schema fault." | Dropped. The same document's §3.10 shipped that revision (v1.8). Its true remainder (the field is retired, any value is a schema fault; `L011`/`L012` keep their numbers) is LP-4.8 and LP-8.7. Its history is in Appendix C (format 7 note, "Known future migrations"). |
| §3.6, `signature` paragraph | Renumbering history of the signing slot; "a non-empty `signature` is still a schema fault (§6, last bullet)". | Condensed to one sentence in §5.3 plus LP-4.8. The renumbering history is in Appendix C. |
| §4.4, last paragraph | "The **eight** fixture stores under `ledger-cli/tests/fixtures/` are further vectors". | Kept as a non-normative paragraph in §4.5 without the count or the path (rule 5). There are **16** fixture stores, and `l007` is stale by design, so the count was wrong. |
| §4.1 | "The reference implementation proves it over every stored digest it holds (`ledger-cli/tests/digests.rs`)." | Moved to the traceability table (§4 below). |
| §3.10.3 | "The reference implementation's one function for these bytes is `ledger_core::hash::signed_bytes`, and `domain_hash` digests its output." | Moved to the traceability table. |
| §3.7 | "In the reference implementation the two are distinct *types* (`RevisitRef`, `BasisRef`)" | Kept without the type names (LP-7.22). The types are in the traceability table. |
| §3.10.7 | "the act is `Act::SetPolicy`, which the role check maps to the `grant-role` capability" | A note in §6.6 without the symbol. |
| §7 | The L1-operation → library-surface table (all Rust symbols), and "Semantic `diff` alone is deferred … arrives with L3". | The behaviour (every write refuses exactly what verification would report) is §12.5, non-normative. The symbols are in the traceability table. The "deferred" sentence is stale: `ledger diff` shipped with L3 (`ledger-core/src/diff.rs`). I dropped it. |
| §3.10.1 | "Every governed verb prints the grant it acted under: `under <grant> (<role>)`." | A non-normative note (§6.2). Message text is not protocol (§1). This lowers a stated behaviour from normative to informative. |
| §3.10.1, §3.10.9, §3.10.5, §8, §8.1 | Flags and commands: `--as`, `--confirm`, `--repository`, `--without-key`, `git config user.signingkey`, `ledger merge --resolve`, `ledger reindex`, `ledger export --format ntriples [--namespace] [--out]`, `ledger coverage`. | Restated as behaviour ("the act names the role", "a recorded arbitration", "explicitly told to proceed unbound"). Commands kept only in notes. The export command line has no destination. |
| §3.10.7 | `--json` carries `unsigned` and `genesis_unbound` | A note in §8.8. The notices themselves are LP-8.31. |
| §8 | "the PRD §5 correctness test, run in CI" | Dropped from LP-9.18, which keeps the rebuild property. Where CI runs it is not protocol. |
| §2 | "`index/` is not created at L0" | Superseded by §8 of the same document: L2 writes the index. LP-3.13 keeps the never-committed rule; LP-9.18 says what the index holds. |

No other normative statement lacks a destination. The full list is §1.

## 3. The 26 disagreements

Numbering follows `ledger/sessions/2026-10-repo-setup.md` §5. "Rule 1" is "the format document wins on everything implemented", "rule 2" is "the draft's new material stays, renamed and marked".

| # | Disagreement | Settled by | Resulting text |
| --- | --- | --- | --- |
| 1 | Acceptance payload lacks `under` | Rule 1 | §4.6 table: `ledger.acceptance.v1` over `decision, version, actor, at, scope, expires_at, under`. |
| 2 | Revocation payload lacks `under`; grant, binding, policy payloads "Extraction" | Rule 1 | §4.6 table gives all five payloads with `under`, and `at` in the policy's. |
| 3 | Points the format fixes marked "Extraction" | Rule 1, rule 6 | Instants LP-3.24 and §3.3; byte grammar LP-4.18; version prefix and field set LP-4.3, LP-4.16; id prefixes §3.3; DSSE payload type LP-4.28; `allowed_signers` LP-4.32; N-Triples LP-9.12; sidecar reference LP-9.3. The draft's "no implementation can claim conformance" sentence is gone from §1. |
| 4 | Draft adds `grounds`, `source_prefix`, `source_method`, `source_keys` | Rule 2, ruling 25 | `grounds` removed: pinned tokens inside `based_on` (§7.3, LP-7.1). The source fields stay in §5.7, **not implemented**, and §4.2 says they are schema faults under LP-4.16 today and join by format amendment. |
| 5 | LP-3.4 "no numbers, booleans or nulls" | Rule 1 | LP-3.4: no floats (a schema fault); `null` is absent; a flag is `"true"` or omitted; an integer payload field is its decimal string; `format` is not hashed. The integer rule comes from the code (§5 below). |
| 6 | Entity list | Rule 1, rule 2 | §5 table: set, change-set, decision, version, acceptance, both revocation shapes, role, grant, grant acceptance, unavailability, availability, key binding, policy, sidecar. Review and basis entity marked not implemented. The count "thirteen" is gone. |
| 7 | Hashed and signed columns | Rule 1 | §5 table: change-set, role and unavailability not hashed; grant and grant acceptance not signed (#82); signable entities LP-4.25. |
| 8 | LP-3.3 "every non-version entity is a prefixed ULID" | Rule 1 | LP-3.3: decisions by `dec:<ns>/<ULID>`, sets and roles by a set-id-shaped id, other log entities by a prefixed ULID. |
| 9 | "Decision: a lineage of versions under one key" | Rule 1 | Terminology: identified by `dec:<namespace>/<ULID>`, linked by `parent`, key optional. |
| 10 | LP-3.9 key "carried to the successor" | Rule 1 | LP-3.9: "free for its successor, which MAY carry it"; the Superseded row in §8.10 says the same. |
| 11 | `A006` described as a search | Rule 1 | LP-6.27 and the §8.6 row are the format's text: the named grant, never a search. The class claim is LP-6.12, not implemented. |
| 12 | Claims and `accept_role` | Rule 1, rule 2, ruling 24 | LP-6.1: seven claims implemented, `trust-source` not implemented. LP-6.17, LP-6.25: `accept_role` as built. LP-6.26: the class table generalises it, `accept_role` its `ordinary` row, not implemented. Policy fields: §4.6, §5.6. |
| 13 | LP-6.2 narrower than the format | Rule 1 | LP-6.2 rewritten; LP-6.16 and LP-6.19 to LP-6.23 carry `under`, fewest claims, the escalation guard and D9 (e). |
| 14 | Fallback ordering listed as open | D9 (e) | Removed from §12; LP-6.16 states covering scope. |
| 15 | `A003` leaves out "accepted" | Rule 1 | LP-6.6, the §8.6 row, LP-8.23. |
| 16 | `L006` scope | Rule 1 | LP-3.7 lists the roles refused, and that `created_by` is not; LP-8.16 has the authority extension. |
| 17 | `L011` / LP-4.13 judge a close by date only | Rule 1 | LP-4.13: dated at or after the close, or landed after it, is `L011`; dated and landed before is a review item, then `L012`. |
| 18 | LP-4.9, LP-4.12 on policies and bindings | Rule 1 | LP-4.12 is now the D7 list; LP-4.30, LP-4.31, LP-4.35, LP-6.28. LP-4.9 kept: "the policy in force before it, which is the policy it replaces". |
| 19 | Key-close rulings missing | Rule 1 | LP-4.37, LP-4.39, and `valid-before` in LP-4.32. |
| 20 | `dsse` against "key material the policy names" | Rule 1 | LP-4.28: ed25519 over PAE by an `ssh-ed25519` key the signer bound in the namespace, open at `at`. |
| 21 | `none` "only for stores that predate signing" | Rule 1 | LP-4.29: governed and unsigned, not restricted to older stores. |
| 22 | Class naming | Rule 1 | LP-8.4: `SCHEMA`, `L001`–`L014`, `G001`–`G006`, `A003`/`A005`/`A006`, `[SIGNERS]`, `[EXPORT]`. |
| 23 | `L001` "has no allocation" | Rule 1 | LP-8.9: the latest version carries no `allocation`. |
| 24 | Derived state | Rule 1, rule 2 | LP-8.33: the seven dispositions, implemented. The draft's states are kept, not implemented, except "needs re-acceptance", which is implemented (LP-4.13). |
| 25 | SC-3.7 batch rows | Not settled | The prompt allows three kinds of change to the server-client protocol, and this is none of them. The ledger protocol now carries the full batch file (§10.1, LP-10.1 to LP-10.5), so SC-3.7 describes less than the text it depends on. Question 4. |
| 26 | SC-2.5 against `allowed_signers` | Ruling 26 | SC-2.5 and step 5 of §3.3 replaced (§9 below). |

**Further disagreements found while merging.** None was in the setup close-out.

| # | Disagreement | Settled by | Resulting text |
| --- | --- | --- | --- |
| 27 | Draft LP-8.2: "a store with a file-gate finding has no graph-stage result". The code runs the graph stage every time (`verify::verify`, `ledger-core/src/verify/mod.rs:196`); the format is silent. | Rule 1, from the code | LP-8.2: both stages always run and both report. |
| 28 | Draft LP-8.1: "the file gate checks one file at a time". `L005`, `L008`, `L014` and `L009` read across files or git. | Rule 1 | LP-8.1 rewritten. |
| 29 | Draft §1: exit codes "not protocol". Format §5.3 fixes them. | Rule 1 | §1 and LP-8.18: exit statuses are protocol for a verifier that reports through one. |
| 30 | Draft lexical forms: set id `[a-z0-9-]+`. Format §3.5 and `set::validate_id` allow dots. | Rule 1 | §3.3 table. |
| 31 | Format §8.1's triple list omits authority records; the code exports policies, bindings, covering grants and their records, and roles (`graph/export.rs` `select`). §3.10.8 already implies them. | Rule 6, from the code | LP-9.11 lists them. |
| 32 | The format says a finding of `L011` names three things; the draft says a finding is class and subject only. | Both kept | LP-4.39 keeps the format's text; CF-8 compares class and subject, so the vectors cannot test it. |
| 33 | LP-3.8 (ruling 16: a namespace is reached only through a pin) against format §3.1 (repositories may reference namespaces they do not own). | Not settled: ruling against implemented format | Both stated (LP-3.8 not implemented, with a note; LP-3.18). Question 6. |
| 34 | Format §8: the graph stage is outside the import surface. Draft: a verifier conforms by reporting the same findings, and rulings 10 and 13 put `A` classes and authority in the protocol. | Not settled | Both stated (LP-8.6; §1). Question 5. |
| 35 | Basis-loss: the format says it is not a gate class; ruling 9 says it follows the `L012` pattern with a deadline, and the draft listed "moved foundation" among new classes. | Not settled, as the prompt asks | LP-7.23, LP-7.26 (implemented) and LP-7.14, LP-7.15 (not implemented) both stated; §7.7 says the point is open. Question 3. |
| 36 | LP-4.18 step 2c strips `\v`; the code does not. | Not settled: no meaning change allowed | Text kept verbatim, with a note. Question 7. |

## 4. Traceability

A read-only survey of `ledger-core` and `ledger-cli` built this table; nothing in it was run. "read" means the code was read and plainly does this; "inferred" means the row rests on names and comments. "**run**" marks the one row I established by running code, and "**read, confirmed**" the rows I read again myself. Core test paths are relative to `ledger-core/src/`; CLI tests start with `ledger-cli/`.

| Requirement | File | Symbol | Test | How established |
|---|---|---|---|---|
| LP-3.1 | ledger-core/src/store.rs | store::load | ledger-cli/tests/graph.rs::the_rebuild_is_byte_identical_after_deleting_the_index | inferred |
| LP-3.2 | ledger-core/src/landing.rs | Landing::compute | none found | inferred |
| LP-3.3 | ledger-core/src/id.rs | DecisionId::from_str, prefixed_ulid_id! | id_tests.rs::change_set_and_acceptance_ids_carry_their_own_schemes | read — differs: `GrantScope::from_str` refuses dots in a `set:` scope, though set ids allow them |
| LP-3.4 | ledger-core/src/canon.rs | canon::scalar_fields | canon_tests.rs::key_and_exported_are_omitted_when_absent_and_exported_hashes_as_a_string | read — differs: no float check; a YAML float in a string field is read as text |
| LP-3.5 | ledger-core/src/verify/history.rs | history::findings | ledger-cli/tests/revisit_format.rs::a_landed_declaration_may_be_raised_but_an_entity_beside_it_may_not_change | read |
| LP-3.6 | ledger-core/src/identity.rs; graph/turtle.rs | Identity::from_str, turtle::mailto | graph/tests.rs::the_emission_carries_prov_attribution_and_open_basis_tokens | read |
| LP-3.7 | ledger-core/src/verify/disposition.rs; verify/authority.rs | disposition::model_acceptor, authority::model_actors | ledger-cli/tests/verbs.rs::a_model_identity_can_author_but_never_sign | read |
| LP-3.8 | — | — | — | not implemented |
| LP-3.9 | ledger-core/src/key.rs | DecisionKey::from_str | key.rs::sixty_four_characters_is_the_limit | read |
| LP-3.10 | ledger-core/src/store.rs | store::yaml_files | store_tests.rs::both_yaml_extensions_are_read | read |
| LP-3.11 | ledger-core/src/author/mod.rs; verify/history.rs | Author::append, history::log_file | ledger-cli/tests/immutability.rs::a_deleted_revocation_fails_l007 | read |
| LP-3.12 | ledger-core/src/store.rs | store::take_log, take_set, structure::role_faults | store_tests.rs::a_file_whose_name_disagrees_with_its_id_is_a_fault | read |
| LP-3.13 | ledger-core/src/init.rs | init::plan_init, apply_init | init.rs::a_fresh_repo_needs_both_directories_plus_the_ignore_line | read |
| LP-3.14 | ledger-core/src/store.rs | store::check_format, format::is_supported | store_tests.rs::an_unknown_format_is_named_rather_than_assumed | read |
| LP-3.15 | ledger-core/src/store.rs; format.rs | store::format_faults, format::needed_for, Revocation::shape_faults | store_tests.rs::a_legacy_revocation_in_a_file_declaring_format_6_or_above_is_a_fault | read |
| LP-3.16 | ledger-core/src/landed.rs | landed::entities | ledger-cli/tests/revisit_format.rs::a_landed_declaration_may_be_raised_but_an_entity_beside_it_may_not_change | read — differs: `format:` is never compared across history, so a lowering that still meets the need, or a raise past it, is not flagged |
| LP-3.17 | ledger-core/src/id.rs | id::validate_ulid | id_tests.rs::a_first_character_above_seven_overflows_the_timestamp | **read, confirmed** |
| LP-3.18 | ledger-core/src/id.rs | id::validate_namespace | none found | inferred |
| LP-3.19 | ledger-core/src/author/version_ops.rs | Author::supersede | author/sign_tests.rs::supersede_builds_a_walkable_chain_and_refuses_forking_it | inferred |
| LP-3.20 | ledger-core/src/changeset.rs | DecisionRecord (deny_unknown_fields) | changeset.rs::a_decision_record_carries_no_second_spelling_of_its_namespace | read |
| LP-3.21 | ledger-core/src/identity.rs | Identity::from_str | identity_tests.rs::an_address_needs_a_local_part_plus_a_domain | **read, confirmed** |
| LP-3.22 | ledger-core/src/identity.rs | Identity::model_or_bot_reason, model_token | identity_tests.rs::model_tokens_match_whole_tokens_never_substrings | **read, confirmed** |
| LP-3.23 | ledger-core/src/identity.rs | Identity::model_or_bot_reason | none found | inferred |
| LP-3.24 | ledger-core/src/authority/payload.rs | payload::stamp | authority/payload_tests.rs::a_policy_always_hashes_its_at | **read, confirmed** |
| LP-3.25 | ledger-core/src/verify/keys.rs | keys::key_changed | verify/keys_tests.rs::l013_judges_the_merged_from_edge_too | read |
| LP-3.26 | ledger-core/src/verify/keys.rs | keys::key_collision | verify/keys_tests.rs::a_superseded_decision_frees_its_key_for_the_successor | read |
| LP-3.27 | ledger-core/src/verify/mod.rs | verify::verify | ledger-cli/tests/gate.rs::l014_two_live_decisions_of_one_namespace_sharing_a_key | read |
| LP-3.28 | ledger-core/src/store.rs | store::version_field_rules | format.rs::a_change_set_needs_the_key_format_only_when_it_names_a_key_or_exports | read |
| LP-3.29 | ledger-core/src/canon.rs | canon::scalar_fields | canon_tests.rs::key_and_exported_are_omitted_when_absent_and_exported_hashes_as_a_string | **read, confirmed** |
| LP-4.1 | ledger-core/src/hash.rs | hash::version_hash, domain_hash | canon_tests.rs::the_pinned_conformance_vector_holds | **read, confirmed** |
| LP-4.2 | ledger-core/src/canon.rs | canon::put | canon_tests.rs::absent_null_and_empty_spellings_hash_alike | read |
| LP-4.3 | ledger-core/src/canon.rs | CANONICAL_FORM | canon_tests.rs::the_domain_prefix_separates_this_form_from_any_other | **read, confirmed** |
| LP-4.4 | ledger-core/src/canon.rs | canon::put_set | canon_tests.rs::formatting_only_changes_leave_the_hash_unchanged | **read, confirmed** |
| LP-4.5 | ledger-core/src/hash.rs | hash::signed_bytes | hash.rs::the_signed_bytes_are_prefix_newline_canonical | **read, confirmed** |
| LP-4.6 | ledger-core/src/signing/check.rs | check::governing | ledger-cli/tests/signing.rs::none_is_valid_only_where_policy_lists_it | read |
| LP-4.7 | ledger-core/src/signing/mod.rs; signing/check.rs | Sidecar::parse_name, check::check | ledger-cli/tests/signing.rs::a_missing_or_tampered_signature_fails_l011 | read — differs: a sidecar on an act no policy governs is never verified |
| LP-4.8 | ledger-core/src/acceptance.rs | Acceptance::schema_faults | acceptance_tests.rs::a_non_empty_inline_signature_is_a_schema_fault_in_every_format | read |
| LP-4.9 | ledger-core/src/signing/check.rs | check::governing | ledger-cli/tests/trust.rs::a_policy_change_is_signed_under_the_policy_it_replaces | read |
| LP-4.10 | ledger-core/src/authority/signers.rs | signers::check, derive_from | ledger-cli/tests/authority.rs::key_bindings_derive_allowed_signers_and_verify_holds_it | **read, confirmed** |
| LP-4.11 | ledger-core/src/verify/history.rs | history::log_file | ledger-cli/tests/immutability.rs::a_deleted_key_close_fails_l007 | read |
| LP-4.12 | ledger-core/src/authority/filing.rs | filing::may_file | authority/filing_tests.rs::a_further_add_and_a_rotate_are_the_principals_own | read — differs: a rotate verifies against any live trusted key of the principal, not only the key it closes |
| LP-4.13 | ledger-core/src/signing/check.rs; signing/review.rs | check::verify_one, review::review_closed | ledger-cli/tests/signing.rs::a_key_closed_after_the_acceptance_is_a_review_item_then_l012_past_the_deadline | read |
| LP-4.14 | ledger-core/src/signing/review.rs | review::review_closed | ledger-cli/tests/inbox.rs::an_acceptance_under_a_since_closed_key_is_affirmed_in_the_same_batch | **read, confirmed** |
| LP-4.15 | ledger-core/src/canon.rs; format.rs | CANONICAL_FORM, SUPPORTED_FORMATS | ledger-cli/tests/digests.rs::every_filed_digest_is_unchanged_by_the_format_5_fields | read |
| LP-4.16 | ledger-core/src/canon.rs; version.rs | canon::scalar_fields, VersionRaw | canon_tests.rs::the_mutation_table_covers_the_whole_canonical_form | **read, confirmed** |
| LP-4.17 | ledger-core/src/canon.rs | canon::scalar_fields | canon_tests.rs::the_tolerance_inputs_are_hashed_rather_than_the_resolved_tier | read |
| LP-4.18 | ledger-core/src/canon.rs | canon::norm, canonical_json | canon_tests.rs::the_canonical_form_is_compact_key_sorted_json | **run** — differs: step 2c leaves U+000B; step 7's float is not refused (LP-3.4) |
| LP-4.19 | ledger-core/src/hash.rs | hash::version_hash | canon_tests.rs::the_domain_prefix_separates_this_form_from_any_other | read |
| LP-4.20 | ledger-core/src/hash.rs | VersionHash::short | hash.rs::a_hash_round_trips_and_exposes_a_short_form | **read, confirmed** |
| LP-4.21 | ledger-core/src/canon_tests.rs | VECTOR, VECTOR_JSON | canon_tests.rs::the_pinned_conformance_vector_holds | read |
| LP-4.22 | ledger-core/src/authority/payload.rs | grant_map, revocation_map, binding_map, policy_map, acceptance_map | authority/payload_tests.rs::the_acceptance_payload_is_the_closed_list_and_omits_absent_fields | **read, confirmed** |
| LP-4.23 | ledger-core/src/authority/payload.rs | payload::revocation_hash | graph/revocation_tests.rs::the_revocation_payload_is_the_closed_four | **read, confirmed** |
| LP-4.24 | ledger-core/src/authority/payload.rs | payload::acceptance_hash | authority/payload_tests.rs::the_acceptance_payload_is_the_closed_list_and_omits_absent_fields | **read, confirmed** |
| LP-4.25 | ledger-core/src/signing/subject.rs | subject::subjects | ledger-cli/tests/export_verifier.rs::the_export_and_the_sidecars_alone_verify_and_rebuild_allowed_signers | read |
| LP-4.26 | ledger-core/src/signing/subject.rs | ssh::sig_namespace | ledger-cli/tests/signing.rs::an_accept_is_signed_into_a_sidecar_and_verifies | read |
| LP-4.27 | ledger-core/src/signing/ssh.rs; signing/check.rs | ssh::verify, check::ssh_key | ledger-cli/tests/signing.rs::an_accept_is_signed_into_a_sidecar_and_verifies | **read, confirmed** |
| LP-4.28 | ledger-core/src/signing/dsse.rs | dsse::verify, dsse::pae | signing/check_tests.rs::a_dsse_envelope_by_the_bound_key_verifies | **read, confirmed** |
| LP-4.29 | ledger-core/src/authority/structure.rs | structure::entry_faults | ledger-cli/tests/signing.rs::none_is_exclusive | read |
| LP-4.30 | ledger-core/src/signing/check.rs | check::check, check::judge | ledger-cli/tests/pre_policy_binding.rs::an_unsigned_binding_before_a_signed_first_policy_fails_l011_and_is_never_trusted | read |
| LP-4.31 | ledger-core/src/signing/check.rs | check::judge_first_policy | ledger-cli/tests/genesis_key.rs::a_later_namespaces_first_policy_is_signed_and_unsigned_it_is_l011 | read |
| LP-4.32 | ledger-core/src/authority/signers.rs | signers::derive_from, HEADER | ledger-cli/tests/authority.rs::key_bindings_derive_allowed_signers_and_verify_holds_it | **read, confirmed** |
| LP-4.33 | ledger-core/src/authority/signers.rs | signers::check | ledger-cli/tests/authority.rs::key_bindings_derive_allowed_signers_and_verify_holds_it | **read, confirmed** |
| LP-4.34 | ledger-core/src/signing/check.rs | check::trust_bindings | ledger-cli/tests/binding_accounting.rs::every_filed_binding_is_trusted_or_named_by_a_finding | read |
| LP-4.35 | ledger-core/src/signing/check.rs; authority/references.rs | trust_bindings, first_policy_of, binding_refs | ledger-cli/tests/pre_policy_binding.rs::a_binding_in_a_namespace_no_policy_governs_is_a_schema_fault_and_init_will_not_govern_it | read |
| LP-4.36 | ledger-core/src/signing/check.rs | check::trust_bindings | ledger-cli/tests/trust.rs::d7_each_binding_act_by_an_allowed_and_a_disallowed_signer | read |
| LP-4.37 | ledger-core/src/authority/key_close.rs | key_close::refusal | ledger-cli/tests/key_ownership.rs::a_hand_filed_binding_of_another_principals_key_is_a_schema_fault | read |
| LP-4.38 | ledger-core/src/author/genesis_key.rs | Author::bind_genesis_key, carry_genesis_key | ledger-cli/tests/genesis_key.rs::with_no_usable_key_init_refuses_and_names_what_is_missing | inferred |
| LP-4.39 | ledger-core/src/authority/key_close.rs; signing/check.rs | closes_among, check::verify_one | ledger-cli/tests/key_across_namespaces.rs::bound_in_a_then_b_and_closed_in_a_the_key_is_closed_in_b_and_vouches_in_no_third | read |
| LP-5.1 | ledger-core/src/graph/authority.rs | authority::emit_revocation | graph/revocation_tests.rs::the_acceptance_node_triple_set_is_fixed_across_a_revocation | read |
| LP-5.2 | ledger-core/src/authority/revocation.rs | Revocation, Revocable | acceptance_tests.rs::a_revocation_names_the_acceptance_it_reverses_with_a_reason | read |
| LP-5.3 | ledger-core/src/authority/payload.rs | acceptance_hash, revocation_hash | authority/payload_tests.rs::no_format_6_digest_moves | read |
| LP-5.4 | ledger-core/src/author/decision.rs | Author::add | ledger-cli/tests/verbs.rs::a_model_identity_can_author_but_never_sign | inferred |
| LP-5.5 | ledger-core/src/verify/keys.rs | keys::key_changed | verify/keys_tests.rs::giving_a_keyless_decision_a_key_is_a_new_version_not_a_finding | read |
| LP-5.6 | — | — | — | not implemented |
| LP-5.7 | ledger-core/src/version.rs; changeset.rs | serde(deny_unknown_fields) | version_tests.rs::an_unknown_key_is_rejected_rather_than_silently_dropped | read |
| LP-5.8 | ledger-core/src/tier.rs | Tolerance::new | tier_tests.rs::an_override_equal_to_the_floor_is_rejected_as_a_no_op | read |
| LP-5.9 | ledger-core/src/tier.rs; verify/disposition.rs | Tolerance::is_stranded_below, disposition::stranded | tier_tests.rs::raising_the_floor_strands_a_member_below_it | read |
| LP-5.10 | ledger-core/src/set.rs | DecisionSet | set.rs::a_set_file_carries_no_membership_list | read |
| LP-5.11 | ledger-core/src/changeset.rs | ChangeSet.created_by, DecisionRecord | graph/tests.rs::g003_a_version_whose_decision_was_never_introduced | read — differs: a decision identity object filed again in a second change-set is not refused |
| LP-5.12 | ledger-core/src/allocation.rs | Allocation::assemble | allocation_tests.rs::the_other_stores_obligations_are_parse_gate_faults | read |
| LP-5.13 | ledger-core/src/version.rs | DecisionVersion::assemble | version_tests.rs::a_version_with_no_allocation_assembles_and_stays_unallocated | read |
| LP-5.14 | ledger-core/src/store.rs; verify/view.rs | version_field_rules, View::derive_latest | graph/tests.rs::a_reconciliation_closes_the_fork_it_names | read |
| LP-5.15 | ledger-core/src/acceptance.rs | AcceptanceScope::from_str | acceptance_tests.rs::a_class_scope_carries_a_real_discharge_pointer | read |
| LP-5.16 | ledger-core/src/discharge.rs | DischargeRef::from_str | discharge_tests.rs::an_unknown_scheme_names_the_schemes_that_exist | read |
| LP-5.17 | ledger-core/src/store.rs | version_field_rules | format.rs::the_contract_format_is_supported | read |
| LP-5.18 | ledger-core/src/discharge.rs | Stage | discharge_tests.rs::stages_order_by_the_ground_they_expose | read |
| LP-5.19 | ledger-core/src/authority/structure.rs | structure::role_faults | authority/role.rs::a_role_file_parses_and_refuses_unknown_keys | read |
| LP-5.20 | ledger-core/src/store.rs | store::entity_rules | ledger-cli/tests/trust.rs::a_policy_change_in_a_format_6_file_is_a_schema_fault | read |
| LP-5.21 | ledger-core/src/authority/revocation.rs; verify/acts.rs | Revocation::shape_faults, acts::revocation_verdict | authority/revocation.rs::each_format_takes_only_its_own_shape | read |
| LP-6.1 | ledger-core/src/authority/role.rs | Capability::ALL | authority/role.rs::the_capability_vocabulary_is_closed_at_seven | read (the `trust-source` part is not implemented) |
| LP-6.2 | ledger-core/src/authority/check.rs | check::authorize, check::judge | authority/check_tests.rs::a_live_accepted_available_primary_grant_authorizes | read |
| LP-6.3 | ledger-core/src/authority/view.rs | Authority::is_accepted, is_live | ledger-cli/tests/authority.rs::a_grant_is_live_only_once_its_holder_accepts_it | read |
| LP-6.4 | ledger-core/src/authority/structure.rs | structure::grant_faults | verify/authority_tests.rs::a_misshapen_genesis_and_a_limited_primary_are_schema_faults | read |
| LP-6.5 | ledger-core/src/authority/structure.rs; graph/shapes.rs | grant_faults, A005 shape | verify/authority_tests.rs::a005_two_live_genesis_grants | read |
| LP-6.6 | ledger-core/src/graph/shapes.rs | GRANT_COLLISION | verify/authority_tests.rs::a003_two_live_grants_at_one_place_in_the_order | read |
| LP-6.7 to LP-6.12 | — | — | — | not implemented (LP-6.10 to LP-6.12, LP-6.7 to LP-6.9) |
| LP-6.13 | ledger-core/src/verify/mod.rs | verify::verify | ledger-cli/tests/signing.rs::a_signed_acceptance_by_a_bound_principal_with_no_grant_fails_a006 | inferred |
| LP-6.14 | ledger-cli/src/commands/terminal.rs; ledger-core/src/author/sign_ops.rs | terminal classification, Author::refuse_key | ledger-cli/tests/signing.rs::an_agent_held_software_key_and_a_software_key_under_sk_are_refused | inferred |
| LP-6.15 | ledger-core/src/authority/view.rs | is_live, is_available, Authority::policy | authority/check_tests.rs::an_availability_ends_an_open_interval_early | read |
| LP-6.16 | ledger-core/src/authority/check.rs | check::candidates, covers, outranking | authority/check_tests.rs::a_set_fallback_waits_on_an_available_star_primary_of_its_role | read |
| LP-6.17 | ledger-core/src/authority/references.rs | references::policy_refs | ledger-cli/tests/authority.rs::the_accept_role_is_never_the_genesis_role | read — differs: the verifier checks only that `accept_role` may `accept-decision`; "never the genesis role" is enforced at write only |
| LP-6.18 | ledger-core/src/verify/acts.rs | acts::unauthorised | ledger-cli/tests/authority.rs::a_namespace_without_policy_is_not_role_checked | read |
| LP-6.19 | ledger-core/src/acceptance.rs; authority/payload.rs | `under` fields | authority/payload_tests.rs::under_moves_the_digest_of_every_payload_that_names_it | read |
| LP-6.20 | ledger-core/src/author/authority_ops.rs; availability_ops.rs | Author::declare_role, Author::unavailable | ledger-cli/tests/authority.rs::policy_changes_and_role_declarations_are_the_genesis_holders | inferred |
| LP-6.21 | ledger-core/src/authority/choice.rs; author/acceptance_ops.rs | choice::choose, decision_authority | authority/choice_tests.rs::on_accept_two_grants_need_as_and_the_narrowest_scope_then_lowest_rank_wins | read |
| LP-6.22 | ledger-core/src/authority/choice.rs | choice::choose | authority/choice_tests.rs::roles_whose_may_sets_are_not_nested_are_not_ordered | read |
| LP-6.23 | ledger-core/src/authority/choice.rs | choice::grantor | authority/choice_tests.rs::below_the_genesis_a_grantor_holding_no_grant_in_the_role_is_refused | read |
| LP-6.24 | ledger-core/src/store.rs | store::entity_rules | ledger-cli/tests/trust.rs::a_policy_change_in_a_format_6_file_is_a_schema_fault | read |
| LP-6.25 | ledger-core/src/verify/acts.rs | acts::unauthorised | ledger-cli/tests/authority.rs::only_a_holder_of_the_accept_role_accepts_in_a_governed_namespace | read |
| LP-6.26 | — | — | — | not implemented |
| LP-6.27 | ledger-core/src/verify/acts.rs | acts::unauthorised, acts::judge | ledger-cli/tests/signing.rs::a_signed_acceptance_by_a_bound_principal_with_no_grant_fails_a006 | read |
| LP-6.28 | ledger-core/src/verify/acts.rs | acts::policy_verdict | ledger-cli/tests/policy_authors.rs::a_policy_change_under_a_non_genesis_grant_role_over_star_fails_a006 | read |
| LP-6.29 | ledger-core/src/verify/acts.rs | acts::revocation_verdict | ledger-cli/tests/legacy_revocation.rs::an_old_style_revocation_backdated_before_the_policy_but_landed_after_fails_a006 | read |
| LP-6.30 | ledger-core/src/authority/view.rs | Authority::as_of, role_landing | ledger-cli/tests/role_position.rs::an_act_that_landed_before_the_role_its_grant_names_fails_a006 | read |
| LP-7.1 to LP-7.18 | — | — | — | not implemented |
| LP-7.19 | ledger-core/src/version.rs | BasisRef::from_str | version_tests.rs::a_basis_pointer_is_a_single_token | **read, confirmed** |
| LP-7.20 | ledger-core/src/canon.rs | canon::put_set | canon_tests.rs::formatting_only_changes_leave_the_hash_unchanged | **read, confirmed** |
| LP-7.21 | ledger-core/src/version.rs | BasisRef | version_tests.rs::a_basis_pointer_is_a_single_token | read (convergence part not implemented) |
| LP-7.22 | ledger-core/src/revisit.rs | RevisitRef | revisit.rs::the_type_is_distinct_from_a_basis_pointer | read |
| LP-7.23 | outside `ledger-*` (the reports live in `ddd-core`) | — | none found | inferred |
| LP-7.24 | ledger-core/src/store.rs | version_field_rules | store_tests.rs::a_lower_format_change_set_carrying_revisit_if_is_a_fault | read |
| LP-7.25 | ledger-core/src/canon.rs | canon::canonical_json | canon_tests.rs::one_token_hashes_differently_as_ground_and_as_a_reopen_edge | **read, confirmed** |
| LP-7.26 | ledger-core/src/finding.rs | ALL_CLASSES | finding.rs::there_are_exactly_fourteen_semantic_classes_plus_the_parse_gate | inferred |
| LP-8.1 | ledger-core/src/verify/mod.rs | verify::verify, Report | graph/tests.rs::graph_findings_fail_verify_as_a_distinct_stage | read |
| LP-8.2 | ledger-core/src/verify/mod.rs | verify::verify | graph/tests.rs::graph_findings_fail_verify_as_a_distinct_stage | **read, confirmed** |
| LP-8.3 | ledger-core/src/verify/view.rs | View::build | verify/mod_tests.rs::latest_is_the_dag_tip_whichever_file_order_the_ulids_impose | inferred |
| LP-8.4 | ledger-core/src/finding.rs; graph/mod.rs | VerifyClass, GraphClass | graph/tests.rs::the_graph_class_set_is_closed | read |
| LP-8.5 | ledger-core/src/verify/keys.rs | key_changed, key_collision | ledger-cli/tests/gate.rs::l013_a_key_renamed_along_the_version_chain | read |
| LP-8.6 | ledger-core/src/verify/mod.rs | verify::graph_stage | none found | inferred |
| LP-8.7 | ledger-core/src/finding.rs | ALL_CLASSES | finding.rs::there_are_exactly_fourteen_semantic_classes_plus_the_parse_gate | **read, confirmed** |
| LP-8.8 | ledger-core/src/store.rs; authority/references.rs | Finding::schema sites, references::duplicate_ids | verify/mod_tests.rs::schema_findings_from_the_store_reach_the_report | read — differs: duplicate `acc:` ids and duplicate decision identity objects are not checked |
| LP-8.9 | ledger-core/src/verify/disposition.rs; verify/integrity.rs | the per-class checks | verify/mod_tests.rs::every_class_the_enum_declares_is_reachable_here | read — differs: `L006` checks an escape's `accepted_by` on latest versions only |
| LP-8.10 | ledger-core/src/verify/view.rs | View::latest_versions | verify/mod_tests.rs::only_the_latest_version_of_a_decision_is_judged | read |
| LP-8.11 | ledger-core/src/verify/view.rs | View::derive_latest | verify/mod_tests.rs::a_forked_chain_is_recorded_and_the_representative_is_order_independent | **read, confirmed** — differs: a forked decision gets a representative "latest" (the first tip in hash order) that the file-gate classes judge; `G004` still fires |
| LP-8.12 | ledger-core/src/verify/disposition.rs | disposition::expired | verify/mod_tests.rs::a_revoked_acceptance_stops_being_judged_for_expiry | **read, confirmed** |
| LP-8.13 | ledger-core/src/verify/integrity.rs | integrity::dangling_acceptance | verify/mod_tests.rs::l008_an_acceptance_signing_a_hash_nobody_filed | read |
| LP-8.14 | ledger-core/src/verify/integrity.rs | integrity::blame_consistency | ledger-cli/tests/gate.rs::an_uncommitted_acceptance_is_skipped_with_the_skip_reported | read |
| LP-8.15 | ledger-core/src/verify/mod.rs | verify::awaiting | verify/mod_tests.rs::allocated_awaiting_acceptance_is_status_never_a_failure | read |
| LP-8.16 | ledger-core/src/authority/references.rs; verify/authority.rs | structure::entry_faults, references::faults, model_actors, hash_mismatches | verify/authority_tests.rs::l007_catches_a_grant_edited_after_it_was_sealed | read |
| LP-8.17 | ledger-core/src/finding.rs | VerifyClass::in_gate | verify/mod_tests.rs::the_readiness_gate_leaves_out_the_two_release_dispositions | **read, confirmed** |
| LP-8.18 | ledger-cli/src/commands/mod.rs | EXIT_OK, EXIT_VIOLATIONS, EXIT_ERROR | ledger-cli/tests/cli.rs::a_gate_that_could_not_run_exits_two_not_one | read |
| LP-8.19 | ledger-core/src/graph/shapes.rs; verify/acts.rs | SHAPES, acts::unauthorised | graph/tests.rs::g001_a_supersession_edge_to_nowhere | read |
| LP-8.20 | ledger-core/src/graph/shapes.rs | G004 shape | graph/tests.rs::a_reconciliation_closes_the_fork_it_names | read |
| LP-8.21 | ledger-core/src/graph/shapes.rs | G005 shape | graph/tests.rs::a_withdrawn_supersession_claim_clears_g005 | read |
| LP-8.22 | ledger-core/src/graph/mod.rs | ALL_GRAPH_CLASSES | graph/tests.rs::the_graph_class_set_is_closed | read |
| LP-8.23 | ledger-core/src/graph/shapes.rs | GRANT_COLLISION, A005 shape | verify/authority_tests.rs::a003_two_live_grants_at_one_place_in_the_order | read |
| LP-8.24 | ledger-core/src/landing.rs | Landing::compute | landing_tests.rs::a_file_lands_at_the_first_first_parent_commit_containing_it | read |
| LP-8.25 | ledger-core/src/landed.rs | landed::entities | landed_tests.rs::every_item_is_keyed_by_its_list_and_identity | read — differs: `parents` is keyed as separate list items, not as part of the header entity |
| LP-8.26 | ledger-core/src/landing.rs | Position::before, not_after | landing_tests.rs::before_needs_landing_no_later_and_an_earlier_at | read |
| LP-8.27 | ledger-core/src/authority/view.rs | role_landing | ledger-cli/tests/role_position.rs::a_role_and_an_act_landed_in_the_same_commit_pass_whatever_the_role_is_dated | read |
| LP-8.28 | ledger-core/src/authority/view.rs | Authority::as_of | verify/order_regressions_tests.rs::a_policy_dated_after_a_backdated_act_still_governs_it | read |
| LP-8.29 | ledger-core/src/landing.rs; revision.rs | Landing::compute, default_base | ledger-cli/tests/signing.rs::a_branch_verified_against_its_base_agrees_with_the_merge_ref | read |
| LP-8.30 | ledger-core/src/verify/history.rs | history::findings | ledger-cli/tests/trust.rs::removing_a_policy_fails_l007_because_opting_in_is_one_way | read |
| LP-8.31 | ledger-core/src/verify/mod.rs | notices | ledger-cli/tests/genesis_key.rs::verify_names_a_none_namespace_in_a_notice_and_passes | read |
| LP-8.32 | ledger-core/src/blame.rs | blame::introducing_author | ledger-cli/tests/gate.rs::l009_an_acceptance_committed_by_somebody_else | read |
| LP-8.33 | ledger-core/src/coverage.rs; verify/state.rs | coverage::coverage | ledger-cli/tests/graph.rs::coverage_distinguishes_all_seven_states_on_the_fixture | read |
| LP-9.1 | ledger-core/src/graph/export.rs | export::export, export::check | graph/export_tests.rs::the_export_carries_exactly_the_index_triples | **read, confirmed** |
| LP-9.2 | ledger-core/src/graph/export.rs | CITATIONS_SUFFIX | graph/export_tests.rs::fresh_exports_pass_and_a_citation_projection_is_ignored | **read, confirmed** |
| LP-9.3 | ledger-core/src/graph/authority.rs | emit_signatures | ledger-cli/tests/export_verifier.rs::bytes_rebuilt_from_the_export_equal_the_signed_bytes_over_every_fixture | **read, confirmed** |
| LP-9.4 | ledger-core/src/graph/authority.rs | emit_revocation | graph/revocation_tests.rs::the_acceptance_node_triple_set_is_fixed_across_a_revocation | read |
| LP-9.5, LP-9.6 | — | — | — | not implemented |
| LP-9.7 | ledger-core/src/inbox/list.rs | inbox::list::list | none found | inferred |
| LP-9.8 | ledger-core/src/inbox/list.rs | list::item | ledger-cli/tests/inbox.rs::three_repositories_twelve_branches_one_sitting_every_branch_green | inferred |
| LP-9.9 | ledger-core/src/inbox/list.rs | PROPOSED query | ledger-cli/tests/inbox.rs::accepting_one_of_a_branchs_two_proposed_decisions_pushes_it_and_the_other_stays_listed | inferred |
| LP-9.10 | ledger-core/src/inbox/list.rs | inbox::list::list | none found | inferred |
| LP-9.11 | ledger-core/src/graph/export.rs | export::select, restrict, Reach::of | graph/export_tests.rs::a_namespace_export_carries_its_own_authority_records_only | **read, confirmed** |
| LP-9.12 | ledger-core/src/graph/ntriples.rs; graph/turtle.rs | ntriples::emit, turtle::literal | graph/export_tests.rs::literals_are_escaped_the_rdf_1_2_canonical_way | **read, confirmed** |
| LP-9.13 | ledger-core/src/graph/export.rs | export::check | graph/export_tests.rs::a_spoken_namespace_without_an_export_and_an_unspoken_export_both_fail | **read, confirmed** |
| LP-9.14 | ledger-cli/tests/common/export_only.rs | export_only::rebuild | ledger-cli/tests/export_verifier.rs::the_export_and_the_sidecars_alone_verify_and_rebuild_allowed_signers | read — differs: exists only as a test helper; its `allowed_signers` takes `valid-before` from the binding's own close rather than the key's earliest close, and does not filter to trusted bindings |
| LP-9.15 | ledger-cli/tests/common/export_only.rs | export_only::verify_all | ledger-cli/tests/export_verifier.rs::the_export_and_the_sidecars_alone_verify_and_rebuild_allowed_signers | inferred |
| LP-9.16 | ledger-core/src/graph/turtle.rs | turtle::emit_edges | graph/tests.rs::the_emission_carries_prov_attribution_and_open_basis_tokens | **read, confirmed** |
| LP-9.17 | ledger-core/src/graph/authority.rs | emit_revocation | graph/revocation_tests.rs::a_legacy_revocation_gets_a_node_minted_from_the_acceptance_it_revokes | read |
| LP-9.18 | ledger-core/src/graph/index.rs | index::write_index, turtle::emit | ledger-cli/tests/graph.rs::the_rebuild_is_byte_identical_after_deleting_the_index | read |
| LP-9.19 | ledger-core/src/graph/turtle.rs | emit_version, emit_acceptance | graph/tests.rs::the_emission_carries_prov_attribution_and_open_basis_tokens | **read, confirmed** |
| LP-10.1 | ledger-core/src/batch_file.rs | batch_file::check | batch_file_tests.rs::a_file_with_no_rows_or_a_duplicate_row_is_refused | read |
| LP-10.2 | ledger-core/src/batch_file.rs | batch_file::manifest | batch_file_tests.rs::the_manifest_covers_every_row_its_branch_and_its_grant | read |
| LP-10.3 | ledger-core/src/author/batch_accept.rs | Author::accept_batch | ledger-cli/tests/batch.rs::a_moved_hash_refuses | read |
| LP-10.4 | ledger-core/src/author/group_accept.rs | Author::sign | ledger-cli/tests/batch.rs::a_batch_signs_every_row_one_sidecar_each_under_one_confirmation | read |
| LP-10.5 | ledger-core/src/author/batch_accept.rs | Author::admit | ledger-cli/tests/batch.rs::another_principals_batch_is_refused | read |

**Where the code differs from the text** (13 rows above). These are findings: the text is the format document's or the code's own documented intent, and I changed neither. Only LP-4.18 (run) and LP-8.11 (read, confirmed) were checked by me. The rest are the survey's reading, and each needs a check before anything is done about it.

- LP-4.18 / LP-3.4: `\v` not stripped; a YAML float in a string field is read as text, not refused.
- LP-8.11: a forked decision gets a representative "latest". The text says it has none, and that no ordering heuristic may pick one.
- LP-3.3: a `set:` grant scope refuses dots, though set ids allow them.
- LP-3.16: `format:` changes are not compared across history.
- LP-4.7: a sidecar on an act no policy governs is not verified, though the text says a present sidecar always has to verify.
- LP-4.12: a `rotate` is accepted when signed by any live key of the principal, not only the key it closes.
- LP-5.11 / LP-8.8: a duplicate `acc:` id, or a decision identity object filed twice, is not a schema fault.
- LP-6.17: "the accept role is never the genesis role" is enforced at write only.
- LP-8.9: `L006` checks an escape's `accepted_by` on latest versions only.
- LP-8.25: `parents` is keyed as separate list items, not with the header.
- LP-9.14: the export-only verifier exists only as a test helper, and it derives `valid-before` per binding.

The symbols the format document named and the protocol no longer does: `ledger_core::hash::signed_bytes` and `domain_hash` (LP-4.5), `ledger-cli/tests/digests.rs` (LP-4.16), `RevisitRef`/`BasisRef` (LP-7.22), `format::needed_for` (LP-3.15), `Act::SetPolicy` and `authorize_named` (LP-6.28, LP-8.23), and the §7 table: `DecisionSet::validate_id`, `version_hash`, `Allocation::assemble`, `Tolerance::new`, `Identity` with `model_or_bot_reason`, `verify::view::View`, `blame::introducing_author`, `canon::canonical_json`.

## 5. Implementation-defined behaviour

| Behaviour | Source | Risk for a second implementation |
| --- | --- | --- |
| Whitespace stripped in LP-4.18 step 2c: space, `\t`, `\n`, `\f`, `\r`, but **not** `\v` | `canon::norm`, `char::is_ascii_whitespace` (established by running it) | A string with a leading or trailing U+000B hashes differently under the text and under the code. No stored digest is known to contain one. |
| Unicode version of NFC | `unicode-normalization = "0.1"` in `ledger-core/Cargo.toml`; no `Cargo.lock` is committed | Two builds can resolve different crate versions with different Unicode tables. A newly assigned character could normalise differently. The text names no Unicode version. |
| Lowercasing an identity | `Identity::from_str`, `str::to_lowercase` (full Unicode) | The text says "lowercase". Non-ASCII local parts or domains (for example `İ`) lowercase differently under ASCII-only and Unicode rules, which moves an escape's `accepted_by` and an acceptance digest. |
| Trimming at parse before canonicalisation | `Identity::from_str` and `BasisRef::from_str` use Unicode `trim`; `canon::norm` then strips ASCII whitespace | A token or identity with leading U+00A0 is trimmed by the reference parser and not by LP-4.18. |
| The YAML subset | `serde_yaml` | Not specified (§6 below). Duplicate keys, anchors, tags, YAML 1.1 scalars (`yes`, `on`), and date-like scalars could be read differently by another parser. |
| "Today" for `L003` and `L012` | `ledger-cli/src/commands/verify.rs` `parse_today`: the UTC calendar date, overridable | Not stated in the text. A verifier using local time can differ for a day. |
| `L003` and `L012` boundaries | `Acceptance::is_expired` (`expires_at < today`); `signing/review.rs` (deadline = the close's UTC date + N days; fails when `today > deadline`) | The text says "before today" and "has passed" without the arithmetic. |
| SSHSIG hash algorithm and sidecar encoding | `signing/ssh.rs` runs `ssh-keygen -Y sign` with its default, and stores its armored output | The text does not fix the algorithm (OpenSSH defaults to SHA-512; SSHSIG allows SHA-256) or require armoring. Inferred from OpenSSH's behaviour, not run here. |
| Verification time and the OpenSSH version | `-Overify-time=<at>` needs OpenSSH 8.9 or later (`ssh::preflight`) | A verifier must evaluate key windows at `at`, in second precision UTC. |
| DSSE details | `signing/dsse.rs`: `keyid` is ignored; base64 padding optional and surrounding whitespace trimmed; unknown envelope fields ignored; any of the signer's bound ed25519 keys may verify | The text says none of this. A stricter verifier would refuse envelopes the reference accepts. |
| `allowed_signers` header bytes | `authority/signers.rs` `HEADER` | Filled into LP-4.32 from the code. The bytes name the reference tool's commands, and a second implementation must reproduce them exactly. |
| Integer rendering | `authority/payload.rs` `policy_map`: `reaccept_within_days` (`u32`) as `to_string()`; export `xsd:integer` | Filled into LP-3.4 and §4.6 from the code. Leading zeros and the range (0 to 2³²−1) are implicit. |
| Export selection of authority records | `graph/export.rs` `select`, `Reach` | Filled into LP-9.11 from the code. A `pattern:` scope never reaches a namespace's export. |
| Landing computation | `landing.rs` over git first-parent history; default base `origin/HEAD` | Stated in LP-8.24 and LP-8.29. How a verifier reads git (CLI, library) and what it does with a shallow clone is not stated. |
| `L009` | `blame::introducing_author` compares the introducing commit's author email | What happens when the author email does not parse as an identity is not stated. |
| Machine-readable report | `verify::Report` (`unsigned`, `genesis_unbound`, `export` omitted when not run) | Notes only. Not protocol. |
| Finding subjects | each check's `Finding::new(class, subject, …)` | CF-8 compares (class, subject) pairs, but the text does not say what the subject of each class is. |
| ULID generation | `mint::UlidMint` | Not specified, as before (§12.4). |
| JSON escaping hex case | `serde_json` writes `\u00XX` with lowercase hex | Matches RFC 8785. Inferred from the library's documented behaviour; I did not read its source or run it. |

## 6. Extraction markers left, with reasons

No literal **Extraction** marker remains; §12.1 of the protocol records how each was filled. Three points are only partly filled:

- **The file grammar as a restricted subset.** The format document never stated one: it relies on the canonical JSON keeping YAML out of the hash. I could not state which YAML constructs a writer may emit without inventing a rule. What would settle it: a principal ruling on the subset, or a test suite of YAML inputs with expected parses.
- **Each class's subject.** Needed for CF-8. The code chooses a subject per check (an acceptance id, a decision id, a file path, `sig/<file>`), but no document states it, and I did not survey every check. What would settle it: a table built from each `Finding::new` call, approved by the principal.
- **The SSHSIG hash algorithm.** Left to the signer (§5 above).


## 7. Test-case candidates

An inventory, not a proposal: no test case was written, and approving one is the principal's (CF-7). Established by reading the suites; the counts in the right-hand columns are the tests' own assertions.

**The §4.4 vector.** The input, canonical JSON and digest of protocol §4.5. `ledger-core/src/canon_tests.rs` pins the digest (`the_pinned_conformance_vector_holds`, `sha256:ac2a6802…37cc`) and the canonical JSON (`the_canonical_form_is_compact_key_sorted_json`). The literal exists only there and in the protocol.

**The vertical-tab case** (added 7 October, ruling 33). A version whose `statement` ends with U+000B keeps the character after LP-4.18 step 2c, so its canonical JSON carries `\u000b`. No fixture or test covers it today. A vector would pin the canonical JSON and the digest the reference implementation computes.

**Committed fixture stores** (`ledger-cli/tests/fixtures/`, 16 stores; the format document said eight). All `gate.rs` runs use `verify --today 2026-08-10 --no-blame`. Its helper `fails_only_with` asserts exit 1, the named class present, and none of `SCHEMA`, `L001`–`L010`, `L013`, `L014` beyond it. It does not exclude `L011`, `L012`, the graph classes or the derived-file stages.

| Fixture | Expected result | Usable as |
| --- | --- | --- |
| `pass` | exit 0, conformant | positive vector |
| `l001`–`l008`, `l010`, `l013` | fails with exactly that class (within the helper's list) | negative vector per class |
| `l007` | fails `L007`; its hash is stale by design | negative vector for `L007` |
| `l014` | fails `L014`, and the graph stage also reports `G006` | negative vector for `L014` with `G006` |
| `forked` | exit 1, `G004`; `L001`, `L007`, `SCHEMA`, `G001`–`G003` absent | negative vector for `G004` (partial isolation) |
| `two-clocks` | conformant; latest follows the parent DAG against ULID order | positive vector for LP-8.11 |
| `coverage` | the seven dispositions at `--today 2027-01-01` | derived-state vector for LP-8.33 |

15 of the 16 hold real stored hashes, held by `gate.rs::fixture_hashes_are_current` and `digests.rs::every_filed_digest_is_unchanged_by_the_format_5_fields`. `export_verifier.rs::bytes_rebuilt_from_the_export_equal_the_signed_bytes_over_every_fixture` rebuilds signed bytes from the export over all 16.

**Byte pins usable as vectors.**

| Test | Pins | Requirement |
| --- | --- | --- |
| `authority/payload_tests.rs::no_format_6_digest_moves` | grant `sha256:fe78d33b…`, genesis grant `sha256:72b02e65…`, binding `sha256:79c8cedb…`, revocation `sha256:60bf0b25…` | LP-4.22 |
| `authority/payload_tests.rs::a_policy_always_hashes_its_at` | `"at":"2026-10-04T00:00:00Z"` in the policy bytes | LP-4.22 |
| `authority/payload_tests.rs::the_acceptance_payload_is_the_closed_list_and_omits_absent_fields` | acceptance prefix line and key set | LP-4.22, LP-4.24 |
| `hash.rs::the_signed_bytes_are_prefix_newline_canonical` | `ledger.acceptance.v1\n{"a":"b"}` | LP-4.5 |
| `canon_tests.rs::key_and_exported_are_omitted_when_absent_and_exported_hashes_as_a_string` | `"exported":"true"` | LP-3.29 |
| `graph/ntriples.rs` tests, `graph/export_tests.rs` | full-IRI terms, typed literals, sort order, LF | LP-9.12 |

No test pins a complete `allowed_signers` file or a complete N-Triples export as a literal.

**Classes with no negative case** (no fixture and no store-building test asserting exactly that class):

- **No committed fixture at all:** `SCHEMA`, `L009`, `L011`, `L012`, `G001`, `G002`, `G003`, `G005`, `A003`, `A005`, `A006`, `[EXPORT]`, `[SIGNERS]`.
- **No test asserting exactly that class in any form:** `L011`, `L012`, `A003`, `A005`, `A006`, `[SIGNERS]`. Each appears only in tests that check the output includes it.
- **`SCHEMA`:** the only exact case injects the finding rather than parsing a file (`verify/mod_tests.rs::schema_findings_from_the_store_reach_the_report`).
- **`G001` to `G006`:** exact only among graph findings, in memory (`graph/tests.rs`), never with the file gate also checked.
- **The basis classes of protocol §8.9** have no implementation and no case.

## 8. Citations not updated

**`.ddd/` seams (not edited, as instructed).** Each should cite:

| Seam | Cites now | Should cite |
| --- | --- | --- |
| `.ddd/seams/seam-ledger-acceptor-identity.yaml` (line 9) | `docs/ledger-format-v1.md` §3.2 | `ledger/spec/ledger-protocol.md` §3.4 (LP-3.22, LP-3.23) |
| `.ddd/seams/seam-ledger-canonical-form.yaml` (lines 8–9) | `docs/ledger-format-migrations.md`; `docs/ledger-format-v1.md` §4.4 | protocol Appendix C; protocol §4.5 (LP-4.21) |
| `.ddd/seams/seam-ledger-disposition-states.yaml` (line 7) | `docs/ledger-format-v1.md` §8 | protocol §8.10 (LP-8.33) |
| `.ddd/seams/seam-ledger-graph-stage.yaml` (line 8) | `docs/ledger-format-v1.md` §8 | protocol §8.6 (LP-8.19, LP-8.22) |
| `.ddd/seams/seam-ledger-verify-classes.yaml` (line 7) | `docs/ledger-format-v1.md` §5.2 | protocol §8.3 (LP-8.9) |

`.ddd/render.html` line 604 renders the canonical-form seam and is generated by `ddd render`; it follows once the seam changes.

**Immutable or hashed records (cannot be edited).**

- `.decisions/log/` basis pointers `format:ledger-format-v1#3.2`, `#3.3`, `#5.2`, `#7`, `#8` and `format:ledger-format-migrations#format-2-spec-v1.3`, `#spec-v1.1`, in seven change-sets. They are hashed content, and editing them would move digests. `docs/decisions/hafeok.ledger.nt` carries the same tokens as `ledger:basedOn` literals. The section map resolves them: §3.2 → 3.4, §3.3 → 5.1, §5.2 → 8.3, §7 → 12.5, §8 → 8.1/8.6/9.1.

**Records of their time (left, as instructed).** `ledger/rulings/*` (including the earlier rulings' "ground"), `ledger/audits/audit-2026-10.md`, `ledger/sessions/*`, `docs/audits/provenance-2026-08.md`, the migration entry in `docs/ddd-format-migrations.md` (line 183), and Appendix C of the protocol itself, which copies the migration notes without rewording and so names `ledger-format-v1.md` three times and `ledger-format-migrations.md` once. Appendix B.1, a verbatim copy, names `ledger-format-migrations.md` once.

**Live text I left, for the principal.**

- `ledger/prd/ledger-cli-prd.md` line 14 (§0 item 4). §0 records rulings, so I did not edit it. It cites §4 of the format document; the destination is protocol §4.3 and §4.6.
- `ledger/prd/ledger-cli-prd.md` headings "## 2. Principles (carry over from ledger-format-v1)" and "## 4. Format changes (ledger-format-v1 → v2)". They name the format by version rather than cite the file, and changing a heading changes the anchor that issue bodies may link to.
- `ledger/spec/authority/ledger-authority.ttl` line 31, an `rdfs:comment` literal ("Draft amendment to ledger-format-v1"). It is vocabulary data, not a comment, so I left it. Its two `#` comments are re-pointed.

**Live citations I re-pointed.** Comments in `ledger-core/src/{canon,hash,format,init,lib,finding,canon_tests}.rs`, `authority/{payload,signers}.rs`, `author/{genesis_key,declare}.rs`, `ledger-cli/tests/revisit_format.rs` and `ledger-cli/tests/common/export_only.rs`; `CLAUDE.md` (the ledger section's Format and Gate bullets and the batch-file reference); `ledger/README.md`; `docs/spec-flow-store-v1.md`; `docs/ddd-cli-prd.md`; `ledger/prd/decision-ledger-prd.md` (two lines); `ledger/prd/ledger-cli-prd.md` line 89; the two `#` comments in `ledger/spec/authority/`. No workflow comment cited the format document: `.github/workflows/product-ci.yml` line 134's "format doc" is the eval format.

**Two comments that are wrong beyond their citation, left as they were apart from the citation.** `ledger-core/src/finding.rs` claimed an `every_class_is_specified` test that does not exist. That sentence is rewritten to say that no test checks it. `ledger-core/src/lib.rs` still says "the twelve failure classes"; there are fourteen. `ledger-cli/tests/gate.rs`'s module comment says eight classes have a fixture; eleven do.

**Issue bodies and other repositories.** Issue bodies in mindovermachine-dev/product-cli cite format sections (for example #65, #66, #69, #70, #79, #81, #82, #86, #96, #104). Other repositories: `Hafeok/decision-cli`, `Hafeok/decision-driven-analyzers` and `Hafeok/Varve`. I could not check whether the analyzers' `docs/rules/ledger-input.md` cites the format document: that repository is outside this session's scope. If it does, it is for the principal to file. The section map resolves all of them.

## 9. Changes made to the server-client protocol

| Line | Before | After | Ruling |
| --- | --- | --- | --- |
| §3.3 step 5 | "…against the union of the `allowed_signers` files of the namespaces it indexes, at the current time." | "…against the key bindings of the namespaces it indexes, with their validity windows, at the current time." | 26 |
| SC-2.5 | "(S) The server verifies the signature against the union of the `allowed_signers` of the namespaces it indexes." | "(S) The server verifies the signature against the key bindings of the namespaces it indexes, with their validity windows. It does not use `allowed_signers`, which accepts acceptance signatures only." | 26 |
| §4 table, Version file | "for holding a ground closure." | "for holding a basis closure." | 23 |
| §4 table, Inbox detail | "with its grounds, their trust status" | "with its bases, their trust status" | 23 |
| SC-4.7 | "reaches every store that grounds on it." | "reaches every store whose versions rest on it as a basis." | 23 |
| Appendix B | — | one row for 6 October 2026 naming these changes and rulings 26 and 23 | — |

**Section numbers.** The server-client protocol cites [LEDGER] by name and never by section number, and the ledger protocol's top-level numbers did not move. Nothing changed.

## 10. Questions for the principal, with options

1. **The edge name `ledger:basis`.** The export already emits `ledger:basis` as a literal on unavailability nodes (`self | grantor | fallback-of-genesis`), and the authority ontology declares it a datatype property (`ledger/spec/authority/ledger-authority.ttl` line 151). Options: (a) another name for the pinned-basis edge, such as `ledger:restsOn` or `ledger:pinnedBasis`; (b) rename the unavailability predicate, which changes the committed export of any store with an unavailability (none in this repository) and the ontology; (c) keep both meanings under one IRI. I recommend (a): it touches nothing that exists.
2. **The unavailability's `basis` field.** Ruling 23 makes "basis" the protocol's word for what a version rests on, and the format already has an unrelated field of that name. Options: (a) keep it and keep the disambiguating note (§5.6, §7); (b) rename it at a future format, which is a schema change. I recommend (a).
3. **Basis-loss past a deadline (disagreement 35).** (a) It stays a report, and "stops being citable" changes only derived state; (b) it becomes a failing class once the deadline passes, as `L012` does, by the `L010` mechanism; (c) a notice before the deadline and a class after it. The rulings and the format document point different ways, so I recorded both.
4. **SC-3.7 batch rows (disagreement 25).** (a) Amend SC-3.7 to say the batch file is the ledger protocol's §10.1 file; (b) leave it until the envelope and batch wire formats are specified (SC Appendix A).
5. **Does the verifier profile require the graph stage and the export (disagreement 34)?** (a) Yes: the verifier profile is the whole of §8, and LP-8.6 then only describes format conformance; (b) no: define a "format verifier" and a "full verifier"; (c) keep LP-8.6 as is.
6. **Namespaces (disagreement 33).** (a) LP-3.8 governs once pins are implemented, and LP-3.18's foreign references then need a pin; (b) LP-3.18 stands for references within one repository and LP-3.8 applies across servers only; (c) other.
7. **`\v` in step 2c (disagreement 36).** (a) Fix the code to strip `\v`: a hashed-meaning change for any string ending in U+000B, so a `CANONICAL_FORM` question if any such string exists; (b) amend the text to the code's set, which is the WHATWG ASCII-whitespace set; (c) leave both and add a vector. A scan of stored statements for U+000B would show whether (a) moves any digest.
8. **Pinned token forms.** (a) Accept `dec:<ns>/<ULID>@sha256:<hash>` and `basis:<ULID>@sha256:<hash>`; (b) other forms. Also whether a version carrying one must declare a new format. Today such a token parses as an opaque pointer at format 1.
9. **Tokens of other schemes carrying `@sha256:`.** 135 tokens in this store already pin a hash (`ddd-content:`, `claim:`, `watched:`). Do they take part in convergence? (a) No: only `dec:` and `basis:` forms are pinned bases; (b) yes, for some schemes.
10. **Two copies of the section map.** The prompt puts it in `ledger/README.md`. The protocol needs it too, to resolve Appendix C's old numbers without citing another document (ruling 22), so it is also the protocol's Appendix C.0. Options: (a) keep both; (b) keep only the protocol's, and have the README name that appendix; (c) keep only the README's.
11. **Citations of the PRDs and the way-of-working document.** The protocol keeps the format document's references to PRD sections (for example PRD §10, §9.4, §4.4) and to the way-of-working document. Does ruling 22's "no citing between prose documents" forbid these? (a) No, it concerns normative text split across documents; (b) yes, inline what they are cited for.
12. **The rulings file's first sentence.** The prompt said to write the section unchanged, and I did, so the file opens "Write this section, unchanged, as …". Keep it, or drop that sentence?
13. **The ledger PRDs.** Should `ledger/prd/ledger-cli-prd.md` §0 item 4 and the two headings (§8 above) be re-pointed, or kept as records?
14. **The code-versus-text findings of §4.** For each: (a) fix the code to the text, in its own pull request with its migration note where a verdict can change; (b) amend the text to the code; (c) record it as a known limit. LP-8.11 matters most: the text forbids exactly the heuristic the code uses to pick a forked decision's "latest". If the code is changed, a forked store keeps failing `G004`, and the file-gate classes then have nothing to judge for that decision.

## 11. Checks run

| Check | Result |
| --- | --- |
| `cargo build` | pass |
| `cargo clippy -- -D warnings -D clippy::unwrap_used` | pass |
| `cargo t`, container environment as given | 2069 passed, 14 failed. All 14 depend on a commit author: `ledger-core` `blame::tests` (3), `ledger-cli` `gate.rs` `blame::*` (3) and `inbox.rs` (8). The container exports `GIT_AUTHOR_EMAIL`, `GIT_AUTHOR_NAME`, `GIT_COMMITTER_EMAIL` and `GIT_COMMITTER_NAME`, which override the identities the tests configure for their commits. |
| `cargo t` with those four variables unset | **2083 passed, 0 failed**, the setup session's count |
| The blame suites on a clean `main` worktree, variables set | the same six blame failures, so the failures predate this change |
| `ledger verify --export` | exit 0, output byte-identical to `main`: "conformant — 284 entries, 93 decision(s)", 3 awaiting acceptance, the two no-policy notices, every export matches |
| Export hashes (`git hash-object`, committed and regenerated) | `hafeok.ddd.nt` `33218d29376b66de7c241a7e2de976159e5802d9`, `hafeok.ledger.nt` `5440946bb5fee2ae1aace691d32d791bc8a29f55`, unchanged |
| `ddd validate` | conformant, 362 entries, before and after |
| `ssh-keygen` | installed (OpenSSH 9.6p1) before the runs |
| Search for `ledger-format-v1` | only `.ddd/` (5 seams and `render.html`), the README section map, the protocol's verbatim Appendices B.1 and C, the hashed `.decisions/log/` pointers and `docs/decisions/hafeok.ledger.nt`, and the records and live leftovers of §8 |

One slip: while the first `cargo t` ran, I stashed and restored the working tree for a few seconds to compare `ddd validate`. Only comments and documents differed. The clean run above was made afterwards, on the final tree.

## Principal's replies, 2026-10-07

Both pull requests were accepted in substance. Leaving the four ruling-against-format conflicts unsettled and keeping the `\v` text verbatim were confirmed as right. The fourteen questions of §10 are ruled, and question 6 led to eight further rulings on namespaces. All are recorded in `ledger/rulings/absorption-replies-rulings-2026-10-07.md`.

### Rulings, by number

| Ruling | Question | In short |
| --- | --- | --- |
| 27 | 1 | The pinned-basis edge is `ledger:pinnedBasis`; `ledger:basis` keeps its meaning on unavailability nodes. |
| 28 | 2 | The unavailability's `basis` field keeps its name. |
| 29 | 3 | Basis-loss is a report until a policy deadline passes, then a failing class; with no deadline it stays a report. |
| 30 | 4 | SC-3.7 names the ledger protocol's batch file as the batch. |
| 31 | 5 | The verifier profile requires the graph stage and the export check. |
| 32 | 6 | A namespace verifies the same wherever it sits; coupling between namespaces is kept minimal. |
| 33 | 7 | Step 2c follows the code: `\v` is not stripped. |
| 34 | 8 | The two pinned token forms are ruled; carrying one as a pinned basis declares a new format once pinning is implemented. |
| 35 | 9 | Other `@sha256:` tokens take no part in convergence for now. |
| 36 | 10 | One section map, in the protocol's appendix; the README names it. |
| 37 | 11 | PRDs and the way-of-working document are cited as rationale only. |
| 38 | 12 | The instruction sentence leaves the 6 October rulings file. |
| 39 | 13 | The ledger CLI PRD's §0 item 4 and two headings stay as records. |
| 40 | 14 | The thirteen code-versus-text findings go to a separate verification session. |
| 41–48 | from 32 | Namespace independence: one form of cross-namespace reference (41), a pin names what is trusted, not where (42), only `based_on` crosses (43), only `exported` decisions are pinnable (44), no file holds two namespaces (45), acyclic dependencies and an instability report (46), authority per namespace (47), authority as its own unit later (48). |

### Pull requests

- mindovermachine-dev/product-cli#110 and mindovermachine-dev/product-cli#111 had already merged when the replies arrived. The two housekeeping changes asked for in #111 went into their own pull request, [mindovermachine-dev/product-cli#112](https://github.com/mindovermachine-dev/product-cli/pull/112): ruling 38 (the sentence removed, nothing else changed) and ruling 36 (the README's copy of the map removed; the README names Appendix C.0). The editorial merge in `main` is untouched, so its "copied exactly" claims stay checkable there.
- The rulings are applied in a pull request stacked on #112 (branch `claude/youthful-hamilton-pz0v9n-rulings-1007`).

### Changes made, with their requirement ids

| Ruling | Where | Change |
| --- | --- | --- |
| 27 | §7.6, LP-9.5, §5.6 note | `ledger:pinnedBasis` replaces the working name in both SPARQL queries and in LP-9.5; Open mark and the collision note removed. |
| 28 | §5.6 note | The disambiguating note kept, naming ruling 28 and the new edge. |
| 29 | §7.7, LP-7.23, LP-7.29 (new), §8.9 | The report stated, then the failing class past a policy deadline, Not implemented, numbered at implementation. LP-7.23 keeps the implemented "neither is a gate class", marked superseded in part by ruling 29. The "not settled" sentence removed. |
| 30 | SC-3.7, SC Appendix B | SC-3.7 says the batch is the ledger protocol's batch selection file (§10.1). Id kept. One Appendix B row. |
| 31 | §1, LP-8.6, §9 intro | The verifier profile is the whole of §8 including the export check; a file-gate-only verifier does not conform. LP-8.6 no longer puts the graph stage outside conformance. |
| 32 | §3.6, LP-3.30 (new) | The principle, Not implemented. |
| 33 | LP-4.18 step 2c, Appendix B | Step 2c lists space, `\t`, `\n`, `\f`, `\r`, and says `\v` is not stripped. The note recording the disagreement is replaced by one saying this is the one place the canonicalisation text departs from the absorbed format document. No digest moves. |
| 34 | §7.3, LP-7.27 (new), §3.3, §5 table | Open marks removed from the two token forms and from `basis:<ULID>`; the format rule stated as LP-7.27. `ledger.basis.v1` and `.decisions/basis/` stay Open (§3.1, §4.6, §5, §7.4). |
| 35 | §7.1, LP-7.28 (new), §7.3 note | Stated in the basis-pointer and convergence text. |
| 36 | `ledger/README.md` (#112) | See above. |
| 37 | §1, and the citations listed below | A conformance bullet says these citations are informative; each is marked "informative". |
| 38 | the 6 October rulings file (#112) | See above. |
| 39, 40 | — | No text change. §12.3 records ruling 40. |
| 41 | LP-3.8, LP-3.18 | LP-3.8 is the ruled rule everywhere: one form of reference, a pinned token under a declared dependency, inside a repository as across servers. LP-3.18's free reference is marked implemented and superseded by ruling 41 once pins exist. The conflict note is removed. |
| 42 | Terminology "Pin", LP-7.11 | A pin names the namespace and its key material, with no server. Open working form of the prefix: `dec:<namespace>/`. |
| 43 | LP-3.31 (new) | Stated. The implemented format does not refuse a cross-namespace `supersedes` inside one store: `G001` asks only that the target is a decision filed anywhere in the store (`ledger-core/src/graph/shapes.rs`, read). |
| 44 | LP-3.32 (new) | Stated. Nothing pins today, and `exported` gates nothing; it is read only by the format-5 rule and the emitter (read). |
| 45 | LP-3.33 (new) | Stated. The implemented format does not refuse it: no rule compares the namespaces of a change-set's entries, the export restricts each change-set per namespace (which presumes such files), and grants of scope `*` and role files belong to no namespace. No committed change-set in this repository holds two namespaces (read, and checked over `.decisions/log/`). |
| 46 | LP-7.30, LP-7.31 (new), §8.9 | The acyclic rule with its failing class, numbered at implementation, and the instability report, not a gate. |
| 47 | §6.7, LP-6.31, LP-6.32 (new), and supersession notes | Authority per namespace stated as Not implemented. Notes added where the implemented text stands (list below). |
| 48 | §6.7, §12.3 | One sentence each. |

**Ruling 47 will supersede, once implemented:**

- **LP-4.12:** the genesis holder's self-bound binding once per store, and a later namespace's first binding signed by a key trusted in another namespace.
- **LP-4.31:** a first policy judged by a key trusted in another namespace.
- **LP-4.32:** `valid-before` taken from a close in any namespace.
- **LP-4.37:** keys judged as belonging to another principal anywhere in the store, and as closed in any namespace.
- **LP-4.38:** the later-namespace opening, which binds the genesis holder's live key from elsewhere.
- **LP-4.39:** a close in one namespace closing the key in every namespace. LP-6.32 is its successor.
- **LP-5.19:** the store-wide `roles/` directory.
- **LP-6.5 and `A005` (LP-8.19, LP-8.23):** the store's one genesis grant of scope `*`.
- **LP-6.16:** scopes `*` and `pattern:` covering every namespace.
- **LP-6.28:** "the genesis grant" read as the store's one.
- **LP-8.31:** the store's one genesis holder in the unbound-genesis notice.
- **LP-9.11:** `*` grants reaching every namespace's export.

LP-6.30 and LP-8.27 (a role placed by its landing) are named in LP-6.31. Their landing rule survives; only the store-wide role file goes.

**Ruling 37, the citations checked.** Each is rationale or provenance, and each is now marked informative:

- §3.4: the PRD's "accepted-by resolves";
- §3.5: the ledger CLI PRD §4;
- LP-4.17: PRD §10;
- §4.6: the PRD §7 revocation payload, whose fields and instant form the table and LP-3.24 already state;
- §5.1: the way-of-working §2.2, whose tier order `T0 < T1 < T2` is stated;
- §5.3: OD-3;
- §5.4 note: the PRD's ground table, whose four stage values are stated;
- LP-7.19: PRD §9.4;
- §12.4: PRD §4.4, PRD §8 and OD-6.

The `OD-6 open` comment inside the verbatim change-set example is left as it was.

**No citation needed its text stated in the protocol:** everything an implementation needs from those passages was already there.

**Server-client protocol, ruling 42.** No line describes a pin as naming a server, so nothing changed for ruling 42. One line bears on it without being a pin description: SC-4.1, "Two servers MAY use the same namespace name for unrelated namespaces". Once a pin names a namespace and key material with no server, two unrelated namespaces of one name are told apart only by their key material. That is for the design session (list 3).

**Test-case candidate added** (§7 above, by ruling 33). A version whose `statement` ends with U+000B. Its canonical JSON keeps the `\u000b`, and the digest is the one the reference implementation computes. No such vector exists today.

**Checks on the stacked pull request.** As before: build, clippy and `cargo t` with the container's git-identity variables unset, and `ledger verify --export` identical to `main` (results in the pull request).

### 1. For the verification session

The thirteen rows of §4 marked "differs", in the order to check them. The first three weaken what a verifier guarantees against a hand-written file.

1. **LP-4.7.** A sidecar on an act no policy governs is not verified, though a present sidecar always has to verify.
2. **LP-6.17.** "The accept role is never the genesis role" is enforced at write only; the verifier checks only that the role may `accept-decision`.
3. **LP-5.11.** A duplicate acceptance id is not refused, nor a decision identity object filed twice.
4. **LP-8.8.** The same gap seen from the `SCHEMA` list: "a duplicate id" is checked for authority records only.
5. **LP-8.11.** A forked decision gets a stand-in "latest" (the first tip in hash order), and the file-gate classes judge it.
6. **LP-4.12.** A `rotate` passes when signed by any live key of the principal, not only the key it closes.
7. **LP-3.16.** `format:` changes are not compared across history, so a lowering that still meets the need, or a raise past it, is not flagged.
8. **LP-8.9.** `L006` checks an escape's `accepted_by` on latest versions only.
9. **LP-3.4.** A YAML float in a string field is read as text, not refused.
10. **LP-4.18.** The float half of step 7, the same point as item 9. The `\v` half is settled by ruling 33. The float half is not ruled.
11. **LP-3.3.** A `set:` grant scope refuses dots, though set ids allow them.
12. **LP-8.25.** `parents` is keyed as separate list items, not with the header entity.
13. **LP-9.14.** The export-only verifier exists only as a test helper, and derives `valid-before` per binding, not per key.

I confirmed item 5 by reading, and the `\v` half of LP-4.18 by running it. The other items are the survey's reading and still need checking. The implementation-defined points of §5, for example integer rendering and the Unicode version, are a separate list and not among the thirteen.

### 2. Issues for the principal to file

- **The NFC Unicode tables are not pinned.** `Cargo.lock` is git-ignored, and `ledger-core` asks for `unicode-normalization = "0.1"`, so the Unicode tables behind NFC (LP-4.18 step 2b) are not pinned. Options:
  - commit the lockfile;
  - pin the crate to an exact version.

  Either way, name the Unicode version in the protocol.
- **Fourteen tests depend on the caller's git identity.** They fail when the environment sets `GIT_AUTHOR_EMAIL`, `GIT_AUTHOR_NAME`, `GIT_COMMITTER_EMAIL` and `GIT_COMMITTER_NAME`, which override the identities the tests configure:
  - `ledger-core` `blame::tests` (3);
  - `ledger-cli` `gate.rs` `blame::*` (3);
  - `inbox.rs` (8).

  The tests should clear those variables for the git commands they run.

### 3. For a later design session on namespace independence

Not designed here. It must settle:

- how grants, roles, key bindings and policies are stored and scoped per namespace, and what a grant scope of `*` or `pattern:` then means;
- whether each namespace gets its own directory under `.decisions/`, so that extraction is a directory move (LP-3.30);
- whether `allowed_signers` becomes one file per namespace, and what the `[SIGNERS]` stage then compares;
- the migration for stores and fixtures that hold authority records today, including every fixture that bootstraps a genesis and binds keys in two namespaces (`key_across_namespaces.rs`, `genesis_key.rs`);
- the exact content of a pin (LP-7.11): key material, policy hash or genesis grant, and the fate of `source_prefix` and the working prefix `dec:<namespace>/`;
- what the export of a namespace carries once authority is per namespace (LP-9.11, LP-9.6);
- how two unrelated namespaces of one name are told apart when a pin names no server (SC-4.1);
- whether a writer filing "a close in every namespace it holds" (LP-6.32) is one act or one per namespace, and how it is ordered (§8.7);
- how a move of a namespace between repositories treats landing order (LP-8.24), which is read from the history of the repository that holds it;
- authority as its own unit (ruling 48), later.

### 4. Still the principal's

- **The five `.ddd/` seams.** Each should cite:

  | Seam | Should cite |
  | --- | --- |
  | `seam-ledger-acceptor-identity.yaml` | protocol §3.4 (LP-3.22, LP-3.23) |
  | `seam-ledger-canonical-form.yaml` | Appendix C, and §4.5 (LP-4.21) |
  | `seam-ledger-disposition-states.yaml` | §8.10 (LP-8.33) |
  | `seam-ledger-graph-stage.yaml` | §8.6 (LP-8.19, LP-8.22) |
  | `seam-ledger-verify-classes.yaml` | §8.3 (LP-8.9) |

  `.ddd/render.html` follows once the seams change.
- **The analyzers' rules document.** Whether `docs/rules/ledger-input.md` in `Hafeok/decision-driven-analyzers` cites the format document. That repository is outside this session's reach.
