# Signing: the 2026-08-11 rulings beside the ledger CLI PRD §7

For a ruling on issue I4. On the left are the five rulings recorded in `docs/decision-ledger-prd.md` §4.5 and milestone L6 (principal: Emil, 2026-08-11; no code was written for them). On the right is `docs/ledger-cli-prd.md` §4 and §7, **as amended 2026-10-01**. The last section covers `L001`, which the import story depends on.

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

## Two further differences the side-by-side surfaces

- **The trust root.** Ruling 4 makes allowed signers "a set of decisions [that] lives in the ledger", with "an explicitly-named **genesis** entry, self-attested and conspicuous". The PRD keeps a plain `ledger/allowed_signers` file seeded by `init` "with the initialising identity", whose changes "are themselves accepted decisions in the policy set". Both govern changes. The difference is that the PRD's genesis is an implicit seed at `init`, while the ruling requires a named one.
- **Revoked keys.** Ruling 5 says validity is judged "at signing time", and a key revoked later produces a finding (`L012`), "never auto-invalidation". The PRD also verifies at signing time (`-Overify-time=<prov:generatedAtTime>`), but has no key revocation at all: rotation "adds a new line, never edits an old one", so a compromised key cannot be closed. The signing time is self-asserted by the key holder, which ruling 5's review trigger partly covers and the PRD does not.

## What `L001` protects (the import story)

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
