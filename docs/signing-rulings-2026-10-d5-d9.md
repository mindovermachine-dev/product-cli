# Rulings D5 to D9, and Session A's twelve questions, 2 October 2026

Status: ruled by the principal, 2 October 2026. Every recommendation on this page was adopted. On D9: `--as` is required only when more than one grant qualifies; fewest-claims is enforced in the verb only; all parts land in Session B. `docs/signing-rulings-2026-10.md` holds the ruled positions and wins where the two differ.

Sources: the project bundle, and the Session A session's answers about PR #80, with file paths and symbols as it reported them. The repository itself was not read for this page.

D5 to D9 come out of Session B step 1 (#70). D5 depends on D6, because "as of the act" needs a definition of when an act happened. The last section goes through §3 of `docs/sessions/2026-10-session-a.md`.

## D5: is the role checked at verify, or only in the verb?

### What the documents and PR #80 say now

- `authorize` carries the role check for `accept`, `revoke` and the grant verbs. Its only callers are the write verbs, through `Author::authorized` (`ledger-core/src/author/authority_ops.rs:61`). The identity verbs use their own checks (D7).
- `verify` never calls `authorize`. About an acceptance's actor it checks `L006` (not a model) and `L009` (the introducing commit's author), and nothing about grants.
- `decision_authority` (`acceptance_ops.rs:60`) returns `Ok(None)` without checking when the namespace has no policy.
- `docs/ledger-format-v1.md:590–592`: "The gate-time counterpart over history — an acceptance whose actor held no such grant (A006) — waits on the decision-class → role mapping." No `A006` exists in code. The policy payload now has `accept_role`.
- `L011` will verify a signature against the derived `allowed_signers`. That proves which principal signed.
- Registry PRD §7: the server verifies an envelope "exactly as `ledger verify` does".
- Hard constraint 4: "the signature is the authority".

### The gap

A principal with a key binding and no grant writes an acceptance file by hand, signs it with `ssh-keygen`, and commits it. The verb never runs, so `authorize` never runs. `L011` passes. The same holds for a holder whose grant has been revoked.

### Positions

| | A: verb only | B: verify, as of the act |
|---|---|---|
| What `verify` proves | The named principal signed these bytes with a key bound to them | That, and the principal held the capability over that scope when the act was made |
| Hand-written acceptance by a bound, ungranted principal | Passes | Fails `A006` |
| Server envelope check (§7) | Needs a second role check outside `verify`, so the server and the CLI differ | One path: `verify` |
| Third party re-verifying a repository | Cannot tell an authorised acceptance from an unauthorised one | Can |
| Cost | None | `authorize` takes the act's position as a parameter; `A006` is wired; every historic acceptance is evaluated against the authority log |
| Main risk | "The signature is the authority" is true of identity only | A later authority entry must not orphan earlier acts; this needs D6 |

- (a) `verify` evaluates the role as of the act (`A006`).

### Second part: is the authority log itself signed?

B is only as strong as the grants it reads. Hard constraint 4 lists acceptance, revocation, review and key-binding as signed entities. Grants and grant acceptances are not in that list. If they are unsigned, a forged grant file naming the genesis holder as `granted_by` satisfies the role check.

Ruling 3 says grants and unavailability "follow the same pattern". If that means "signed", this part is already ruled and only its implementation issue is missing: neither session prompt has a step for it.

The grant payload (`ledger.authority-grant.v1`) carries no `at`, and a grant acceptance has no payload of its own. Signing them needs both, as additive format changes. That is the reason this part is its own issue.

The grant payload also names its role by id only. A role file edited after the grant changes what the grant gives without touching the grant's hash or its signature. Either roles are write-once and `verify` holds them so (Session A question 8), or the grant payload binds the role's content.

- (b) Grants are signed by the grantor, who must pass the same check for `grant-role`. Grant acceptances are signed by the holder. The chain ends at the genesis grant, anchored by `externalRef` and the self-bound key.

### Third part: acts before a namespace's first policy

Session A landed "a namespace without a policy is not role-checked; `init --namespace` opts a namespace in", so that the repository's 91 existing acceptances keep a valid path. Its close-out §3 question 2 offers: as built; refuse `accept` outright in an ungoverned namespace; a store-level switch.

| | A: namespace switch, as landed | B: position rule |
|---|---|---|
| Rule | No policy, no role check | An act before its namespace's first policy (D6) is not role-checked; every other act is |
| The 91 acceptances once their namespace opts in | Pass, because `verify` checks no acceptance against grants; no test covers it. Under (a), the genesis grant lands after all of them, so they are orphaned unless a second rule exempts them | Stand, as pre-policy acts |
| Opting out again | No verb removes a policy; a deleted policy file is not checked | Fails: a policy present at an earlier commit and absent at the verified one |
| What `verify` says of an unchecked namespace | Nothing; only `policy show` says it | Names it as unchecked |

- (c) The position rule replaces the namespace switch.

### Recommendation

B, all three parts.

1. `verify` is the only gate that sees a file nobody made with the verb, and it is the server's write check by definition. A rule enforced only in the verb is a convention.
2. `authorize` exists. The change is one parameter and one class, not a new mechanism.
3. `A006` widens from "orphaned acceptance" to "act without authority", covering revocations now and reviews in R1. That changes a class's meaning, so it belongs in the ruling.
4. No retroactivity: a grant revocation, supersession or unavailability affects only acts that are not before it under D6. Acts before it stand without review. A grant revocation ends authority; unlike a key close, it does not suggest that earlier acts were someone else's.
5. Staging: (a) and (c) in #70. (b) as its own issue and PR, landed before any real namespace has a second grant and before R1.5. Until (b) lands, a forged grant defeats (a), and the only defence is that grants are few and visible in review.
6. On (c): the switch needs a second rule for the day a namespace opts in, and the position rule is that rule. It is the reasoning behind `none` for pre-v2 stores, applied to the role instead of the signature. It does not touch D4: the signature requirement for pre-v2 acceptances stays with `none` in policy.

## D6: what orders an act against a key close or a grant revocation?

### What the documents and PR #80 say now

- D3 and "Keys closed later": an acceptance dated after the close fails `L011`; dated before, it is listed under `L012` and stays citable until the policy deadline, if any.
- Session B step 1: `-Overify-time=<at>`.
- The original PRD §7: time is "as written in the acceptance, backed by the landing commit's date". The Session B prompt did not carry the second clause.
- Ruling 5: commits are not in the export.
- Nothing resolves a landing commit today. `L009` is the only class that reads git, and it reads an author email found with `git log -S`. CI checks out full history.

### The gap

`at` is a payload field written by the signer. A key closed because it was compromised can still sign an acceptance with `at` before the close, which lands in `L012` and is citable. Under D5, a holder whose grant was revoked can backdate the same way. Closing a key or revoking a grant contains nothing if the party being contained writes the timestamp.

### Positions

| | A: `at` as written | B: `at` bounded by commit date | C: landing order and `at` |
|---|---|---|---|
| Rule | Compare `at` with the close | `at` must be no later than the landing commit's committer date and within a tolerance of it | An act is before a close or revocation only if it landed before it on the first-parent history and its `at` is earlier |
| Who controls the evidence | The signer | The committer, through `GIT_COMMITTER_DATE` | Whoever can rewrite the default branch's history |
| Backdated acceptance under a closed key | `L012`, citable | `L012` if the commit date is also forged | `L011` |
| Export-only verifier | Full check | Cannot check | Cannot check |
| Cost | None | A new landing-commit function | The same function on the first-parent line; `verify` on a branch must compute against the base |

### Definitions for C

- **Landing.** An entity's landing commit is the first commit on the first-parent history of the verified commit whose tree contains the entity. Entities on an unmerged branch land at the tip, after everything on the base.
- **Before.** An act is before a terminating entry (key close, grant revocation, grant supersession, policy change) when its landing commit is a strict first-parent ancestor of the entry's and its `at` is earlier. When both land in the same commit, `at` decides. Where an entry carries no signed `at` (a policy change until D8, a grant until D5 (b)), landing alone decides.
- **Enabling entries** (key-binding add, grant acceptance) cover an act when they are not after it: landing no later, `at` no later.
- `at` keeps the jobs a clock is for: key validity windows, unavailability intervals, expiry.

### Consequences of C

- It tightens D3: "dated before the close" becomes "dated and landed before the close". This is a superseding ruling, not a reading of the existing one.
- A backdated close still works as intended. Filing the close with `at` set to the time of compromise invalidates what landed in between, because both conditions must hold.
- Acceptances signed under a key that is closed while they sit on an unmerged branch fail `L011` at the merge result and must be re-signed. A rotation that is not a compromise is therefore `identity add`, let open branches drain, then `identity revoke`.
- Local `verify` on a branch and CI on the merge ref must agree, so `verify` needs the base.
- The export-only verifier checks signatures and key windows by `at` and nothing about order. That limit is stated in the format document. A committed export is held byte-identical by a `verify` that does check order, so an export from a green repository contains no act that fails it.
- "The policy in force before it" (D1) gets the same definition: the policy at the first parent of the change's landing commit. The `replaces` hash in the policy payload already orders policies among themselves.
- Assumption: the default branch's history is not rewritten. `verify` cannot prove that; the ruleset does, and any earlier clone detects it.

### Recommendation

C.

1. A and B both take the ordering evidence from the party being checked. The signer writes `at`; the committer writes the commit date; anyone who can push the acceptance can set both.
2. B costs the same new function as C and buys less.
3. First-parent order on a protected default branch is the only ordering in the repository that the signer does not author. It keeps hard constraint 3: git and nothing else.
4. The real cost is re-signing on open branches at a key close, and it falls where a re-acceptance is wanted anyway.
5. The export-only limit is a loss against A. A's full check is a check of a self-asserted field, so the loss is small.

## D7: what authenticates a principal's first key?

### What the documents and PR #80 say now

- Trust-root ruling: the genesis holder's first binding is self-bound; "later bindings are signed under the policy in force". That says which policy, not who.
- The closed capability vocabulary has no capability for binding a key.
- As landed (`ledger-core/src/author/identity_ops.rs`), unsigned until #70:
  - `add`: the principal is always the actor. The genesis holder files freely. Anyone else needs a live grant covering the namespace, with any role, and cannot file a namespace's first binding.
  - `rotate`: only the principal, on its own open binding.
  - `revoke`: the principal, or the genesis holder.
- These checks are in the verbs. `authorize` is not involved.

### The gap

Once bindings are signed, a `rotate` is signed by the key it closes, and a further `add` by a key the principal already has. A principal's first key has no earlier key to sign it. Self-signed, it proves possession of the key and nothing about the address. The grant precondition says the address may have a key, not that this key is theirs.

### Positions

| | A: as landed, self-signed | B: genesis vouches for the first key | C: any holder with `grant-role` vouches |
|---|---|---|---|
| A principal's first `add` | Filed and signed by the principal with the new key | Filed and signed by the genesis holder: `by` is the genesis holder, `principal` is the new holder | Filed and signed by a grantor |
| Further `add`, `rotate` | The principal's live key | The principal's live key | The principal's live key |
| `revoke` | The principal's live key, or the genesis holder | Same | The principal's live key, or a grantor |
| What links the first key to the address | Review of the commit | The genesis holder's signature | A grantor's signature |
| Who can bind a key to someone else's address | Anyone who can write the file and claim to be the actor | The genesis holder | Every grantor |

### Recommendation

B, with `rotate` and `revoke` kept as landed.

1. Binding a key to an address is the power to sign as that address. For a first key, something other than the key itself has to say so, and the root is the only authority that exists at that point in every store.
2. The payload already has both `by` and `principal`, so B needs no format change. The verb change is that the genesis holder can name a principal other than itself.
3. Delegating it later through a new capability is additive; taking it back would not be.
4. Rotation stays with the principal, so key hygiene does not wait on the genesis holder. Recovery from a lost or stolen key is the genesis holder's `revoke`, then a new first `add`.
5. A stolen live key can rotate itself. The remedy is the genesis holder's `revoke` with `at` set to the time of compromise, which D6 makes effective.
6. Whatever is ruled, the check moves into `verify` as well as the verbs, for the reason in D5.
7. Not included: proof of possession by the new key under B. A second `ssh` signature on one entity does not fit D2's one sidecar per scheme.

## D8: `at` in the policy payload

### What PR #80 says now

- `ledger.namespace-policy.v1` hashes `{id, namespace, schemes, require_sk, accept_role, reaccept_within_days, replaces, by}`. There is no `at`.
- `ledger.identity-binding.v1` and `ledger.revocation.v1` both hash `at`.
- #70 signs a policy change under the policy in force before it, and `ssh-keygen -Y verify` needs `-Overify-time` to check the signing key's validity window.

### Positions

| | A: add `at` to the payload | B: leave it out |
|---|---|---|
| Format | Additive: hashed when present, omitted when absent, required in files of the format #70 introduces. No existing digest moves | None |
| Verify time for a policy change's signature | The signed `at` | A timestamp outside the signed bytes, or no window check |
| D6 for policy changes | Landing and `at`, as for a key close | Landing only |
| Cost | A format change outside #70's current list | A policy change signed by a since-closed key cannot be distinguished by time |

### Recommendation

A, added to #70's list before Session B starts. Otherwise the session stops at that bullet under its own constraint: no format change outside the issues' lists.

## D9: every act names the grant it is made under

### The principal's position (2 October, in conversation)

Different roles for different scopes. The actor states the role an act is made under. The genesis role is for acts on the authority structure, not for day-to-day acceptance. Always the role with the fewest claims. Fallback order (question 7) is by covering scope.

### What PR #80 says now

- `authorize` (`ledger-core/src/authority/check.rs:99`) walks the actor's grants in `BTreeMap` order of grant id (`view.rs:27`) and returns the first that passes `judge`. With several qualifying grants the winner is the lowest ULID, in practice the earliest minted. Nothing ranks candidates by role, scope or order.
- The choice is not recorded. `accept` prints "under … (…)" to stdout (`acceptance_ops.rs:106–107`). `Acceptance`, `Revocation` and `Grant` have no field for the authorising grant.
- The escalation guard (`authority_ops.rs:243`) compares the role of that one result. An actor with two qualifying grants in different roles can be refused a grant the other role allows, and cannot choose.
- `init --namespace` defaults `--role` to `steward` (`cli_enum.rs:178`). `bootstrap` (`authority_ops.rs:105`) declares it with `Capability::ALL`, all seven, when no such role exists. It reuses an existing role of that id without checking its `may` set.
- Ruling 4: the acceptance payload is the closed list `{decision, version, actor, at, scope, expires_at}`, and the revocation payload is `{revokes, actor, at, reason}`.

### Positions

| | A: as built | B: the act names its grant |
|---|---|---|
| Which grant authorises an act | The actor's lowest-id grant that passes | The one the act names |
| Recorded | No | `under`, a grant id, in the signed payload |
| `A006` (D5) | Searches for any grant that would have passed | Checks the named grant |
| Escalation guard (question 5) | Can refuse wrongly | Compares the named grant's role |
| Audit | Cannot say whether an acceptance was made as steward or as architect | Can |
| Format | None | `under` added to five payloads; supersedes ruling 4's closed lists |

### Parts of B

- (a) **Record.** `under` in the payloads of an acceptance, a revocation, a grant, a policy change, and a key-binding filed by someone other than its principal. Hashed when present, omitted when absent. It is absent on the genesis grant, the genesis holder's self-bound binding, a principal's acts on its own keys, and pre-policy acts. The record names the grant, not the role: one principal can hold a role over two scopes or at two ranks, and the role is derivable from the grant.
- (b) **Choice.** `--as <role>` on the verbs. The candidates are the actor's grants that pass `judge` for this act and target. With one candidate, it is used and shown in the confirmation. With several, `--as` is required; within the named role the narrowest covering scope wins, then the lowest rank. The alternative is `--as` always required.
- (c) **Fewest claims.** The verb refuses a grant when another candidate's role has a strictly smaller `may` set.
- (d) **Verify.** Under D5, `A006` checks the named grant: held by the actor; live, accepted and available as of the act; its role `may` do the act; its scope covers the target; the fallback rule holds. A governed act with no `under` fails.
- (e) **Fallback order** (question 7). Within one role and per target: a grant may act only while no live, available grant of the same role at a lower rank covers the target (`covers`). Equal rank acts concurrently. `A003` is unchanged.
- (f) **Genesis role.** `init --namespace` declares the root role with the four authority capabilities (`grant-role`, `revoke-grant`, `declare-unavailability`, `rotate-genesis`) and without the three decision capabilities. The policy's `accept_role` is a different role, granted separately, to the genesis holder too if that person is to accept. `bootstrap` refuses an existing role id that lacks the root capabilities.

### Recommendation

B, all parts.

1. As built, which grant authorises an act depends on ULID order. That is an accident of minting, and it already makes the escalation guard refuse wrongly.
2. Now is the cheapest moment. `ledger.acceptance.v1` does not exist yet, so no acceptance digest has been computed without the field. No revocation exists on disk either.
3. On (b), `--as` only when there is a choice. The confirmation shows the grant and role before signing and the signature covers `under`, so the role is stated either way. Always requiring the flag costs one argument per batch and decides nothing when one grant qualifies. This is the principal's call.
4. On (c), the verb and not `verify`. Acting under a broader role than needed confers no power the actor lacks, so it is hygiene; D5 is about power the actor does not have. Failing history for hygiene is the wrong weight.
5. On (f): with all seven capabilities in the default genesis role, the first holder accepts as genesis from the first day, and under (e) an available genesis holder would suppress every fallback in that role. With the split, an override by the genesis holder is a visible entry: a grant revocation, or a grant of the accept role to themselves.
6. If #70 must stay small, the minimum is (a) on acceptances and revocations, (b) and (d). The other payloads can gain `under` with D5 (b); (c), (e) and (f) are verb and `init` behaviour and move no digest.

## Session A close-out §3: the twelve questions

Each has a default, the one built. "(a)" is always "as built". The recommendation is for the principal to rule on.

| # | Question | Recommendation | Reason |
|---|---|---|---|
| 1 | The selection dry run stays scriptable | (a) | It writes nothing. Hard constraint 4 is about producing an act, not about reading. |
| 2 | A namespace without a policy is not role-checked | (a), refined by D5 (c), now. (b) as the end state | The position rule defines what (a) leaves open. Once #70 has landed, `accept` in an ungoverned namespace should refuse; otherwise acceptances there stay unsigned and unchecked for good. (b) today breaks every fixture that accepts without grants, so it is its own issue after Session B. |
| 3 | Genesis is per store | (a) | It follows from scope `*`. The store is the trust boundary; namespaces with different owners belong in different repositories. |
| 4 | Namespace scopes match exactly | (a), and rule that a hierarchy, if ever wanted, is a new scope form | Reading `ns:hafeok` as a prefix later would widen every existing grant and, under D5, change the validity of past acts. |
| 5 | Escalation guard: a grantor grants only its own role | (a), on the grant the act names (D9) | It holds as long as roles are write-once (8). As built it compares the role of whichever grant `authorize` found first, and can refuse wrongly. The cost of the guard is that only the genesis holder can delegate something narrower. (c) is the principled form and can come later as a vocabulary addition. |
| 6 | Who changes policy | (a) | Policy sets the signature requirement, so changing it is a stronger power than granting. (c) would give it to every grantor over `*`. (b) is additive when delegation is needed. |
| 7 | Fallback semantics | (a), by covering scope, within one role | (b) makes unavailability and rank mean nothing. As built compares identical scope strings; the principal's position is covering scope. Defined in D9 (e): per target, through `covers`, equal rank concurrent. With the genesis role split in D9 (f), an available genesis holder suppresses nobody in the accept role. |
| 8 | Roles are write-once | Rule it as intended, and have `verify` hold it | 5 and D5 both rely on a role's `may` set not changing under an existing grant, and the grant names its role by id only. A role change is then a new role and new grants that supersede. |
| 9 | Grant acceptance does not require a TTY | Gate it | Session B step 0 does, with the other verbs that write an authority record. |
| 10 | The legacy revocation node has no `ledger:id` | (a) | No store or fixture on disk has one; they exist only as in-memory test data. The shape failure is correct, and the remedy for a store that has one is to re-file it as a `rev:` entity. (c) weakens a shape for data that does not exist. |
| 11 | The `no-genesis` limit withholds nothing yet | No ruling; an issue for `rotate-genesis` | With one genesis per store, policy changes at the genesis, and D7 as B, the genesis holder is a single point of failure. The rotation should exist before any store other than the principal's own depends on it. |
| 12 | Merge class for role files | No new class | With 8 ruled write-once, a change to a landed role file is a fault, and two branches declaring the same role id is a conflict a person resolves. The class it is reported under does not change the outcome. |

## What the Session B prompt assumes

The rulings match what the prompt assumes. No bullet marked [D5] to [D9], [Q7] or [Q8] is to be skipped.
