# The decision ledger's documents

This directory holds the documents of the decision ledger: the record substrate
whose store is `.decisions/` and whose binary is `ledger`. The crates stay at the
workspace root (`ledger-core/`, `ledger-cli/`) and the committed exports stay at
`docs/decisions/*.nt`. The ledger has one normative text,
`spec/ledger-protocol.md`, which absorbed the former format document and its
migration record (ruling 22). The root `CLAUDE.md`, section "Decision ledger",
governs the crates.

## Directories

**`spec/`** holds the ledger's specification. `ledger-protocol.md` is the Decision
Ledger Protocol 1.0, the normative format, and `server-client-protocol.md` the
Decision Ledger Server-Client Protocol, a draft of 5 October 2026. `spec/authority/` is
the format-6 authority ontology: `ledger-authority.ttl`, its SHACL shapes
`ledger-authority-shapes.ttl`, and the two samples the shapes are checked against.
`spec/tests/` is the protocol test suite, empty today.

**`rulings/`** holds the principal's rulings, each recorded beside the questions it
answers. A ruling wins over the document that asked the question.

**`prd/`** holds the product requirements: the decision ledger PRD (L0 to L6), the
ledger CLI PRD, and the decision registry PRD.

**`audits/`** holds read-only audits of the ledger against its requirements.

**`sessions/`** holds the prompts that drove implementation sessions, the
close-out each session wrote, and the worksheets that sequenced work over the
ledger's pending entries.

## Status

Which document is normative today.

| Document | Status |
| --- | --- |
| `ledger/spec/ledger-protocol.md` | **Normative** for the store format, hashing, signing, authority, the gate and the export. Requirements it marks *Not implemented* are ruled but not built. Its Appendix C is the migration record. |
| `ledger/spec/server-client-protocol.md` | **Draft.** It depends on the ledger protocol, which governs where they differ. |
| `ledger/spec/authority/` | The vocabulary and shapes the protocol's §5.6 projects. |
| `ledger/spec/tests/` | Empty. A test case binds once the principal approves it. |
| `ledger/rulings/` | The principal's rulings, as recorded. |
| `ledger/prd/` | Requirements, amended by the rulings. Not a specification. |
| `ledger/audits/`, `ledger/sessions/` | Records. Not normative. |

## Path map

Base names never changed; only the directory did. Issue bodies, rulings, the log
in `.decisions/`, the `.ddd/` store and other repositories cite the old paths.
This map is how those citations resolve.

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
| `docs/acceptance-worksheet-2026-08.md` | `ledger/sessions/acceptance-worksheet-2026-08.md` |

A basis pointer such as `prd:decision-ledger-prd#4.2.1` names a document by its
base name, not its path, so the move does not touch it.

## Section map

`docs/ledger-format-v1.md` and `docs/ledger-format-migrations.md` were absorbed
into `spec/ledger-protocol.md` and deleted (ruling 22). Issue bodies, rulings,
session close-outs, the `.ddd/` store, the hashed basis pointers in
`.decisions/log/` (`format:ledger-format-v1#…`) and other repositories cite the
format document's sections. This map is how those citations resolve. The
protocol carries the same map as its Appendix C.0. Read "ground" in the earlier
rulings, in the sense of what a version rests on, as "basis" (ruling 23).

| Format document § | Protocol section | Requirement ids |
| --- | --- | --- |
| Header (status, scope, audience) | Status; 1; Appendix B.1 | — |
| §1 What the format is for | 2 (purpose); 4.1 (the two versions) | LP-4.15 |
| §2 Storage layout | 3.1 | LP-3.10 to LP-3.13 |
| §3 Schemas (every file declares `format`) | 3.2 | LP-3.14 |
| §3.1 Identifiers | 3, 3.3 | LP-3.3, LP-3.17 to LP-3.20 |
| §3.2 Identity | 3.4 | LP-3.6, LP-3.7, LP-3.21 to LP-3.23 |
| §3.3 Tolerance | 5.1 | LP-5.8, LP-5.9 |
| §3.4 Discharge pointers | 5.4 | LP-5.16 to LP-5.18 |
| §3.5 Set file | 5.2 | LP-5.10 |
| §3.6 Change-set file | 5.3; 7.1 (`based_on`) | LP-5.7, LP-5.11 to LP-5.15, LP-7.19, LP-7.20 |
| §3.7 The reopen edge, `revisit_if` | 7.2; 3.2 (declare what you need) | LP-7.22 to LP-7.26, LP-3.15 |
| §3.8 The decision key and the export flag | 3.5 | LP-3.9, LP-3.25 to LP-3.29 |
| §3.9 Authority records (intro) | 5.6 | — |
| §3.9.1 Files | 5.6 | LP-5.19 |
| §3.9.2 Log entries | 5.6 | LP-5.20, LP-5.21 |
| §3.9.3 Hashing | 4.6 | LP-4.22, LP-4.23 |
| §3.9.4 What the gate checks | 8.4; 6.1 (liveness, role check) | LP-8.16, LP-6.15 to LP-6.18 |
| §3.9.5 `allowed_signers` | 4.9 | LP-4.10, LP-4.32, LP-4.33 |
| §3.10 Signing (intro) | 4.8 | — |
| §3.10.1 New fields | 6.2 | LP-6.19 to LP-6.24, LP-4.8 |
| §3.10.2 Payloads | 4.6 | LP-4.22, LP-4.24, LP-3.24 |
| §3.10.3 The signed bytes | 4.7 | LP-4.5 |
| §3.10.4 Sidecars and schemes | 4.8 | LP-4.6, LP-4.7, LP-4.9, LP-4.25 to LP-4.31 |
| §3.10.5 Trusted bindings and the first key (D7) | 4.10 | LP-4.12, LP-4.34 to LP-4.38 |
| §3.10.6 Order: landing and `at` (D6) | 8.7; 4.11 (closed keys) | LP-8.24 to LP-8.29, LP-4.13, LP-4.14, LP-4.39 |
| §3.10.7 The role check over history and history rules | 6.6; 8.7; 8.8; 3.2 | LP-6.27 to LP-6.30, LP-8.30, LP-8.31, LP-3.16 |
| §3.10.8 The export and the export-only verifier | 9.3 | LP-9.3, LP-9.14, LP-9.15 |
| §3.10.9 The batch selection file | 10.1 | LP-10.1 to LP-10.5 |
| §4 Canonicalisation and hashing (intro) | 4 | — |
| §4.1 The hashed field set | 4.2 | LP-4.16, LP-4.17 |
| §4.2 The algorithm | 4.3 | LP-4.18, LP-3.4 |
| §4.3 The digest | 4.4 | LP-4.19, LP-4.20 |
| §4.4 Conformance vector | 4.5 | LP-4.21 |
| §5 The gate (intro) | 8.2 | LP-8.7 |
| §5.1 The parse gate | 8.2 | LP-8.8 |
| §5.2 The fourteen | 8.3 | LP-8.9 to LP-8.15 |
| §5.3 Gates and exit codes | 8.5 | LP-8.17, LP-8.18 |
| §6 Open edges | 12.4 | — |
| §7 What L1 needs from L0 | 12.5 | — |
| §8 The graph stage (L2) | 8.1, 8.6, 9.1, 8.10 | LP-8.6, LP-8.19 to LP-8.23, LP-9.4, LP-9.16 to LP-9.19, LP-8.33 |
| §8.1 The committed export and the export stage | 9.2 | LP-9.1, LP-9.2, LP-9.11 to LP-9.13 |
| Migrations document | Appendix C | — |
