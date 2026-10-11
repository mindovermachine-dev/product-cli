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
`.decisions/ns/<ns>/log/` (`format:ledger-format-v1#…`) and other repositories cite the
format document's sections. The one map from those sections to the protocol's
is the protocol's Appendix C.0 (ruling 36). Read "ground" in the earlier
rulings, in the sense of what a version rests on, as "basis" (ruling 23).
