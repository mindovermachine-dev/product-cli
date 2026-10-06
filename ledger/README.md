# The decision ledger's documents

This directory holds the documents of the decision ledger: the record substrate
whose store is `.decisions/` and whose binary is `ledger`. The crates stay at the
workspace root (`ledger-core/`, `ledger-cli/`), the committed exports stay at
`docs/decisions/*.nt`, and the format document stays at `docs/ledger-format-v1.md`
with its migrations at `docs/ledger-format-migrations.md` until the absorption
session. The root `CLAUDE.md`, section "Decision ledger", governs the crates.

## Directories

**`spec/`** holds the ledger's specification. `spec/authority/` is the format-6
authority ontology: `ledger-authority.ttl`, its SHACL shapes
`ledger-authority-shapes.ttl`, and the two samples the shapes are checked against.

**`rulings/`** holds the principal's rulings, each recorded beside the questions it
answers. A ruling wins over the document that asked the question.

**`prd/`** holds the product requirements: the decision ledger PRD (L0 to L6), the
ledger CLI PRD, and the decision registry PRD.

**`audits/`** holds read-only audits of the ledger against its requirements.

**`sessions/`** holds the prompts that drove implementation sessions and the
close-out each session wrote.

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

A basis pointer such as `prd:decision-ledger-prd#4.2.1` names a document by its
base name, not its path, so the move does not touch it.
