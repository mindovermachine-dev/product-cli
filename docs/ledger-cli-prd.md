# Decision Ledger CLI — product requirements

Draft 1, 2 October 2026. Scope: the open-source `decision` CLI (decision-cli). The Context& service layer (DSSE/Sigstore, step-ca, eIDAS, hosted ledger, fan-out) is out of scope and referenced only where the open design must leave a seam for it.

## 1. Purpose

The ledger is the source of truth for decisions. Code cites decisions through `DecisionDriven.Analyzers`; the analyzers' generator reads the ledger's export. The CLI is how people file, revise, accept and verify decisions, and how the export is produced. Its first job is to make acceptance a single, deliberate, human, signed act, because acceptance is the only operation in the model that carries responsibility, and it is currently done by hand-editing files.

## 2. Principles (carry over from ledger-format-v1)

- Files are the truth; the graph is a read model. Every entity is a file whose hash is its identity; hashed content is strings only; the format refuses to restate facts.
- File gate before graph stage: single-file checks reject bad files; cross-file properties are computed at verify time, never materialised.
- Identities are `mailto:` IRIs. A model is never a holder.
- Everything the CLI does must be reproducible from the repository alone, with no service in the loop.

## 3. Users

- A maintainer of a repository that cites decisions (first: Varve, decision-driven-analyzers, contextand-architecture).
- A holder with `accept-decision` for a namespace, reviewing a red PR.
- An agent (Claude Code) filing new, unaccepted decisions during work. Agents never accept.
- A consumer verifying an export without the ledger's own tooling.

## 4. Format changes (ledger-format-v1 → v2)

| Change | Why |
|---|---|
| `ledger:key` on `DecisionVersion`: `^[A-Z][A-Za-z0-9]{0,63}$`, hashed, unique per (namespace, key) among live decisions, immutable across versions of one decision, carried to the successor on supersession | Generated C# type names; stable citations across supersession |
| `ledger:exported` on `DecisionVersion`: boolean, hashed | A decision citable from other namespaces; default false |
| `ledger:Commit` a `prov:Activity`, keyed `urn:git:sha1:<hex>` or `urn:git:sha256:<hex>`; change-sets `prov:wasInformedBy` the commit that landed their file | Citations are blamed to commits; the citation projection needs the node |
| `ledger:Citation` a `prov:Entity` (produced by the analyzers' report, never hashed): `ledger:ofDecision`, `ledger:citesVersion`, `ledger:symbol`, `ledger:attribute`, `ledger:exceptionScope`, `prov:wasGeneratedBy <commit>` | Read model of code → decision |
| `ledger:signature` and `ledger:signatureScheme` on `Acceptance`; signature stored as a sidecar; acceptance hash covers content, not signature | Signed acceptances (§7) |
| `ledger:publicIri` on `DecisionVersion`, optional | Dereferenceable alias (`https://…/{ns}/{set}/{key}`) for the lineage; URN identity unchanged |
| Namespace policy entity: allowed signature schemes, required key types, `allowed_signers` snapshot reference, threshold (signers required) per set or namespace | Per-namespace acceptance policy |

Gates added: key syntax and immutability (file gate); key uniqueness among live decisions (graph stage, SPARQL shape); exported decisions must have an unrevoked acceptance before export (export-time check); signature verifies against policy (graph stage).

## 5. Commands

All commands run inside a repository with a `ledger/` directory. All writes are to files; all commits are the user's (`--commit` opt-in creates a signed, signed-off commit with `Refs #n`).

| Command | Behaviour |
|---|---|
| `decision init --namespace <ns>` | Creates `ledger/`, namespace entity, empty policy, `allowed_signers` with the initialising identity |
| `decision new --set <set> --key <Key> --statement "…" [--exported]` | New decision with first version; unaccepted; prints the generated type name |
| `decision revise <key> --statement "…"` | New version, `prov:wasRevisionOf` the tip; key unchanged |
| `decision supersede <old-key> --key <NewKey?> --statement "…"` | New decision superseding the old; key carried unless `--key` given; old tip remains, unaccepted citations of it diverge |
| `decision revoke <key> --reason "…"` | Revocation triples on the tip's acceptances (no decision-level retirement in v2; open item) |
| `decision review [--diff <ref>] [--pr <n>]` | Lists decisions cited in the changed code whose tip has no unrevoked acceptance: key, set, statement, citing symbols. Reads citations from the analyzers' report output or by scanning `typeof(` citations in the diff |
| `decision accept <key>… [--scope version\|class:<ref>] [--commit]` | For each key: builds the acceptance node, canonicalises (§7), signs with SSH, writes acceptance and sidecar, re-exports. Refuses non-interactive invocation, refuses if the holder lacks `accept-decision`, refuses a software key when policy requires `-sk`, refuses an agent-reported unconfirmed key |
| `decision verify [--export] [--signatures] [--all]` | File gate, graph stage, L-class checks against git, signature verification with `-Overify-time`, export freshness. Exit code is the CI gate |
| `decision export --format ntriples --namespace <ns> [--out docs/decisions/<ns>.nt]` | Deterministic, sorted N-Triples of the namespace: decisions, versions, acceptances with signature references, policy and `allowed_signers` snapshot; only exported decisions when `--exported-only` |
| `decision import --from docs/decisions/*.md` | One-time import of interim set files: keys, statements, acceptances (as unsigned acceptances flagged `ledger:signatureScheme "none"`, which policy may reject later); deletes the markdown on success |
| `decision policy set|show` | Edits the namespace policy; the change itself requires acceptance by a current holder |
| `decision identity add|rotate` | Edits `allowed_signers`: principal, key, `namespaces=ledger-accept@<ns>`, validity window; rotation adds a new line, never edits an old one |
| `decision show <key>` / `decision list [--unaccepted] [--exported]` | Inspection |

## 6. Review loop (the user story this PRD exists for)

1. A PR arrives red: CS0618 on every citation of an unaccepted decision.
2. `gh pr checkout n && decision review --pr n` prints the keys, statements and citing symbols.
3. The holder reads, then `decision accept Key1 Key2 --commit`, touches the hardware key per acceptance (or confirms per key for software keys), and pushes.
4. `decision verify` in CI confirms signatures and export freshness; the generator reads the export; the PR goes green.
5. Rejected keys: the holder tells the author which; the author takes the alternative and removes the citation. Nothing in the CLI accepts on anyone's behalf.

Target: a holder accepts twenty decisions in under five minutes without opening an editor.

## 7. Signatures (open-source scheme: SSH)

- Canonical form: RDFC-1.0 of the acceptance node's triples (excluding `ledger:signature`), serialised as canonical N-Quads, UTF-8.
- Signing: `ssh-keygen -Y sign -f <key> -n ledger-accept@<namespace>` over the canonical bytes. Sidecar file `<acceptance-id>.sig` in SSHSIG format.
- Identity binding: `ledger/allowed_signers`, one line per principal and key: `mailto:… namespaces="ledger-accept@<ns>" valid-after=… valid-before=… <keytype> <key>`. Changes are themselves accepted decisions in the policy set.
- Verification: `ssh-keygen -Y verify -f allowed_signers -I <principal> -n ledger-accept@<ns> -Overify-time=<prov:generatedAtTime>`.
- Time: as written in the acceptance, backed by the landing commit's date; no timestamp authority in the open source.
- Hardware keys: `-sk` recommended; required per namespace by policy.
- Seam: `ledger:signatureScheme` is extensible. Other schemes (DSSE/in-toto, Sigstore, X.509, eIDAS) are verifiers registered by name; the open CLI ships `ssh` and `none` (import only).

## 8. Integration with the analyzers

- The generator reads `docs/decisions/<ns>.nt`; when both `.nt` and interim `.md` exist for a namespace, DDGEN reports it.
- `ledger:key` → nested type name; `ledger:exported` → citable cross-namespace; unrevoked acceptance on tip → citable in release builds; no decision-level retirement in v2, so `[Obsolete(error: true)]` has no ledger source until a later format change.
- The report's citation projection is emitted as `ledger:Citation` N-Triples and may be committed under `docs/decisions/<ns>.citations.nt`; `decision review` reads it when present.

## 9. Non-goals

- Any network service, hosted acceptance, or web UI.
- DSSE, Sigstore, CA integration, timestamp authorities, eIDAS. Seams only.
- Decision-level retirement (open item; tracked for v3).
- Editing hashed files in place; every change is a new entity.

## 10. Acceptance criteria

- Varve's `docs/decisions/*.md` imports, exports, and the generated decision types are byte-identical before and after; the markdown is removed.
- `decision accept` on a hardware key produces an acceptance that `ssh-keygen -Y verify` validates independently of the CLI.
- An acceptance signed by a key not in `allowed_signers`, or outside its validity window, or with the wrong SSHSIG namespace, fails `decision verify`.
- `decision accept` run without a TTY, or by an identity without `accept-decision`, or with a software key under a `-sk` policy, exits non-zero and writes nothing.
- A committed export edited by hand fails `decision verify --export`.
- Two live decisions with the same key in one namespace fail the graph stage.
- Supersession carries the key; the generator's output for the citing repository compiles unchanged and the report shows the diverged cited version.
- The whole flow runs offline.

## 11. Open questions

- Decision-level retirement: a decision explicitly withdrawn with no successor needs a representation; until then the analyzers' error-level obsolete has no source.
- Threshold acceptances (two holders) per set: policy supports it; `accept` writes one acceptance per holder; the generator treats the decision as accepted only when the threshold is met. Confirm the generator reads the policy.
- Whether `review` should rely on the report's projection (accurate, needs a build) or a diff scan (fast, approximate). Proposal: diff scan by default, projection when present.
- Imported acceptances with scheme `none`: grandfathered forever, or re-signed in a one-time ceremony. Proposal: policy decides per namespace; default grandfathered with a visible flag.
