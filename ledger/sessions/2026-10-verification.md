# Verification of the thirteen code-versus-text findings

Session of 7 October 2026, under ruling 40. It checks, by running the reference implementation, the thirteen rows that §4 of `ledger/sessions/2026-10-protocol-absorption.md` marks "differs", in the order of list 1 of the principal's replies. It fixes nothing: no code, test, fixture, digest, protocol text, ruling, store or export changed.

The repository had not moved: `main` was at `55c3bcf` when this session started, the commit the prompt names.

## Summary

| # | Finding | Requirement | Verdict | Authority? | Cases in this repository | Lean |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | A sidecar on an act no policy governs is never verified | LP-4.7 | confirmed | no | 0 | fix the code |
| 2 | "The accept role is never the genesis role" is not held at verification | LP-6.17 | confirmed | **yes** | 0 | fix the code |
| 3 | A duplicate `acc:` id, and a decision identity object filed twice, are not refused | LP-5.11, LP-8.8 | confirmed, with a wider reach than claimed | **yes** | 0 | fix the code |
| 4 | A forked decision gets a stand-in latest that the file gate judges | LP-8.11 | confirmed | no | 1 (the `forked` fixture) | fix the code |
| 5 | A `rotate` passes when signed by any live key of its principal | LP-4.12 | confirmed | no | 0 | fix the code |
| 6 | `format:` changes are not compared across history | LP-3.16 | confirmed | no | 0 non-corrections; 3 legitimate corrections | amend the text |
| 7 | `L006` judges an escape's `accepted_by` on latest versions only | LP-8.9 | confirmed | no | 0 | fix the code |
| 8 | A YAML float in a hashed string field is read as text | LP-3.4, LP-4.18 step 7 | confirmed | no | 0 | fix the code |
| 9 | A `set:` grant scope refuses a dot | LP-3.3 | confirmed | no | 0 | fix the code |
| 10 | `parents` is keyed apart from the header entity | LP-8.25 | confirmed | no | 0 | fix the code |
| 11 | The export-only verifier is a test helper and derives `valid-before` per binding | LP-9.14 | different | **yes, for an export-only reader** | 0 | amend the text, with ruling 47 |

The prompt numbers the thirteen 1 to 13 and pairs 3 with 4 and 9 with 10. This report pairs them the same way, so it has eleven findings. Its numbering follows the output layout the prompt gives, so the report's 4 to 11 are the prompt's 5 to 13:

| Report | Prompt |
| --- | --- |
| 1, 2, 3 | 1, 2, 3 and 4 |
| 4, 5, 6, 7 | 5, 6, 7, 8 |
| 8 | 9 and 10 |
| 9, 10, 11 | 11, 12, 13 |

"Authority" uses the prompt's test: whether the gap opens a path by which an act passes verification without a valid signature by a holder with the role. "Cases" counts what the 16 committed stores in this repository hold: `.decisions/` and the 15 fixture stores under `ledger-cli/tests/fixtures/`. No committed store holds an authority record or a sidecar, so every authority finding has zero cases. Stores that tests build at run time are not counted.

Section 13 adds two differences found while reproducing: an explicit `null` in a required hashed field, and a genesis role that carries a decision capability, which nothing refuses. No finding was refuted.

## How the reproductions run

Every reproduction is a shell script over the built binary, in an empty directory outside the repository. Each script sources `lib.sh` below. To rerun one:

```bash
cargo build -p ledger-cli
export LEDGER=$PWD/target/debug/ledger REPO=$PWD WORK=$(mktemp -d)
bash f01.sh      # with lib.sh and f01.sh in the current directory
```

They need git, OpenSSH's `ssh-keygen` (the container lacked it; OpenSSH 9.6p1 was installed) and `script(1)`. Verbs that sign or write an authority record refuse a non-terminal caller, so `tl` runs them under `script(1)`, as the test suites do. `lib.sh` unsets the four `GIT_AUTHOR_*` and `GIT_COMMITTER_*` variables. Hand-written files carry a placeholder digest, and `rehash` pastes the digest that `L007` reports, as `CLAUDE.md` describes. ULIDs, timestamps and digests over timestamps differ on every run. The outputs below are from one run, with the scratch path shortened to `$WORK`.

`lib.sh`:

```bash
# lib.sh: shared helpers. Needs LEDGER (the built `ledger` binary), WORK (an
# empty scratch directory), git, ssh-keygen (OpenSSH) and script(1).
set -u
unset GIT_AUTHOR_NAME GIT_AUTHOR_EMAIL GIT_COMMITTER_NAME GIT_COMMITTER_EMAIL
OWNER=owner@customer.example
NS=fixture.ledger
FIX=${REPO:-/home/user/product-cli}/ledger-cli/tests/fixtures
pl() { "$LEDGER" --root "$R" "$@"; }                     # plain pipes
tl() {                                                    # at a terminal, as a person
  local line="'$LEDGER' --root '$R'"; local a
  for a in "$@"; do line="$line '${a//\'/\'\\\'\'}'"; done
  script -qefc "$line" /dev/null </dev/null | tr -d '\r'; return "${PIPESTATUS[0]}"
}
commit() { git -C "$R" add -A && git -C "$R" commit -qm "$1"; }
fresh() {                                                 # fresh <name> [email]
  R=$WORK/$1; rm -rf "$R"; mkdir -p "$R"
  git -C "$R" init -q --initial-branch=main
  git -C "$R" config user.name Fixture; git -C "$R" config user.email "${2:-$OWNER}"
  git -C "$R" config commit.gpgsign false; pl init >/dev/null
}
keygen() { mkdir -p "$WORK/keys"; local k=$WORK/keys/$1; rm -f "$k" "$k.pub"; ssh-keygen -q -t ed25519 -N "" -C "$1" -f "$k"; echo "$k"; }
from_fixture() { cp "$FIX/$2/.decisions/sets/"* "$R/.decisions/sets/"; [ "$1" = log ] && cp "$FIX/$2/.decisions/log/"* "$R/.decisions/log/"; return 0; }
# governed <name>: a namespace under an `ssh` policy, the owner's key bound,
# the owner granted the accept role `acceptor`; committed.
governed() {
  fresh "$1"
  pl declare --set ledger-design --tolerance-floor T1 >/dev/null
  tl init --namespace $NS --external-ref "contract 2026/117" --without-key >/dev/null
  OKEY=$(keygen "owner-$1"); git -C "$R" config user.signingkey "$OKEY"
  tl identity add --namespace $NS --key-file "$OKEY.pub" >/dev/null
  GRANT=$(tl grant new acceptor --to $OWNER --scope ns:$NS | grep -o 'grant:[0-9A-Z]*' | head -1)
  tl grant accept "$GRANT" >/dev/null
  commit governed
}
add() { pl add --set ledger-design --namespace "${2:-$NS}" --statement "$1" --store constraint --discharge analyzer:DEC001 | grep -o 'dec:[^ ]*' | head -1; }
# rehash <file>: replace each placeholder digest with the one L007 reports.
rehash() {
  local i l7 old new
  for i in 1 2 3 4 5; do
    l7=$(pl verify --no-blame 2>&1 | grep -m1 'L007.*hashes to') || break
    old=$(echo "$l7" | grep -o 'stored hash [0-9a-f]*' | awk '{print $3}')
    new=$(echo "$l7" | grep -o 'hashes to sha256:[0-9a-f]*' | sed 's/hashes to sha256://')
    sed -i "s/sha256:${old}[0-9a-f]*/sha256:$new/g" "$1"
  done
}
v() { pl verify --no-blame "$@"; echo "exit $?"; }
```

## 1. LP-4.7: a sidecar on an act no policy governs is never verified

**The text.** LP-4.7: "A verifier MUST verify every sidecar present." LP-4.30 repeats it: "A sidecar that is present always has to verify." LP-4.30 then says "An acceptance or revocation before its namespace's first policy is not checked (D5 (c))."

**The code.** `signing::check::check` (`ledger-core/src/signing/check.rs`) judges a non-binding entity only through `governing(…)`. When no policy governs the entity, it reaches `else { continue }` and its sidecars are never opened. That covers both cases: a namespace with no policy, and an act before its namespace's first policy. A binding is never skipped this way: one in a namespace with no policy is already a schema fault.

**Reproduction.** Two stores. In each, an acceptance filed by `ledger accept` gets a sidecar whose content is the text `not a signature`. In 1a the namespace has no policy. In 1b the acceptance is committed first, and the namespace is then put under an `ssh` policy.

```bash
# Finding 1 (LP-4.7): a sidecar that is not a signature, on two acceptances
# no policy governs.
. ./lib.sh
echo "## 1a: a namespace with no policy"
fresh f01a; pl declare --set ledger-design --tolerance-floor T1 >/dev/null
D=$(add "Money is decimal."); tl accept "$D" >/dev/null
U=$(grep -rho '^- id: acc:[0-9A-Z]*' "$R/.decisions/log" | cut -d: -f3)
mkdir -p "$R/.decisions/sig"; printf 'not a signature\n' > "$R/.decisions/sig/$U.ssh.sig"
commit f01a; (cd "$R" && find .decisions -type f | sort); v
pl export --format ntriples >/dev/null; grep -h "sig:$U" "$R/docs/decisions/$NS.nt"
echo "## 1b: an acceptance before its namespace's first policy (D5 (c))"
fresh f01b; pl declare --set ledger-design --tolerance-floor T1 >/dev/null
D=$(add "Money is decimal."); tl accept "$D" >/dev/null
U=$(grep -rho '^- id: acc:[0-9A-Z]*' "$R/.decisions/log" | cut -d: -f3)
mkdir -p "$R/.decisions/sig"; printf 'not a signature\n' > "$R/.decisions/sig/$U.ssh.sig"
commit pre-policy
tl init --namespace $NS --external-ref "contract 2026/117" --without-key >/dev/null
K=$(keygen owner-f01b); git -C "$R" config user.signingkey "$K"
tl identity add --namespace $NS --key-file "$K.pub" >/dev/null; commit policy
pl policy show --namespace $NS | sed -n 2p; v
```

```text
## 1a: a namespace with no policy
.decisions/log/01M4ASDM8QDXZBJHPSVF4QFDE2.yml
.decisions/log/01M4ASDMAGH7WSVV877N1HSBEB.yml
.decisions/sets/ledger-design.yml
.decisions/sig/01M4ASDMAGE4ZBYC21H6K9ZWTP.ssh.sig
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 4 entries, 1 decision(s)
notice: namespace `fixture.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace fixture.ledger` opts it in)
exit 0
<urn:acc:01M4ASDMAGE4ZBYC21H6K9ZWTP> <urn:ledger:ns#signature> <urn:sig:01M4ASDMAGE4ZBYC21H6K9ZWTP.ssh.sig> .
<urn:sig:01M4ASDMAGE4ZBYC21H6K9ZWTP.ssh.sig> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <urn:ledger:ns#Signature> .
<urn:sig:01M4ASDMAGE4ZBYC21H6K9ZWTP.ssh.sig> <urn:ledger:ns#signatureFile> "sig/01M4ASDMAGE4ZBYC21H6K9ZWTP.ssh.sig" .
<urn:sig:01M4ASDMAGE4ZBYC21H6K9ZWTP.ssh.sig> <urn:ledger:ns#signatureScheme> "ssh" .
## 1b: an acceptance before its namespace's first policy (D5 (c))
  schemes: ssh
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 10 entries, 1 decision(s)
exit 0
```

**Verdict: confirmed.** Both stores are conformant, and the export carries a `ledger:signature` node for the sidecar that is not a signature.

**Reach.**

- *Passes what the text refuses:* yes. A store holding a present sidecar that does not verify passes, and so does its export, which asserts a signature that is not there.
- *Authority:* no. The acts concerned need no signature and no role check under the text either (LP-6.18; D5 (c)), so no act gains standing it would otherwise lack. What breaks is the signature record: the export says "signed", and the export-only helper (finding 11), which verifies every `ledger:signature` node, would report the failure that the repository verifier does not.
- *Cases:* 0. No committed store has a `sig/` directory.

**Choices.**

- **Fix the code to the text.** Verify every present sidecar. Both stores above turn red, and no committed store changes verdict. No digest moves. Two questions come with it:
  - Which class? `L011` reads "a signature the namespace's policy requires is absent or does not verify", and here no policy requires one. Reporting it as `L011` stretches that wording. `SCHEMA` fits a malformed sidecar, but not a well-formed signature by the wrong key. A new class is a protocol change (LP-8.7).
  - Against which keys? In a namespace with no policy no binding can be trusted, since such a binding is a schema fault, so no sidecar there can ever verify. Before a first policy, only bindings judged under that policy exist. In practice the fix reduces to "a sidecar on an act no policy governs is a finding".
- **Amend the text to the code.** LP-4.7 and LP-4.30 would say: "A verifier MUST verify every sidecar on an entity a policy governs; a sidecar on any other entity is not read." A second implementation would have to skip such sidecars too. The export would still carry them as signature nodes unless LP-9.3 is narrowed to match.
- **Record it as a known limit.** A store and its export can claim signatures that do not exist on ungoverned and pre-policy acts. A reader of the export cannot tell such a sidecar from a real one without verifying it.

*Lean:* fix the code, reporting the sidecar as `SCHEMA` ("a sidecar on an act no policy governs"). That fits LP-8.8's existing "a sidecar that is misnamed or names no signable entity" without adding a class.

## 2. LP-6.17: "the accept role is never the genesis role" is not held at verification

**The text.** LP-6.17: "In a namespace with a policy, accepting and revoking an acceptance count only grants of the policy's `accept_role`, which is never the genesis role. The genesis (root) role carries `grant-role`, `revoke-grant`, `declare-unavailability` and `rotate-genesis` and none of the decision capabilities (D9 (f)). A policy whose `accept_role` is no declared role that may `accept-decision` is a schema fault."

**The code.** `authority::references::policy_refs` checks only that `accept_role` names a declared role that may `accept-decision`. The writer refuses an `accept_role` equal to the genesis role, in `init --namespace` (`author/authority_ops.rs`) and `policy set` (`author/policy_ops.rs`). Nothing at verification compares the two roles, and nothing checks the genesis role's capability set.

**Reproduction.** `init --namespace` is run without committing. Then, by hand, `accept-decision` is added to the genesis role `steward`, the policy's `accept_role` is set to `steward` with its digest re-pasted, and the unused `acceptor` role file is removed. The owner then binds a key and runs the ordinary `ledger accept`.

```bash
# Finding 2 (LP-6.17): the genesis role widened to accept-decision and named
# as the policy's accept role.
. ./lib.sh
fresh f02; pl declare --set ledger-design --tolerance-floor T1 >/dev/null
tl init --namespace $NS --external-ref "contract 2026/117" --without-key >/dev/null
F=$(grep -l '^policies:' "$R"/.decisions/log/*.yml)
sed -i 's/^  accept_role: acceptor$/  accept_role: steward/' "$F"
sed -i 's/^- rotate-genesis$/- rotate-genesis\n- accept-decision/' "$R/.decisions/roles/steward.yml"
rm "$R/.decisions/roles/acceptor.yml"
rehash "$F"; commit "genesis role is the accept role"
cat "$R/.decisions/roles/steward.yml"; grep -A9 '^policies:' "$F"
K=$(keygen owner-f02); git -C "$R" config user.signingkey "$K"
tl identity add --namespace $NS --key-file "$K.pub" >/dev/null; commit key
D=$(add "Money is decimal."); tl accept "$D"; commit accept
v
```

```text
format: 6
id: steward
title: Genesis steward
owner: owner@customer.example
may:
- grant-role
- revoke-grant
- declare-unavailability
- rotate-genesis
- accept-decision
created_at: 2026-10-07
policies:
- id: pol:01M4ASCNGEG3FSBJZMKHE8P61N
  namespace: fixture.ledger
  schemes:
  - ssh
  accept_role: steward
  by: owner@customer.example
  under: grant:01M4ASCNGE98147RT482F86NDY
  at: 2026-10-07T08:58:56.397633539Z
  hash: sha256:f5895b1d43b1590e461cba123c6004f12248127b1c8d3b35fd17a3529c9fd5ce
owner@customer.example accepted 5a61d5d95ec7 of dec:fixture.ledger/01M4ASCP06R8485P8JBS1TJ2QC — the signature names this exact state
under grant:01M4ASCNGE98147RT482F86NDY (`steward`)
signed — sig/01M4ASCP7G81PX8F546APRQG33.ssh.sig
filed $WORK/f02/.decisions/log/01M4ASCP7Y78HHT41D6EC6RWNQ.yml
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 9 entries, 1 decision(s)
exit 0
```

**Verdict: confirmed.** The store is conformant. The writer's own `accept` then signs under the genesis grant (`under grant:… (steward)`), since the policy it reads names `steward`.

**Reach.**

- *Passes what the text refuses:* yes. A policy whose accept role is the genesis role, and a genesis role carrying `accept-decision`.
- *Authority:* **yes.** The acceptance is validly signed, but by a holder of a role the text says can never authorise acceptance. D9 (f) separates acting on the authority structure from acting on decisions, and this store erases that separation: the genesis grant alone accepts, with no separate grant of the accept role. Separately, the second half of LP-6.17 (the genesis role carries no decision capability) is refused by neither the verifier nor the writer; see section 13.2.
- *Cases:* 0. No committed store holds a policy or a role file.

**Choices.**

- **Fix the code to the text.** At verification, a policy whose `accept_role` equals the role of the namespace's genesis grant is `SCHEMA`, as is a genesis grant whose role may do any decision capability. The store above turns red, and no committed store changes. No digest moves. The rule needs "the genesis role" defined at a policy's position: the role of the live genesis grant as of the policy. Ruling 47 will make that per namespace.
- **Amend the text to the code.** LP-6.17 would make "never the genesis role" a writer rule (W) only. A second implementation's verifier would then accept a genesis role that accepts, and the separation D9 (f) ruled would rest on every writer being honest.
- **Record it as a known limit.** A hand-written store can let the genesis holder accept with the genesis grant, and verification will not say so.

*Lean:* fix the code. D9 (f) is a ruling, and as things stand it holds only against people who use the reference writer.

## 3. LP-5.11 and LP-8.8: a duplicate acceptance id, and a decision identity object filed twice

**The text.** LP-5.11: "A decision identity object appears in the change-set that introduces the decision, and only there." LP-8.8 counts "a duplicate id" among the `SCHEMA` faults.

**The code.** `authority::references::duplicate_ids` counts the ids of grants, grant acceptances, unavailabilities, availabilities, `rev:` revocations, key bindings and policies only. `acc:` ids and `decisions:` entries are not counted. Sets, roles and change-sets have their own duplicate checks (`store.rs`, `authority/structure.rs`). Two consequences:

- `View` keeps revoked acceptances as a set of ids (`verify/view.rs`), so one revocation of an id revokes every acceptance that carries it.
- `verify::acts::revocation_verdict` and `signing::subject::acceptance_namespace` both resolve an acceptance id to the *first* acceptance with that id in change-set order. They take that acceptance's namespace to decide whether the revocation is governed, and in which namespace it is signed.

**Reproduction.**

- 3a: a copy of the `pass` fixture, plus a second change-set that files the same decision identity object and the acceptance id `acc:…RDXX` again, for a different actor.
- 3b: the second actor then revokes "their" acceptance in the legacy shape.
- 3c: in a governed namespace, the genesis holder accepts a decision with a signed acceptance. A hand-written change-set with an earlier ULID then files a decision in an ungoverned namespace `open.ns`, an acceptance of it reusing the governed acceptance's id, and a legacy revocation of that id by `mallory@example`, who holds nothing. It is committed as mallory, so `L009` (run here, no `--no-blame`) is satisfied.

```bash
# Findings 3 and 4 (LP-5.11, LP-8.8): one acceptance id and one decision
# identity object, each filed twice.
. ./lib.sh
echo "## 3a: the pass fixture plus a second change-set"
fresh f03a fixture-human@example; from_fixture log pass
cat > "$R/.decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDY5.yml" <<'YML'
format: 1
id: cs:01K2C4YQJ3F8M0PT5W7NZ9RDY5
created_at: 2026-08-11T09:14:22Z
created_by: someone-else@example
decisions:
  - id: dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
    created_at: 2026-08-11T09:14:22Z
    created_by: someone-else@example
acceptances:
  - id: acc:01K2C4YQJ3F8M0PT5W7NZ9RDXX
    decision: dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
    version: sha256:6e8bedf516cbf54c1a20416f33df5afc556b6f318345ad81d26e7b2657444b6d
    actor: second-human@example
    at: 2026-08-11T09:20:00Z
    expires_at: 2027-08-11
YML
commit f03a; v; pl status | sed -n 2p
pl export --format ntriples >/dev/null
grep -h 'acc:01K2C4YQJ3F8M0PT5W7NZ9RDXX> <http://www.w3.org/ns/prov#wasAttributedTo' "$R"/docs/decisions/*.nt
rm -rf "$R/docs"
echo "## 3b: second-human revokes the shared id"
cat > "$R/.decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDY6.yml" <<'YML'
format: 1
id: cs:01K2C4YQJ3F8M0PT5W7NZ9RDY6
created_at: 2026-08-12T09:14:22Z
created_by: second-human@example
revocations:
  - acceptance: acc:01K2C4YQJ3F8M0PT5W7NZ9RDXX
    at: 2026-08-12T09:00:00Z
    by: second-human@example
    reason: withdrawn
YML
commit revoke; v; pl status | sed -n 2p
echo "## 3c: an unsigned legacy revocation reaches a governed acceptance"
governed f03c; D=$(add "Money is decimal."); tl accept "$D" >/dev/null; commit accepted
pl status | sed -n 2p
ACC=$(grep -rho '^- id: acc:[0-9A-Z]*' "$R/.decisions/log" | cut -d' ' -f3)
F="$R/.decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDZ0.yml"
cat > "$F" <<YML
format: 1
id: cs:01K2C4YQJ3F8M0PT5W7NZ9RDZ0
created_at: 2026-08-10T09:14:22Z
created_by: mallory@example
decisions:
  - id: dec:open.ns/01K2C4YQJ3F8M0PT5W7NZ9RDZ1
    created_at: 2026-08-10T09:14:22Z
    created_by: mallory@example
versions:
  - decision: dec:open.ns/01K2C4YQJ3F8M0PT5W7NZ9RDZ1
    hash: sha256:0000000000000000000000000000000000000000000000000000000000000000
    set: ledger-design
    statement: Anything at all.
    allocation: constraint
    discharge: [analyzer:X]
    tolerance_floor_at_creation: T1
acceptances:
  - id: $ACC
    decision: dec:open.ns/01K2C4YQJ3F8M0PT5W7NZ9RDZ1
    version: sha256:0000000000000000000000000000000000000000000000000000000000000000
    actor: mallory@example
    at: 2026-08-10T09:20:00Z
revocations:
  - acceptance: $ACC
    at: 2026-08-10T09:30:00Z
    by: mallory@example
    reason: withdrawn
YML
rehash "$F"; git -C "$R" config user.email mallory@example; commit mallory
echo "governed acceptance: $ACC"; pl verify; echo "exit $?"; pl status | sed -n 2p
```

```text
## 3a: the pass fixture plus a second change-set
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 6 entries, 1 decision(s)
notice: namespace `fixture.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace fixture.ledger` opts it in)
exit 0
  dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV [6e8bedf516cb] decided
<urn:acc:01K2C4YQJ3F8M0PT5W7NZ9RDXX> <http://www.w3.org/ns/prov#wasAttributedTo> <mailto:fixture-human@example> .
<urn:acc:01K2C4YQJ3F8M0PT5W7NZ9RDXX> <http://www.w3.org/ns/prov#wasAttributedTo> <mailto:second-human@example> .
## 3b: second-human revokes the shared id
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 7 entries, 1 decision(s)
1 allocated, awaiting acceptance:
  - dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
notice: namespace `fixture.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace fixture.ledger` opts it in)
exit 0
  dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV [6e8bedf516cb] awaiting-acceptance
## 3c: an unsigned legacy revocation reaches a governed acceptance
  dec:fixture.ledger/01M4ASDPD1Z0N84V71PTBE1RSS [10fb8d66e75d] decided
governed acceptance: acc:01M4ASDPHNSZX0AF98QF9R0ZGM
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 16 entries, 2 decision(s)
2 allocated, awaiting acceptance:
  - dec:fixture.ledger/01M4ASDPD1Z0N84V71PTBE1RSS
  - dec:open.ns/01K2C4YQJ3F8M0PT5W7NZ9RDZ1
notice: namespace `open.ns` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace open.ns` opts it in)
exit 0
  dec:fixture.ledger/01M4ASDPD1Z0N84V71PTBE1RSS [10fb8d66e75d] awaiting-acceptance
```

**Verdict: confirmed, with a wider reach than the survey stated.**

- 3a: conformant. The export merges the two acceptances into one node with two actors, two times and two expiries, and the decision node gains two creators.
- 3b: conformant. One holder's revocation revokes the other holder's acceptance, and the decision falls from `decided` to awaiting acceptance.
- 3c: conformant. An unsigned revocation by someone with no grant unsays a signed acceptance in a governed namespace. `A006` looks up the first acceptance with the id, finds it in `open.ns`, which has no policy, and judges nothing. The signature gate takes the revocation's namespace from the same first match and requires nothing.

**Reach.**

- *Passes what the text refuses:* yes. Duplicate ids, a decision introduced twice, and through them revocations that reach acceptances they do not name uniquely.
- *Authority:* **yes** (3c). A revocation of an acceptance in a governed namespace needs a grant of the accept role (LP-6.17) and a signature (LP-4.25). Here one passes with neither. Inference, not run: no path was found by which a duplicate id lets an *acceptance* pass unsigned. Each acceptance is a separate subject and needs its own verifying sidecar, so a governed copy without one fails `L011`.
- *Cases:* 0. No duplicate `acc:` id and no decision identity object filed twice in any of the 16 stores (91 acceptances in `.decisions/`, 8 in the fixtures).

**Choices.**

- **Fix the code to the text.** Count `acc:` ids and decision identity objects in the duplicate check, as `SCHEMA`. 3a, 3b and 3c all turn red, and no committed store changes. No digest moves: neither the acceptance's id nor a decision identity object is hashed.
- **Amend the text to the code.** LP-8.8's "a duplicate id" would be narrowed to authority records, and LP-5.11 marked writer-only. A second implementation would then have to reproduce the first-match resolution of an acceptance id, the cross-namespace effect in 3c included, and the text would have to say what one revocation of a shared id revokes.
- **Record it as a known limit.** A hand-written change-set can revoke a governed acceptance without a grant or a signature, by reusing its id in an ungoverned namespace.

*Lean:* fix the code. An identity that is not unique cannot name what it revokes.

## 4. LP-8.11: a forked decision gets a stand-in latest

**The text.** LP-8.11: "A chain that cannot name one tip … has no latest: an implementation MUST NOT resolve the ambiguity by any ordering heuristic." LP-8.10 has `L001`, `L003`, `L005`, `L010` and `L014` judge only the latest version.

**The code.** `View::derive_latest` (`verify/view.rs`) records the fork for `G004`, and also stores the tip with the smallest hash as the decision's `latest`. Every latest-only class then judges that tip. Its own comment says so ("a deterministic representative … nothing may treat that pick as a resolution").

**Reproduction.** The `forked` fixture as committed (5a), then the right writer's revision with its `allocation` and `discharge` removed and its digest re-pasted. In 5b the unallocated tip hashes `9f84…`, which sorts before the other tip `a289…`. In 5c a one-word change to its statement gives `ffbc…`, which sorts after.

```bash
# Finding 5 (LP-8.11): the forked fixture, then the same defect (a tip with
# no allocation) placed on the tip that sorts first, and on the one that
# sorts last.
. ./lib.sh
right() {  # right <statement> <hash>: the right writer's revision, unallocated
cat > "$R/.decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDY1.yml" <<YML
format: 1
id: cs:01K2C4YQJ3F8M0PT5W7NZ9RDY1
created_at: 2026-08-10T10:00:01Z
created_by: fixture-human@example
versions:
  - decision: dec:fixture.forked/01K2C4YQJ3F8M0PT5W7NZ9RDXV
    parent: sha256:29fb5a0a3934f4e3bc307b6ea03a9932a6a9a85f04cb2d97a87917000b790a39
    hash: $2
    set: forked
    statement: $1
    tolerance_floor_at_creation: T1
YML
}
echo "## 5a: the fixture as committed"
fresh f05a fixture-human@example; from_fixture log forked; commit f; v
echo "## 5b: the right tip unallocated, hash 9f84… (sorts before a289…)"
fresh f05b fixture-human@example; from_fixture log forked
right "The right writer's revision." sha256:9f84919ba8c8e2013269795c5ebe7ed87a0b39f5648edb6b6ab88fb67d26a825
commit f; v
echo "## 5c: the right tip unallocated, hash ffbc… (sorts after a289…)"
fresh f05c fixture-human@example; from_fixture log forked
right "The right writer's revision, unallocated 1." sha256:ffbcf1ff4357fff3296f017cca8d8fb574b69d8a3948ed7309ff28ce111abd5e
commit f; v; pl status | sed -n 2p
```

```text
## 5a: the fixture as committed
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 1 finding(s):
graph stage — 1 finding(s):
  - [G004] dec:fixture.forked/01K2C4YQJ3F8M0PT5W7NZ9RDXV: the version chain forks: 523a5b4b0e93 and a289f499364f are both tips — two writers diverged, and only `ledger merge --resolve` may settle which content stands
1 allocated, awaiting acceptance:
  - dec:fixture.forked/01K2C4YQJ3F8M0PT5W7NZ9RDXV
notice: namespace `fixture.forked` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace fixture.forked` opts it in)
exit 1
## 5b: the right tip unallocated, hash 9f84… (sorts before a289…)
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 2 finding(s):
  - [L001] dec:fixture.forked/01K2C4YQJ3F8M0PT5W7NZ9RDXV: no allocation — demand is never reduced by leaving it unassigned (§2.1)
graph stage — 1 finding(s):
  - [G004] dec:fixture.forked/01K2C4YQJ3F8M0PT5W7NZ9RDXV: the version chain forks: 9f84919ba8c8 and a289f499364f are both tips — two writers diverged, and only `ledger merge --resolve` may settle which content stands
notice: namespace `fixture.forked` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace fixture.forked` opts it in)
exit 1
## 5c: the right tip unallocated, hash ffbc… (sorts after a289…)
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 1 finding(s):
graph stage — 1 finding(s):
  - [G004] dec:fixture.forked/01K2C4YQJ3F8M0PT5W7NZ9RDXV: the version chain forks: a289f499364f and ffbcf1ff4357 are both tips — two writers diverged, and only `ledger merge --resolve` may settle which content stands
1 allocated, awaiting acceptance:
  - dec:fixture.forked/01K2C4YQJ3F8M0PT5W7NZ9RDXV
notice: namespace `fixture.forked` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace fixture.forked` opts it in)
exit 1
  dec:fixture.forked/01K2C4YQJ3F8M0PT5W7NZ9RDXV [a289f499364f] awaiting-acceptance — chain forked into 2 tips, awaiting `ledger merge --resolve`
```

**Verdict: confirmed.** The same defect, a tip with no allocation, draws `L001` when that tip sorts first by hash (5b) and nothing when it sorts last (5c). `status` reads the decision at the stand-in's hash. Every forked store still fails, on `G004`.

**Reach.**

- *Passes what the text refuses:* no store passes, since `G004` runs under both gates. What the gap changes is the content of the report: findings on a forked decision depend on a hash-order pick, and the "allocated, awaiting acceptance" list includes it.
- *Authority:* no.
- *Cases:* 1, the `forked` fixture. Its file-gate verdict does not change under either choice: both tips there are allocated and unaccepted.

**Choices.**

- **Fix the code to the text.** A forked decision has no latest. The latest-only classes skip it, and `G004` is its one finding. Verdicts change on no committed store. `verify/mod_tests.rs::a_forked_chain_is_recorded_and_the_representative_is_order_independent` asserts the representative and would be rewritten. Every reader of `View::latest` (`status`, `coverage`, the authoring verbs, `merge`) needs a stated behaviour for a decision with none. Inferred from the names: `author/sign_tests.rs` already refuses forked chains, so the verbs are largely there. No digest moves.
- **Amend the text to the code.** LP-8.11 would say that the reference picks the smallest-hash tip for reporting only, and that it never resolves the fork. A second implementation would have to pick the same tip to report the same file-gate findings (CF-8 compares class and subject pairs).
- **Record it as a known limit.** The file gate's findings on a forked decision depend on hash order. `G004` keeps the store red.

*Lean:* fix the code. The text says MUST NOT, and the representative leaks into what `status` shows.

## 5. LP-4.12: a `rotate` signed by another live key of its principal

**The text.** LP-4.12: "every further `add`, and every `rotate`, is the principal's own, signed by a live key of theirs (a `rotate` by the key it closes)". D7's gap paragraph says the same (`ledger/rulings/signing-rulings-2026-10-d5-d9.md`).

**The code.** `authority::filing::may_file` checks who filed the binding, and names no signing key. The signature is judged in `signing::check::verify_one`, which matches the sidecar's key against every open trusted binding of the signer in the namespace. The writer signs with whatever key `git config user.signingkey` names.

**Reproduction.** In a governed store the owner holds K1 (the self-bound key) and binds a second key K2. With K2 configured as the signing key, the owner rotates K1 to a new key K3. The script reads the signer's public key out of the SSHSIG sidecar and compares it with the three keys.

```bash
# Finding 6 (LP-4.12): the owner holds K1 and K2, and rotates K1 to K3
# signing with K2.
. ./lib.sh
governed f06
K1=$(grep -h -B1 '^  act: add' "$R"/.decisions/log/*.yml | grep -o 'key:[0-9A-Z]*' | head -1)
K2=$(keygen k2-f06); tl identity add --namespace $NS --key-file "$K2.pub" >/dev/null; commit k2
git -C "$R" config user.signingkey "$K2"
K3=$(keygen k3-f06); tl identity rotate --key-file "$K3.pub" "$K1"; echo "exit $?"; commit rotate
ROT=$(grep -l '^  act: rotate' "$R"/.decisions/log/*.yml); U=$(grep -o 'id: key:[0-9A-Z]*' "$ROT" | cut -d: -f3)
python3 -I - "$R/.decisions/sig/$U.ssh.sig" "$WORK/keys/owner-f06.pub" "$K2.pub" "$K3.pub" <<'PY'
import sys, base64, struct
lines = open(sys.argv[1]).read().split("\n")
blob = base64.b64decode("".join(l for l in lines if l and not l.startswith("-----")))
n = struct.unpack(">I", blob[10:14])[0]; signer = blob[14:14 + n]   # SSHSIG: magic, version, publickey
for p in sys.argv[2:]:
    key = base64.b64decode(open(p).read().split()[1])
    print(p.rsplit("/", 1)[1], "signed the rotate" if key == signer else "did not sign")
PY
v
```

```text
identity rotate: owner@customer.example in `fixture.ledger` — key:01M4ASCTDB6CMSVXS61RHG1DF9
regenerated allowed_signers
filed $WORK/f06/.decisions/log/01M4ASCTDK120EQ2GA4ERE7JA4.yml
exit 0
owner-f06.pub did not sign
k2-f06.pub signed the rotate
k3-f06.pub did not sign
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 11 entries, 0 decision(s)
exit 0
```

**Verdict: confirmed.** The writer filed the rotate signed by K2, and `verify` is conformant.

**Reach.**

- *Passes what the text refuses:* yes. A `rotate` not signed by the key it closes.
- *Authority:* no. The signature is by a live key of the principal, and under the text that key could already do the same in two acts: revoke K1 (LP-4.12: "a `revoke` is the principal's", which D7's table signs with "the principal's live key") and add K3 (a further `add`, signed by a live key). The gap loses the proof of possession of the closed key, not the principal's authority.
- *Cases:* 0. No committed store holds a key binding.

**Choices.**

- **Fix the code to the text.** At verification, a `rotate` verifies only against the key of the binding it closes, and the writer signs a rotate with that key (refusing when the configured key is another). The reproduction turns red at `L011`, and no committed store changes. No digest moves. A store that holds such a rotate would also lose the rotated key's line from `allowed_signers`, since an untrusted binding never reaches it.
- **Amend the text to the code.** LP-4.12 would say that a `rotate` is signed by any live key of the principal. D7's gap paragraph is a ruling record, and the text would then differ from it.
- **Record it as a known limit.** A rotate does not prove possession of the key it closes.

*Lean:* fix the code. It is small, and it is what D7 recorded.

## 6. LP-3.16: `format:` changes are not compared across history

**The text.** LP-3.16: "A correction may only **raise** the declaration, only **to the lowest format the file's content needs** … Lowering a declaration, or raising it past what the content needs, is not a correction." LP-3.5 allows only that exception to immutability.

**The code.** `landed::entities` skips `format` (`("format", _) => {}`), so `verify::history` never compares it. The static rule (LP-3.15) still refuses a declaration below what the content needs, or at or above a retiring format.

**Reproduction.** The `pass` fixture's change-set (content needs format 1) lands declaring format 3. It is then lowered to 1, and then raised to 5, each in its own commit.

```bash
# Finding 7 (LP-3.16): a landed format declaration lowered, then raised past
# what the content needs.
. ./lib.sh
fresh f07 fixture-human@example; from_fixture log pass
F=$R/.decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDXW.yml
sed -i 's/^format: 1$/format: 3/' "$F"; commit "landed at format 3"; pl verify --no-blame | sed -n 2p
sed -i 's/^format: 3$/format: 1/' "$F"; commit "lowered to 1"
git -C "$R" log -1 -p --format= | grep '^[-+]format'; v
sed -i 's/^format: 1$/format: 5/' "$F"; commit "raised to 5"
git -C "$R" log -1 -p --format= | grep '^[-+]format'; v
```

```text
conformant — 4 entries, 1 decision(s)
-format: 3
+format: 1
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 4 entries, 1 decision(s)
notice: namespace `fixture.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace fixture.ledger` opts it in)
exit 0
-format: 1
+format: 5
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 4 entries, 1 decision(s)
notice: namespace `fixture.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace fixture.ledger` opts it in)
exit 0
```

**Verdict: confirmed.** Both edits are conformant.

**Reach.**

- *Passes what the text refuses:* yes. Lowering a landed declaration that still meets the need, and raising one past it.
- *Authority:* no. `format` is not hashed, and LP-3.15 still rejects every declaration a field cannot live under, so no rule's reading changes.
- *Cases:* 0 non-corrections. Three landed files changed `format:`, all `1 → 4` in `.decisions/log/` (`01KZX70EMPA47TBR0PFKX4M32Z`, `01KZX70EQGQCB1B190TS9FZ1A2`, `01KZX70ET1GMR2012XKEP5EWDW`). Each carries `revisit_if`, which needs 4. These are the #81 corrections, and a fix must keep them passing.

**Choices.**

- **Fix the code to the text.** `history` compares a log file's `format` across its first-parent versions. A change is accepted only when it raises to exactly the lowest format the current content needs, with no other entity changed in the same commit. The reproduction turns red, and the three corrections stay green. Which class? A changed declaration is not a changed entity, so `L007` does not literally fit. `SCHEMA` fits "a format declaration below what a field needs" (LP-8.8) only by extension. No digest moves.
- **Amend the text to the code.** LP-3.16 would say that a format declaration may change freely provided LP-3.15 holds at the verified commit.
- **Record it as a known limit.** A landed declaration can wander between valid values. Nothing hashed changes.

*Lean:* amend the text. The rule protects no hashed byte, and LP-3.15 already holds every reading that depends on `format`. But the rule was ruled on 5 October (#81), so this is the principal's call.

## 7. LP-8.9: `L006` judges an escape's `accepted_by` on latest versions only

**The text.** LP-8.9 `L006`: "an acceptance actor, or an escape's `accepted_by`, is refused by section 3.4". LP-8.10 lists the latest-only classes as `L001`, `L003`, `L005`, `L010` and `L014`, and `L006` is not among them. LP-3.7: a model is never "an escape's `accepted_by`".

**The code.** `verify::disposition::model_acceptor` checks every acceptance's actor, but only `view.latest_versions()` for an escape's acceptor.

**Reproduction.** 8a: version 1 is an escape whose `accepted_by` is `claude@example.com`, and version 2, its child, re-allocates the decision to a constraint. 8b, the control: version 1 alone.

```bash
# Finding 8 (LP-8.9): an escape priced by a model identity, on a version that
# is no longer the latest.
. ./lib.sh
escape() {
cat <<'YML'
format: 1
id: cs:01K2C4YQJ3F8M0PT5W7NZ9RDXW
created_at: 2026-08-10T09:14:22Z
created_by: fixture-human@example
decisions:
  - id: dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
    created_at: 2026-08-10T09:14:22Z
    created_by: fixture-human@example
versions:
  - decision: dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
    hash: sha256:acb9e35642e2572fb26618ff51a031cb4ca9559c82ed3a853f9ab8e97547a013
    set: ledger-fixture
    statement: Export column order matches the legacy CSV contract.
    allocation: escaped
    exposure: Downstream consumer may break silently.
    accepted_by: claude@example.com
    review_by: 2027-01-01
    tolerance_floor_at_creation: T1
YML
}
echo "## 8a: the escape, then a child version that re-allocates it"
fresh f08a fixture-human@example; from_fixture sets pass
F=$R/.decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDXW.yml
{ escape; cat <<'YML'
  - decision: dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
    parent: sha256:acb9e35642e2572fb26618ff51a031cb4ca9559c82ed3a853f9ab8e97547a013
    hash: sha256:eaa2baa585ef7f9d523afab5387411cfadbb36dfe791241fe6434518d54e1cd5
    set: ledger-fixture
    statement: Export column order matches the legacy CSV contract.
    allocation: constraint
    discharge: [analyzer:DEC001-csv-order]
    tolerance_floor_at_creation: T1
YML
} > "$F"; commit f; v
echo "## 8b (control): the escape alone, so it is the latest"
fresh f08b fixture-human@example; from_fixture sets pass
escape > "$R/.decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDXW.yml"; commit f; v
```

```text
## 8a: the escape, then a child version that re-allocates it
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 4 entries, 1 decision(s)
1 allocated, awaiting acceptance:
  - dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
notice: namespace `fixture.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace fixture.ledger` opts it in)
exit 0
## 8b (control): the escape alone, so it is the latest
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 1 finding(s):
  - [L006] dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV: escape acceptor `claude` carries the model-actor token `claude`
1 allocated, awaiting acceptance:
  - dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
notice: namespace `fixture.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace fixture.ledger` opts it in)
exit 1
```

**Verdict: confirmed.** 8a is conformant, and 8b fails `L006` on the same version.

**Reach.**

- *Passes what the text refuses:* yes. A model identity as the acceptor of an escape that has since been superseded within its chain.
- *Authority:* no by the prompt's test. An escape's `accepted_by` is not a signed act. It does touch LP-3.7 ("a model is never a holder").
- *Cases:* 0. One escape in `.decisions/` (accepted by `emk@delegate.dk`) and three in the fixtures, none by a model identity.

**Choices.**

- **Fix the code to the text.** Judge the escape's acceptor on every version. 8a turns red, and no committed store changes. No digest moves.
- **Amend the text to the code.** Add `L006`'s escape half to LP-8.10's latest-only list. A second implementation would then skip superseded escapes too.
- **Record it as a known limit.** A model can stand as an escape's acceptor in history, once a later version has replaced the escape.

*Lean:* fix the code. LP-3.7 is a hard constraint, and acceptances are already judged on every version.

## 8. LP-3.4 and LP-4.18 step 7: a YAML float in a hashed string field

**The text.** LP-3.4: "Hashed content is strings only. A floating-point value is a schema fault." LP-4.18 step 7: "No floating-point value may appear in hashed content. A float is a schema fault." Step 2c, the `\v` half of LP-4.18, is settled by ruling 33 and is not reopened here.

**The code.** String fields are `String` in the wire structs (`version.rs`), and `serde_yaml` 0.9 hands a plain scalar to a `String` field as its source text. Nothing checks whether the scalar would resolve to a float.

**Reproduction.** A version whose `statement` is a plain scalar, under several spellings, then `1.5` committed with its digest.

```bash
# Findings 9 and 10 (LP-3.4, LP-4.18 step 7): a YAML float in the hashed
# string field `statement`.
. ./lib.sh
version() {  # version <statement scalar> <hash>
cat <<YML
format: 1
id: cs:01K2C4YQJ3F8M0PT5W7NZ9RDXW
created_at: 2026-08-10T09:14:22Z
created_by: fixture-human@example
decisions:
  - id: dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
    created_at: 2026-08-10T09:14:22Z
    created_by: fixture-human@example
versions:
  - decision: dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
    hash: $2
    set: ledger-fixture
    statement: $1
    allocation: constraint
    discharge: [analyzer:DEC001-no-float-money]
    tolerance_floor_at_creation: T1
YML
}
echo "## 9a: what each scalar hashes to (no file committed)"
for s in "1.5" "'1.5'" "1.50" "1e3" ".inf" "42"; do
  fresh f09x fixture-human@example; from_fixture sets pass
  version "$s" sha256:0000000000000000000000000000000000000000000000000000000000000000 > "$R/.decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDXW.yml"
  printf '%-6s ' "$s"; pl verify --no-blame 2>&1 | grep -E 'SCHEMA|L007' | grep -o 'SCHEMA.*\|hashes to .*'
done
echo "## 9b: the float committed with its digest"
fresh f09 fixture-human@example; from_fixture sets pass
version 1.5 sha256:ee2961751af58d606ad3d4679a7ab8109fa90bbd4e8afe944694026a2c817f32 > "$R/.decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDXW.yml"
commit f; v
```

```text
## 9a: what each scalar hashes to (no file committed)
1.5    hashes to sha256:ee2961751af58d606ad3d4679a7ab8109fa90bbd4e8afe944694026a2c817f32
'1.5'  hashes to sha256:ee2961751af58d606ad3d4679a7ab8109fa90bbd4e8afe944694026a2c817f32
1.50   hashes to sha256:669bbed2d62c68ad932c211f5272d49524e7b6da99643c14bdcc5c63a149bc93
1e3    hashes to sha256:569df7ff2b348ed840bb3e821e51890ab5fdee92a2709fc8177f3b54a2e9ee42
.inf   hashes to sha256:8c72e8ca70337c0799d861dae9003f75d1202e0c86bdacb85207c5a2e0405dcd
42     hashes to sha256:5c00bc83488b638ca18baa2d1e8cde98a5579d08daa7f7ec151eceeed3aebe11
## 9b: the float committed with its digest
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 3 entries, 1 decision(s)
1 allocated, awaiting acceptance:
  - dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
notice: namespace `fixture.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace fixture.ledger` opts it in)
exit 0
```

**Verdict: confirmed.** No spelling is refused. `1.5` and `'1.5'` hash identically, and `1.50` hashes differently from `1.5`, so the hashed string is the scalar's source text as written. The committed store is conformant.

**Reach.**

- *Passes what the text refuses:* yes. A float in hashed content.
- *Authority:* no.
- *Cases:* 0. A scan of every plain scalar in the 236 YAML files of the 16 committed stores (3,480 scalars) found none that YAML 1.2's core schema types as a float, outside the unhashed `format`.

**Choices.**

- **Fix the code to the text.** Refuse, as `SCHEMA`, a plain (unquoted) scalar in a hashed string field that the YAML 1.2 core schema resolves to a float. This needs the scalar's style, which a `String` field does not keep. It means a custom deserializer or a second pass over the `serde_yaml::Value`. No committed store changes and no stored digest moves. The fix still defines which inputs reach the canonical form, which makes it a canonical-form question: `CANONICAL_FORM` would not change, since no accepted input changes its digest, but a file that verified before would stop verifying.
- **Amend the text to the code.** LP-3.4 and step 7 would say that every scalar in a hashed string field is hashed as its source text, quoted or not, and that there is no float in the format. A second implementation could not use a typed YAML loader: one would render `1.50` as `1.5` and hash something else. This also is a canonical-form statement.
- **Record it as a known limit.** A second implementation with a typed loader computes a different digest for such a version, or refuses it. The ledger's digests would then be implementation-defined for those inputs.

*Lean:* fix the code. Refusing keeps the canonical input defined by the text rather than by one YAML library.

## 9. LP-3.3: a `set:` grant scope refuses a dot

**The text.** LP-3.3: "A set and a role are identified by an id of the set-id form". The set file's id is "lowercase alphanumerics, dashes, dots" (section 5.2).

**The code.** `DecisionSet::validate_id` allows dots. `GrantScope::from_str` (`authority/grant.rs`) allows a `set:` payload of lowercase, digits and dashes only.

**Reproduction.** In a governed store, `declare --set money.rules` succeeds, and `grant new … --scope set:money.rules` is refused. A writer-filed grant over `set:money-rules` is then edited to `set:money.rules`.

```bash
# Finding 11 (LP-3.3): a set id with a dot, and a grant scoped to it.
. ./lib.sh
governed f11
pl declare --set money.rules --tolerance-floor T1; commit set
tl grant new acceptor --to architect@customer.example --scope set:money.rules; echo "exit $?"
tl grant new acceptor --to architect@customer.example --scope set:money-rules >/dev/null
F=$(grep -l 'set:money-rules' "$R"/.decisions/log/*.yml); sed -i 's/set:money-rules/set:money.rules/' "$F"
grep -A9 '^grants:' "$F"; v
```

```text
declared set `money.rules` — floor T1, owner owner@customer.example
filed $WORK/f11/.decisions/sets/money.rules.yml
error: `set:money.rules`: a set id is lowercase alphanumerics and dashes
exit 2
grants:
- id: grant:01M4ASCX4Z8VXKC32YJS0GTHG7
  role: acceptor
  scope: set:money.rules
  holder: architect@customer.example
  granted_by: owner@customer.example
  order: primary
  under: grant:01M4ASCWGCM5G305HGR1Y9TMVX
  at: 2026-10-07T08:59:04.221794953Z
  hash: sha256:a3e15fb4a3024ea5de10c02348b737eb6e9d1b25b4cf9290c0bdeebca22b46d8
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 1 finding(s):
  - [SCHEMA] 01M4ASCX4Z8CMBVHKK8Z57N5EJ.yml: change-set file does not parse: grants[0]: `set:money.rules`: a set id is lowercase alphanumerics and dashes at line 7 column 3
exit 1
```

**Verdict: confirmed.** The writer refuses the scope, and the verifier fails the whole change-set at parse.

**Reach.**

- *Passes what the text refuses:* no. The code is stricter than the text: a set with a dot in its id cannot be the scope of a grant.
- *Authority:* no.
- *Cases:* 0. No set id with a dot, and no `set:` grant, in any committed store.

**Choices.**

- **Fix the code to the text.** Accept dots in a `set:` scope. Nothing committed changes. No digest moves: the scope is hashed as written.
- **Amend the text to the code.** Restrict set ids to no dots, which would turn a valid set into a schema fault, or say that a set with a dot cannot be a grant's scope.
- **Record it as a known limit.** Sets with dotted ids cannot carry set-scoped grants.

*Lean:* fix the code. It is a one-line inconsistency.

## 10. LP-8.25: `parents` is keyed apart from the header entity

**The text.** LP-8.25: an entity is one item of one entity list, "or the file's header fields (`id`, `created_at`, `created_by`, `parents`, `note`) taken together."

**The code.** `landed::entities` files every YAML sequence as an entity list. `parents` is a sequence, so each parent becomes its own entity (`parents/<text>`), and adding one reads as a new entity landing, not as a change to the header.

**Reproduction.** A parent is appended to a landed change-set's header. In the control, a `note` is appended instead.

```bash
# Finding 12 (LP-8.25): a parent appended to a landed change-set header; a
# note appended as the control.
. ./lib.sh
for field in "parents: [cs:01K2C4YQJ3F8M0PT5W7NZ9RDX0]" "note: added later"; do
  echo "## add \`$field\` to the landed header"
  fresh f12 fixture-human@example; from_fixture log pass; commit landed
  sed -i "s/^created_by: fixture-human@example\$/created_by: fixture-human@example\n$field/" "$R/.decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDXW.yml"
  commit edit; v
done
```

```text
## add `parents: [cs:01K2C4YQJ3F8M0PT5W7NZ9RDX0]` to the landed header
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 4 entries, 1 decision(s)
notice: namespace `fixture.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace fixture.ledger` opts it in)
exit 0
## add `note: added later` to the landed header
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 1 finding(s):
  - [L007] .decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDXW.yml: the change-set header landed in 680f9bd57d38 (.decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDXW.yml) and has changed — a landed entity is never edited or removed
notice: namespace `fixture.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace fixture.ledger` opts it in)
exit 1
```

**Verdict: confirmed.** The added parent is conformant, while the added note fails `L007` on the header.

**Reach.**

- *Passes what the text refuses:* yes. A landed change-set's `parents` can grow.
- *Authority:* no. `parents` is unhashed and, by the text's own table (`parents` on a change-set: "may be written; still unresolved"), read by no rule. That the added parent names no filed change-set is therefore not a difference.
- *Cases:* 0. No committed change-set carries `parents`.

**Choices.**

- **Fix the code to the text.** Fold `parents` into the header entity. The reproduction turns red. Nothing committed changes. No digest moves.
- **Amend the text to the code.** LP-8.25 would key each parent as its own entity, so that appending a parent to a landed header is allowed. Whether a landed change-set may gain a parent is a merge (L3) question the text leaves open.
- **Record it as a known limit.** A landed change-set's `parents` can be extended without a finding.

*Lean:* fix the code, since the change-set DAG is unresolved and immutability is the safe default until it is.

## 11. LP-9.14: the export-only verifier

**The text.** LP-9.14: "An export-only verifier rebuilds the signed bytes by these rules … It rebuilds `allowed_signers` from the key-binding nodes (section 4.9 …) and verifies each signature as section 4.8 says." Section 4.9 (LP-4.32): "`valid-before` is the `at` of the earliest `rotate` or `revoke` that closed its **key**, in any namespace … only **trusted** bindings are written". LP-9.15: "an export from a green repository holds no act that fails it." Ruling 47 will supersede LP-4.32's "in any namespace" by the binding's own namespace (LP-6.31).

**The code.**

- The only export-only verifier is `ledger-cli/tests/common/export_only.rs`, a test helper. No binary or library function ships one.
- Its `allowed_signers` takes `valid-before` from a close that names that very binding, and writes a line for every `add` and `rotate` node, trusted or not.
- Wider than the survey said: `graph::export::select` keeps only the exported namespace's own key bindings (`kept.key_bindings.retain(|b| b.namespace == reach.namespace)`). A close in another namespace never reaches the export, so no export-only verifier could take `valid-before` per key "in any namespace" from one namespace's export, however it was written.

**Reproduction.** The helper is run unmodified. A scratch crate outside the repository includes it by path, `#[path = "…/ledger-cli/tests/common/export_only.rs"]`, and its `main` prints `export_only::allowed_signers(&export_only::parse(text))` for the export file named on the command line. `$XO` is that binary.

`Cargo.toml` and `src/main.rs` of the scratch crate:

```toml
[package]
name = "xo"
version = "0.0.0"
edition = "2021"
publish = false

[workspace]

[dependencies]
ledger-core = { path = "/home/user/product-cli/ledger-core" }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
```

```rust
//! Runs the repository's own test helper, unmodified, on one export.
#[path = "/home/user/product-cli/ledger-cli/tests/common/export_only.rs"]
#[allow(dead_code)]
mod export_only;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let text = std::fs::read_to_string(&args[1]).expect("export");
    let g = export_only::parse(&text);
    match args.get(2).map(String::as_str) {
        Some("verify") => println!("{:?}", export_only::verify_all(&g, std::path::Path::new(&args[3]))),
        _ => print!("{}", export_only::allowed_signers(&g)),
    }
}
```

13a: the owner's key is bound in `fixture.ledger` and, by `init --namespace second.ns`, in `second.ns`. It is then revoked in `fixture.ledger`. The script compares the committed `allowed_signers` line for `second.ns` with the helper's, made from `second.ns.nt` alone. It then signs a message now with the closed key under `ledger-accept@second.ns`, and verifies it against each line.

13b: a key binding D7 does not allow (mallory binds her own first key) is hand-filed, and the export is regenerated.

```bash
# Finding 13 (LP-9.14): the repository's export-only helper, run unmodified
# (XO is a binary that calls its `allowed_signers`; see the report).
. ./lib.sh
echo "## 13a: a key bound in two namespaces, closed in one"
governed f13a
tl init --namespace second.ns >/dev/null; commit second
KA=$(grep -h -B1 '^  act: add' "$R"/.decisions/log/*.yml | grep -o 'key:[0-9A-Z]*' | head -1)
tl identity revoke "$KA" >/dev/null; commit revoke
add "A." >/dev/null; add "B." second.ns >/dev/null
pl export --format ntriples >/dev/null; commit export
pl verify --no-blame --export | sed -n 2p
for f in "$R"/docs/decisions/*.nt; do echo "$(basename "$f"): binding acts $(grep -o 'bindingAct> "[a-z]*"' "$f" | cut -d'"' -f2 | tr '\n' ' ')"; done
grep second.ns "$R/.decisions/allowed_signers" > "$WORK/as-committed"
"$XO" "$R/docs/decisions/second.ns.nt" | grep -v '^#' > "$WORK/as-helper"
echo "committed: $(cat "$WORK/as-committed")"; echo "helper:    $(cat "$WORK/as-helper")"
printf 'any bytes\n' > "$WORK/msg"; rm -f "$WORK/msg.sig"
ssh-keygen -q -Y sign -f "$WORK/keys/owner-f13a" -n ledger-accept@second.ns "$WORK/msg"
NOW=$(date -u +%Y%m%d%H%M%SZ)
for f in as-helper as-committed; do echo "-- $f at $NOW"
  ssh-keygen -Y verify -f "$WORK/$f" -I $OWNER -n ledger-accept@second.ns -s "$WORK/msg.sig" -Overify-time=$NOW < "$WORK/msg" 2>&1; done
echo "## 13b: a binding D7 does not allow, in the export"
governed f13b; add "A." >/dev/null; commit dec
MK=$(keygen mallory-f13b); F=$R/.decisions/log/01M4AS9000000000000000000A.yml
cat > "$F" <<YML
format: 7
id: cs:01M4AS9000000000000000000A
created_at: 2026-10-07T09:30:00Z
created_by: mallory@example
key_bindings:
- id: key:01M4AS9000000000000000000B
  act: add
  principal: mallory@example
  namespace: fixture.ledger
  key_type: ssh-ed25519
  key: $(cut -d' ' -f2 "$MK.pub")
  by: mallory@example
  at: 2026-10-07T09:30:00Z
  hash: sha256:0000000000000000000000000000000000000000000000000000000000000000
YML
rehash "$F"; pl export --format ntriples >/dev/null; commit mallory
v --export
echo "committed allowed_signers: $(grep -v '^#' "$R/.decisions/allowed_signers" | cut -d' ' -f1 | tr '\n' ' ')"
echo "helper allowed_signers:    $("$XO" "$R/docs/decisions/$NS.nt" | grep -v '^#' | cut -d' ' -f1 | tr '\n' ' ')"
```

```text
## 13a: a key bound in two namespaces, closed in one
conformant — 16 entries, 2 decision(s)
fixture.ledger.nt: binding acts add revoke 
second.ns.nt: binding acts add 
committed: owner@customer.example namespaces="ledger-accept@second.ns",valid-after="20261007085905Z",valid-before="20261007085906Z" ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIC+KB14iD1moRSAu9hz0UtmqYO4ILLByX4J5PLi6vB16
helper:    owner@customer.example namespaces="ledger-accept@second.ns",valid-after="20261007085905Z" ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIC+KB14iD1moRSAu9hz0UtmqYO4ILLByX4J5PLi6vB16
-- as-helper at 20261007085907Z
Good "ledger-accept@second.ns" signature for owner@customer.example with ED25519 key SHA256:D64HqQnZTDzdEn8/X4yzdFXVwoi9G6b0buEXQLeMWAc
-- as-committed at 20261007085907Z
$WORK/as-committed:1: key has expired: verify time 2026-10-07T08:59:07 > valid-before 2026-10-07T08:59:06
Could not verify signature.
## 13b: a binding D7 does not allow, in the export
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 1 finding(s):
  - [SCHEMA] key:01M4AS9000000000000000000B: D7: mallory@example has no live key in `fixture.ledger`: a principal's first key is filed by the genesis holder (D7)
1 allocated, awaiting acceptance:
  - dec:fixture.ledger/01M4ASD1AZT9Z8WDGD5PJ6EJNJ
export: every committed export matches the log byte for byte
exit 1
committed allowed_signers: owner@customer.example 
helper allowed_signers:    mallory@example owner@customer.example 
```

**Verdict: different.** The code differs as claimed: a test helper only, `valid-before` per binding, no filter to trusted bindings. It also differs in a way the survey did not name: the export does not carry the closes the text's per-key rule needs.

- In 13a the repository is green, `verify --export` included. The repository verifier closes the key in `second.ns` too, while the helper, reading `second.ns.nt`, leaves it open, and a signature made after the close verifies against the helper's line and not the committed one.
- In 13b the export matches the log byte for byte while the store is red, and the helper's `allowed_signers` names mallory, whom the repository never trusted.

**Reach.**

- *Passes what the text refuses:* yes, for an export-only reader. A signature by a key closed in another namespace, and a signature by a key whose binding D7 refuses, both verify against the helper's derivation.
- *Authority:* **yes, for an export-only reader only.** An act signed by a closed or untrusted key passes export-only verification. The repository verifier is not affected, and LP-9.15's guarantee covers exports from green repositories only. 13a shows that a green repository's export still yields an `allowed_signers` that differs from the committed one.
- *Cases:* 0. The two committed exports (`docs/decisions/hafeok.ddd.nt`, `hafeok.ledger.nt`) carry no key binding.

**Choices.**

- **Fix the code to the text.** Ship an export-only verifier as a library function or a verb, and make the export carry what LP-4.32 needs: every close of a key the namespace binds, from any namespace, and enough to judge trust (D7 needs the genesis grant and the bindings before each one). The exports change, though no committed export holds bindings, and no stored digest moves. It cuts across ruling 47 and the namespace-independence design session (list 3 of the replies), which this session does not begin.
- **Amend the text to the code.** LP-9.14 would say: `valid-before` from closes present in the export, which are the namespace's own, and every binding in the export is taken as trusted. LP-9.15 would then state the further limit that a key closed elsewhere stays open to an export-only reader. Once ruling 47 is implemented, "a close in the binding's own namespace" is the rule, and a key is bound once per namespace, so per binding and per key coincide inside one namespace. The helper's `valid-before` then matches the text without change.
- **Record it as a known limit.** Until ruling 47 is implemented, an export-only reader of one namespace cannot see a close filed in another, nor tell a trusted binding from an untrusted one in an export of a red repository.

*Lean:* amend the text. State the per-namespace rule now as what one namespace's export can support, which is where ruling 47 lands anyway. Ship a verifier only after the design session settles what an export carries.

## 12. Findings refuted, and what the survey misread

None was refuted. Three readings were narrower than what running showed:

- **3 (LP-5.11, LP-8.8).** The survey saw an unchecked duplicate. Running shows a revocation path. Because an acceptance id resolves to its first filing, a duplicate in an ungoverned namespace makes an unsigned, ungranted revocation reach a signed acceptance in a governed one (3c). That is why the finding is marked authority.
- **2 (LP-6.17).** "Enforced at write only" holds for the `accept_role` half. The genesis role's capability set (no decision capability) is enforced nowhere: the writer adopts a hand-written root role that also carries `accept-decision` (section 13.2).
- **11 (LP-9.14).** Beyond the helper's per-binding derivation, the export itself drops other namespaces' bindings, so the per-key rule cannot be implemented over one namespace's export.

## 13. Anything else found while reproducing

### 13.1 An explicit `null` in a required hashed field is hashed as text

**The text.** LP-3.4: "An explicit `null` is absent." LP-4.18 step 3: an explicit `null` is treated as absent. `statement` is required (`version.rs` refuses an empty one).

**Reproduction.** The `pass` fixture's version with `statement: null`, then `statement: ~`. The acceptance is dropped because the version's digest changes.

```bash
# Section 13: an explicit YAML null in the required hashed field `statement`.
. ./lib.sh
for s in null "~"; do
  fresh f14 fixture-human@example; from_fixture sets pass
  sed "s/^    statement: .*/    statement: $s/; s/^    hash: .*/    hash: sha256:0000000000000000000000000000000000000000000000000000000000000000/" \
    "$FIX/pass/.decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDXW.yml" | sed '/^acceptances:/,$d' > "$R/.decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDXW.yml"
  rehash "$R/.decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDXW.yml"
  echo "## statement: $s -> $(grep -o 'hash: .*' "$R/.decisions/log/01K2C4YQJ3F8M0PT5W7NZ9RDXW.yml")"
  commit f; v
done
```

```text
## statement: null -> hash: sha256:f9575694b70ce7cd3355dfa9b1ce89b29fa7c5f5f50e5d82c0c06a0ae3d7dc4a
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 3 entries, 1 decision(s)
1 allocated, awaiting acceptance:
  - dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
notice: namespace `fixture.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace fixture.ledger` opts it in)
exit 0
## statement: ~ -> hash: sha256:3861c5311715e941248d4e8de8eb395a4faa3c686bc4415bd7a30e523881c354
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 3 entries, 1 decision(s)
1 allocated, awaiting acceptance:
  - dec:fixture.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
notice: namespace `fixture.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace fixture.ledger` opts it in)
exit 0
```

**Result.** Both are conformant, and they hash differently from each other. The code hashes the source texts `"null"` and `"~"` where the text says the field is absent, which here would mean a version without its required statement. This is the same mechanism as finding 8, since a plain scalar reaches a `String` field as its source text, but a different rule. An optional field given `null` deserializes as absent, so the difference is confined to required string fields: `statement` and `set`. A `set: null` names an undeclared set `null` and is already a schema fault. No committed store carries either.

### 13.2 A genesis role carrying a decision capability is refused by nothing

**The text.** LP-6.17: "The genesis (root) role carries `grant-role`, `revoke-grant`, `declare-unavailability` and `rotate-genesis` and none of the decision capabilities (D9 (f))."

**Reproduction.** A hand-written `steward` role carrying the four root capabilities and `accept-decision`, then `ledger init --namespace`.

```bash
# Finding 2, the writer's side: `init` adopts a hand-written root role that
# also carries accept-decision.
. ./lib.sh
fresh f02b; pl declare --set ledger-design --tolerance-floor T1 >/dev/null
mkdir -p "$R/.decisions/roles"
printf 'format: 6\nid: steward\nowner: owner@customer.example\nmay: [grant-role, revoke-grant, declare-unavailability, rotate-genesis, accept-decision]\ncreated_at: 2026-10-07\n' > "$R/.decisions/roles/steward.yml"
tl init --namespace $NS --external-ref "contract 2026/117" --without-key | grep -v '^warning'; echo "exit $?"
commit init; v
```

```text
.decisions/ is already initialised
genesis grant:01M4ASEK820W78Y03S2M2ZS3XS — owner@customer.example holds `steward` over *
declared role `acceptor` — may accept-decision; held by nobody until granted
namespace `fixture.ledger` under policy pol:01M4ASEK82NEQRWWHJ69PDAPHE (accept role `acceptor`)
filed $WORK/f02b/.decisions/log/01M4ASEK82VTPSKAKYQRD83TAX.yml
exit 0
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 6 entries, 0 decision(s)
notice: the genesis holder owner@customer.example has no trusted key — governed namespace(s) `fixture.ledger`: the first self-bound binding to land for that address will be the one trusted (D7); bind one with `ledger identity add --namespace <ns>`
exit 0
```

**Result.** `init` adopts the role as the genesis role and `verify` is conformant. `bootstrap` refuses only a role that *lacks* a root capability, which is what D9 (f) ruled ("`bootstrap` refuses an existing role id that lacks the root capabilities"). The ruling is not reargued here: the text's "none of the decision capabilities" is the part neither writer nor verifier holds. On its own this grants nothing, because acceptance counts only grants of `accept_role`. Combined with finding 2 it is how the genesis grant comes to accept.

## 14. Questions for the principal

Authority findings first. Each lists the choices of its section, in the same order.

1. **Finding 3 (LP-5.11, LP-8.8), authority.** Should duplicate `acc:` ids and decision identity objects filed twice be `SCHEMA`?
   - (a) yes, both, at verification;
   - (b) amend the text to authority ids only, and state the first-match resolution;
   - (c) known limit.
2. **Finding 2 (LP-6.17) with section 13.2, authority.** Should the verifier refuse a policy whose `accept_role` is the genesis role, and a genesis role that carries a decision capability?
   - (a) both, as `SCHEMA`;
   - (b) the `accept_role` half only;
   - (c) amend the text to a writer rule;
   - (d) known limit.
   - If (a) or (b): should `bootstrap` also refuse an existing root role that carries a decision capability? That would extend D9 (f).
3. **Finding 11 (LP-9.14), authority for export-only readers.**
   - (a) amend LP-9.14 and LP-9.15 now to what one namespace's export supports, and wait for ruling 47;
   - (b) ship an export-only verifier and widen the export now;
   - (c) known limit.
   - Under (b): which closes and trust evidence should an export carry? That falls within list 3 of the replies.
4. **Finding 1 (LP-4.7).**
   - (a) a present sidecar on an act no policy governs is `SCHEMA`;
   - (b) it is `L011`, with `L011`'s wording widened;
   - (c) amend LP-4.7 and LP-4.30 to governed acts, and decide whether the export still carries such sidecars;
   - (d) known limit.
5. **Finding 5 (LP-4.12).**
   - (a) verify a `rotate` against the key it closes, and have the writer sign with that key;
   - (b) amend the text to any live key of the principal;
   - (c) known limit.
6. **Finding 4 (LP-8.11).**
   - (a) a forked decision has no latest, and only `G004` reports on it;
   - (b) amend the text to a reporting-only representative;
   - (c) known limit.
7. **Finding 8 (LP-3.4, LP-4.18 step 7).**
   - (a) refuse a plain scalar that resolves to a float, as `SCHEMA`;
   - (b) amend the text so that a scalar in a hashed string field is its source text;
   - (c) known limit.
   - Under (a) or (b): does it carry an Appendix C note, given that no digest moves?
8. **Section 13.1 (LP-3.4, LP-4.18 step 3).**
   - (a) an explicit `null` or `~` in a required string field is absent, and so a schema fault;
   - (b) amend the text so that it is the text `null`;
   - (c) known limit.
   - Best decided with question 7.
9. **Finding 7 (LP-8.9).**
   - (a) judge an escape's acceptor on every version;
   - (b) add it to LP-8.10's latest-only list;
   - (c) known limit.
10. **Finding 6 (LP-3.16).**
    - (a) compare `format:` across history, under which class;
    - (b) amend LP-3.16 so that any declaration LP-3.15 accepts is allowed;
    - (c) known limit.
11. **Finding 9 (LP-3.3).**
    - (a) accept dots in a `set:` scope;
    - (b) forbid dots in set ids;
    - (c) known limit.
12. **Finding 10 (LP-8.25).**
    - (a) fold `parents` into the header entity;
    - (b) amend the text to key parents apart;
    - (c) known limit.

## 15. Proposed follow-up

One issue for each "fix the code" choice, to be filed by the principal. None is opened here.

| Issue title | Protocol section | Size | Depends on | Appendix C note |
| --- | --- | --- | --- | --- |
| Refuse duplicate acceptance ids and re-filed decision identity objects | §5.3 (LP-5.11), §8.2 (LP-8.8) | S | — | No: no format or digest change |
| Hold "the accept role is never the genesis role" at verification | §6.1 (LP-6.17), §8.4 (LP-8.16) | S | — | No |
| Refuse a genesis role that carries a decision capability (verify and `bootstrap`) | §6.1 (LP-6.17) | S | the previous issue; question 2's extension of D9 (f) | No |
| Ship an export-only verifier and carry what it needs in the export | §9.3 (LP-9.14, LP-9.15), §4.9 (LP-4.32) | L | ruling 47 implemented; the namespace-independence design session | Yes: the export's content changes |
| Verify every present sidecar, governed or not | §4.8 (LP-4.7, LP-4.30), §8.2 (LP-8.8) | S | question 4's choice of class | No |
| Verify a `rotate` against the key it closes | §4.10 (LP-4.12) | S | — | No |
| Give a forked decision no latest in the file gate | §8.3 (LP-8.10, LP-8.11) | M | — | No |
| Refuse a YAML float in a hashed string field | §3 (LP-3.4), §4.3 (LP-4.18 step 7) | M | — | The principal's call (question 7): no digest moves, but a file that verified stops verifying |
| Treat an explicit null in a required string field as absent | §3 (LP-3.4), §4.3 (LP-4.18 step 3) | S | the previous issue (same deserializer) | As the previous issue |
| Judge an escape's `accepted_by` on every version (`L006`) | §8.3 (LP-8.9, LP-8.10) | S | — | No |
| Compare `format:` declarations across a log file's history | §3.2 (LP-3.16), §8.7 (LP-8.30) | M | question 10's choice of class | No |
| Accept dots in a `set:` grant scope | §3 (LP-3.3), §6.1 (LP-6.16) | S | — | No |
| Key a change-set's `parents` with its header entity | §8.7 (LP-8.25) | S | — | No |

## 16. Checks run

- **Reproductions.** Every script in this report was run from an empty work directory against `ledger` built from `55c3bcf`, and the outputs above are from that run. 3c was run twice: the first run's id lookup matched inside a `gacc:` id, and the pattern was tightened before the run shown.
- **Committed-store scans.** Over `.decisions/` and the 15 fixture stores:
  - sidecars and `sig/` directories;
  - authority records;
  - duplicate `acc:` ids and decision identity objects;
  - forked chains (`ledger verify` over every fixture);
  - `format:` changes across first-parent history;
  - escapes and their acceptors;
  - plain scalars a YAML 1.2 core schema types as a float;
  - set ids with dots;
  - `parents`;
  - key bindings in the committed exports.
- **Gates** (git-identity variables unset, as `CLAUDE.md` and the prompt require):

  - `cargo build`: exit 0.
  - `cargo t`: exit 0, 2,083 passed, 0 failed, 2 ignored, summed over every `test result:` line.
  - `cargo clippy -- -D warnings -D clippy::unwrap_used`: exit 0.
  - `dotnet test` (`eval-dotnet/`, `spec-flow/`) was not run, since no .NET code is touched.

- **`ledger verify --export`** on this branch, compared with `main` at `55c3bcf`: exit 0 on both, and the output is byte-identical: `conformant — 284 entries, 93 decision(s)`, three allocated and awaiting acceptance, two notices for namespaces with no policy, and "every committed export matches the log byte for byte".

- **What changed.** One file was added, this one, and nothing else changed. The reproduction stores, the scratch crate and their outputs live outside the repository and are not committed.
