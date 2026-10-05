# Signing: the 2026-08-11 rulings beside the ledger CLI PRD §7

For a ruling on issue I4. On the left are the five rulings recorded in `docs/decision-ledger-prd.md` §4.5 and milestone L6 (principal: Emil, 2026-08-11; no code was written for them). On the right is `docs/ledger-cli-prd.md` §4 and §7, **as amended 2026-10-01**. The last section covers `L001`, which the import story depends on.

> D5 to D9 were ruled the same day; see "D5 to D9 (ruled 2026-10-02)" below.
>
> **All ruled 2026-10-02 on #65** (now closed): namespace policy (D1); a sidecar with the inline field retired (D2); `L011`/`L012` kept, new classes from `L013` (D3); `ssh`, then `dsse` verification, `none` for pre-v2 stores only, `webauthn` later (D4); the trust root as a governed projection; `L012` as a review trigger with an optional deadline set by policy. The ruling text is in `docs/ledger-cli-prd.md` §0, items 7–12. The comparison below is the record the rulings were made on.

## Already ruled (2026-10-01), shown for reference

| | 2026-08-11 | PRD as amended | Status |
|---|---|---|---|
| What is signed | **Ruling 1.** "a canonical payload — decision id, version hash, actor identity, signing timestamp", never the commit | `{decision, version, actor, at, scope, expires_at}` under the canonical-JSON law, prefix `ledger.acceptance.v1`. A revocation is its own signed payload (`{revokes, actor, at, reason}`), and an acceptance is immutable | **Ruled.** It keeps ruling 1's shape and its "not the commit", and adds scope and expiry, so a class-scoped acceptance cannot be re-scoped under its signature. Confirm that this supersedes ruling 1's field list. |

## The four disagreements named in I4

**D1. When a signature is required.**

| 2026-08-11 (ruling 3) | PRD §4, §5, §7 |
|---|---|
| "Tier-gated, not universal. A signature is required above the tolerance floor: T2 acceptances are signed; T1 keeps today's claimed identity." The up-only floor means the requirement "cannot be shopped out". | Every acceptance is signed. Policy is per namespace: allowed schemes, required key types (`-sk`), and a threshold "per set or namespace". |

The two compose if policy is the mechanism and tier is its predicate ("signed at effective tier ≥ X"). What still needs ruling:
- whether policy may require signatures **below** T2, which the PRD does but ruling 3 does not allow for;
- whether a namespace policy may relax the tier rule, which ruling 3's "cannot be shopped out" forbids.

Today none of this repository's 91 acceptances is signed. Both sets have a T1 floor, and five filed versions override to T2. Under the PRD as written, all 91 would need re-signing or grandfathering. Under ruling 3, only acceptances of T2-effective versions would.

**D2. Where the signature lives.**

| 2026-08-11 (ruling 1) | PRD §4, §7 |
|---|---|
| "stored detached, in the `signature` field L0 reserved" | A sidecar file `<acceptance-id>.sig` (SSHSIG), with `ledger:signature` and `ledger:signatureScheme` on the node |

A sidecar keeps armoured blobs out of hashed-adjacent YAML and lets `ssh-keygen -Y verify` read the file directly (PRD §10). The reserved field avoids a new file kind and a pairing rule (an acceptance without its sidecar, or a sidecar without its acceptance). Either one keeps the version hash unchanged. The ruling should also say what the inline field holds under a sidecar regime: empty, the sidecar's digest, or its path.

**D3. Gate class numbering.**

| 2026-08-11 | PRD §4 |
|---|---|
| `L011` means "a required signature is absent or invalid against the trust root as of its signing timestamp". `L012` means "acceptances exist under a since-revoked key", which is a review trigger. Ten classes become twelve. | New gates: key syntax, key immutability, key uniqueness (all file gate, per the 2026-10-01 ruling); signature-verifies-against-policy; exported-needs-acceptance (at export time). It also adds revocation signing and revoker role (ruling 3, 2026-10-01). |

The open questions:
- Do `L011` and `L012` keep their reserved meanings, so that the key classes take `L013` onward?
- Or are the numbers reassigned in landing order?
- Does a revocation's missing or invalid signature fold into `L011`, or get its own class?

Each new class is a format-spec amendment by the `L010` mechanism, and the closed-count tests change with it.

**D4. Which signature formats ship.**

| 2026-08-11 (ruling 2) | PRD §7, §9 |
|---|---|
| "Formats are git's own — `gpg.format`: `openpgp` \| `ssh` \| `x509`." X.509 is "the bridge to the org product, where Entra-issued certificates make dev-side and org-side identity one mechanism". | It ships `ssh`, plus `none` (for import only). Other schemes are "verifiers registered by name". CA integration is a non-goal. |

The PRD's registry seam can hold all three. The decision is whether `openpgp` and `x509` are in the open-source CLI's v2 or are seams only. The PRD's §9 non-goal ("CA integration… Seams only") puts X.509 outside the v2 scope that ruling 2 included.

> **Amended 2026-10-04 (PR #89 review).** `none` is exclusive. A policy lists `none` alone or not at all; a policy listing `none` with another scheme is a schema fault, and `ledger policy set` refuses it. `none` means governed and unsigned: the namespace is role-checked, and no signature is required. This amends D4's "`none` for pre-v2 stores only" reading, under which `[ssh, none]` could be read as "signed, optionally".

## D5 to D9 (ruled 2026-10-02)

The positions, reasons and the walk through Session A's twelve questions are in `docs/signing-rulings-2026-10-d5-d9.md`. That page holds the positions and reasons; these notes record what was ruled, and win where the two differ.

**D5. Is the role checked at verify, or only in the verb?**

> **Ruled 2026-10-02: B, all three parts.** (a) `verify` evaluates the role as of the act (`A006`); (c) the position rule replaces the namespace switch — both land in #70. (b) signed grants and grant acceptances is its own issue.

**D6. What orders an act against a key close or a grant revocation?**

> **Ruled 2026-10-02: C**, landing order on the first-parent history and `at`. This supersedes D3's (and PRD §0 item 12's) "dated before the close": an act is before a close only if it is dated and landed before it.

**D7. What authenticates a principal's first key?**

> **Ruled 2026-10-02: B.** The genesis holder vouches for a principal's first key. `rotate` and `revoke` stay as landed.

**D8. `at` in the policy payload.**

> **Ruled 2026-10-02: A.** The policy payload gains `at`, added to #70's format list.

**D9. Every act names the grant it is made under.**

> **Ruled 2026-10-02: B, all parts, all in Session B.** `--as` is required only when more than one grant qualifies. Fewest-claims is enforced in the verb only. `under` in the payloads supersedes ruling 4's closed lists (PRD §0 item 4).

**Session A close-out §3** (`docs/sessions/2026-10-session-a.md`): 1 (a). 2 (a) with D5 (c) now, and (b) as an issue after Session B. 3 (a). 4 (a); a hierarchy, if ever, is a new scope form. 5 (a), on the grant the act names. 6 (a). 7 (a), by covering scope within one role, per D9 (e). 8 write-once, held by `verify`. 9 gate it. 10 (a). 11 no ruling; an issue for `rotate-genesis`. 12 no new class.

## Two further differences the side-by-side surfaces

- **The trust root.** Ruling 4 makes allowed signers "a set of decisions [that] lives in the ledger", with "an explicitly-named **genesis** entry, self-attested and conspicuous". The PRD keeps a plain `ledger/allowed_signers` file seeded by `init` "with the initialising identity", whose changes "are themselves accepted decisions in the policy set". Both govern changes. The difference is that the PRD's genesis is an implicit seed at `init`, while the ruling requires a named one.
- **Revoked keys.** Ruling 5 says validity is judged "at signing time", and a key revoked later produces a finding (`L012`), "never auto-invalidation". The PRD also verifies at signing time (`-Overify-time=<prov:generatedAtTime>`), but has no key revocation at all: rotation "adds a new line, never edits an old one", so a compromised key cannot be closed. The signing time is self-asserted by the key holder, which ruling 5's review trigger partly covers and the PRD does not.

## What `L001` protects (the import story)

> **Ruled 2026-10-02 on #65.** Options 3 and 4 are rejected. The ruling is to allocate each imported decision by what enforces it, derived from how it is cited:
> - cited from an analyzer rule: `constraint` with `analyzer:<rule>`;
> - cited only through `[DesignDecision(Scope=…)]`: `escaped`;
> - cited as affirmative design behind a human merge gate: `judgment`;
> - not cited: left unallocated, with `L001` failing on it.
>
> Interim acceptances are not imported; the holder accepts at import. The text below is the analysis the ruling was made on. The ruling itself, and #73's scope, are on the issues.

`L001` fails when a decision's latest version has no `allocation` (`ledger-core/src/verify/disposition.rs` `unallocated`). It protects the ledger's founding claim (`decision-ledger-prd.md` §2.1): for a declared assurance level, specification demand is constant and **fully allocated** across four stores:

- `constraint`: an extra-actor (analyzer or compiler) decides before the act;
- `criterion`: a test or telemetry decides after it;
- `judgment`: a named accountable human decides during it;
- `escaped`: priced and accepted, with exposure, acceptor and review date.

"Total never shrinks… Silent escape is not [legitimate]." A decision with no allocation is demand that nobody holds: governing, recorded, and enforced by nothing. That is exactly the silent escape the format exists to make visible.

The gate fails on it rather than reporting it as status for a reason. Leaving a decision unassigned does not reduce demand (the `L001` message cites §2.1). It runs under **readiness** (blocks produce), not only completeness, because starting work with unallocated decisions means building against rules nobody discharges.

**What this means for importing Varve's 513 decisions:**

1. **Import unallocated.** The import is honest, but CI fails `L001` 513 times until someone allocates each decision. Each allocation is a new version, so it moves the hash, and the imported acceptance then signs a superseded version.
2. **Import with an allocation chosen by the importer.** This fabricates the allocation claim, and every imported acceptance signs content its acceptor never saw.
3. **Rule a default allocation for the class of decision.** For example, `constraint` with discharge `analyzer:DDGEN`, for decisions that exist to be cited by generated types. The generator and analyzers are then the extra-actor that enforces the citation. This is the only option where the acceptance and the allocation describe the same content. It needs a ruling that this discharge is real, not a label.
4. **Rule a new, named state.** For example, "allocation pending" declared by set policy, which would be a format amendment to `L001` itself.

`ground: uncharacterised` on a set does not exempt it. `L001` judges every latest version.

## Correcting a landed file's `format:` declaration (ruled 2026-10-05, #81)

> **Ruled 2026-10-05 (principal), option 1 of #81.** A landed log file's `format:` declaration may be corrected. It may only be raised, only to the lowest format the file's content needs, and nothing else in the file may change.

The case: spec §3.7 makes a lower-format file carrying `revisit_if` a schema fault, the loader had no row for it, and three of this repository's change-sets carried the field under `format: 1` (#81 listed four; the fourth, `01KZX70S86QGXVCA5GW5WSY6XA`, names `revisit_if` only in its text and needs format 1). `format` is not hashed (it is a `ChangeSet` field, not a `VersionRaw` one) and is not an entity under the landed-entity rule (`ledger-core/src/landed.rs`), so raising it moves no digest and touches no acceptance. Options 2 (strike the fault from §3.7) and 3 (a pinned exemption in the gate) were not taken. The rule is in spec §3.10.7; the record of the three corrected files is in `docs/ledger-format-migrations.md`.

> **Ruled 2026-10-05 (principal), on #91.** Declaring a higher format than the content needs is not a fault. A writer declares the lowest format its content needs (`format::needed_for`); a file that declares less than it needs is a schema fault; a higher declaration is not. Recorded in spec §3.7. This leaves the correction rule above as it stands: a correction of a landed declaration still raises it only to the lowest format the content needs.
