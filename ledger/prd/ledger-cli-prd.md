# Decision Ledger CLI — product requirements

Draft 1, 2 October 2026. Scope: the open-source `decision` CLI (decision-cli). The Context& service layer (DSSE/Sigstore, step-ca, eIDAS, hosted ledger, fan-out) is out of scope and referenced only where the open design must leave a seam for it.

**Amended 2026-10-01** by the principal's rulings on `ledger/audits/audit-2026-10.md` §9 — see §0. **Amended 2026-10-02** by D6 (item 12) and D9 (item 4), `ledger/rulings/signing-rulings-2026-10-d5-d9.md`.

## 0. Amendments (2026-10-01)

Ruled on the audit of the implementation (`ledger-core` / `ledger-cli`, binary `ledger`, store `.decisions/`). Each is folded into the sections below; this list is the index.

1. **Entities are not one file each (audit C1).** The unit of a file is the *act*: a change-set (`.decisions/log/<ulid>.yml`) holding the decisions, versions, acceptances and revocations one act filed. Versions are content-addressed (`sha256:` over the canonical form); acceptances and revocations gain a content hash for signing (§4, §7) alongside their ULID ids. The file layout is not changed.
2. **The key rules are file-gate classes (audit C2).** The file gate is the format's import surface — every rule an outside implementation must reproduce, including cross-entry rules over the whole log (as `L005` and `L008` already are). Key syntax, key immutability and key uniqueness among live decisions are all file-gate classes; uniqueness additionally has a graph-stage SPARQL cross-check. Nothing a generated type name depends on is graph-only.
3. **Revocation is its own signed entity; acceptances are immutable.** A `ledger:Revocation` names the acceptance it revokes (`ledger:revokes`), carries its reason and attribution, and is signed. Revoking requires the role. No triple is ever added to an acceptance node after it is created, so the acceptance's signature covers its fixed content and stays valid when it is revoked.
4. **The signed payload is a closed field list under the ledger's one canonicalisation law (audit C4).** Not RDFC-1.0 over a node. The acceptance payload is `{decision, version, actor, at, scope, expires_at}` — scope and expiry are signed, so an acceptance cannot be re-scoped under its signature — canonicalised as the format's canonical JSON (§4 of `ledger-format-v1.md`) under its own domain-separation prefix. The revocation payload is likewise closed. The export carries every payload field as a triple, so a consumer can rebuild the signed bytes from the export alone. *(Amended 2026-10-02, D9: every act names the grant it is made under. `under`, a grant id, joins the payloads of an acceptance, a revocation, a grant, a policy change and a key-binding filed by someone other than its principal — hashed when present, omitted when absent. This supersedes the closed lists above; see `ledger/rulings/signing-rulings-2026-10-d5-d9.md`.)*
5. **The export keeps `ledger:set` as an IRI** (`<urn:ledger-set:<id>>`). The analyzers' reader takes the set id from the IRI's local part, and keeps every `rdf:type` value of a node rather than the last.
6. **`mailto:` is the identity's IRI form; the file keeps the bare address** (refined 2026-10-02). Parse normalises `mailto:x@y` and `x@y` to one identity; the stored and hashed form stays the bare address `x@y`, so no digest moves and the canonical form is unchanged; the graph projection and the export emit `<mailto:x@y>`, and equality compares that form.
7. **When a signature is required: namespace policy, not tier** (#65, D1). Tier is a property of a decision; signing is a property of what the namespace promises its citers. Policy declares the required schemes. A decision cannot leave its namespace, because the namespace is in its id, so the requirement cannot be shopped out per decision. A policy change is itself accepted, signed under the policy in force *before* the change. `none` is a scheme only where policy lists it, and the generator treats `none` acceptances as citable only in such namespaces.
8. **Where it is stored: a sidecar** (#65, D2), at `.decisions/sig/<acc-ulid>.<scheme>.sig`, one per scheme, all verified together. The inline `signature` field is retired: it is required empty permanently, and the gate refuses any value, so there is one place a signature can be.
9. **Class numbers** (#65, D3). `L011` (a required signature is absent or invalid; the requirement now comes from namespace policy) and `L012` (acceptances under a since-closed key, a review trigger) keep their August roles. New classes take the next free numbers, `L013` onward.
10. **Schemes** (#65, D4). `ssh` ships first, as the open-source scheme. `dsse` comes second, for the hosted service, and the open CLI ships DSSE *verification* (never signing), so a hosted ledger stays verifiable from the repository alone. `none` covers pre-v2 stores only, where namespace policy lists it. `webauthn` comes later, by ADR. There are no others. Losing X.509's single mechanism is a consequence accepted knowingly: there are two schemes, a holder has a key in each scheme policy lists, and the hosted path is gated by Entra rather than by a shared certificate chain.
11. **The trust root is a governed projection** (#65). `allowed_signers` is a derived file, built from key-binding entries (`identity add | rotate | revoke`, each an accepted act in the ledger) and held byte-identical by `verify`, exactly as the export is. Bootstrap: the genesis holder's first binding is signed by the key it binds, and the genesis grant's `externalRef` is the out-of-band mandate for it. Every later binding is signed under the policy in force.
12. **Keys closed later** (#65). Validity is judged at the acceptance's time against the key's window. An acceptance dated after the close, or landed after it, fails `L011`. An acceptance dated **and landed** before it is listed by `L012` for the holder to re-accept or affirm. *(Amended 2026-10-02, D6: "before" is landing order on the first-parent history and `at`, both; this supersedes D3's "dated before the close". See `ledger/rulings/signing-rulings-2026-10-d5-d9.md`.)* An affirmation is a new acceptance by the holder with a live key, not a new entity: the reader takes the latest valid acceptance for a version, and the old one stays as history. The listing is a registry inbox item ("needs re-acceptance"). A namespace policy may set a deadline after which unaffirmed acceptances under a closed key stop being citable.

## 1. Purpose

The ledger is the source of truth for decisions. Code cites decisions through `DecisionDriven.Analyzers`; the analyzers' generator reads the ledger's export. The CLI is how people file, revise, accept and verify decisions, and how the export is produced. Its first job is to make acceptance a single, deliberate, human, signed act, because acceptance is the only operation in the model that carries responsibility, and it is currently done by hand-editing files.

## 2. Principles (carry over from ledger-format-v1)

- Files are the truth; the graph is a read model. A file holds one act (a change-set); every hashed entity is identified by the hash of its content; hashed content is strings only; the format refuses to restate facts.
- File gate before graph stage: the file gate is the format's import surface — single-file and cross-entry rules every implementation must reproduce, computed at verify time and never materialised; the graph stage is the reference implementation's SPARQL cross-check and is never the only home of a rule.
- Identities are `mailto:` IRIs in the graph and in equality; files and hashes keep the bare address, and parse normalises either spelling to one identity. A model is never a holder.
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
| `ledger:exported` on `DecisionVersion`: hashed as the string `"true"` when set, absent (omitted from the canonical form) when false — hashed content is strings only | A decision citable from other namespaces; default false |
| `ledger:Commit` a `prov:Activity`, keyed `urn:git:sha1:<hex>` or `urn:git:sha256:<hex>`; change-sets `prov:wasInformedBy` the commit that landed their file | Citations are blamed to commits; the citation projection needs the node |
| `ledger:Citation` a `prov:Entity` (produced by the analyzers' report, never hashed): `ledger:ofDecision`, `ledger:citesVersion`, `ledger:symbol`, `ledger:attribute`, `ledger:exceptionScope`, `prov:wasGeneratedBy <commit>` | Read model of code → decision |
| `ledger:signature` and `ledger:signatureScheme` on `Acceptance` and `Revocation`; signature stored as a sidecar at `.decisions/sig/<id-ulid>.<scheme>.sig`, one per scheme; the inline `signature` field is retired (required empty, permanently); the entity's hash covers its fixed content (the closed payload, §7), never the signature | Signed acceptances and revocations (§7) |
| `ledger:Revocation` a `prov:Entity`: `ledger:revokes <acceptance>`, `ledger:revocationReason`, `prov:wasAttributedTo`, `prov:generatedAtTime`, signed; revoking requires the role. An `Acceptance` node is immutable after creation — no revocation triple is ever written on it | A revocation that edited the acceptance would break the acceptance's signature; revoking is an act of authority, so it is attributed and signed like accepting |
| `ledger:publicIri` on `DecisionVersion`, optional | Dereferenceable alias (`https://…/{ns}/{set}/{key}`) for the lineage; URN identity unchanged |
| Namespace policy entity: allowed signature schemes, required key types, `allowed_signers` snapshot reference, threshold (signers required) per set or namespace | Per-namespace acceptance policy |

Gates added: key syntax, key immutability, and key uniqueness among live decisions (file gate, by the `L010` amendment mechanism; uniqueness also has a graph-stage SPARQL cross-check); exported decisions must have an unrevoked acceptance before export (export-time check); signature verifies against policy (graph stage).

## 5. Commands

All commands run inside a repository with a `ledger/` directory. All writes are to files; all commits are the user's (`--commit` opt-in creates a signed, signed-off commit with `Refs #n`).

| Command | Behaviour |
|---|---|
| `decision init --namespace <ns>` | Creates `ledger/`, namespace entity, empty policy, `allowed_signers` with the initialising identity |
| `decision new --set <set> --key <Key> --statement "…" [--exported]` | New decision with first version; unaccepted; prints the generated type name |
| `decision revise <key> --statement "…"` | New version, `prov:wasRevisionOf` the tip; key unchanged |
| `decision supersede <old-key> --key <NewKey?> --statement "…"` | New decision superseding the old; key carried unless `--key` given; old tip remains, unaccepted citations of it diverge |
| `decision revoke <key> --reason "…"` | Files one signed `ledger:Revocation` per live acceptance of the tip (`ledger:revokes <acceptance>`); the acceptances themselves are never edited. Refuses non-interactive invocation and an identity without the role, exactly as `accept` does (no decision-level retirement in v2; open item) |
| `decision review [--diff <ref>] [--pr <n>]` | Lists decisions cited in the changed code whose tip has no unrevoked acceptance: key, set, statement, citing symbols. Reads citations from the analyzers' report output or by scanning `typeof(` citations in the diff |
| `decision accept <key>… [--scope version\|class:<ref>] [--commit]` | For each key: builds the acceptance node, canonicalises (§7), signs with SSH, writes acceptance and sidecar, re-exports. Refuses non-interactive invocation, refuses if the holder lacks `accept-decision`, refuses a software key when policy requires `-sk`, refuses an agent-reported unconfirmed key |
| `decision verify [--export] [--signatures] [--all]` | File gate, graph stage, L-class checks against git, signature verification with `-Overify-time`, export freshness. Exit code is the CI gate |
| `decision export --format ntriples --namespace <ns> [--out docs/decisions/<ns>.nt]` | Deterministic, sorted N-Triples of the namespace: decisions, versions, acceptances with signature references, policy and `allowed_signers` snapshot; only exported decisions when `--exported-only` |
| `decision import --from docs/decisions/*.md` | One-time import of interim set files: keys, statements, and an allocation derived from how each key is cited (#65). Interim `accepted-by` lines are **not** imported as acceptances; the holder accepts the imported versions at import, through `accept --batch`. Deletes the Markdown on success |
| `decision policy set|show` | Edits the namespace policy; the change itself requires acceptance by a current holder |
| `decision identity add|rotate|revoke` | Files a key-binding entry (principal, key, `namespaces=ledger-accept@<ns>`, validity window) as an accepted act; `allowed_signers` is regenerated from these entries and held byte-identical by `verify`. Revocation closes a key's window; the entries stay append-only |
| `decision show <key>` / `decision list [--unaccepted] [--exported]` | Inspection |

## 6. Review loop (the user story this PRD exists for)

1. A PR arrives red: CS0618 on every citation of an unaccepted decision.
2. `gh pr checkout n && decision review --pr n` prints the keys, statements and citing symbols.
3. The holder reads, then `decision accept Key1 Key2 --commit`, touches the hardware key per acceptance (or confirms per key for software keys), and pushes.
4. `decision verify` in CI confirms signatures and export freshness; the generator reads the export; the PR goes green.
5. Rejected keys: the holder tells the author which; the author takes the alternative and removes the citation. Nothing in the CLI accepts on anyone's behalf.

Target: a holder accepts twenty decisions in under five minutes without opening an editor.

## 7. Signatures (open-source scheme: SSH)

- Canonical form: the closed payload of the entity — for an acceptance `{decision, version, actor, at, scope, expires_at}`, for a revocation `{revokes, actor, at, reason}` — canonicalised by the format's canonical-JSON law (`ledger/spec/ledger-protocol.md` §4.3: normalised strings, absent keys omitted, keys code-point sorted, no insignificant whitespace), under a domain-separation prefix per entity (`ledger.acceptance.v1`, `ledger.revocation.v1`), UTF-8. The signature never covers a node's whole triple set: a node's triples are a read model and may gain triples; the payload is fixed at creation. Every payload field is exported as a triple, so a consumer rebuilds the signed bytes from the export alone. *(Amended 2026-10-01; was RDFC-1.0 of the acceptance node's triples.)*
- Signing: `ssh-keygen -Y sign -f <key> -n ledger-accept@<namespace>` over the canonical bytes. Sidecar file `<acceptance-id>.sig` (or `<revocation-id>.sig`) in SSHSIG format.
- Identity binding: `allowed_signers`, one line per principal and key: `mailto:… namespaces="ledger-accept@<ns>" valid-after=… valid-before=… <keytype> <key>`. The file is derived from key-binding entries and never edited by hand (§0 item 11).
- Verification: `ssh-keygen -Y verify -f allowed_signers -I <principal> -n ledger-accept@<ns> -Overify-time=<prov:generatedAtTime>`.
- Time: as written in the acceptance, backed by the landing commit's date; no timestamp authority in the open source.
- Hardware keys: `-sk` recommended; required per namespace by policy.
- Seam: `ledger:signatureScheme` is extensible. Other schemes (DSSE/in-toto, Sigstore, X.509, eIDAS) are verifiers registered by name; the open CLI ships `ssh` signing and verification, `dsse` verification, and `none` for pre-v2 stores where policy lists it (§0 item 10).

## 8. Integration with the analyzers

- The generator reads `docs/decisions/<ns>.nt`; when both `.nt` and interim `.md` exist for a namespace, DDGEN reports it.
- Export shape the reader relies on: `ledger:set` is the IRI `<urn:ledger-set:<id>>`, and the reader takes the set id from its local part; a node carries several `rdf:type` values (`ledger:Decision` and `prov:Entity`, …) and the reader keeps all of them; a revocation is its own `ledger:Revocation` node with `ledger:revokes <acceptance>`, and an acceptance is accepted-and-unrevoked when no `ledger:Revocation` revokes it.
- `ledger:key` → nested type name; `ledger:exported` → citable cross-namespace; unrevoked acceptance on tip → citable in release builds; no decision-level retirement in v2, so `[Obsolete(error: true)]` has no ledger source until a later format change.
- The report's citation projection is emitted as `ledger:Citation` N-Triples and may be committed under `docs/decisions/<ns>.citations.nt`; `decision review` reads it when present.

## 9. Non-goals

- Any network service, hosted acceptance, or web UI.
- DSSE signing, Sigstore, CA integration, timestamp authorities, eIDAS: seams only. DSSE *verification* ships in the open CLI (§0 item 10).
- Decision-level retirement (open item; tracked for v3).
- Editing hashed files in place; every change is a new entity.

## 10. Acceptance criteria

- Varve's `docs/decisions/*.md` imports, exports, and the generated decision types are byte-identical before and after; the markdown is removed.
- `decision accept` on a hardware key produces an acceptance that `ssh-keygen -Y verify` validates independently of the CLI.
- An acceptance signed by a key not in `allowed_signers`, or outside its validity window, or with the wrong SSHSIG namespace, fails `decision verify`.
- `decision accept` run without a TTY, or by an identity without `accept-decision`, or with a software key under a `-sk` policy, exits non-zero and writes nothing.
- A committed export edited by hand fails `decision verify --export`.
- Two live decisions with the same key in one namespace fail the file gate (and the graph-stage cross-check).
- A revoked acceptance's signature still verifies; the revocation is a separately signed entity.
- Supersession carries the key; the generator's output for the citing repository compiles unchanged and the report shows the diverged cited version.
- The whole flow runs offline.

## 11. Open questions

- Decision-level retirement: a decision explicitly withdrawn with no successor needs a representation; until then the analyzers' error-level obsolete has no source.
- Threshold acceptances (two holders) per set: policy supports it; `accept` writes one acceptance per holder; the generator treats the decision as accepted only when the threshold is met. Confirm the generator reads the policy.
- Whether `review` should rely on the report's projection (accurate, needs a build) or a diff scan (fast, approximate). Proposal: diff scan by default, projection when present.
- Imported acceptances with scheme `none`: grandfathered forever, or re-signed in a one-time ceremony. Proposal: policy decides per namespace; default grandfathered with a visible flag.
