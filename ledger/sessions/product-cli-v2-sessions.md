# Claude Code prompts: ledger v2 implementation in product-cli

Two sessions, run in `mindovermachine-dev/product-cli` after PR #77 (export, PRDs, rulings) and PR #78 (authority amendment) are merged. Each step is one PR that lands green on its own; no step starts until the previous PR is merged or its branch is the base. Read `CLAUDE.md` first and follow it for clippy, conventions and commit form.

Rulings are closed. `ledger/prd/ledger-cli-prd.md` §0 items 1–12 and the "Ruled" notes in `ledger/rulings/signing-rulings-2026-10.md` are not to be re-opened. If an issue body and a ruling disagree, the ruling wins; say so in the PR description and amend the issue. If implementing a step needs a decision the rulings do not cover, stop at that step, write the question into the close-out report with the options you see, and continue with the next step that does not depend on it.

Every PR: tests for every new class and verb (violating and conforming), the format document and migration note updated in the same PR, exports regenerated (`ledger export --format ntriples`) so `verify --export` passes, closed-count tests (`finding.rs`, `gate.rs` `fails_only_with`) updated, 100% of existing tests passing or changed with the reason in the PR description.

Amended 2 October 2026: Session B only. First for step 0 carry-overs, proposed rulings, key-binding signing, batch signing, branch discovery and the export-only verifier's input; then for what Session A landed in PR #80, as its session reported it with file paths and symbols. Session A is unchanged.

---

## Session A — format 5, interactivity, authority, revocation

Order: #67 → #71 → #69 → #66. #70 and #79 are Session B.

### Read first
`docs/ledger-format-v1.md`, `ledger/prd/ledger-cli-prd.md` (§0, §4, §5, §7), `ledger/rulings/signing-rulings-2026-10.md`, `ledger/spec/authority/` (all four files, as amended for ruling 3), `ledger/audits/audit-2026-10.md` §1–§4 and §9, then the four issue bodies. Then the emitter (`turtle.rs`), `acceptance.rs`, `verify/disposition.rs`, `format::needed_for`, `canon::put`/`put_set`/`domain_hash`, and the `L010` amendment mechanism, so you know where a new class and a new field go before writing either.

### Step 1 — #67: spec v1.6 / format 5
- `key` on `VersionRaw` (`^[A-Z][A-Za-z0-9]{0,63}$`, SCHEMA fault at parse), `exported` hashed as the string `"true"` when set, absent when false. Both hashed when present, omitted when absent; `CANONICAL_FORM` stays `v1`; prove with a test that every existing fixture digest is unchanged.
- File-gate classes by the `L010` mechanism: key immutability across `parent`/`merged_from`; key uniqueness among live decisions per namespace. Numbers: next free after `L012` (`L013`, `L014`); `L011`/`L012` stay reserved for Session B.
- SPARQL cross-check shape for uniqueness in the graph stage.
- Emitter: `ledger:key`, `ledger:exported`.
- Migration note: giving an existing decision a key is a new version and needs re-acceptance.

### Step 2 — #71: non-interactive refusal
- TTY check on stdin for `accept` (both forms) and `revoke`. No environment override, no flag. Keep the manifest step.
- Tests: move the `assert_cmd` tests that run `accept` to a PTY driver, or inject an interactivity predicate into the core and keep the real check in the binary. Pick one, state why in the PR. Add the negative test: piped stdin exits non-zero and writes nothing.

### Step 3 — #69: authority model
Start from `ledger/spec/authority/`; the vocabulary is the specification, the shapes are the graph-stage checks (`A003`, `A005`; `A006` deferred).
- File schema: `.decisions/roles/<id>.yml`; grants, grant acceptances, unavailability, availability, revocations and key bindings as log entries.
- Hashes: `ledger.authority-grant.v1`, `ledger.revocation.v1`, and a key-binding hash `ledger.identity-binding.v1`, all under the canonical-JSON law. Record each prefix in the format document.
- Verbs: `init --namespace`, role and grant verbs, grant acceptance, `identity add|rotate|revoke` (append only, never edit), declare and end unavailability, `policy` show/set.
- Namespace policy file with: required signature schemes (`ssh`, `dsse`, `none`), `-sk` requirement, the `accept-decision` role mapping, the optional re-acceptance deadline. A policy change is a log entry signed under the policy in force before it; without signing (Session B) it is a plain entry carrying the hash of the policy it replaces, and the check that it was signed under the old policy is listed as a Session B TODO in the close-out, not silently dropped.
- `allowed_signers` as a derived file: generated from key-binding entries, with `valid-after`/`valid-before` from `add`/`rotate`/`revoke`, held byte-identical by `verify` the way the export is. Genesis bootstrap: the genesis holder's first binding is marked self-bound and references the genesis grant's `externalRef`.
- The role check: does the actor hold a live, accepted, available grant whose role `may` do this act over this scope? Exposed as one function `accept`, `revoke` and the grant verbs all call. Test every arm: no grant, grant not accepted, unavailable, wrong scope, fallback limit.
- Emitter for every node; shapes wired as graph-stage classes with their `A` ids.

### Step 4 — #66: revocation as its own entity
- `<urn:rev:…>` node per the amended vocabulary; `rev:<ULID>` id; no triple ever added to an acceptance after creation (add a test that asserts the acceptance node's triple set is fixed across a revocation).
- `L006` extended to `Revocation.by`.
- Closed payload `{revokes, actor, at, reason}` under `ledger.revocation.v1`; role check from step 3.
- Regenerate exports. Note in the PR that Hafeok/decision-driven-analyzers#83 reads both shapes during transition.

### Session A close-out
Write `ledger/sessions/2026-10-session-a.md`:
1. PR list with what each landed, and any issue text you amended because a ruling won.
2. Format changes: every new field, class id, hash prefix, and the fixture-digest proof.
3. Questions that stopped a step, with options.
4. Session B TODOs you deferred (at minimum: policy-change-under-old-policy check).
5. Test counts before and after; PTY or predicate choice and why.

---

## Session B — signing and the R0 inbox

Order: step 0 (carry-overs) → #70 → `accept --batch` → #79. Prerequisite: Session A merged (PR #80) with its close-out, `ledger/sessions/2026-10-session-a.md`, on the default branch.

Five rulings that this session needs were proposed after the first version of this prompt: D5 (authority is checked at verify, as of the act; acts before a namespace's first policy are not role-checked), D6 (acts are ordered by landing, not by timestamp alone), D7 (who signs a key-binding), D8 (`at` in the policy payload) and D9 (every act names the grant it is made under; fallback order by covering scope; the genesis role carries no decision capability). They are closed only if `ledger/rulings/signing-rulings-2026-10.md` records them as ruled. If one is not there when you start, do not implement the bullets marked with it: implement the rest of the step and list the marked bullets in the close-out as blocked on the ruling. Session A's close-out §3 lists twelve questions for the principal; where one is still open and a bullet depends on it, treat it the same way. Never implement a proposal.

### Read first
Session A's close-out, `ledger/prd/ledger-cli-prd.md` §4 and §7, `ledger/rulings/signing-rulings-2026-10.md` (including D5 to D9 if ruled), `ledger/prd/decision-registry-prd.md` (§2, §5, §7, §8, §9 R0, §11), then #70 and #79. Confirm `ssh-keygen -Y sign/verify` is available in CI and in the dev container before writing code; if it is not, the first PR is the CI change.

What Session A landed, as its session reported it. Confirm each against the code in the first PR description. Where the code differs, the code is the fact and the difference goes in the close-out:
- `require_terminal` (`ledger-cli/src/commands/common.rs`) refuses non-interactive `accept <dec>`, `accept --set|--group --confirm`, `revoke` and `grant revoke`. Tests run under a PTY driver (`script(1)`); there is no injected predicate. The selection dry run stays scriptable.
- `authorize` carries the role check for `accept`, `revoke` and the grant verbs. Its only callers are the write verbs, through `Author::authorized` (`ledger-core/src/author/authority_ops.rs`); `verify` never calls it and checks no acceptance against grants. `decision_authority` (`acceptance_ops.rs`) returns `Ok(None)` when the namespace has no policy. The identity verbs do not call `authorize`: `bind`, `bootstrap_or_grant` and `closable` in `ledger-core/src/author/identity_ops.rs` decide who may file a binding, and the principal is always the actor.
- `authorize` (`ledger-core/src/authority/check.rs`) returns the first of the actor's grants that passes `judge`, in grant-id order. The choice is printed by `accept` and recorded nowhere. `init --namespace` defaults the genesis role to `steward` and `bootstrap` (`authority_ops.rs`) declares it with all seven capabilities.
- A grantor below the genesis grants only its own role, over its own exact scope (`covers`, `ledger-core/src/authority/check.rs`).
- Hash prefixes and payloads are in `ledger-core/src/authority/payload.rs`: `ledger.authority-grant.v1`, `ledger.revocation.v1` (`{revokes, actor, at, reason}`, one entity whether it revokes an acceptance or a grant), `ledger.identity-binding.v1`, `ledger.namespace-policy.v1`. The grant and policy payloads carry no `at`. There is no `ledger.acceptance.v1` yet.
- `domain_hash` (`ledger-core/src/hash.rs`) digests prefix bytes, `0x0A`, canonical bytes, in separate `update` calls. No function returns that byte sequence.
- `verify` has a `[SIGNERS]` stage for the derived `allowed_signers` and prints nothing about policy. `policy set` always files a successor with `replaces`; no verb removes a policy.
- `L009` is the only class that reads git. `introducing_author` (`ledger-core/src/blame.rs`) returns an author email found with `git log -S`; nothing resolves a landing commit. CI checks out full history.
- The export carries `KeyBinding` and `NamespacePolicy` nodes for the exported namespace (`restrict`, `ledger-core/src/graph/export.rs`).
- One genesis per store, not per namespace; policy changes are the genesis holder's act; a namespace without a policy is not role-checked and `init --namespace` opts it in.

Also establish, with symbols: whether `merge --resolve` can write an acceptance or revocation that neither side of the merge had.

### Step 0 — #85: carry-overs from Session A
One PR.
- Non-interactive refusal on every verb that writes an authority record: `init --namespace`, `role declare`, `grant new`, `grant accept`, `unavailable`, `available`, `identity add|rotate|revoke` and `policy set`, in addition to the four that have it. Through `require_terminal` and the same PTY driver; no environment override, no flag, no predicate. The verbs that file decisions (`declare`, `add`, `allocate`, `escape`, `revise`, `supersede`) and the derived-file verbs (`identity sync`, `reindex`, `merge --install`, `merge-driver`) stay scriptable. One test holds the closed list of verbs on each side, so a new verb has to be classified. A negative test per refusing verb: piped stdin exits non-zero and leaves `.decisions/` byte-identical.
- If `merge --resolve` can write an acceptance or revocation that neither side had, it refuses non-interactive use too. If it only chooses between existing entities, say so in the PR and leave it.
- [Q7] Fallback order by covering scope, as D9 (e) defines it: within one role and per target, a grant may act only while no live, available grant of the same role at a lower rank covers the target (`covers`); equal rank acts concurrently. Tests: a `fallback-1` over a set with a primary over `*` in the same role, available and unavailable; the same with the primary in another role; two grants of equal rank.
- [D9] `init --namespace` declares the root role with `grant-role`, `revoke-grant`, `declare-unavailability` and `rotate-genesis` only, and the policy's `accept_role` names a different role. `bootstrap` refuses an existing role id that lacks those four. Fixtures that accept as the genesis holder gain a grant of the accept role; say in the PR how many changed.
- Grant scope: add the missing test that a grantor whose scope is not `*` is refused a grant over any scope but its own (`covers`).

### Step 1 — #70: signed acceptances, revocations, key-bindings and policy changes
- Acceptance payload: `ledger.acceptance.v1` over the closed list `{decision, version, actor, at, scope, expires_at}` (ruling 4), plus [D9] `under`. No existing digest moves, because acceptances had none. Landed files are not rewritten; state in the PR whether the hash is stored or computed.
- Signed bytes: the byte sequence `domain_hash` digests, that is prefix bytes, `0x0A`, canonical bytes. Add one function that returns it; `domain_hash` and signing both call it; Session A's digest test proves no digest moved. The format document states the sequence byte for byte.
- [D9] `under`: the id of the grant an act is made under, in the payloads of an acceptance, a revocation, a grant, a policy change, and a key-binding filed by someone other than its principal. Hashed when present, omitted when absent; Session A's digest test proves no existing digest moved. `--as <role>` on the verbs: with one candidate grant it is used and shown in the confirmation; with several, `--as` is required, and within the role the narrowest covering scope wins, then the lowest rank. The verb refuses a grant when another candidate's role has a strictly smaller `may` set. The escalation guard compares the named grant's role. Tests: two qualifying grants in different roles with and without `--as`; the broader role refused; the guard no longer refusing a grant the named role allows.
- `ssh`: `ssh-keygen -Y sign -n ledger-accept@<ns>`; verify with `-f <derived allowed_signers> -I <principal> -Overify-time=<at>`. `<ns>` is the ledger namespace of the store that holds the entity. It is never a flag and never derived from the principal.
- `dsse`: verification only. An envelope at `.decisions/sig/<ulid>.dsse.sig`; verify against the public key material the policy names; signing refuses with a message naming the hosted service.
- `none`: valid only where policy lists it; the gate refuses a `none` acceptance in any other namespace.
- Sidecars at `.decisions/sig/<ulid>.<scheme>.sig`; the inline `signature` field required empty, gate refuses any value.
- Key-bindings are signed entities (hard constraint 4; trust-root ruling) under `ledger.identity-binding.v1`. The genesis holder's first binding is self-bound against the genesis grant's `externalRef`. A `rotate`, and an `add` by a principal who already has a live key, are signed by that live key. Where the policy in force requires a signature, an unsigned non-genesis binding fails the gate and is not written into `allowed_signers`. Tests: an unsigned binding for an existing holder's principal; a binding signed by a key that is not live.
- [D7] A principal's first binding, and who else may sign a `revoke`: as D7 rules, checked at verify as well as in the verbs, by the functions the verbs call (`bootstrap_or_grant`, `closable`). Tests: each of `add`, `rotate` and `revoke` by a signer D7 allows and by one it does not, including a hand-written binding file. Until D7 is recorded, do not choose a signer for a non-genesis principal's first binding: where policy requires a signature the gate refuses it, and the close-out says so.
- [D8] The policy payload gains `at`, hashed when present and omitted when absent, required in files of the format this step introduces. `-Overify-time` for a policy change is that `at`. Until D8 is recorded, do not add the field; sign policy changes without a key-window check on a time, and list it in the close-out.
- Policy change: Session A's deferred check. A policy change is signed under the policy in force before it, by the genesis holder as Session A landed it.
- Classes: `L011` required signature absent or invalid, including dated after the key's close; `L012` acceptances under a since-closed key dated before the close, a review trigger, with the policy deadline making them non-citable after it. Reader takes the latest valid acceptance of a version.
- [D6] Landing: add a function that returns an entity's landing commit, the first commit on the first-parent history of the verified commit whose tree contains the entity. Not `git log -S`: a pickaxe matches the id wherever the string appears. "Before the close" means before by landing and by `at`, as D6 defines both. An acceptance whose `at` precedes the close but which landed after it fails `L011`, not `L012`. Test: an acceptance signed with a closed key, `at` backdated, committed after the close entry landed. `verify` on a branch computes landing against the base so that it gives the result the merge would; state in the PR how (flag or default) and prove local and merge-ref results agree with a test.
- [D5] `A006`: every acceptance and every revocation is by an actor who, as of the act, held a live, accepted, available grant whose role `may` do it over that scope. Computed at verify by `authorize`, the function the verbs call, with the act's position (D6) as a parameter; nothing materialised. Under [D9] it checks the grant the act names and does not search for another, and a governed act with no `under` fails. Tests: a hand-written, validly signed acceptance by a bound principal with no grant; one made after the grant's revocation landed with `at` backdated; one made before a later revocation, which stands. D5's second part (signed grants and grant acceptances) is not in this session unless an issue for it exists when you start; list in the close-out what it would touch.
- [D5] Pre-policy acts. Session A's rule (a namespace without a policy is not role-checked) becomes a position rule: an act that is before its namespace's first policy (D6) is not role-checked; every other act is. `verify` names each namespace that has no policy as unchecked in its output, as a notice and not a failure. Opting in is one-way: a store in which a namespace had a policy at an earlier first-parent commit and has none at the verified commit fails; say in the PR which class reports it. Tests: a namespace with acceptances, then `init --namespace`, `verify` green with the earlier acceptances standing; an acceptance by an ungranted principal landing after the policy fails `A006`; the removed-policy store fails. `L011` is untouched: the signature requirement for pre-v2 acceptances stays with `none` in policy (D4).
- [Q8] If Session A's question 8 is ruled write-once: `verify` fails when a landed role file's content differs from its content at its landing commit (the [D6] landing function). Test: a role file edited after landing to add a capability. Say in the PR which class reports it.
- `accept` and `revoke` refuse: identity without the role, software key under a `-sk` policy, agent-reported unconfirmed key.
- Export: every payload field plus a reference per sidecar, so an export-only verifier can rebuild the signed bytes. Add that verifier as a test with its input fixed: the export file and the sidecar files; no entity file from `.decisions/` and no git. It rebuilds `allowed_signers` from the export's `KeyBinding` nodes, and the result equals the committed derived file byte for byte. Document the reconstruction rules in the format document: identities with `mailto:` stripped, `at` in the exact lexical form that was hashed, absent fields omitted, `scope` in wire form, [D9] `under` as the grant id. Property test over every fixture: bytes rebuilt from the export equal the bytes that were signed.
- [D6] Document the export-only verifier's limit in the format document: it checks signatures and key windows by `at`; it cannot check landing order, which `verify` checks in the repository.
- Upstream list. Session A's close-out has none, so this one covers #80 as well. For the analyzers' reader contract, list every class and predicate that can appear in the export and could not before #80, and state these four points: predicates such as `ledger:id`, `ledger:hash`, `ledger:scope` and `ledger:namespace` now occur on non-decision nodes, so the reader must dispatch on `rdf:type`; `ledger:revokes` can target a grant as well as an acceptance; `ledger:revokedAt` and `ledger:revokedBy` no longer appear on acceptances; legacy revocation nodes (`urn:rev:legacy-<acc-ulid>`) carry no `ledger:id`.

### Step 2 — #86: `accept --batch <file>`
- Selection file: enumerated (repository, decision, version hash) rows, pinned by a manifest digest exactly as `--set|--group --confirm` is. Signs exactly that list; refuses on drift (any version hash moved, any row missing).
- [D9] `--as <role>` applies to the whole batch. The grant is resolved per row, shown per row in the manifest, and covered by the manifest digest. A row with no qualifying grant in that role refuses the batch.
- One confirmation per batch, one signature per acceptance. The holder confirms the manifest digest once; the CLI then produces one sidecar per acceptance (D2). Under a `-sk` policy that is one touch per acceptance. No signature over the manifest and no signature covering more than one acceptance: either would be a format change.
- The same file format the inbox writes; document it in the format document as the hand-off artefact.

### Step 3 — #79: `ledger inbox`
- Config: list of local clone paths. Index: in-memory Oxigraph dataset, one named graph per (repository, branch), built from each clone's committed `docs/decisions/<ns>.nt`. Graph-stage checks, the `A` shapes included, run per named graph and never over the union: with one genesis per store, two repositories in one dataset would otherwise fail `A005`. Test it with two stores, each with its own genesis.
- Branches: git does not record which branches have an open PR. R0 indexes the default branch plus every remote-tracking branch whose committed export differs from the default branch's. A PR number is a label: read from `refs/pull/<n>/head` when the clone has fetched those refs and the head commit matches, absent otherwise, and grouping falls back to the branch. No GitHub API (§10.3). Record the difference from registry PRD §8 ("every branch with an open PR") in the close-out for the PRD to be amended.
- List per holder: proposed decisions in namespaces where the holder has `accept-decision`, grouped by repository and PR or branch, newest first, with key, statement, set, namespace, [D9] the grant it would be accepted under, proposer and identity class, rationale, citing symbols, firing rule, diff against predecessor. "Needs re-acceptance" items (`L012`, and [D6] acceptances on a branch that fail `L011` against the base) in their own state with the policy deadline when set. A namespace with no policy has no holders, so its proposed decisions are in nobody's list; the inbox names such namespaces as unchecked and does not omit them silently.
- Accept: writes the batch file, calls `accept --batch`, commits on the branch as the holder, pushes. Affirmations go in the same batch. An action on a stale item re-reads the branch before signing and refuses if the version hash moved. A branch the clone's remote cannot be pushed to is listed read-only with the reason.
- Refusals: agent identities, non-interactive sessions, any attempt to accept for another principal. Tests prove each.
- Rebuild equivalence: if R0 has an incremental path, the index rebuilt from exports equals the incrementally maintained one, as a property test. If R0 rebuilds on every invocation, say so in the PR and leave the test to R1.5; do not write a test that compares a rebuild with itself.
- Acceptance criterion in a test fixture: three repositories, twelve branches with proposed decisions, one sitting, every branch's `verify` green afterwards. Every namespace in the fixture is initialised with `init --namespace`, and each repository has its own genesis.

### Session B close-out
`ledger/sessions/2026-10-session-b.md`, same structure as A, plus:
- the measured time for twenty acceptances through the inbox on the fixture, and the number of confirmations and signatures that took;
- what in R1 and R1.5 the R0 design already leaves room for or blocks: `ledger:Review`, reject, changes-requested, and commits written by a server, given that `L009` reads the introducing commit's author;
- every [D5] to [D9], [Q7] or [Q8] bullet left out because the ruling was not recorded, and every bullet left out because a Session A question was still open;
- what signing grants and grant acceptances would touch (D5, second part), including the missing `at` in the grant payload;
- the upstream list from step 1.

---

## Constraints for both sessions
- Quote file paths and symbol names in every PR description for every claim.
- No new dependency without a note in the PR description giving the alternative in the BCL-equivalent (`std`) and why it was insufficient.
- No format change outside the issues' lists. If one seems necessary, it is a question in the close-out.
- Never weaken a gate to make a test pass; change the fixture or the test, and say which.
- Do not open issues upstream in the analyzers repository; list them in the close-out for the principal.
