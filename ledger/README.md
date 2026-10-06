# The decision ledger's documents

This directory holds the documents of the decision ledger: the record substrate
whose store is `.decisions/` and whose binary is `ledger`. The crates stay at the
workspace root (`ledger-core/`, `ledger-cli/`), the committed exports stay at
`docs/decisions/*.nt`, and the format document stays at `docs/ledger-format-v1.md`
with its migrations at `docs/ledger-format-migrations.md` until the absorption
session. The root `CLAUDE.md`, section "Decision ledger", governs the crates.

## Directories

**`spec/`** holds the ledger's specification. `ledger-protocol.md` is the Decision
Ledger Protocol 1.0 Editor's Draft and `server-client-protocol.md` the Decision
Ledger Server-Client Protocol, both drafts of 5 October 2026. `spec/authority/` is
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
| `docs/ledger-format-v1.md` | **Normative** for the store format, hashing and the gate, until the absorption session lands. |
| `docs/ledger-format-migrations.md` | Normative migration notes for the format document. |
| `ledger/spec/ledger-protocol.md` | **Draft.** Where it and the format document differ, the format document governs. |
| `ledger/spec/server-client-protocol.md` | **Draft.** Where it and the format document differ, the format document governs. |
| `ledger/spec/authority/` | The vocabulary and shapes the format document's §3.9 projects. |
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
