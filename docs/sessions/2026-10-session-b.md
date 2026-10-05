# Session B close-out — signing and the R0 inbox

2026-10-05. Order worked: step 0 (#85) → step 1 (#70) → step 2 (#86) →
step 3 (#79). One pull request per step.

## 1. What landed

| Step | Issue | Pull request | Landed |
|---|---|---|---|
| 0 | #85 | #88 (merged) | Session A's carry-overs: the non-interactive refusal on every verb that writes an authority record, the D9 root role and fallback order, and the grant-scope test. The PR description is the record. |
| 1 | #70 | #89 (merged) | Spec v1.8 / format 7: signed bytes, `ledger.acceptance.v1`, `under`, `at` in the policy payload, sidecars, `ssh` sign/verify, `dsse` verify, exclusive `none`, `L011`, `L012`, `A006`, landing per entity (D6), immutable landed entities (`L007`), D7 in one function, `--as` (D9), and the export-only verifier. |
| 2 | #86 | #90 | `accept --batch <file> [--repository] [--branch]` and the `ledger.acceptance-batch.v1` selection file (format §3.10.9). |
| 3 | #79 | #92 (stacked on #90) | `ledger inbox list|accept`, the named-graph dataset (`product_core::pf::sparql_dataset`), and graph shapes run per named graph. |

**Issue text amended.** #82 now also covers signing grant revocations (4
October). No other issue body was edited.

**Differences from Session A's report, found in the code.**

- **The derived `allowed_signers` was unreadable by `ssh-keygen`.** It separated options with spaces, and OpenSSH needs commas. It was fixed in #89 (`signers::derive_from`). No committed store carried one.
- **`authorize` has two more callers** than the report named: `role declare` (`Act::DeclareRole`), and `unavailable` filed on someone else's grant (`Act::DeclareUnavailability`). Neither records `under`, because neither has a hashed payload (format §3.10.1).

### Ruled during the session

**On 4 October 2026, with their callers recorded:**
- `acceptor` carries only `accept-decision`.
- `policy set` refuses the genesis role as the accept role.

**On 4 October 2026, in the #89 review:**
- **`none` is exclusive.** A policy listing `none` with another scheme is a schema fault, and `policy set` refuses it. `none` means governed and unsigned; this amends D4 (`docs/signing-rulings-2026-10.md`).
- **`L012`** as built: a review item until the policy's deadline, a finding after it.
- **Agent-held software keys** are refused, as built.
- **Grant revocations stay unsigned** until #82.
- **DSSE key material** stays an open question (§3).

**On 5 October 2026, in the #89 review:**
- **Key closes are terminating**, and the change stays: under `[none]`, it is the only check that refuses a self-filed key dated back inside a closed window. The test is `closed_key.rs` `under_none_a_principals_own_add_dated_before_their_keys_close_is_refused_and_never_trusted`.

### Interpretations made in the code, for confirmation

Each is stated in its PR.

- `L012` fails only after the deadline (ruled as built).
- A policy change filed below format 7 is now a schema fault, so "format-6 policy changes are unchecked" no longer applies.
- **How `accept` is authorised.** Only grants of the policy's `accept_role` count (`decision_authority`, `acceptance_ops.rs`), so on `accept` the candidates are all in one role. Options:
  - (a) keep this;
  - (b) count any role whose `may` holds `accept-decision`, with `accept_role` as a filter only when a policy sets one. Under (b), D9 (c) would apply to `accept` too.
- **The batch file's rows carry an optional `branch`**, beyond #86's (repository, decision, version). One inbox sitting spans several branches of one repository, and each checkout signs only its own rows.

## 2. Format changes

**Classes:**
- `L011` (required signature absent or invalid) and `L012` (acceptance under a since-closed key, past the deadline), both in the file gate.
- `A006`, in the graph stage.
- `L007` extended to cover landed entities.
- The closed counts went from 13 to 15 (`ALL_CLASSES`) and from 8 to 9 (`ALL_GRAPH_CLASSES`).

**Hash prefixes:**
- new: `ledger.acceptance.v1`;
- reused: `ledger.acceptance-manifest.v1`, for the batch manifest under the selector `"batch"`;
- new form tag, not a hash prefix: `ledger.acceptance-batch.v1`.

**Fields:**
- `under` on acceptances, `rev:` revocations, grants, policies and key bindings;
- `at` in the policy payload, always hashed. A policy is a format-7 entry.

**Files:**
- sidecars, `.decisions/sig/<ulid>.<scheme>.sig`;
- the batch selection file, a hand-off artefact that is never committed.

**Digest proof:**
- `tests/digests.rs` still re-derives every stored version digest.
- `authority/payload_tests.rs` pins the grant, genesis-grant, binding and revocation digests against `main` at `88b3de1`.
- The format-6 policy pin was dropped, because the policy payload now always carries `at`.

## 3. Questions for the principal, with options

1. **Passphrase-less private key file.** `refuse_key` accepts a software private key file on disk with no passphrase. Only an agent-held software key is refused. Options:
   - (a) keep this: possession of the file is the control, and `require_sk` is the stronger setting;
   - (b) refuse a key that `ssh-keygen -y -P "" -f <key>` opens;
   - (c) warn only;
   - (d) refuse every software key for signed acts.
2. **DSSE.**
   - **Where the key comes from.** Today it is the signer's `ssh-ed25519` key bindings. The alternatives are a key the policy names (a format change) or the hosted service's published key set.
   - **Which key types the verifier accepts.** Only ed25519 is implemented, and the hosted signer's keys may not be ed25519 (for example ECDSA P-256 from a cloud KMS, or RSA).
   - **The payload type.** #89 defines `application/vnd.ledger.signed-bytes.v1`, while the platform PRD describes an in-toto statement. Options: keep signed-bytes; adopt in-toto with the signed-bytes digest as its subject; or accept both.
3. **Registry PRD §8 needs amending.** It says "every branch with an open PR". Git does not record open PRs, so R0 indexes the default branch plus every remote-tracking branch whose committed export differs from the default branch's. A PR number is only a label, read from `refs/pull/<n>/head` when that ref is fetched and matches.
4. **Citing symbols and the firing rule are not in the export.** The inbox reads them from `based_on` pointers by scheme: `symbol:` and `code:` for citations, `rule:` and `analyzer:` for the rule. Should R1 give them their own predicates?
5. **An acceptance that fails `L011` against the base cannot be affirmed by re-accepting it.** The failing acceptance stays in the log. The inbox lists it (`fails-on-base`) but leaves it out of the batch. Should the remedy be a revocation, or is it R1's?
6. **A holder who rotates keys on the default branch cannot sign on branches cut before the rotation.** Those branches don't know the new key, so `accept --batch` refuses there until the branch merges the default branch. The inbox reports this per branch. Should `accept --batch` read the base's bindings (`--base`), as `verify` does?

### Proposals (from the principal, 5 October; not this session's work)

1. **A property test.** For an act at a fixed landing position, an earlier `at` never improves the verdict against key closes, grant revocations or policies.
   - Three review rounds on #89 each found a hole of that shape: an appended entity landing with its file; a policy dated after a backdated act; a key close treated as enabling.
   - Unavailability windows are the known exception.
2. **`init --namespace` does not bind the genesis holder's key**, so the first self-bound binding to land for that address is the one trusted. Binding the key in the same act as `init` would close that window.
3. **A namespace's first policy is unsigned**, even in a store where the genesis holder already has a trusted key.
4. **Under `[none]`, no signature is required.** From the code:
   - **These checks still hold:**
     - D7 (`may_file` runs before any policy is read, in `trust_bindings`);
     - `A006`;
     - `L007`, landed entities immutable;
     - the schema and structure faults, `none` exclusivity included;
     - the `[SIGNERS]` match of the committed `allowed_signers`;
     - the version gate `L001`–`L010`, `L013` and `L014`;
     - the graph shapes;
     - a sidecar that *is* present, which is still verified (`L011` if invalid).
   - **These do not:**
     - `L011` for an absent signature;
     - `L012`, which arises only from a signature;
     - the signature chain over key bindings: a binding D7 admits is trusted unsigned, and only D7-admitted bindings reach `allowed_signers`;
     - the signature on a policy change;
     - the `ssh-keygen` preflight, unless an `ssh` sidecar exists.

## 4. Items the session prompt asks for

**Time for twenty acceptances through the inbox.** On the fixture (three repositories, twelve branches, twenty proposed decisions; `ledger-cli/tests/inbox.rs`):
- 11.0 to 11.5 s from the confirming `ledger inbox accept --all --confirm` to the end;
- **one confirmation** and **twenty signatures**, one sidecar per acceptance;
- per branch: a fetch, a worktree, `accept --batch`, the export, a commit, a push and `verify --base`.

That leaves the five-minute target for reading time. The listing run before it takes a few seconds more, because it creates a worktree and runs `verify --base … --json` per branch to find re-acceptance items.

**What R0 leaves room for, or blocks, in R1 and R1.5.**

- **`ledger:Review`, reject and changes-requested (R1).**
  - The batch file's rows are the natural carrier: a `verdict` column, under a new form tag (`ledger.acceptance-batch.v2`), since the manifest covers the rows.
  - `accept --batch` would become a review verb over the same pinned selection.
  - Nothing in R0 blocks this. The index already has the version, its predecessor and the proposer per item.
- **Commits written by a server (R1.5, hosted).**
  - `L009` reads the introducing commit's author and requires it to be the acceptance's actor.
  - The inbox commits as the holder, so it passes. A server-written commit (a GitHub App) would fail `L009` unless it is authored as the holder, and an author field the server sets freely would empty `L009` of meaning.
  - Now that acceptances are signed, the proposal for the principal is that `L009` stand down for an acceptance whose required signature verifies: the signature, not the commit author, is the authority (registry PRD §7, "writes are self-authenticating").
- **Browser triage hands off a batch (R1.5).** The batch file is that hand-off, and `accept --batch` already signs it. The open server needs only to produce the file, with the same manifest law.
- **Incremental index (R1.5).** R0 rebuilds the index every invocation, so no rebuild-equals-incremental test was written (#79's last bullet). It belongs with the first maintained index.

**Bullets left out because a ruling was not recorded:** none. D5–D9, Q7 and Q8 were all recorded before the steps that needed them, and no Session A question still open blocked a bullet.

**What signing grants and grant acceptances would touch (D5, second part; #82):**
- `ledger.authority-grant.v1` (`authority/payload.rs` `grant_map`) has **no `at`**. Adding it needs a format bump, and the format-6 grant digests are pinned.
- A new `ledger.grant-acceptance.v1` payload over `{grant, signs, actor, at}`, with no `under`.
- `signing/subject.rs` would sign grants (by `granted_by`), grant acceptances (by the holder), and revocations of grants (by the actor).
- `trust_bindings` ordering: grants are judged after the bindings they are signed with, and before the acts made under them.
- `A006` would use only signed grants. A forged grant would then fail `L011` and every act under it `A006`.
- The verbs `grant new`, `grant accept` and `grant revoke` would sign, with sidecars.
- The export would carry grant signatures, and the export-only verifier would rebuild them.

## 5. Tests

| Point | Workspace `cargo t` |
|---|---|
| `main` after #88 | 1,933 passed |
| #89 merged | 1,999 passed |
| #90 (step 2) | 2,010 passed |
| step 3 | 2,020 passed, 0 failed |

- **New suites:**
  - `signing.rs`, `trust.rs`, `immutability.rs`, `closed_key.rs`, `export_verifier.rs` (#89);
  - `batch.rs` (#90);
  - `inbox.rs` (step 3);
  - unit suites `landing_tests`, `landed_tests`, `filing_tests`, `choice_tests`, `payload_tests`, `signing/check_tests`, `signing/dsse_tests`, `batch_file_tests`, `inbox/index_tests` and `sparql_dataset`.
- **Terminals:** a PTY driven by `script(1)`, as in Session A. The terminal check is classified per verb in `commands/terminal.rs`, and `inbox accept --confirm` is on the terminal side.
- **A flaky-looking failure, root-caused.** Two signing tests dated a hand acceptance from the first key binding, which could precede the grant acceptance in the same second. They are re-dated after the second key's binding (#89).

## 6. Upstream list: `hafeok/decision-driven-analyzers`

Covers #80 and Session B. Nothing was filed upstream.

**Classes that can appear in the export and could not before #80:**
- `ledger:Role`, `ledger:Grant`, `ledger:GrantAcceptance`, `ledger:Unavailability`, `ledger:Availability`, `ledger:Revocation`, `ledger:KeyBinding`, `ledger:NamespacePolicy` (#80);
- `ledger:Signature` (#89).

**Predicates new since before #80:**
- `ledger:key`, `ledger:exported`;
- `ledger:may`;
- `ledger:role`, `ledger:holder`, `ledger:grantedBy`, `ledger:order`, `ledger:rank`, `ledger:limit`, `ledger:genesis`, `ledger:externalRef`, `ledger:supersedesGrant`;
- `ledger:grant`, `ledger:signsHash`;
- `ledger:from`, `ledger:until`, `ledger:basis`, `ledger:reason`, `ledger:ends`, `ledger:availableAt`;
- `ledger:revokes`;
- `ledger:bindingAct`, `ledger:principal`, `ledger:keyType`, `ledger:publicKey`, `ledger:closes`, `ledger:selfBound`, `ledger:mandate`;
- `ledger:requiresScheme`, `ledger:requiresSecurityKey`, `ledger:acceptRole`, `ledger:reacceptWithinDays`, `ledger:replacesPolicy` (#80);
- `ledger:under`, `ledger:signature`, `ledger:signatureScheme`, `ledger:signatureFile` (#89).

**Points for the reader:**
1. Predicates such as `ledger:id`, `ledger:hash`, `ledger:scope` and `ledger:namespace` now occur on non-decision nodes, so the reader must dispatch on `rdf:type`.
2. `ledger:revokes` can target a grant as well as an acceptance.
3. `ledger:revokedAt` and `ledger:revokedBy` no longer appear on acceptances.
4. Legacy revocation nodes (`urn:rev:legacy-<acc-ulid>`) carry no `ledger:id`.
5. An acceptance is citable only if `verify` is green. The export carries the signatures, but not landing order.
