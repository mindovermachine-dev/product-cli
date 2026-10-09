# Namespace independence: design session

Session of 7 October 2026, under rulings 32 and 41 to 48. It writes the design as a PRD, `ledger/prd/namespace-independence-prd.md`, and this record. It decides nothing. It changes no code, test, fixture, protocol text, ruling, store, export or `.ddd/` file.

## The repository

The prompt was written against `main` at `55c3bcf`. When this session started, `main` was at `e20fadc`. The only change between the two is `ledger/sessions/2026-10-verification.md`, added by mindovermachine-dev/product-cli#114, which this session read. No code changed between them, so every claim about code holds for both commits.

A session fixing the verified findings may be running at the same time. Nothing here depends on its outcome. The PRD notes where it touches the same code (N12, the duplicate acceptance id).

The local clone was shallow: 193 commits, 51 on the first-parent line. The experiment needs real landing order, so it ran on a separate clone, unshallowed from GitHub: 855 commits, 501 on the first-parent line.

## What was read

In the order the prompt gives.

1. **Rulings.** Every file in `ledger/rulings/`:
   - `absorption-replies-rulings-2026-10-07.md`;
   - `basis-and-absorption-rulings-2026-10-06.md`;
   - `ground-and-protocol-rulings-2026-10.md`;
   - `signing-rulings-2026-10.md`;
   - `signing-rulings-2026-10-d5-d9.md`, for D6's position table and definitions.

   Also `ledger/sessions/2026-10-protocol-absorption.md`, "Principal's replies, 2026-10-07": list 3 and the list of requirements ruling 47 will supersede.
2. **The protocol**, `ledger/spec/ledger-protocol.md`:
   - sections 1 to 3, 4.6 to 4.11, 5 (all), 6, 7, 8, 9, 10.1 and 12;
   - every requirement marked superseded by rulings 41 to 47: LP-3.8, LP-3.18, LP-4.12, LP-4.31, LP-4.32, LP-4.37, LP-4.38, LP-4.39, LP-5.19, LP-6.5, LP-6.16, LP-6.28, LP-8.23, LP-8.31 and LP-9.11.
3. **The verification session**, `ledger/sessions/2026-10-verification.md`: its method and `lib.sh`, finding 3 (cases 3a to 3c), finding 11 (cases 13a and 13b), and §12 to §16.
4. **The code.**
   - `ledger-core/src/authority/`: every file, with `key_close.rs`, `signers.rs`, `filing.rs`, `view.rs`, `check.rs`, `structure.rs` and `references.rs` read in full.
   - `landing.rs`, `landed.rs`, `store.rs`, `init.rs`, `blame.rs`.
   - `graph/export.rs` (`select`, `Reach`), `graph/shapes.rs` (the `A003` and `A005` shapes, `graph_findings_in`).
   - `signing/subject.rs`, `signing/check.rs`.
   - `verify/mod.rs`, `verify/acts.rs`, `verify/history.rs`, `verify/keys.rs`, `verify/authority.rs`, `verify/integrity.rs` (`blame_consistency`).
   - `author/authority_ops.rs` (`init_namespace`, `bootstrap`, `join_genesis`), and `ledger-cli/src/commands/init.rs`.
5. **The tests that encode sharing.**
   - `ledger-cli/tests/key_across_namespaces.rs` (six tests) and `genesis_key.rs` (the four tests on a later namespace).
   - Every `init --namespace` call in `ledger-cli/tests/*.rs`, to find the stores built with two governed namespaces.
   - The 15 committed fixture stores under `ledger-cli/tests/fixtures/`, scanned: one namespace each, one set each, no role files, no authority records.

## Survey of this repository's store

From a scan of `.decisions/`, by `grep` and a Python YAML read:

- **Namespaces.** Two, both with no policy.
  - `hafeok.ddd`: 164 log files, set `ddd-governance`.
  - `hafeok.ledger`: 23 log files, set `ledger-design`.
- **Mixed files.** No log file holds entities of both namespaces.
- **References between them.**
  - No version carries a `dec:` token in `based_on` or `revisit_if`.
  - No version carries `supersedes`.
  - No version is `exported`.
- **Authority.** No authority record, role file, sidecar or `allowed_signers`.
- **Format declarations.** 151 files declare format 1, 33 format 3, 3 format 4.
- **Acceptors.** Every acceptance's actor is `emk@delegate.dk`.

## Environment

- `ledger` built from `e20fadc` with `cargo build -p ledger-cli`, and copied to `$SP/tools/ledger`.
- `git filter-repo` 2.47.0, installed into the scratchpad from its wheel.
- OpenSSH 9.6p1, installed: the container lacked `ssh-keygen`.
- `script(1)` from util-linux, for the verbs that refuse a non-terminal caller.
- The four `GIT_AUTHOR_*` and `GIT_COMMITTER_*` variables are unset in `lib.sh`.

`$SP` stands for the session's scratchpad directory. In the outputs below the absolute path is replaced by `$SP`. ULIDs, timestamps and key fingerprints differ on every run. Every output is from one run of all six scripts, in the order shown.

## The experiment: commands

### `lib.sh` and `glib.sh`

```bash
# lib.sh: shared helpers for the extraction experiment.
set -u
unset GIT_AUTHOR_NAME GIT_AUTHOR_EMAIL GIT_COMMITTER_NAME GIT_COMMITTER_EMAIL
SP=${SP:?the session scratchpad directory}
L=$SP/tools/ledger                      # ledger built from e20fadc
GFR="python3 $SP/tools/gfr/git_filter_repo.py"
FULL=$SP/exp/full                       # unshallowed clone of main at e20fadc
NS=hafeok.ddd
v() { "$L" --root "$1" verify "${@:2}"; echo "exit $?"; }
# the files that hold hafeok.ddd: its log files, its set, its export
ddd_files() { (cd "$1" && grep -l "dec:$NS/" .decisions/log/*.yml; echo .decisions/sets/ddd-governance.yml; echo docs/decisions/$NS.nt); }
```

```bash
# glib.sh: helpers for the governed store, after the verification session's lib.sh.
. "$(dirname "$0")/lib.sh"
OWNER=owner@customer.example; SECOND=second@customer.example
A=alpha.ns; B=beta.ns
pl() { "$L" --root "$R" "$@"; }
tl() {                                                    # at a terminal, as a person
  local line="'$L' --root '$R'"; local a
  for a in "$@"; do line="$line '${a//\'/\'\\\'\'}'"; done
  script -qefc "$line" /dev/null </dev/null | tr -d '\r'; return "${PIPESTATUS[0]}"
}
as() { git -C "$R" config user.email "$1"; git -C "$R" config user.signingkey "$2"; }
commit() { git -C "$R" add -A && git -C "$R" commit -qm "$1"; }
keygen() { mkdir -p "$SP/exp/keys"; local k=$SP/exp/keys/$1; rm -f "$k" "$k.pub"; ssh-keygen -q -t ed25519 -N "" -C "$1" -f "$k"; echo "$k"; }
add() { pl add --set "$2" --namespace "$3" --statement "$1" --store constraint --discharge analyzer:DEC001 | grep -o 'dec:[^ ]*' | head -1; }
word() { grep -o "$1[0-9A-Z]*" | head -1; }
```

### `classify.py`: which namespace each entity of each log file reaches

```python
"""classify.py <store>: the namespace(s) each entity of each log file reaches."""
import sys, glob, os, yaml
root = sys.argv[1]
want = sys.argv[2] if len(sys.argv) > 2 else None
grants = {}
files = sorted(glob.glob(root + "/.decisions/log/*.yml"))
docs = {f: yaml.safe_load(open(f)) for f in files}
sets = {}
for d in docs.values():
    for v in d.get("versions") or []:
        sets[v["set"]] = v["decision"].split(":")[1].split("/")[0]
for d in docs.values():
    for g in d.get("grants") or []:
        s = g["scope"]
        grants[g["id"]] = ("*" if s == "*" else s[3:] if s.startswith("ns:") else sets.get(s[4:], "?") if s.startswith("set:") else "?")
accs = {}
for d in docs.values():
    for a in d.get("acceptances") or []:
        accs[a["id"]] = a["decision"].split(":")[1].split("/")[0]
def ns_of(kind, e):
    if kind in ("decisions",): return e["id"].split(":")[1].split("/")[0]
    if kind in ("versions", "acceptances"): return e["decision"].split(":")[1].split("/")[0]
    if kind in ("key_bindings", "policies"): return e["namespace"]
    if kind == "grants": return grants[e["id"]]
    if kind == "grant_acceptances": return grants.get(e["grant"], "?")
    if kind == "revocations":
        t = e.get("revokes") or e.get("acceptance")
        return accs.get(t) or grants.get(t, "?")
    return "?"
out = []
for f, d in docs.items():
    seen = []
    for kind, items in d.items():
        if isinstance(items, list) and kind != "parents":
            for e in items:
                seen.append((kind, ns_of(kind, e), e.get("id") or e.get("hash")))
    nss = sorted({n for _, n, _ in seen})
    if want is None:
        print(os.path.basename(f), ",".join(nss), " ".join(f"{k}:{n}" for k, n, _ in seen))
    elif want in nss or "*" in nss:
        print(f"{os.path.basename(f)} {'pure' if nss == [want] else 'mixed'} " + " ".join(i.split(':',1)[1] for _,_,i in seen if i and ':' in i and not i.startswith('sha256')))
```

### E1: `hafeok.ddd` copied into a fresh repository; the source after removal

```bash
# E1: copy hafeok.ddd's files into a fresh repository, one commit, then
# remove them from the source in one commit.
. "$(dirname "$0")/lib.sh"
W=$SP/exp/e1; rm -rf "$W"; mkdir -p "$W"
D=$W/ddd; git init -q --initial-branch=main "$D"
git -C "$D" config user.name Extractor; git -C "$D" config user.email extractor@example
git -C "$D" config commit.gpgsign false
ddd_files "$FULL" > "$W/files"; echo "files: $(wc -l < "$W/files")"
(cd "$FULL" && tar cf - -T "$W/files") | tar xf - -C "$D"
printf '.decisions/index/\n' > "$D/.gitignore"
git -C "$D" add -A; git -C "$D" commit -qm "extract hafeok.ddd (copy)"
echo "## E1a: target, committed by extractor@example"; v "$D" --export > "$W/e1a.out" 2>&1; grep -v "^  - \[L009\]" "$W/e1a.out"; grep -c "\[L009\]" "$W/e1a.out"
echo "## E1b: target, --no-blame"; v "$D" --export --no-blame | tail -2
echo "## E1c: target, committed as emk@delegate.dk (the acceptor)"
git -C "$D" commit -q --amend --reset-author --no-edit -c user.email=emk@delegate.dk 2>/dev/null || \
  git -C "$D" -c user.email=emk@delegate.dk -c user.name=emk commit -q --amend --reset-author --no-edit
git -C "$D" log -1 --format='author %ae'; v "$D" --export | tail -3
echo "## E1d: source after the files are removed in one commit"
S=$W/src; git clone -q "$FULL" "$S"; git -C "$S" remote remove origin
git -C "$S" config user.name Extractor; git -C "$S" config user.email extractor@example
(cd "$S" && xargs git rm -q < "$W/files"); git -C "$S" commit -qm "move hafeok.ddd out"
v "$S" --export > "$W/src.out" 2>&1; sed -n 2p "$W/src.out"; grep -o "^  - \[[A-Z0-9]*\]" "$W/src.out" | sort | uniq -c; grep -v "^  - " "$W/src.out"
```

### E2: `hafeok.ddd` with its history carried; then merged into a repository with history

```bash
# E2: carry hafeok.ddd's files with their git history into a fresh
# repository (git filter-repo, path filter), then into a repository that
# already has history (merge of unrelated histories).
. "$(dirname "$0")/lib.sh"
W=$SP/exp/e2; rm -rf "$W"; mkdir -p "$W"
ddd_files "$FULL" > "$W/files"
D=$W/ddd; git clone -q --no-local "$FULL" "$D"
$GFR --source "$D" --target "$D" --force --quiet --paths-from-file "$W/files" 2>&1 | tail -2
git -C "$D" remote remove origin 2>/dev/null
echo "commits: $(git -C "$D" rev-list --count HEAD), first-parent: $(git -C "$D" rev-list --first-parent --count HEAD), source first-parent: $(git -C "$FULL" rev-list --first-parent --count HEAD)"
echo "tree: $(git -C "$D" ls-files | wc -l) files; identical bytes: $(cd "$D" && git ls-files | while read f; do cmp -s "$f" "$FULL/$f" || echo DIFF "$f"; done | wc -l) differ"
echo "## E2a: the filtered history, as is"; v "$D" --export
echo "## E2b: landing index of every log file: source first-parent index -> filtered first-parent index"
idx() { git -C "$1" rev-list --first-parent --reverse HEAD | awk '{print $1, NR-1}' | sort > "$W/order"
  git -C "$1" log --first-parent --diff-merges=first-parent --reverse --name-only --diff-filter=A --format='C %H' HEAD -- .decisions/log \
   | awk '/^C /{c=$2; next} NF{print c, $1}' | sort > "$W/adds"
  join "$W/order" "$W/adds" | awk '{print $3, $2}' | sort -u -k1,1; }
idx "$FULL" > "$W/full.idx"; idx "$D" > "$W/ddd.idx"
join "$W/full.idx" "$W/ddd.idx" | awk '{print $2, "->", $3}' | sort -n | uniq -c
echo "## E2c: into a repository that already has history, merged as unrelated history"
T=$W/target; git init -q --initial-branch=main "$T"
git -C "$T" config user.name Owner; git -C "$T" config user.email owner@target.example; git -C "$T" config commit.gpgsign false
echo hi > "$T/README"; git -C "$T" add -A; git -C "$T" commit -qm "existing history"
git -C "$T" fetch -q "$D" HEAD:extracted
git -C "$T" merge -q --allow-unrelated-histories --no-edit extracted -m "bring in hafeok.ddd"
echo "first-parent: $(git -C "$T" rev-list --first-parent --count HEAD), all: $(git -C "$T" rev-list --count HEAD)"
v "$T" --export
```

### E3: a governed store with two namespaces

```bash
# E3: a governed store with two namespaces, alpha.ns and beta.ns, built with
# the verbs as today's rules allow; then a key close in alpha.ns.
. "$(dirname "$0")/glib.sh"
R=$SP/exp/e3/store; rm -rf "$SP/exp/e3"; mkdir -p "$R"
git -C "$R" init -q --initial-branch=main; git -C "$R" config user.name Person; git -C "$R" config commit.gpgsign false
K1=$(keygen owner-k1); K2=$(keygen owner-k2); KS=$(keygen second)
as $OWNER "$K1"; pl init >/dev/null
pl declare --set set-a --tolerance-floor T1 >/dev/null; pl declare --set set-b --tolerance-floor T1 >/dev/null; commit sets
echo "## init alpha.ns (genesis, self-bound K1)"; tl init --namespace $A --external-ref "contract 2026/117" | grep -v '^$'; commit "init alpha"
echo "## init beta.ns (joins the genesis; K1 carried over)"; tl init --namespace $B | grep -v '^$'; commit "init beta"
GA=$(tl grant new acceptor --to $OWNER --scope ns:$A | word grant:); tl grant accept "$GA" >/dev/null
GB=$(tl grant new acceptor --to $OWNER --scope ns:$B | word grant:); tl grant accept "$GB" >/dev/null; commit "owner acceptor grants"
echo "## second's first key in beta.ns, filed by the genesis holder; second's acceptor grant"
tl identity add --namespace $B --for $SECOND --key-file "$KS.pub" | grep -v '^$'
GS=$(tl grant new acceptor --to $SECOND --scope set:set-b | word grant:); commit "second: key and grant"
as $SECOND "$KS"; tl grant accept "$GS" >/dev/null; commit "second accepts grant"
as $OWNER "$K1"
DA=$(add "Alpha decides." set-a $A); DB=$(add "Beta decides." set-b $B); DB2=$(add "Beta decides again." set-b $B); commit decisions
echo "DA=$DA DB=$DB DB2=$DB2" | tee "$SP/exp/e3/ids"
tl accept "$DA" --as acceptor | grep -v '^$' | head -2; tl accept "$DB" | grep -v '^$' | head -2; commit "owner accepts"
as $SECOND "$KS"; tl accept "$DB2" | grep -v '^$' | head -2; commit "second accepts"
pl export --format ntriples >/dev/null; commit export
echo "## verify before the close"; v "$R" --export
sleep 1.1
echo "## owner rotates K1 to K2 in alpha.ns: today the close ends K1 in beta.ns too"
as $OWNER "$K1"; KA=$(python3 - "$R" <<'PY'
import sys,glob,yaml
for f in sorted(glob.glob(sys.argv[1]+"/.decisions/log/*.yml")):
    d=yaml.safe_load(open(f))
    for b in d.get("key_bindings",[]) or []:
        if b["namespace"]=="alpha.ns" and b["act"]=="add" and b["principal"]=="owner@customer.example": print(b["id"])
PY
)
tl identity rotate "$KA" --key-file "$K2.pub" | grep -v '^$'; commit "rotate in alpha"
pl export --format ntriples >/dev/null; commit export2
echo "## verify after the close"; v "$R" --export
echo "## allowed_signers"; cat "$R/.decisions/allowed_signers"
echo "## files"; (cd "$R" && find .decisions docs -type f | sort)
```

### E4: `beta.ns` extracted from E3's store

```bash
# E4: extract beta.ns from the governed store E3 built, carrying history
# (git filter-repo) — (a) beta's own files, (b) plus the file holding the
# genesis grant, (c) by copying into one commit; then the source side.
. "$(dirname "$0")/glib.sh"
S=$SP/exp/e3/store; W=$SP/exp/e4; rm -rf "$W"; mkdir -p "$W"
pick() { # pick <pure|all>: beta's log files, their sidecars, set, export, roles
  python3 "$SP/scripts/classify.py" "$S" $B > "$W/beta.cls"
  awk -v m="$1" '($2=="pure" || m=="all"){print ".decisions/log/"$1}' "$W/beta.cls"
  awk -v m="$1" '($2=="pure" || m=="all"){for(i=3;i<=NF;i++) print $i}' "$W/beta.cls" | while read u; do
    [ -f "$S/.decisions/sig/$u.ssh.sig" ] && echo ".decisions/sig/$u.ssh.sig"; done
  echo .decisions/sets/set-b.yml; echo docs/decisions/$B.nt; echo .decisions/roles/acceptor.yml; echo .decisions/roles/steward.yml
  echo .decisions/allowed_signers; echo .gitignore
}
extract() { # extract <dir> <pure|all>
  local D=$1; pick "$2" > "$D.files"; git clone -q --no-local "$S" "$D"
  $GFR --source "$D" --target "$D" --force --quiet --paths-from-file "$D.files" >/dev/null 2>&1
  git -C "$D" remote remove origin 2>/dev/null; git -C "$D" config user.name Person; git -C "$D" config commit.gpgsign false
  echo "kept $(git -C "$D" ls-files | wc -l) files over $(git -C "$D" rev-list --first-parent --count HEAD) first-parent commits (source $(git -C "$S" rev-list --first-parent --count HEAD))"
}
echo "## E4a: beta.ns's own files, history carried"
extract "$W/a" pure; v "$W/a" --export
echo "## E4a': the same, allowed_signers regenerated (ledger identity sync) and committed"
R=$W/a; as $OWNER "$SP/exp/keys/owner-k2"; pl identity sync >/dev/null; git -C "$R" diff --stat | tail -1; commit sync; v "$R" --export
echo "## E4b: plus the change-set that holds the genesis grant (and alpha.ns's policy and binding), history carried"
extract "$W/b" all; v "$W/b" --export
R=$W/b; pl identity sync >/dev/null; git -C "$R" diff --stat | tail -1; commit sync; echo "-- synced"; v "$R" --export
echo "## E4c: E4b's files copied into one commit authored by the owner"
D=$W/c; mkdir -p "$D"; git init -q --initial-branch=main "$D"; (cd "$S" && tar cf - -T "$W/b.files") | tar xf - -C "$D"
R=$D; git -C "$R" config user.name Person; git -C "$R" config commit.gpgsign false; as $OWNER "$SP/exp/keys/owner-k2"; commit copy; v "$R" --export
echo "## E4d: the source after beta.ns's own files are removed"
R=$W/src; git clone -q --no-local "$S" "$R"; git -C "$R" remote remove origin; git -C "$R" config user.name Person; git -C "$R" config commit.gpgsign false; as $OWNER "$SP/exp/keys/owner-k2"
(cd "$R" && grep -v 'roles/\|allowed_signers\|gitignore' "$W/a.files" | xargs git rm -q); commit "move beta out"
v "$R" --export > "$W/src.out" 2>&1; sed -n 2p "$W/src.out"; grep -o '^  - \[[A-Z0-9]*\]' "$W/src.out" | sort | uniq -c; grep -v '^  - \[L007\]' "$W/src.out" | sed 1,2d
echo "-- after identity sync"; pl identity sync >/dev/null; commit sync; v "$R" --export > "$W/src2.out" 2>&1; grep -o '^  - \[[A-Z0-9]*\]\|^\[[A-Z]*\]' "$W/src2.out" | sort | uniq -c; grep -v '^  - \[L007\]' "$W/src2.out" | sed 1,2d
```

### E5: landing order after a move

```bash
# E5: landing order after a move. An acceptance dated before its namespace's
# first policy but landed after it is governed (LP-8.28). Moved (a) with
# history, (b) by copy into one commit, (c) by merging the carried history
# into an existing repository as unrelated history.
. "$(dirname "$0")/glib.sh"
W=$SP/exp/e5; rm -rf "$W"; mkdir -p "$W"; R=$W/src
git init -q --initial-branch=main "$R"; git -C "$R" config user.name Person; git -C "$R" config commit.gpgsign false
K=$(keygen owner-e5); as $OWNER "$K"; pl init >/dev/null; pl declare --set set-g --tolerance-floor T1 >/dev/null
D=$(add "Gamma decides." set-g gamma.ns); commit "decision"
EARLY=$(date -u +%Y-%m-%dT%H:%M:%SZ); sleep 1.1
tl init --namespace gamma.ns --external-ref "contract 2026/117" >/dev/null; commit "policy"
H=$(python3 -c "
import yaml,glob
for f in glob.glob('$R/.decisions/log/*.yml'):
    for v in (yaml.safe_load(open(f)).get('versions') or []): print(v['hash'])")
cat > "$R/.decisions/log/01M4B0000000000000000000AA.yml" <<YML
format: 1
id: cs:01M4B0000000000000000000AA
created_at: $EARLY
created_by: $OWNER
acceptances:
  - id: acc:01M4B0000000000000000000AB
    decision: $D
    version: $H
    actor: $OWNER
    at: $EARLY
YML
commit "backdated acceptance, landed after the policy"; pl export --format ntriples >/dev/null; commit export
echo "## E5-source"; v "$R" --export
echo "## E5a: carried history (filter-repo over every path)"
A=$W/a; git clone -q --no-local "$R" "$A"; git -C "$A" remote remove origin
$GFR --source "$A" --target "$A" --force --quiet --path .decisions --path docs --path .gitignore >/dev/null 2>&1; v "$A" --export | sed 1d
echo "## E5b: copied into one commit"
B=$W/b; mkdir -p "$B"; git init -q --initial-branch=main "$B"; git -C "$B" config user.name Person; git -C "$B" config commit.gpgsign false
(cd "$R" && git ls-files | tar cf - -T -) | tar xf - -C "$B"; R=$B; as $OWNER "$K"; commit copy; v "$B" --export | sed 1d
echo "## E5c: carried history merged into an existing repository as unrelated history"
C=$W/c; git init -q --initial-branch=main "$C"; git -C "$C" config user.name Owner; git -C "$C" config user.email $OWNER; git -C "$C" config commit.gpgsign false
echo hi > "$C/README"; git -C "$C" add -A; git -C "$C" commit -qm "existing"
git -C "$C" fetch -q "$A" HEAD:carried; git -C "$C" merge -q --allow-unrelated-histories --no-edit carried
v "$C" --export | sed 1d
echo "## E5d: the same merge, verified with --base at the merge's first parent (the target before the move)"
v "$C" --export --base HEAD~1 | sed 1d
```

### Coupling-inventory stores: a shared set, a cross-namespace `supersedes`, a mixed change-set

```bash
# Minimal stores for coupling-inventory entries not shown by E1-E5.
. "$(dirname "$0")/glib.sh"
W=$SP/exp/c; rm -rf "$W"; mkdir -p "$W"
fresh() { R=$W/$1; mkdir -p "$R"; git -C "$R" init -q --initial-branch=main; git -C "$R" config user.name Person; git -C "$R" config user.email $OWNER; git -C "$R" config commit.gpgsign false; pl init >/dev/null; }
echo "## C-set: one set, decisions of a.ns and b.ns; a.ns's owner raises the floor"
fresh set; pl declare --set shared --tolerance-floor T0 >/dev/null
DA=$(add "A." shared a.ns); DB=$(add "B." shared b.ns); commit two
sed -i 's/^tolerance_floor: T0/tolerance_floor: T1/' "$R/.decisions/sets/shared.yml"; commit raise
v "$R" --no-blame | grep -v '^landing\|notice'
echo "## C-sup: a decision of a.ns supersedes one of b.ns; b.ns's status, coverage and key"
fresh sup; pl declare --set s --tolerance-floor T1 >/dev/null
DB=$(pl add --set s --namespace b.ns --statement B. --store constraint --discharge analyzer:X --key Shared | grep -o "dec:[^ ]*" | head -1); DA=$(pl add --set s --namespace a.ns --statement A. --store constraint --discharge analyzer:X | grep -o "dec:[^ ]*" | head -1); commit two
pl supersede "$DB" --by "$DA" 2>&1 | tail -1; commit sup
v "$R" --no-blame | grep -v '^landing\|notice'; pl status | grep -i "b.ns\|supersed"; pl coverage 2>&1 | grep -i "supersed\|b.ns" | head -4
echo "## C-cs: one change-set holding versions of a.ns and b.ns"
fresh cs; pl declare --set s --tolerance-floor T1 >/dev/null; DA=$(add "A." s a.ns); DB=$(add "B." s b.ns)
F2=$(grep -l "$DB" "$R"/.decisions/log/*.yml); F1=$(grep -l "$DA" "$R"/.decisions/log/*.yml)
python3 - "$F1" "$F2" <<'PY'
import sys,yaml
a=yaml.safe_load(open(sys.argv[1])); b=yaml.safe_load(open(sys.argv[2]))
a['decisions']+=b['decisions']; a['versions']+=b['versions']
open(sys.argv[1],'w').write(yaml.safe_dump(a,sort_keys=False))
PY
rm "$F2"; pl export --format ntriples >/dev/null; commit merged-file
v "$R" --no-blame --export | grep -v '^landing\|notice'
CS=$(basename "$F1" .yml); for f in "$R"/docs/decisions/*.nt; do echo "$(basename $f): $(grep -c "urn:cs:$CS>" "$f") triples on cs:$CS"; done
```

## The experiment: results

### E1

```text
files: 166
## E1a: target, committed by extractor@example
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 79 finding(s):
2 allocated, awaiting acceptance:
  - dec:hafeok.ddd/01KZTGGMEACBFMTC1RJJ8T90GS
  - dec:hafeok.ddd/01KZTGGX5ABSQ2PVTQ32NPKVNE
notice: namespace `hafeok.ddd` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ddd` opts it in)
export: every committed export matches the log byte for byte
exit 1
79
## E1b: target, --no-blame
export: every committed export matches the log byte for byte
exit 0
## E1c: target, committed as emk@delegate.dk (the acceptor)
author emk@delegate.dk
notice: namespace `hafeok.ddd` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ddd` opts it in)
export: every committed export matches the log byte for byte
exit 0
## E1d: source after the files are removed in one commit
non-conformant — 408 finding(s):
    408   - [L007]
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 408 finding(s):
1 allocated, awaiting acceptance:
notice: namespace `hafeok.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ledger` opts it in)
export: every committed export matches the log byte for byte
exit 1
```

E1a's findings in full (`$W/e1a.out`):

```text
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 79 finding(s):
  - [L009] acc:01KZXCED6PQD7XQBXA36C6EHZD: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXGPY5GEYHNG2XWGRNBZZK2: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXGRAK0QAH4D0NC1BE1FCXJ: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXGSFTA7QRC9VJGPS79B7P8: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEF9JJP3PXDXYCQJCJTCA: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEFD53HNEP6K0KEWHXVH8: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEFGXYNXSNTE861V522PM: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEFM9Q0VAFWNAB24JPKPB: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEFQH3EW45NNARZ4MYWJ1: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEFTW74VV7R374KGEXSCB: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEFYB7FD64SQMNV0412DZ: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEG2A5F861KT52YRBE47Z: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEG6AFS6VCKHK895FJ9VD: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEGA6620VD6A3PM08V1KX: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEGD6NBB32DHWR3JFEJY5: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEGG8HNTP5HS2QM8CQX57: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEGJXVQQ34ZJ12NCW7WM5: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEGNHJ1GAF26773321DVK: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEGRJ9N38Z0DNTX7CTSVR: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEGV9YS9DT6A3QTA812VT: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEGXWPZEVYGE4D5F4CP28: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEH0M6F6FPWVDN052GF6F: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEH3E6J1NP00Z0P3GHASG: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEH6AB51TA5PBNCMV9320: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEH9FZZG5T0SFXFN8YPAP: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEHCMWGTDBX6YEBN6PKQF: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEHFAA5FPVVH1VBV06PZJ: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEHHWMPSMKPH620NPN4VQ: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEHN3NF577MAPKDPMATDF: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEHR16B996XRWGYCZWMRP: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEHV8G6Y3S8YZTRRYTGW1: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEHY1EDFXND6EX4ZP0QEB: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEJ0Q2CXY533JKG568H69: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEJ3J9F39EP95B42M9RRN: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEJ6JZ39DT8Y44XBWV088: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEJ9J0C47V0K26514CKGK: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEJCHC108WN1SGFM4V0E1: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEJFTCGP73S3JP36F71SN: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEJKMZMZNCPHGKVY80VJR: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEJQ4FN3Y8VHB1Y700VJ3: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEJV4NE1KWE8AQAX73QV3: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEJYAPPCK4G8E1Y81MR4D: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEK1JE3CVEVC9SHTKTM6J: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEK4PZGAEZ2NDKMT5WG0R: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEK87KN0HPPWNB8D8E56A: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEKBWEMS7CGBHTRW5DQM9: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEKF3Y4R99H5ZQMTCSSS2: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEKJBS16R98TMDFZ3WF26: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEKN8YP2XPSN0JCA4NNVQ: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEKRD7ZG32GRWQB811BWZ: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEKVDAAD9ECWZ9DFC99V9: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEKYF9N87P8V49NFQP366: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEM1CJJXNDND6GGGE6MM5: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEM5X9QA8H3TAXE3YTHQS: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEM9784MA8V9V7VA2ZKET: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEMCM2J1WZM27D9JC43MM: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEMG0BSFTHB4BSWXZH1CB: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEMK08M3HQW1M82XNHK6R: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEMP4DJJ381N3R15VTAKV: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEMSFTTJK1N7KPS7RD77V: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEMWR8D9G8F2634HT6GN4: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEN042JV0FM1STY6X0ZX2: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEN47STG2NS9SSWYQGW9C: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEN8ZRRGPQ0GNPK71EWQ2: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHENDA39SSMJXH3NB9P8M3: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHENGJPTGZ5S7AVK2R9CM4: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHENKTGWT98QZ89QZZP780: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHENQEDP1P4BRSA4A30BV8: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHENTSYAY1M5FH9Y63NVQZ: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHENYP3SSER2QEAGHD6NZ8: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEP2E06G6H33XSV6TJZE2: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEP5Z52Z1EDVYB0S40SZH: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEP9MKT408MG5FVSZTKBK: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEPD4VWGZRFBDWYAKQ8WA: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEPGKYGZW6TZER72K2TZJ: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEPMAWJMBMMSDSRNGGQGP: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEPRDYQX0J556E1YD04ZG: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEPWKFZ7RH95EKD5DV2JD: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
  - [L009] acc:01KZXHEQ0QPRXAW09KNDH63XDC: names emk@delegate.dk but was introduced by commit author extractor@example — an acceptance is signed by the actor who deposits it
2 allocated, awaiting acceptance:
  - dec:hafeok.ddd/01KZTGGMEACBFMTC1RJJ8T90GS
  - dec:hafeok.ddd/01KZTGGX5ABSQ2PVTQ32NPKVNE
notice: namespace `hafeok.ddd` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ddd` opts it in)
export: every committed export matches the log byte for byte
exit 1
```

E1d's findings (`$W/src.out`): 408 lines of one form, one per landed entity of the removed files. By kind:
- 164 change-set headers;
- 85 `versions` entries;
- 80 `decisions` entries;
- 79 `acceptances` entries.

The first three:

```text
  - [L007] .decisions/log/01KZTGGGS1TF32NE5ZREW3A55B.yml: the change-set header landed in 75b5472c844e (.decisions/log/01KZTGGGS1TF32NE5ZREW3A55B.yml) and is gone — a landed entity is never edited or removed
  - [L007] .decisions/log/01KZTGGGXG6CVNYHVZWD4VSXXN.yml: the change-set header landed in 75b5472c844e (.decisions/log/01KZTGGGXG6CVNYHVZWD4VSXXN.yml) and is gone — a landed entity is never edited or removed
  - [L007] .decisions/log/01KZTGGH25SFKCGSFCXJMTM7VC.yml: the change-set header landed in 75b5472c844e (.decisions/log/01KZTGGH25SFKCGSFCXJMTM7VC.yml) and is gone — a landed entity is never edited or removed
```

### E2

```text
New history written in 0.24 seconds; now repacking/cleaning...
Completely finished after 0.39 seconds.
commits: 9, first-parent: 6, source first-parent: 501
tree: 166 files; identical bytes: 0 differ
## E2a: the filtered history, as is
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 245 entries, 80 decision(s)
2 allocated, awaiting acceptance:
  - dec:hafeok.ddd/01KZTGGMEACBFMTC1RJJ8T90GS
  - dec:hafeok.ddd/01KZTGGX5ABSQ2PVTQ32NPKVNE
notice: namespace `hafeok.ddd` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ddd` opts it in)
export: every committed export matches the log byte for byte
exit 0
## E2b: landing index of every log file: source first-parent index -> filtered first-parent index
     79 449 -> 0
      5 451 -> 1
     79 452 -> 2
      1 477 -> 3
## E2c: into a repository that already has history, merged as unrelated history
first-parent: 2, all: 11
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 245 entries, 80 decision(s)
2 allocated, awaiting acceptance:
  - dec:hafeok.ddd/01KZTGGMEACBFMTC1RJJ8T90GS
  - dec:hafeok.ddd/01KZTGGX5ABSQ2PVTQ32NPKVNE
notice: namespace `hafeok.ddd` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ddd` opts it in)
export: every committed export matches the log byte for byte
exit 0
```

### E3

```text
## init alpha.ns (genesis, self-bound K1)
.decisions/ is already initialised
genesis grant:01M4AZWKA588JCEK5FFSZGF7XE — owner@customer.example holds `steward` over *
declared role `acceptor` — may accept-decision; held by nobody until granted
namespace `alpha.ns` under policy pol:01M4AZWKA51PBD1CY2BP1AMSB6 (accept role `acceptor`)
bound $SP/exp/keys/owner-k1 as owner@customer.example's first key (ssh-ed25519), self-bound; it signs the policy — key:01M4AZWKA7H3HP11SDMYDAPEJT
regenerated allowed_signers
filed $SP/exp/e3/store/.decisions/log/01M4AZWKA4NXW8EQ7CN7ZBDGTQ.yml
## init beta.ns (joins the genesis; K1 carried over)
.decisions/ is already initialised
namespace `beta.ns` under policy pol:01M4AZWKFEDX0GSR7FFF7YNWFK (accept role `acceptor`)
bound $SP/exp/keys/owner-k1 (ssh-ed25519) for owner@customer.example in `beta.ns`, signed by their key trusted elsewhere — key:01M4AZWKFGPA0ZD641MTGBYAVM
signed pol:01M4AZWKFEDX0GSR7FFF7YNWFK with owner@customer.example's live key
regenerated allowed_signers
filed $SP/exp/e3/store/.decisions/log/01M4AZWKFE4WJEVSZQWBWY6C8A.yml
## second's first key in beta.ns, filed by the genesis holder; second's acceptor grant
identity add: second@customer.example in `beta.ns` — key:01M4AZWMHPS8Y0K6JW3YRCHGFR
regenerated allowed_signers
filed $SP/exp/e3/store/.decisions/log/01M4AZWMHXKB93FS6J4E2Y9FQ7.yml
DA=dec:alpha.ns/01M4AZWNBH4NR4R9DQ362G00TH DB=dec:beta.ns/01M4AZWNKCE7DN4EPHCK02JVE9 DB2=dec:beta.ns/01M4AZWNTX2W09A05SDW4Q7XZS
owner@customer.example accepted 017d46917661 of dec:alpha.ns/01M4AZWNBH4NR4R9DQ362G00TH — the signature names this exact state
under grant:01M4AZWKPZMJY98HJ89D37EWPY (`acceptor`)
owner@customer.example accepted 0c6ef8598e9f of dec:beta.ns/01M4AZWNKCE7DN4EPHCK02JVE9 — the signature names this exact state
under grant:01M4AZWM3WW471XRK9HH16P8B1 (`acceptor`)
second@customer.example accepted 754d8d179971 of dec:beta.ns/01M4AZWNTX2W09A05SDW4Q7XZS — the signature names this exact state
under grant:01M4AZWMTYQ327XKWZ055YFZ7Q (`acceptor`)
## verify before the close
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 26 entries, 3 decision(s)
export: every committed export matches the log byte for byte
exit 0
## owner rotates K1 to K2 in alpha.ns: today the close ends K1 in beta.ns too
identity rotate: owner@customer.example in `alpha.ns` — key:01M4AZWRTB1B82VKW5NMDTV6GR
regenerated allowed_signers
filed $SP/exp/e3/store/.decisions/log/01M4AZWRTJVNC73VHP1C3ZNCE2.yml
## verify after the close
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 27 entries, 3 decision(s)
2 acceptance(s) under a since-closed key need re-acceptance (L012 after the deadline):
  - acc:01M4AZWP5DZ103RT7AV3N4R820 (owner@customer.example): key key:01M4AZWKA7H3HP11SDMYDAPEJT closed by key:01M4AZWRTB1B82VKW5NMDTV6GR — re-accept or affirm
  - acc:01M4AZWPJ38KYZNT184NG20T5C (owner@customer.example): key key:01M4AZWKA7H3HP11SDMYDAPEJT closed by key:01M4AZWRTB1B82VKW5NMDTV6GR — re-accept or affirm
export: every committed export matches the log byte for byte
exit 0
## allowed_signers
# Derived from the key-binding entries in .decisions/log by `ledger identity`.
# Never edit by hand: `ledger verify` holds this file byte-identical to the log.
owner@customer.example namespaces="ledger-accept@alpha.ns",valid-after="20261007105229Z",valid-before="20261007105235Z" ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIBnxYmUYtadIRno32+YTrEDKOkf+TF4I4TE7n/2ULRTu
owner@customer.example namespaces="ledger-accept@alpha.ns",valid-after="20261007105235Z" ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIE81DKiF4AL9fgIcAsN6UqiPbttrq9zoSA+456Dg76Jm
owner@customer.example namespaces="ledger-accept@beta.ns",valid-after="20261007105230Z",valid-before="20261007105235Z" ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIBnxYmUYtadIRno32+YTrEDKOkf+TF4I4TE7n/2ULRTu
second@customer.example namespaces="ledger-accept@beta.ns",valid-after="20261007105231Z" ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHr1t4FR4OROhV2ka5dM/ZbnPYeWdnticTutG2pmNVnF
## files
.decisions/allowed_signers
.decisions/log/01M4AZWKA4NXW8EQ7CN7ZBDGTQ.yml
.decisions/log/01M4AZWKFE4WJEVSZQWBWY6C8A.yml
.decisions/log/01M4AZWKPZAHZFVYWGV4G3MG2S.yml
.decisions/log/01M4AZWKXJ6JKACM2ADTDFYK0V.yml
.decisions/log/01M4AZWM3W7E1B0T0GAKRG1X90.yml
.decisions/log/01M4AZWMAF67J3Q4TGP5ZRGFDR.yml
.decisions/log/01M4AZWMHXKB93FS6J4E2Y9FQ7.yml
.decisions/log/01M4AZWMV045ZN8ZM2ZM8C55QR.yml
.decisions/log/01M4AZWN39HRN6441PBB89CKFR.yml
.decisions/log/01M4AZWNBHZ5C4HZN5VKYK8T6C.yml
.decisions/log/01M4AZWNKCKF5GNSFCKAS0T90A.yml
.decisions/log/01M4AZWNTXR6DGB9GD7NM080MH.yml
.decisions/log/01M4AZWP5KH5FFAZ5ZQ0B52XCK.yml
.decisions/log/01M4AZWPJA06RGJ2993APWYZV7.yml
.decisions/log/01M4AZWQ0EETR684XR27W23KYH.yml
.decisions/log/01M4AZWRTJVNC73VHP1C3ZNCE2.yml
.decisions/roles/acceptor.yml
.decisions/roles/steward.yml
.decisions/sets/set-a.yml
.decisions/sets/set-b.yml
.decisions/sig/01M4AZWKA51PBD1CY2BP1AMSB6.ssh.sig
.decisions/sig/01M4AZWKA7H3HP11SDMYDAPEJT.ssh.sig
.decisions/sig/01M4AZWKFEDX0GSR7FFF7YNWFK.ssh.sig
.decisions/sig/01M4AZWKFGPA0ZD641MTGBYAVM.ssh.sig
.decisions/sig/01M4AZWMHPS8Y0K6JW3YRCHGFR.ssh.sig
.decisions/sig/01M4AZWP5DZ103RT7AV3N4R820.ssh.sig
.decisions/sig/01M4AZWPJ38KYZNT184NG20T5C.ssh.sig
.decisions/sig/01M4AZWQ07S9GCHNK95QYNSEMV.ssh.sig
.decisions/sig/01M4AZWRTB1B82VKW5NMDTV6GR.ssh.sig
docs/decisions/alpha.ns.nt
docs/decisions/beta.ns.nt
```

### E4

```text
## E4a: beta.ns's own files, history carried
kept 21 files over 11 first-parent commits (source 12)
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 8 finding(s):
  - [SCHEMA] key:01M4AZWKFGPA0ZD641MTGBYAVM: D7: owner@customer.example has no live key in `beta.ns`: a principal's first key is filed by the genesis holder (D7)
  - [SCHEMA] key:01M4AZWMHPS8Y0K6JW3YRCHGFR: D7: a principal's act on its own keys names no `under`
  - [L011] acc:01M4AZWPJ38KYZNT184NG20T5C: its `ssh` signature does not hold: signed by SHA256:wXMSHU/5GFRAHZ89MzWR8XsXWe7G/uLpvWXZrkdNiFo, which is no key bound to owner@customer.example in `beta.ns`
  - [L011] acc:01M4AZWQ07S9GCHNK95QYNSEMV: its `ssh` signature does not hold: signed by SHA256:Xkq4nA239v4rcpUFmUcT3UeBohLu6U20E/+eFpQQH8Q, which is no key bound to second@customer.example in `beta.ns`
  - [L011] pol:01M4AZWKFEDX0GSR7FFF7YNWFK: its `ssh` signature does not hold: signed by SHA256:wXMSHU/5GFRAHZ89MzWR8XsXWe7G/uLpvWXZrkdNiFo, which is no key bound to owner@customer.example in `beta.ns`
graph stage — 1 finding(s):
  - [A006] pol:01M4AZWKFEDX0GSR7FFF7YNWFK: owner@customer.example sets `beta.ns`'s policy with no live genesis grant as of the policy — a policy is the genesis holder's act
export stage — 1 finding(s):
  - [EXPORT] docs/decisions/beta.ns.nt: does not match the log: 36 line(s) the log does not produce, 0 line(s) it lacks — regenerate with `ledger export --format ntriples --namespace beta.ns`
trust-root stage — 1 finding(s):
  - [SIGNERS] .decisions/allowed_signers: allowed_signers is committed but the log binds no key — it trusts what nothing filed
exit 1
## E4a': the same, allowed_signers regenerated (ledger identity sync) and committed
On branch main
nothing to commit, working tree clean
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 8 finding(s):
  - [SCHEMA] key:01M4AZWKFGPA0ZD641MTGBYAVM: D7: owner@customer.example has no live key in `beta.ns`: a principal's first key is filed by the genesis holder (D7)
  - [SCHEMA] key:01M4AZWMHPS8Y0K6JW3YRCHGFR: D7: a principal's act on its own keys names no `under`
  - [L011] acc:01M4AZWPJ38KYZNT184NG20T5C: its `ssh` signature does not hold: signed by SHA256:wXMSHU/5GFRAHZ89MzWR8XsXWe7G/uLpvWXZrkdNiFo, which is no key bound to owner@customer.example in `beta.ns`
  - [L011] acc:01M4AZWQ07S9GCHNK95QYNSEMV: its `ssh` signature does not hold: signed by SHA256:Xkq4nA239v4rcpUFmUcT3UeBohLu6U20E/+eFpQQH8Q, which is no key bound to second@customer.example in `beta.ns`
  - [L011] pol:01M4AZWKFEDX0GSR7FFF7YNWFK: its `ssh` signature does not hold: signed by SHA256:wXMSHU/5GFRAHZ89MzWR8XsXWe7G/uLpvWXZrkdNiFo, which is no key bound to owner@customer.example in `beta.ns`
graph stage — 1 finding(s):
  - [A006] pol:01M4AZWKFEDX0GSR7FFF7YNWFK: owner@customer.example sets `beta.ns`'s policy with no live genesis grant as of the policy — a policy is the genesis holder's act
export stage — 1 finding(s):
  - [EXPORT] docs/decisions/beta.ns.nt: does not match the log: 36 line(s) the log does not produce, 0 line(s) it lacks — regenerate with `ledger export --format ntriples --namespace beta.ns`
trust-root stage — 1 finding(s):
  - [SIGNERS] .decisions/allowed_signers: allowed_signers is committed but the log binds no key — it trusts what nothing filed
exit 1
## E4b: plus the change-set that holds the genesis grant (and alpha.ns's policy and binding), history carried
kept 24 files over 11 first-parent commits (source 12)
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 1 finding(s):
trust-root stage — 1 finding(s):
  - [SIGNERS] .decisions/allowed_signers: does not match the key bindings in the log — regenerate it with `ledger identity sync` — it is never edited by hand
export: every committed export matches the log byte for byte
exit 1
 1 file changed, 2 insertions(+), 3 deletions(-)
-- synced
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 20 entries, 2 decision(s)
export: every committed export matches the log byte for byte
exit 0
## E4c: E4b's files copied into one commit authored by the owner
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 2 finding(s):
  - [L009] acc:01M4AZWQ07S9GCHNK95QYNSEMV: names second@customer.example but was introduced by commit author owner@customer.example — an acceptance is signed by the actor who deposits it
trust-root stage — 1 finding(s):
  - [SIGNERS] .decisions/allowed_signers: does not match the key bindings in the log — regenerate it with `ledger identity sync` — it is never edited by hand
export: every committed export matches the log byte for byte
exit 1
## E4d: the source after beta.ns's own files are removed
non-conformant — 29 finding(s):
     28   - [L007]
      1   - [SIGNERS]
trust-root stage — 1 finding(s):
  - [SIGNERS] .decisions/allowed_signers: does not match the key bindings in the log — regenerate it with `ledger identity sync` — it is never edited by hand
1 acceptance(s) under a since-closed key need re-acceptance (L012 after the deadline):
  - acc:01M4AZWP5DZ103RT7AV3N4R820 (owner@customer.example): key key:01M4AZWKA7H3HP11SDMYDAPEJT closed by key:01M4AZWRTB1B82VKW5NMDTV6GR — re-accept or affirm
export: every committed export matches the log byte for byte
exit 1
-- after identity sync
     28   - [L007]
1 acceptance(s) under a since-closed key need re-acceptance (L012 after the deadline):
  - acc:01M4AZWP5DZ103RT7AV3N4R820 (owner@customer.example): key key:01M4AZWKA7H3HP11SDMYDAPEJT closed by key:01M4AZWRTB1B82VKW5NMDTV6GR — re-accept or affirm
export: every committed export matches the log byte for byte
exit 1
```

### E5

```text
## E5-source
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 2 finding(s):
  - [L011] acc:01M4B0000000000000000000AB: `gamma.ns`'s policy requires a `ssh` signature; none is filed
graph stage — 1 finding(s):
  - [A006] acc:01M4B0000000000000000000AB: a governed act names no grant (`under`) — owner@customer.example acts under one
export: every committed export matches the log byte for byte
exit 1
## E5a: carried history (filter-repo over every path)
non-conformant — 2 finding(s):
  - [L011] acc:01M4B0000000000000000000AB: `gamma.ns`'s policy requires a `ssh` signature; none is filed
graph stage — 1 finding(s):
  - [A006] acc:01M4B0000000000000000000AB: a governed act names no grant (`under`) — owner@customer.example acts under one
export: every committed export matches the log byte for byte
exit 1
## E5b: copied into one commit
conformant — 10 entries, 1 decision(s)
export: every committed export matches the log byte for byte
exit 0
## E5c: carried history merged into an existing repository as unrelated history
conformant — 10 entries, 1 decision(s)
export: every committed export matches the log byte for byte
exit 0
## E5d: the same merge, verified with --base at the merge's first parent (the target before the move)
conformant — 10 entries, 1 decision(s)
export: every committed export matches the log byte for byte
exit 0
```

### Coupling-inventory stores

```text
## C-set: one set, decisions of a.ns and b.ns; a.ns's owner raises the floor
non-conformant — 2 finding(s):
  - [L005] dec:a.ns/01M4AZWYGNV9YV1E13B1W91QQH: effective tier T0 is below set `shared`'s floor T1 — the raise re-opens acceptance at the new floor (§4.2.1)
  - [L005] dec:b.ns/01M4AZWYHN19TXP9795C1XARC4: effective tier T0 is below set `shared`'s floor T1 — the raise re-opens acceptance at the new floor (§4.2.1)
2 allocated, awaiting acceptance:
  - dec:a.ns/01M4AZWYGNV9YV1E13B1W91QQH
  - dec:b.ns/01M4AZWYHN19TXP9795C1XARC4
notice: namespace `a.ns` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace a.ns` opts it in)
notice: namespace `b.ns` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace b.ns` opts it in)
exit 1
## C-sup: a decision of a.ns supersedes one of b.ns; b.ns's status, coverage and key
filed $SP/exp/c/sup/.decisions/log/01M4AZWYRN01265Y6SHWTG4CXR.yml
conformant — 6 entries, 2 decision(s)
2 allocated, awaiting acceptance:
  - dec:a.ns/01M4AZWYQ5984MF4DAEEFEJRBN
  - dec:b.ns/01M4AZWYP4NM456RH2QGSC4AQR
exit 0
  dec:b.ns/01M4AZWYP4NM456RH2QGSC4AQR [ab4690df342f] superseded by dec:a.ns/01M4AZWYQ5984MF4DAEEFEJRBN
  s: awaiting-acceptance 1 · superseded 1
  b.ns: superseded 1
  dec:b.ns/01M4AZWYP4NM456RH2QGSC4AQR -> dec:a.ns/01M4AZWYQ5984MF4DAEEFEJRBN
## C-cs: one change-set holding versions of a.ns and b.ns
conformant — 5 entries, 2 decision(s)
2 allocated, awaiting acceptance:
  - dec:a.ns/01M4AZWYX4A0524VW9W328S34B
  - dec:b.ns/01M4AZWYY5W3JP8C79XWB7GVGM
export: every committed export matches the log byte for byte
exit 0
a.ns.nt: 6 triples on cs:01M4AZWYX5N8F528FFGZ302314
b.ns.nt: 6 triples on cs:01M4AZWYX5N8F528FFGZ302314
```

Two earlier runs of these scripts were thrown away, and they explain two lines in the scripts:
- E4 first ran in clones that still had `origin`. `verify` then computed landing against `origin/HEAD` and overlaid the base's log files on the branch (LP-8.29), so files removed in the branch came back. Every clone now drops its remote.
- C-cs first merged two landed files, which is `L007`. It now writes the mixed change-set before its first commit.

## What the experiment shows

1. **The target side of this repository's extraction works today.**
   - With history carried, it is conformant (E2a).
   - A copy fails only `L009`, once per acceptance (E1a, 79 findings). It passes when the sole acceptor makes the copy commit (E1c).
   - No hash changed and no file's bytes changed in any case (E2: 166 files, 0 differ).
2. **The source side does not work.** Removing a namespace's files is `L007` for every landed entity (E1d: 408; E4d: 28).
3. **In a governed store, a namespace cannot be extracted alone today.**
   - Its genesis grant, roles, trusted keys and first-policy signature live in another namespace's files (E4a: D7 twice, `L011` three times, `A006`, `[EXPORT]`, `[SIGNERS]`).
   - Taking the file that holds the genesis grant along makes it verify (E4b). But that file also holds the other namespace's policy and binding, and a rotate left behind changes a verdict: an acceptance stops awaiting re-acceptance.
4. **A move that collapses landing order is permissive** (E5). An unsigned acceptance with no grant, dated before the policy and landed after it, fails at the source. It passes after a copy and after a merge into a repository with history of its own. Carrying the history into a fresh repository keeps the verdict (E5a).
5. **Shared sets, cross-namespace `supersedes`, and change-sets with two namespaces all pass today, and each couples verdicts or exports** (C-set, C-sup, C-cs).

## What could not be determined

- **Re-laying out a store into directories under today's code.** The loader reads only the flat layout (`store::load`), so the effect of a re-layout on landing and `L009` was reasoned, not run. The PRD marks it as inference (§3.5, §3.11).
- **Whether `git log --follow -S` gives the introducing author across a rename** for every acceptance in this repository. Not run, and marked as inference in §3.11.
- **Whether the writer always binds and signs with the same key in a later namespace.** The PRD's migration step M4 depends on it. E3 shows one run where it did ("bound … owner-k1 … signed by their key trusted elsewhere", with only K1 configured). The code path `author::genesis_key` was read, not tested with two keys.
- **`A003` and `A005` on two separately founded genesis grants.** Read from the SPARQL in `graph::shapes`, not run.
- **Stores outside this repository with two governed namespaces.** None was available, so how much of migration step M4's residue exists in practice is unknown.
- **Whether `A001`, `A002` and `A004` are reserved.** They appear in no class list in the protocol or the code read.
- **The pinning format's number.** It depends on whether the basis work lands before or after the move act.
- **Whether the analyzers' reader tolerates namespaced set and role IRIs.** That repository is outside this session's reach.

## Questions for the principal (first draft; ruled by rulings 62 to 80)

The PRD's §7, in one list. Questions that change the layout or the format come first, and each lean is listed first among its options.

1. **Layout.** A directory per namespace, `.decisions/ns/<ns>/` (lean), or the flat layout with each file's namespace inferred or declared?
2. **The one-namespace form.** Does a flat store with one namespace stay valid, so that single-namespace stores need no migration (lean), or does every store use directories?
3. **Landing order after a move.** Ruling 32 and D6 cannot both hold as written. Options:
   - pre-move order rests on the namespace's own signed landing record, through a move act (lean);
   - governed namespaces move only into fresh repositories, with their history carried;
   - moved entities are ordered by `at` alone;
   - a per-namespace hash chain;
   - only namespaces with no policy move.
4. **Leaving the source.** Ruling 32 and LP-8.30 cannot both hold. Options:
   - after a landed move act, the whole namespace may be removed in one commit (lean);
   - a whole-directory removal with no act;
   - the source keeps a frozen copy.

   Under the lean, AC-1 adds the move act. Is that within "no hash changes"?
5. **Format numbers.** The layout as spec revision v1.9 with no format number; the move act as format 8; pins with the pinning format (lean). Or a format number for the layout?
6. **The genesis grant's scope.** Keep `*`, read as "this namespace" (lean); require `ns:<own>` on new genesis grants; or rewrite, which moves digests?
7. **Migration of shared authority.**
   - Copy shared records byte-identically into each namespace, keep their original landing, read a carried-over first `add` signed by its own key as the opening binding, and record an exemption for any residue (lean).
   - Or the residue stays red.
   - Or store-wide rules for old files, which leaves those namespaces coupled.
8. **A close in every namespace.** One change-set per namespace from one invocation, plus a repository notice for a key closed in one namespace and open in another (lean), or without the notice?
9. **A pin's key material.** Genesis grant hash and anchor binding hash (lean); genesis hash alone; a key set; or a policy hash?
10. **Two unrelated namespaces of one name.** Told apart by genesis hash, with one live pin per name (lean), or a local alias in the token, which changes ruling 34's form?
11. **Where pinned material is held.** A vendored export snapshot held by the dependent, even inside one repository (lean); log files; or the named versions only?
12. **Pinning an ungoverned namespace.** Not possible (lean), or `content-addressed`?
13. **Set and role IRIs.** Namespaced (lean), or unchanged?
14. **`supersedes` across namespaces in existing stores.** Judged on live claims, so a revision repairs it (lean), or on every version?
15. **Unpinned `dec:` tokens of another namespace.** Refused from the pinning format (lean), or opaque forever?
16. **Class ids.** `G007` for cycles, `G008` for a non-exported pin, `A007` for a move mismatch (lean)? Are `A001`, `A002` and `A004` free?
17. **`exported` on which version.** On the pinned version itself (lean), or on the decision's tip?
18. **`L009` for arrived acceptances.** Compared with the landing record's author (lean), or skipped and reported?
19. **Instability for an isolated namespace.** Reported as "isolated" (lean), or omitted?

## Principal's replies, 2026-10-07

The design was accepted as the basis, with three changes to the leans. All nineteen questions are ruled, as rulings 62 to 80 in `ledger/rulings/namespace-design-rulings-2026-10-07.md`. Ruling 61 amends allocation at import and is outside this design. Ruling 81 follows from 64.

### Rulings, by number

| Ruling | Answers | In short |
| --- | --- | --- |
| 61 | — | Allocation at import comes from a reviewed triage; an unplaced decision is imported unallocated and fails `L001` |
| 62 | Q1 | Each namespace has its own directory, `.decisions/ns/<namespace>/` |
| 63 | Q2 | Every store uses that layout; no flat one-namespace form (**differs from the lean**) |
| 64 | Q3 | The landing record is written once at departure and fixed by a signed move act, not held continuously; D6 before a move (**differs from the lean**) |
| 65 | Q4 | Whole removal after a landed move act; partial removal stays a finding; the move act is within "no hash changes" |
| 66 | Q5 | Layout v1.9 with no format number; move act format 8; pins after |
| 67 | Q6 | Genesis scope `*`, read as this namespace |
| 68 | Q7 | No migration path for shared authority; such a store is re-founded per namespace (**differs from the lean**) |
| 69 | Q8 | One change-set per namespace, in one commit; a notice for a key closed in one namespace and open in another |
| 70 | Q9 | Key material: genesis grant hash and first trusted binding hash |
| 71 | Q10 | Told apart by genesis hash; one live pin per name |
| 72 | Q11 | An export snapshot held by the dependent, inside one repository too |
| 73 | Q12 | An ungoverned namespace cannot be pinned |
| 74 | Q13 | `urn:ledger-set:<ns>/<id>` and `urn:ledger-role:<ns>/<id>` |
| 75 | Q14 | A cross-namespace `supersedes` in an existing store is judged on live claims only |
| 76 | Q15 | Unpinned cross-namespace `dec:` refused from the pinning format |
| 77 | Q16 | `G007`, `G008`, `A007`; `A001`, `A002`, `A004` stay unused |
| 78 | Q17 | The pinned version itself carries `exported` |
| 79 | Q18 | `L009` on an arrived acceptance compares with the record's author |
| 80 | Q19 | "Isolated" |
| 81 | from 64 | No landing ordinals or introducing authors in the export |

### Ruling 64's reasoning, checked

The principal asked to be told, and the work stopped, if the reasoning behind ruling 64 is wrong. It is not.

A continuously held record gives each entity an ordinal on the default branch. A pull request computes its rows against its base, where its entities land at the tip (LP-8.29). Take two pull requests open on one base, each adding an entity to the namespace:
- both write the same next ordinal;
- whichever merges second carries a wrong value, whether or not git reports a textual conflict;
- the default branch then fails the check until the record is regenerated.

This was checked by reasoning against `landing::Landing::compute` and the first draft's own definition of the ordinal, not run. The design now confines the merge-order dependence to the one pull request that lands a move act (PRD §3.5, N-Q3).

### What changed in the PRD

- **Header.** The document states the ruled design, and marks the points not separately ruled as "accepted with the design". It now rests on `846975a`, which adds rulings 49 to 60 and amends LP-9.14 and LP-9.15 (ruling 51).
- **Sections 1 and 2.** Unchanged, byte for byte.
- **Section 3** states the ruled design topic by topic, citing rulings.
  - **§3.1 (ruling 63).** The one-namespace flat form is removed. A new §3.1.1 answers the history question:
    - the five readers that read history;
    - the two path patterns, told apart by path alone;
    - landing, immutability and `L009` keyed by entity across both;
    - the plain statement that one layout cannot be had without reading the flat layout in history, and what that costs.
  - **§3.5 (ruling 64).** Reworked:
    - no record before a move, and D6 as written;
    - the record written once, by the writer filing the move act, at `ns/<ns>/landing/<move-ulid>`;
    - checked against git by the source's verifier at the commit where the act lands;
    - read by the target for arrived entities, with later entities ordered from the target's history after the act;
    - a second move copies the earlier record's rows, and successive records are checked to agree;
    - the continuous check and the `[LANDING]` stage removed;
    - a re-layout uses no record and no move act, only landing keyed by entity.
  - **§3.9 (ruling 81).** Ordinals and authors are removed from the export, and the landing records are not emitted. It states what an export-only reader can no longer check, and what a dependent holding a pinned snapshot cannot know about the pinned namespace. No fix is designed: that is question N-Q2.
  - **§3.11 (ruling 68).** M2 to M4 and the exemption are removed. The migration is the re-layout of this repository and of the fixtures. The tests to rewrite are named, and the Appendix C note telling an earlier governed store to re-found is drafted. The section notes that ruling 68 rests on there being no governed multi-namespace store outside this repository, which is the principal's to answer.
- **Sections 4 to 6** are brought into line.
  - Added AC-63, AC-64, AC-65, AC-68 and AC-81.
  - The protocol text gains LP-3.35 (history readers), LP-8.34 (the record at departure) and LP-8.35 (frozen after a move act), and drops the derived stage.
  - The landing-record issue is folded into the move-act issue (11). Issue 2 grows to L, and issues 9 and 13 shrink. The order and the split are kept.
- **Section 7** maps Q1 to Q19 to rulings 62 to 80 and lists five questions the rulings raise:
  - N-Q1, the flat layout in history;
  - N-Q2, what a pinned snapshot cannot tell its holder;
  - N-Q3, the namespace frozen at its move act;
  - N-Q4, a signed move act in a namespace with no policy. This is the one place two rulings cannot both hold as written: AC-1 moves `hafeok.ddd`, which has no policy;
  - N-Q5, LP-9.6 against ruling 81.
- **Appendix A**, "Options considered", holds the options not chosen, in one table.

### For the principal to file: the analyzers' repository

**Title:** The ledger export's set and role IRIs carry the namespace (ruling 74)

**What changes.** From specification revision v1.9, every export at `docs/decisions/<ns>.nt` writes:
- a set as `<urn:ledger-set:<ns>/<id>>` instead of `<urn:ledger-set:<id>>`;
- a role as `<urn:ledger-role:<ns>/<id>>` instead of `<urn:ledger-role:<id>>`.

This covers `ledger:set` on every version, the set nodes, `ledger:acceptRole` on policies, and a grant's role. No hash changes; only the IRI text does.

**What the reader must check, and may need to change.** Nothing of that repository was read in this session, so each point is a check, not a finding.
1. *Taking the set id.*
   - A reader that takes the set id as the last `/`-separated segment of the IRI (LP-9.8) needs no change. Set ids and role ids contain no `/`, and a namespace contains no `/`.
   - A reader that strips the prefix `urn:ledger-set:` and keeps the rest would now get `<ns>/<id>`, and must take the part after the last `/`.
2. *Reading several exports into one graph.* Sets of the same id in two namespaces were one node and are now two. Anything keyed by the set IRI, such as grouping, counting or generated names, must decide whether it wants per-namespace sets.
3. *Matching against a literal IRI or a fixed prefix.* Fixtures or SPARQL that match `urn:ledger-set:`, `urn:ledger-role:`, or a whole set IRI written out literally, must be updated.
4. *Role IRIs.* A reader that ignores them is unaffected.

**When.** The change lands with issue 13 of the PRD (export per namespace), after the layout and authority work. Exports regenerate in the same change.

## Principal's replies to the questions the rulings raised, 2026-10-07

Three of the five questions are ruled, as rulings 82 to 84 under a new heading "Questions the rulings raised" in `ledger/rulings/namespace-design-rulings-2026-10-07.md`. N-Q2 and N-Q5 go to the pinning design.

| Ruling | Answers | In short |
| --- | --- | --- |
| 82 | N-Q1 | Reading the flat layout in history is a legacy capability, not part of the verifier profile. A verifier without it refuses a repository whose history predates v1.9, and never passes one by collapsing landing order. |
| 83 | N-Q3 | A namespace is frozen in its source from its move act on. A later entity of it there is `A007`, and a move act landing after another entity of its namespace is refiled. |
| 84 | N-Q4 | A move act always exists. It is signed where the namespace's policy requires a signature, and is unsigned and unchecked in a namespace with no policy. |

### What changed in the PRD

- **§3.1.1 (ruling 82).**
  - Every verifier detects flat history, with one path-limited `git log --first-parent` over the flat paths.
  - A verifier without the capability refuses such a repository with exit status 2, naming the first flat commit. Exit 2 rather than a finding is this design's reading of "refuses", and the PRD says so.
  - The six readers of history are now the legacy capability, and the cost of that capability is optional.
  - The reference implementation needs the capability, because this repository's history predates v1.9.
- **§3.5 (rulings 83 and 84).**
  - Every move has a move act. Under policy it is the genesis holder's act, judged by `A006` and signed where the policy requires. With no policy it is unsigned and unchecked.
  - The namespace is frozen from the move act on. A move act whose pull request lands after another entity is refiled with a new record.
  - In a namespace with no policy, the checks on the record still run: against git in the source, and on its digest and manifest in the target. "Unchecked" is read as applying to who filed the act, not to whether the record is true. That reading is this design's, and the PRD marks it so.
- **AC-1** moves `hafeok.ddd` with an unsigned, unchecked move act, uses a verifier with the legacy capability, and checks the freeze. The dependency on N-Q4 is gone.
- **New acceptance criteria.** AC-82 (with and without the capability), AC-83 (the freeze and the refiled act) and AC-84 (the signed act under policy, the unsigned act without). AC-63 keeps only the schema fault at the verified commit.
- **§5.** LP-3.35 states the capability and the refusal, LP-6.33 states ruling 84, and LP-8.35 states ruling 83. LP-9.6 is left as it is, with a note that ruling 81 narrows what it can deliver.
- **§6.** Issue 2 now covers the detection, the refusal and the capability. Issue 12 adds the new criteria.
- **§7** maps N-Q1, N-Q3 and N-Q4 to rulings 82 to 84, and lists N-Q2 and N-Q5 as open for the pinning design.

## Checks

### First draft (commit `4a35c9b`)

- **What changed.** Two files were added: the PRD and this record. The experiment's repositories, scripts, keys and outputs live in the scratchpad and are not committed.
- **Gates**, with the four git-identity variables unset:
  - `cargo build`: exit 0.
  - `cargo clippy -- -D warnings -D clippy::unwrap_used`: exit 0.
  - `cargo t`: exit 0. 2,083 passed, 0 failed and 2 ignored, summed over the 90 `test result:` lines.
  - `dotnet test` was not run, since no .NET code is touched.
- **`ledger verify --export`** on that branch and on `main` at `e20fadc` (the unshallowed clone): exit 0 on both, and byte-identical output after the first line.

### Revision

- **What changed.** The branch merges `main` at `846975a`, which brings rulings 49 to 60 and the amended LP-9.14 and LP-9.15. The pull request's own change against `main`:
  - adds `ledger/rulings/namespace-design-rulings-2026-10-07.md`;
  - changes the PRD and this record.

  Nothing else.
- **Gates**, with the four git-identity variables unset:
  - `cargo build`: exit 0.
  - `cargo clippy -- -D warnings -D clippy::unwrap_used`: exit 0.
  - `cargo t`: exit 0. 2,083 passed, 0 failed and 2 ignored, over 90 `test result:` lines.
- **`ledger verify --export`**, built from this branch and run on this branch and on a worktree of `main` at `846975a`: exit 0 on both, and byte-identical output, shown below.

### Second revision (rulings 82 to 84)

- **What changed.** The rulings file gains rulings 82 to 84. The PRD and this record are updated. Nothing else changed, and `main` has not moved from `846975a`.
- **Gates**, with the four git-identity variables unset:
  - `cargo build`: exit 0.
  - `cargo clippy -- -D warnings -D clippy::unwrap_used`: exit 0.
  - `cargo t`: exit 0. 2,083 passed, 0 failed and 2 ignored, over 90 `test result:` lines.
- **`ledger verify --export`**: byte-identical output on this branch and on a worktree of `main` at `846975a`, the same as shown below.

The first draft's `verify` output:

```text
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 284 entries, 93 decision(s)
3 allocated, awaiting acceptance:
  - dec:hafeok.ddd/01KZTGGMEACBFMTC1RJJ8T90GS
  - dec:hafeok.ddd/01KZTGGX5ABSQ2PVTQ32NPKVNE
  - dec:hafeok.ledger/01KZXJX693301CZSY4XNP643XY
notice: namespace `hafeok.ddd` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ddd` opts it in)
notice: namespace `hafeok.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ledger` opts it in)
export: every committed export matches the log byte for byte
exit 0
```

## Order from the acts

Session of 9 October 2026, on the direction the principal accepted: the order of a namespace's acts comes from the acts themselves, a terminating entry naming what came before it. It tests that direction against the code and the attack cases and rewrites the move part of the PRD around it (§3.5, §4.2, §5.2, §6, §7 and Appendix A.1 of `ledger/prd/namespace-independence-prd.md`). It decides nothing. It changes no code, test, fixture, protocol text, ruling, store, export or `.ddd/` file.

### The repository

The prompt was written with `main` at `18ebc86` and pull request #128 (this branch, head `c7d7e4f`) unmerged. When this session started, `main` was at `918a08d`, the merge of #128: the repository had moved, and the repository wins. So this revision is not a change to #128. The branch `claude/focused-goodall-qh7sm7` was restarted from `918a08d`, keeping its name, and the commit was pushed there. The pull request is opened from a new branch off `main` instead (see the end of this record), because that branch is #128's and #128 has merged. Every claim about code below holds for `918a08d`, which carries the ten verification fixes (rulings 49 to 60) that the prompt says had merged.

The local clone has `origin/main` and full history; the experiment did not need this repository's history beyond its store, which case K copies.

### What was read

In the order the prompt gives.

1. **Rulings.** Every file in `ledger/rulings/` on `main`: `namespace-design-rulings-2026-10-07.md` (61 to 84), `verification-rulings-2026-10-07.md`, `absorption-replies-rulings-2026-10-07.md`, `basis-and-absorption-rulings-2026-10-06.md`, `ground-and-protocol-rulings-2026-10.md`, `signing-rulings-2026-10.md`, and `signing-rulings-2026-10-d5-d9.md`, D5 to D9 in full.
2. **The PRD**, `ledger/prd/namespace-independence-prd.md` as merged, whole; §2 and §3.5 twice.
3. **The protocol**, `ledger/spec/ledger-protocol.md` on `main`: 3.6, 4.6, 4.10, 4.11, 5.6, 6.1 to 6.7, 8.7 (LP-8.24 to LP-8.32), 8.8, 9 (all), and Appendix C's head and note headings.
4. **The code**, every file in full: `ledger-core/src/landing.rs`, `landed.rs`, `verify/history.rs`, `verify/acts.rs`, `verify/order_grid.rs`, `verify/genesis_role.rs`, `verify/mod.rs`, `verify/integrity.rs`; `signing/subject.rs`, `signing/check.rs`, `signing/review.rs`, `signing/ssh.rs` (`sign`, `verify`, `signer_fingerprint`); `authority/view.rs`, `key_close.rs`, `signers.rs`, `binding.rs`, `policy.rs`, `revocation.rs`, `grant.rs`, `role.rs`, `availability.rs`, `filing.rs`, `check.rs`, `structure.rs`, `references.rs`, `payload.rs`; `blame.rs`, `revision.rs`, `canon.rs`, `hash.rs`, `format.rs`, `acceptance.rs`; `author/mod.rs`, `identity_ops.rs`, `policy_ops.rs`, `authority_ops.rs` (`authorized`, `init_namespace`, `revoke_grant`, `revocation`), `acceptance_ops.rs` (outline), `genesis_key.rs`, `sign_ops.rs`; `graph/export.rs` (`select`, `Reach`, `restrict`); `ledger-cli/src/commands/verify.rs`.
5. **The tests that encode D6.** `ledger-core/src/verify/order_tests.rs` and `order_regressions_tests.rs` in full; the names and setups of every test in `ledger-core/src/landing_tests.rs`, `authority/filing_tests.rs`, `signing/check_tests.rs`, `authority/key_close_tests.rs`, `verify/authority_tests.rs`, `verify/genesis_role_tests.rs`, `blame_tests.rs`, and in `ledger-cli/tests/`: `role_position.rs` (in full), `closed_key.rs` (setup), `pre_policy_binding.rs`, `rotate_key.rs`, `policy_authors.rs`, `immutability.rs`, `trust.rs`, `genesis_key.rs` (the forged self-bound test in full), `signing.rs` (the backdated, branch, no-grant and revocation tests in full), `key_across_namespaces.rs`, `key_ownership.rs`, `format_history.rs`, `landed_header.rs`, `legacy_revocation.rs`; the helpers `common/mod.rs`, `common/hand.rs`, `common/export_only.rs` (outline).
6. **Session records.** This record's earlier sections; the headings of `2026-10-verification-fixes.md` and `2026-10-session-b.md`.

### Environment

- `ledger` built from `918a08d` with `cargo build -p ledger-cli` (`target/debug/ledger`).
- OpenSSH `ssh-keygen` and util-linux `script(1)` were already installed.
- The four `GIT_AUTHOR_*` and `GIT_COMMITTER_*` variables are unset in `lib.sh` and for every gate run; `GIT_CONFIG_GLOBAL` and `GIT_CONFIG_SYSTEM` point at `/dev/null` in the stores.
- `git filter-repo` was not installed: no case needed carried history, and E2a, E2c, E5a and E5c are cited from the first session where this revision relies on them.
- Everything else lives in the scratchpad, `$SP/attacks/`, and is not committed: the shell library, the hand-filing helper, eleven case scripts, the prototype, the stores and the outputs.

### Step 1: every use of order

Every place where a verdict depends on landing order, a position, or "before" and "after", from the files read in full. "Terminating" and "enabling" follow D6's words. The last column is the concrete attack the order stops, or "nothing".

| # | File and symbol | Requirement | Terminating / enabling | What landing order protects against |
| --- | --- | --- | --- | --- |
| O1 | `landing::Position::before`, `not_after` | LP-8.26 | The primitive for both | The signer of `at` ordering their own act: landed later, dated earlier |
| O2 | `authority::view::Authority::as_of` | LP-8.26, LP-8.27, LP-8.28 | Roles enabling by landing alone; grants, grant acceptances and opening bindings enabling by `not_after`; revocations and key closes terminating by `before`; policies governing by `before`; intervals by landing | A grant, grant acceptance or role filed later and dated earlier enabling an act (a grant's `at` is outside its payload); an act backdated past a revocation, close or policy |
| O3 | `verify::acts::unauthorised` (`A006` on acceptances) | LP-6.27 | Grant revocation terminating; grant, grant acceptance, role enabling | Case C: a revoked grantee's backdated acceptance |
| O4 | `verify::acts::policy_verdict` | LP-6.28 | The genesis grant as of the policy, enabling | Nothing beyond O2: which genesis grant is live at the policy |
| O5 | `verify::acts::revocation_verdict` | LP-6.29, LP-6.27 | The first policy governing a legacy revocation; a grant's revocation | A legacy-shape revocation backdated before the first policy and landed after it (`legacy_revocation.rs`) |
| O6 | `signing::check::governing` | LP-8.28 | The policy in force at the act, governing; a policy change judged under the one it replaces by hash | Case D: an act backdated past the first policy; an act dodging a policy change by its date |
| O7 | `signing::check::check` (the `first_policy` branch and `auth.policy(ns)` being `None`) | D5 (c), ruling 52 | The first policy governing | As D; a sidecar on an act the position puts before the policy (`ungoverned_sidecars`) |
| O8 | `signing::check::trust_bindings` with `may_file` over `as_of` | LP-4.34, LP-4.12 | Bindings judged in landing order against the trust built so far; the genesis grant enabling the filer | A first key filed by the genesis holder before the genesis grant landed; a binding by a filer who became allowed only later |
| O9 | `signing::check::judge_first_policy` (`held`) | LP-4.31, #96 | A key opened `not_after` the policy, enabling; its close terminating | A first policy signed by a key bound later and backdated |
| O10 | `signing::check::verify_one` | LP-4.13 | The key's binding enabling (`not_after`); its closes terminating (`before`): `L011` or `Closed` | Cases A, B, E, F, G: an act signed by a closed key, dated inside the window, landed after the close |
| O11 | `signing::check::ordered` | LP-8.26 | The order of the trust pass: (index, `at`, id) | A binding whose signing key's binding lands later |
| O12 | `signing::review::review_closed` | LP-4.13 (`L012`) | Consumes O10's `Closed` verdicts; the deadline by clock | Nothing beyond O10 |
| O13 | `verify::genesis_role::findings` | Ruling 50 | The genesis grant as of the policy, enabling | Nothing: the fault is a fault under whichever genesis is live |
| O14 | `authority::filing::self_bound` over `as_of` | LP-4.12, LP-4.38 | The other bindings of the principal, enabling by `not_after` | Case H: an impostor's self-bound binding. As implemented it protects less than LP-4.38 says: two self-bound bindings, one landed first and dated later, one landed second and dated earlier, are both trusted *(run)* |
| O15 | `authority::filing::may_file`, the `open_window` branches, and `closing` | LP-4.12 | Opening bindings enabling; closes terminating | A principal's own `add` dated back inside a window a landed `revoke` closed (`closed_key.rs`, `order_regressions_tests.rs` hole 3); a close dated before the window it closes finds none (case B2's bound) |
| O16 | `verify::history::findings` over `landing::touched_after_landing`, `file_versions`, `content_at`, `landed::entities` | LP-8.30, ruling 58 | Neither: the landing commit of each entity | Editing or removing a landed entity, role file or sidecar; lowering or over-raising a `format:` declaration |
| O17 | `blame::introducing_author`, `verify::integrity::blame_consistency` | LP-8.32 | Neither: the introducing commit | An unsigned acceptance deposited by someone other than its actor (case K) |
| O18 | `landing::Landing::compute` with a base, `revision::overlay_base` | LP-8.29 | Branch entities at the tip; the base's files overlaid | A branch verdict that differs from the merge's (case E1) |
| O19 | `author::Author::gate` (`Options::offline`, `history: false`), `acceptance_ops::refuse_duplicate` (`Landing::unknown`) | — | None: the writers read no order | Nothing. The write-time gate is order-blind by design; CI is where order bites |
| O20 | `signing::check::dsse_key`, `authority::signers::derive_from` | LP-4.32 | Clock only: `k.at <= s.at`, `valid-before` from the close's `at` | Nothing: they are not ordered by landing; `verify_one` applies the close after them |

Under D, O1 to O15 read names and signed `at` (PRD §3.5.1 to §3.5.5); O16 to O18 stay git's (§3.5.7); O19 and O20 are unchanged.

### Step 2: the attack stores

Each case is one script under `$SP/attacks/`, sourcing `lib.sh` and calling `hand.py` for what a forger could write. `$SP` stands for the scratchpad directory, replaced in the outputs below. ULIDs, timestamps and key fingerprints differ on every run. Every output is from the run recorded here, with blame off (`--no-blame`) except in case K, which is about `L009`.

#### `lib.sh`, `hand.py`, `d.py` and `run_d.sh`

```bash
# lib.sh: shared helpers for the attack stores (order from the acts).
# Sourced by each case script. Never pushed.
set -u
SP=$SP
L=/home/user/product-cli/target/debug/ledger
HAND="python3 -I $SP/attacks/hand.py"
unset GIT_AUTHOR_NAME GIT_AUTHOR_EMAIL GIT_COMMITTER_NAME GIT_COMMITTER_EMAIL
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null
MANDATE="contract 2026/117"

# A verb that must run at a terminal (#71, #85): drive it under script(1).
tty() { # tty <root> <args...>
  local root=$1; shift
  local line; line=$(printf '%q ' "$L" --root "$root" "$@")
  script -qefc "$line" /dev/null </dev/null | tr -d '\r'
}
piped() { local root=$1; shift; "$L" --root "$root" "$@"; }

# mkrepo <dir> <email>: a git repository with an initialised store and set.
mkrepo() {
  local dir=$1 email=$2
  rm -rf "$dir"; mkdir -p "$dir"
  git -C "$dir" init -q --initial-branch=main
  git -C "$dir" config user.name "Fixture"
  git -C "$dir" config user.email "$email"
  piped "$dir" init >/dev/null
  piped "$dir" declare --set ledger-design --tolerance-floor T1 >/dev/null
}
act_as() { git -C "$1" config user.email "$2"; }
keygen() { # keygen <dir> <name> -> prints the private key path
  mkdir -p "$1/keys"; ssh-keygen -q -t ed25519 -N "" -C "$2" -f "$1/keys/$2" </dev/null; echo "$1/keys/$2"
}
use_key() { git -C "$1" config user.signingkey "$2"; }
commit() { git -C "$1" add -A; git -C "$1" commit -q -m "$2"; }
verify() { # verify <dir> [args]: print the report and the exit code
  local dir=$1; shift
  "$L" --root "$dir" verify --no-blame "$@" 2>&1; echo "exit $?"
}
word() { # word <prefix> : first word starting with prefix on stdin
  tr -d '(),`' | tr ' ' '\n' | grep "^$1" | head -1
}
# add_decision <dir> <ns> <statement> -> dec id
add_decision() { piped "$1" add --set ledger-design --namespace "$2" --statement "$3" --store constraint --discharge analyzer:DEC001 | word dec:; }
# govern <dir> <email> <ns>: a key for the genesis holder, init under policy
# (self-bound, ssh), an acceptor grant to <email>, accepted. Prints the grant id.
govern() {
  local dir=$1 email=$2 ns=$3
  local key; key=$(keygen "$dir" genesis)
  use_key "$dir" "$key"
  tty "$dir" init --namespace "$ns" --external-ref "$MANDATE" >/dev/null
  local grant; grant=$(tty "$dir" grant new acceptor --to "$email" --scope "ns:$ns" | word grant:)
  tty "$dir" grant accept "$grant" >/dev/null
  echo "$grant"
}
# ids out of the store
bindings() { python3 -I "$SP/attacks/hand.py" list --root "$1" key_bindings; }
grants() { python3 -I "$SP/attacks/hand.py" list --root "$1" grants; }
acceptances() { python3 -I "$SP/attacks/hand.py" list --root "$1" acceptances; }
at_of() { python3 -I "$SP/attacks/hand.py" at --root "$1" "$2"; }
plus() { python3 -c "import sys,datetime,re; t=datetime.datetime.strptime(re.sub(r'\\.\\d+','',sys.argv[1]),'%Y-%m-%dT%H:%M:%SZ')+datetime.timedelta(seconds=int(sys.argv[2])); print(t.strftime('%Y-%m-%dT%H:%M:%SZ'))" "$1" "$2"; }
section() { echo; echo "## $*"; }
```

```python
#!/usr/bin/env python3
"""hand.py: write ledger entities by hand, as a forger could (never pushed).

Implements the protocol's canonical-JSON law for the closed payloads of
an acceptance (`ledger.acceptance.v1`) and a key binding
(`ledger.identity-binding.v1`): strings normalised (LF, NFC, ASCII trim,
empty omitted), sets sorted and deduplicated, keys sorted, compact; the
digest is sha256 over `prefix || 0x0A || json`. Signing is
`ssh-keygen -Y sign -n ledger-accept@<ns>` over the same bytes.

Subcommands:
  list   --root R <list>                 ids of every entry of that list
  at     --root R <id>                   the `at` of an entry
  accept --root R --decision D --actor A --at T [--under G] [--key K]
  close  --root R --act revoke|rotate --closes KEY --by B --at T
         [--under G] --sign K [--new-key PUB]
"""
import argparse, glob, hashlib, json, os, random, re, subprocess, sys, time, unicodedata
import yaml

ALPHABET = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"

def ulid():
    ms = int(time.time() * 1000)
    rnd = random.getrandbits(80)
    n = (ms << 80) | rnd
    out = ""
    for _ in range(26):
        out = ALPHABET[n & 31] + out
        n >>= 5
    return out

def norm(s):
    s = s.replace("\r\n", "\n").replace("\r", "\n")
    s = unicodedata.normalize("NFC", s)
    s = s.strip(" \t\n\x0c\r")
    return s or None

def put(m, k, v):
    if v is None: return
    v = norm(str(v))
    if v is not None: m[k] = v

def put_set(m, k, items):
    s = sorted({norm(str(i)) for i in items if norm(str(i)) is not None})
    if s: m[k] = s

def canonical(m):
    return json.dumps(m, sort_keys=True, separators=(",", ":"), ensure_ascii=False)

def signed_bytes(prefix, m):
    return prefix.encode() + b"\n" + canonical(m).encode()

def digest(prefix, m):
    return "sha256:" + hashlib.sha256(signed_bytes(prefix, m)).hexdigest()

def logs(root):
    for f in sorted(glob.glob(os.path.join(root, ".decisions/log/*.yml"))):
        with open(f) as fh:
            yield f, yaml.load(fh, Loader=yaml.BaseLoader)

def entries(root, name):
    for f, d in logs(root):
        for it in d.get(name) or []:
            yield it

def tip_version(root, decision):
    versions = [v for v in entries(root, "versions") if v["decision"] == decision]
    parents = {v.get("parent") for v in versions}
    tips = [v for v in versions if v["hash"] not in parents]
    assert len(tips) == 1, tips
    return tips[0]["hash"]

def acceptance_map(a):
    m = {}
    put(m, "decision", a["decision"]); put(m, "version", a["version"]); put(m, "actor", a["actor"])
    put(m, "at", a["at"]); put(m, "scope", a.get("scope", "version")); put(m, "expires_at", a.get("expires_at"))
    put(m, "under", a.get("under"))
    return m

def binding_map(b):
    m = {}
    for k in ["id", "act", "principal", "namespace", "key_type", "key", "closes"]:
        put(m, k, b.get(k))
    if b.get("self_bound"): put(m, "self_bound", "true")
    put(m, "mandate", b.get("mandate")); put(m, "by", b.get("by")); put(m, "under", b.get("under")); put(m, "at", b.get("at"))
    return m

def ssh_sign(key, ns, data):
    out = subprocess.run(["ssh-keygen", "-q", "-Y", "sign", "-f", key, "-n", f"ledger-accept@{ns}"], input=data, capture_output=True)
    assert out.returncode == 0 and out.stdout, out.stderr.decode()
    return out.stdout

def write_cs(root, cs, sidecars):
    os.makedirs(os.path.join(root, ".decisions/sig"), exist_ok=True)
    path = os.path.join(root, ".decisions/log", cs["id"][3:] + ".yml")
    with open(path, "w") as fh:
        yaml.safe_dump(cs, fh, sort_keys=False)
    for name, data in sidecars:
        with open(os.path.join(root, ".decisions/sig", name), "wb") as fh:
            fh.write(data)
    return path

def secs(t):
    return re.sub(r"\.\d+", "", t)

def cmd_accept(a):
    a.at = secs(a.at)
    ns = a.decision[4:].split("/")[0]
    acc = {"id": "acc:" + ulid(), "decision": a.decision, "version": tip_version(a.root, a.decision), "actor": a.actor, "at": a.at, "scope": "version"}
    if a.under: acc["under"] = a.under
    cs = {"format": 7, "id": "cs:" + ulid(), "created_at": a.at, "created_by": a.actor, "acceptances": [acc]}
    side = []
    if a.key:
        side.append((acc["id"][4:] + ".ssh.sig", ssh_sign(a.key, ns, signed_bytes("ledger.acceptance.v1", acceptance_map(acc)))))
    write_cs(a.root, cs, side)
    print(acc["id"])

def cmd_close(a):
    a.at = secs(a.at)
    target = next(b for b in entries(a.root, "key_bindings") if b["id"] == a.closes)
    b = {"id": "key:" + ulid(), "act": a.act, "principal": target["principal"], "namespace": target["namespace"]}
    if a.act == "rotate":
        t, k = open(a.new_key).read().split()[:2]
        b["key_type"] = t; b["key"] = k
    b["closes"] = a.closes; b["by"] = a.by
    if a.under: b["under"] = a.under
    b["at"] = a.at
    b["hash"] = digest("ledger.identity-binding.v1", binding_map(b))
    cs = {"format": 7, "id": "cs:" + ulid(), "created_at": a.at, "created_by": a.by, "key_bindings": [b]}
    side = [(b["id"][4:] + ".ssh.sig", ssh_sign(a.sign, b["namespace"], signed_bytes("ledger.identity-binding.v1", binding_map(b))))]
    write_cs(a.root, cs, side)
    print(b["id"])

def cmd_bind(a):
    a.at = secs(a.at)
    t, k = open(a.new_key).read().split()[:2]
    b = {"id": "key:" + ulid(), "act": "add", "principal": a.principal, "namespace": a.ns, "key_type": t, "key": k}
    if a.mandate: b["self_bound"] = True; b["mandate"] = a.mandate
    b["by"] = a.by
    if a.under: b["under"] = a.under
    b["at"] = a.at
    b["hash"] = digest("ledger.identity-binding.v1", binding_map(b))
    cs = {"format": 7, "id": "cs:" + ulid(), "created_at": a.at, "created_by": a.by, "key_bindings": [b]}
    side = [(b["id"][4:] + ".ssh.sig", ssh_sign(a.sign, a.ns, signed_bytes("ledger.identity-binding.v1", binding_map(b))))]
    write_cs(a.root, cs, side)
    print(b["id"])

def main():
    p = argparse.ArgumentParser(); sub = p.add_subparsers(dest="cmd")
    s = sub.add_parser("list"); s.add_argument("--root", required=True); s.add_argument("name")
    s = sub.add_parser("at"); s.add_argument("--root", required=True); s.add_argument("id")
    s = sub.add_parser("accept"); s.add_argument("--root", required=True); s.add_argument("--decision", required=True); s.add_argument("--actor", required=True); s.add_argument("--at", required=True); s.add_argument("--under"); s.add_argument("--key")
    s = sub.add_parser("close"); s.add_argument("--root", required=True); s.add_argument("--act", required=True); s.add_argument("--closes", required=True); s.add_argument("--by", required=True); s.add_argument("--at", required=True); s.add_argument("--under"); s.add_argument("--sign", required=True); s.add_argument("--new-key")
    s = sub.add_parser("bind"); s.add_argument("--root", required=True); s.add_argument("--principal", required=True); s.add_argument("--by", required=True); s.add_argument("--ns", required=True); s.add_argument("--new-key", required=True); s.add_argument("--mandate"); s.add_argument("--under"); s.add_argument("--at", required=True); s.add_argument("--sign", required=True)
    a = p.parse_args()
    if a.cmd == "list":
        for it in entries(a.root, a.name): print(it["id"])
    elif a.cmd == "at":
        for name in ["acceptances", "key_bindings", "grants", "grant_acceptances", "revocations", "policies"]:
            for it in entries(a.root, name):
                if it.get("id") == a.id: print(it["at"]); return
        sys.exit("no such id")
    elif a.cmd == "accept": cmd_accept(a)
    elif a.cmd == "close": cmd_close(a)
    elif a.cmd == "bind": cmd_bind(a)

main()
```

```python
#!/usr/bin/env python3
"""d.py: a throwaway reading of position D over a store (never pushed).

What it computes. For every acceptance in the store: the binding whose key
signed it (read from the SSHSIG blob in its sidecar), every close of that
key, and D's verdict against each close: BEFORE when the close names the
acceptance and the acceptance's `at` is earlier, else AFTER. Against the
grant the acceptance names: the grant's revocation, if any, judged the
same way. Against the namespace's first policy: a pre-policy act when the
policy names it and dates after it, else governed. Names are supplied in a
JSON file, {entry id: [act ids]}, standing in for the field this design
proposes; an entry absent from the file names nothing.

What it does not compute. Signatures are not verified, D7 trust is not
judged, grants are not role-checked, `at` is taken as written, and nothing
is read from git. It decides nothing about enabling entries.
"""
import base64, glob, json, os, re, struct, sys
import yaml

def entries(root, name):
    for f in sorted(glob.glob(os.path.join(root, ".decisions/log/*.yml"))):
        for it in yaml.load(open(f), Loader=yaml.BaseLoader).get(name) or []:
            yield it

def secs(t): return re.sub(r"\.\d+", "", t)

def sshsig_key(path):
    data = open(path, "rb").read()
    body = base64.b64decode(b"".join(l for l in data.splitlines() if not l.startswith(b"-----")))
    assert body[:6] == b"SSHSIG"
    off = 10  # magic + version
    (n,) = struct.unpack(">I", body[off:off + 4]); key = body[off + 4:off + 4 + n]
    return base64.b64encode(key).decode()

def main(root, names_path):
    names = json.load(open(names_path)) if names_path else {}
    bindings = list(entries(root, "key_bindings"))
    by_key = {b["key"]: b for b in bindings if b.get("key")}
    closes = [b for b in bindings if b.get("closes")]
    revs = {r["revokes"]: r for r in entries(root, "revocations") if r.get("revokes")}
    pols = sorted(entries(root, "policies"), key=lambda p: p["at"])
    first = {p["namespace"]: p for p in reversed(pols) if not p.get("replaces")}
    def named(entry, act): return act in names.get(entry["id"], [])
    for a in entries(root, "acceptances"):
        ns = a["decision"][4:].split("/")[0]; at = secs(a["at"]); out = [a["id"], f"at {at}"]
        p = first.get(ns)
        if p is None: out.append("no policy: unchecked"); print("  ".join(out)); continue
        if named(p, a["id"]) and at < secs(p["at"]): out.append(f"pre-policy (named by {p['id']})"); print("  ".join(out)); continue
        out.append("governed")
        g = a.get("under")
        if g and g in revs:
            r = revs[g]
            out.append(f"grant revoked {r['id']}: " + ("BEFORE it (named, dated earlier)" if named(r, a["id"]) and at < secs(r["at"]) else "AFTER it -> A006"))
        sig = os.path.join(root, ".decisions/sig", a["id"][4:] + ".ssh.sig")
        if not os.path.exists(sig): out.append("unsigned"); print("  ".join(out)); continue
        b = by_key.get(sshsig_key(sig))
        if b is None: out.append("signed by a key no binding carries -> L011"); print("  ".join(out)); continue
        out.append(f"signed by {b['id']}")
        same = [c for c in closes if any(o["id"] == c["closes"] and o.get("key") == b["key"] and o["principal"] == b["principal"] for o in bindings)]
        for c in same:
            out.append(f"close {c['id']} at {secs(c['at'])}: " + ("BEFORE it (named, dated earlier) -> review item" if named(c, a["id"]) and at < secs(c["at"]) else "AFTER it -> L011"))
        print("  ".join(out))

main(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else None)
```

```bash
#!/bin/bash
# run_d.sh: the position-D prototype over each attack store, with the names
# each terminating entry's writer would have had at hand when it was filed.
source "$(dirname "$0")/lib.sh"
ids() { $HAND list --root "$1" "$2"; }
run() { echo; echo "## $1: $2"; echo "names: $(cat "$3")"; python3 -I "$SP/attacks/d.py" "$1" "$3"; }
N=$SP/attacks/names; mkdir -p "$N"
# A: the verb's revoke was filed before the forged act existed: it names nothing.
echo '{}' > $N/a.json; run repos/a "the close names nothing (the act did not exist)" $N/a.json
# B: the thief's rotate names (i) nothing (ii) the legit a1; the genesis holder's revoke of K2 names nothing.
R=repos/b; ROT=$(ids $R key_bindings | sed -n 3p); A1=$(ids $R acceptances | head -1)
echo '{}' > $N/b1.json; run $R "the thief's rotate names nothing; the revoke of K2 names nothing" $N/b1.json
printf '{"%s": ["%s"]}' "$ROT" "$A1" > $N/b2.json; run $R "the thief's rotate names the legit a1; the revoke of K2 names nothing" $N/b2.json
# C: the grant's revocation names the one act under it at the time, a1.
R=repos/c; REV=$(ids $R revocations | head -1); A1=$(ids $R acceptances | head -1)
printf '{"%s": ["%s"]}' "$REV" "$A1" > $N/c.json; run $R "the revocation names a1" $N/c.json
# D: the first policy names the one pre-policy acceptance, a0.
R=repos/d; POL=$(ids $R policies | head -1); A0=$(ids $R acceptances | head -1)
printf '{"%s": ["%s"]}' "$POL" "$A0" > $N/d.json; run $R "the first policy names a0" $N/d.json
# E1: the close on main names nothing (the topic acceptance is not in its base).
echo '{}' > $N/e1.json; run repos/e1 "the close on main names nothing" $N/e1.json
# E2: the close on its branch names a1, which was in its base; a2 merged later.
R=repos/e2; CL=$(ids $R key_bindings | tail -1); A1=$(ids $R acceptances | head -1)
printf '{"%s": ["%s"]}' "$CL" "$A1" > $N/e2.json; run $R "the close on the branch names a1 and not a2" $N/e2.json
# F: the verb's close names a1 (F1) or omits it (the omission).
R=repos/f; CL=$(ids $R key_bindings | tail -1); A1=$(ids $R acceptances | head -1)
printf '{"%s": ["%s"]}' "$CL" "$A1" > $N/f1.json; run $R "the close names a1" $N/f1.json
echo '{}' > $N/f2.json; run $R "the closer omits a1" $N/f2.json
# G: the close names the drawer act filed with it (a1); a2 was filed later. (i) named by content digest at close time (ii) not named.
R=repos/g; CL=$(ids $R key_bindings | tail -1); A1=$(ids $R acceptances | head -1); A2=$(ids $R acceptances | tail -1)
printf '{"%s": ["%s", "%s"]}' "$CL" "$A1" "$A2" > $N/g1.json; run $R "the close names both drawer acts (both existed as bytes when it was written)" $N/g1.json
printf '{"%s": ["%s"]}' "$CL" "$A1" > $N/g2.json; run $R "the close names only the drawer act filed with it" $N/g2.json
```

#### Case A: a closed key signs an act dated before the close

The script, `case_a.sh`:

```bash
#!/bin/bash
# Case A: a closed key signs an act dated before the close (D6, LP-4.13).
source "$(dirname "$0")/lib.sh"
R=$SP/attacks/repos/a; OWNER=owner@customer.example; NS=fixture.ledger
mkrepo "$R" $OWNER
GRANT=$(govern "$R" $OWNER $NS)
D1=$(add_decision "$R" $NS "Money is decimal.")
commit "$R" "governed"
K1=$(bindings "$R" | head -1)
section "store: genesis holder $OWNER self-bound K1=$K1, acceptor grant $GRANT, decision $D1"
K2=$(keygen "$R" owner-next)
tty "$R" identity add --namespace $NS --key-file "$K2.pub" | tail -2
K2ID=$(bindings "$R" | tail -1)
INSIDE=$(plus "$(at_of "$R" "$K2ID")" 1)
sleep 2.1
use_key "$R" "$K2"
section "the holder revokes K1 with the verb (at = now, signed by K2)"
tty "$R" identity revoke "$K1" | tail -2
commit "$R" "closed K1"
CLOSE=$(bindings "$R" | tail -1)
echo "close $CLOSE at $(at_of "$R" "$CLOSE"); the forged act is dated $INSIDE (inside K1's window)"
section "hand-filed acceptance by $OWNER, signed with K1, dated inside the window, landed after the close"
ACC=$($HAND accept --root "$R" --decision "$D1" --actor $OWNER --at "$INSIDE" --under "$GRANT" --key "$R/keys/genesis")
commit "$R" "backdated acceptance $ACC"
section "verify today"
verify "$R"
```

Its output, today's `ledger` from `918a08d`:

```text

## store: genesis holder owner@customer.example self-bound K1=key:01M4G97APM4C95CZZ3PX6YWYSV, acceptor grant grant:01M4G97AV5RZNXRRJQVRY64QYQ, decision dec:fixture.ledger/01M4G97B51JCX2BNXTJC0K6WAW
regenerated allowed_signers
filed $SP/attacks/repos/a/.decisions/log/01M4G97BEN1CZKAC2Y535GNJG8.yml

## the holder revokes K1 with the verb (at = now, signed by K2)
regenerated allowed_signers
filed $SP/attacks/repos/a/.decisions/log/01M4G97E0B9N4KGT53RQ0EZABN.yml
close key:01M4G97E017PXTFJBJ8E3QNSXY at 2026-10-09T12:11:54.237419412Z; the forged act is dated 2026-10-09T12:11:52Z (inside K1's window)

## hand-filed acceptance by owner@customer.example, signed with K1, dated inside the window, landed after the close

## verify today
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 1 finding(s):
  - [L011] acc:01M4G97EHAZSHWM4PC0WRMVPD1: its `ssh` signature does not hold: its key key:01M4G97APM4C95CZZ3PX6YWYSV was closed in `fixture.ledger` by key:01M4G97E017PXTFJBJ8E3QNSXY, and the act is not dated and landed before the close (D6) — a close ends the key in every namespace, so it signs nothing in `fixture.ledger`
exit 1
```

#### Case B: a stolen key rotates itself; the holder's revoke

The script, `case_b.sh`:

```bash
#!/bin/bash
# Case B: a stolen key rotates itself and signs forged acts; the genesis
# holder's revoke, dated at the compromise, names neither (D6, D7 (5)).
source "$(dirname "$0")/lib.sh"
R=$SP/attacks/repos/b; G=genesis@customer.example; H=holder@customer.example; NS=fixture.ledger
mkrepo "$R" $G
GK=$(keygen "$R" genesis); use_key "$R" "$GK"
tty "$R" init --namespace $NS --external-ref "$MANDATE" >/dev/null
GENESIS=$(grants "$R" | head -1)
GRANT=$(tty "$R" grant new acceptor --to $H --scope "ns:$NS" | word grant:)
act_as "$R" $H; tty "$R" grant accept "$GRANT" >/dev/null; act_as "$R" $G
KH=$(keygen "$R" holder)
tty "$R" identity add --namespace $NS --for $H --key-file "$KH.pub" | head -1
KHID=$(bindings "$R" | tail -1)
section "genesis $G (grant $GENESIS); holder $H with K1=$KHID bound by the genesis holder; acceptor grant $GRANT"
act_as "$R" $H; use_key "$R" "$KH"
D1=$(add_decision "$R" $NS "Money is decimal.")
tty "$R" accept "$D1" | grep -i "accepted\|under" | head -2
A1=$(acceptances "$R" | tail -1)
commit "$R" "legit: $H accepts $D1 ($A1) with K1"
sleep 2.1
section "the thief, holding K1 and $H's git identity: rotate K1 to K2, then accept with K2"
K2=$(keygen "$R" thief)
tty "$R" identity rotate "$KHID" --key-file "$K2.pub" | head -1
ROT=$(bindings "$R" | tail -1); TROT=$(at_of "$R" "$ROT")
use_key "$R" "$K2"
D2=$(add_decision "$R" $NS "Time is UTC.")
tty "$R" accept "$D2" | grep -i "accepted\|under" | head -2
A2=$(acceptances "$R" | tail -1)
commit "$R" "compromise: rotate $ROT at $TROT, forged $A2"
echo "rotate $ROT at $TROT; forged acceptance $A2 at $(at_of "$R" "$A2")"
rm -rf "$R-verb"; cp -r "$R" "$R-verb"
section "B1: the genesis holder revokes K2 with the verb (at = now, after the forged act)"
act_as "$R-verb" $G; use_key "$R-verb" "$R-verb/keys/genesis"
sleep 1.1
tty "$R-verb" identity revoke "$ROT" | head -1
commit "$R-verb" "genesis revokes K2 now"
verify "$R-verb"
section "B2: the genesis holder's revoke of K2 hand-filed with at = one second after the rotate ($(plus "$TROT" 1); a close dated before the window it closes finds no window, so the compromise time is bounded below by the rotate), under the genesis grant, signed with the genesis key"
act_as "$R" $G
REV=$($HAND close --root "$R" --act revoke --closes "$ROT" --by $G --at "$(plus "$TROT" 1)" --under "$GENESIS" --sign "$GK")
commit "$R" "genesis revokes K2 at the compromise ($REV)"
verify "$R"
```

Its output, today's `ledger` from `918a08d`:

```text
identity add: holder@customer.example in `fixture.ledger` — key:01M4G9413H9FE34X0M88RN30A5

## genesis genesis@customer.example (grant grant:01M4G940GFQH5AF86VRB1H1HEV); holder holder@customer.example with K1=key:01M4G9413H9FE34X0M88RN30A5 bound by the genesis holder; acceptor grant grant:01M4G940QJHR5S42EC8H33NR4R
holder@customer.example accepted fd243727fdd7 of dec:fixture.ledger/01M4G941FBNQF97T2F28SFRYDG — the signature names this exact state
under grant:01M4G940QJHR5S42EC8H33NR4R (`acceptor`)

## the thief, holding K1 and holder@customer.example's git identity: rotate K1 to K2, then accept with K2
identity rotate: holder@customer.example in `fixture.ledger` — key:01M4G9447039Z33W2RB2T9GNNP
holder@customer.example accepted ce73c6ce1d4c of dec:fixture.ledger/01M4G944RMH28PC5V27PCJ3671 — the signature names this exact state
under grant:01M4G940QJHR5S42EC8H33NR4R (`acceptor`)
rotate key:01M4G9447039Z33W2RB2T9GNNP at 2026-10-09T12:10:05.915566506Z; forged acceptance acc:01M4G9455BPQTA6623MJCC7G65 at 2026-10-09T12:10:06.799871160Z

## B1: the genesis holder revokes K2 with the verb (at = now, after the forged act)
identity revoke: holder@customer.example in `fixture.ledger` — key:01M4G946YK5735F9QE3TEVGTRX
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 18 entries, 2 decision(s)
2 acceptance(s) under a since-closed key need re-acceptance (L012 after the deadline):
  - acc:01M4G941RMTWBGN8WJ4A7V0JP1 (holder@customer.example): key key:01M4G9413H9FE34X0M88RN30A5 closed by key:01M4G9447039Z33W2RB2T9GNNP — re-accept or affirm
  - acc:01M4G9455BPQTA6623MJCC7G65 (holder@customer.example): key key:01M4G9447039Z33W2RB2T9GNNP closed by key:01M4G946YK5735F9QE3TEVGTRX — re-accept or affirm
exit 0

## B2: the genesis holder's revoke of K2 hand-filed with at = one second after the rotate (2026-10-09T12:10:06Z; a close dated before the window it closes finds no window, so the compromise time is bounded below by the rotate), under the genesis grant, signed with the genesis key
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 2 finding(s):
  - [L011] acc:01M4G9455BPQTA6623MJCC7G65: its `ssh` signature does not hold: its key key:01M4G9447039Z33W2RB2T9GNNP was closed in `fixture.ledger` by key:01M4G947V67N43851K07J3YQWV, and the act is not dated and landed before the close (D6) — a close ends the key in every namespace, so it signs nothing in `fixture.ledger`
trust-root stage — 1 finding(s):
  - [SIGNERS] .decisions/allowed_signers: does not match the key bindings in the log — regenerate it with `ledger identity sync` — it is never edited by hand
1 acceptance(s) under a since-closed key need re-acceptance (L012 after the deadline):
  - acc:01M4G941RMTWBGN8WJ4A7V0JP1 (holder@customer.example): key key:01M4G9413H9FE34X0M88RN30A5 closed by key:01M4G9447039Z33W2RB2T9GNNP — re-accept or affirm
exit 1
```

#### Case C: a revoked grantee backdates an act

The script, `case_c.sh`:

```bash
#!/bin/bash
# Case C: a revoked grantee backdates an act (D5, LP-6.27).
source "$(dirname "$0")/lib.sh"
R=$SP/attacks/repos/c; G=genesis@customer.example; H=holder@customer.example; NS=fixture.ledger
mkrepo "$R" $G
GK=$(keygen "$R" genesis); use_key "$R" "$GK"
tty "$R" init --namespace $NS --external-ref "$MANDATE" >/dev/null
GRANT=$(tty "$R" grant new acceptor --to $H --scope "ns:$NS" | word grant:)
act_as "$R" $H; tty "$R" grant accept "$GRANT" >/dev/null; act_as "$R" $G
KH=$(keygen "$R" holder)
tty "$R" identity add --namespace $NS --for $H --key-file "$KH.pub" >/dev/null
act_as "$R" $H; use_key "$R" "$KH"
D1=$(add_decision "$R" $NS "Money is decimal."); D2=$(add_decision "$R" $NS "Time is UTC.")
tty "$R" accept "$D1" >/dev/null
A1=$(acceptances "$R" | tail -1)
commit "$R" "legit $A1 under $GRANT"
sleep 2.1
section "the genesis holder revokes $GRANT with the verb"
act_as "$R" $G; use_key "$R" "$GK"
tty "$R" grant revoke "$GRANT" --reason "moved teams" | head -1
commit "$R" "revoked"
REV=$($HAND list --root "$R" revocations | tail -1); echo "revocation $REV at $(at_of "$R" "$REV"); legit $A1 at $(at_of "$R" "$A1")"
section "the revoked holder hand-files an acceptance of $D2 under the revoked grant, dated before the revocation, signed with their live key"
act_as "$R" $H
A2=$($HAND accept --root "$R" --decision "$D2" --actor $H --at "$(plus "$(at_of "$R" "$A1")" 1)" --under "$GRANT" --key "$KH")
commit "$R" "backdated $A2"
verify "$R"
```

Its output, today's `ledger` from `918a08d`:

```text

## the genesis holder revokes grant:01M4G8Y19XQDA5ZPYTZ5A8DRNN with the verb
revoked grant:01M4G8Y19XQDA5ZPYTZ5A8DRNN — moved teams
revocation rev:01M4G8Y4RPJKJKYYXQ6PBA1E97 at 2026-10-09T12:06:49.871920952Z; legit acc:01M4G8Y2B75PJNC8S63P40P1YR at 2026-10-09T12:06:47.339629002Z

## the revoked holder hand-files an acceptance of dec:fixture.ledger/01M4G8Y23B47FNYEZYF7MZEDWB under the revoked grant, dated before the revocation, signed with their live key
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 1 finding(s):
graph stage — 1 finding(s):
  - [A006] acc:01M4G8Y5GD8YH7EK1ZEMCK5FRC: holder@customer.example holds grant:01M4G8Y19XQDA5ZPYTZ5A8DRNN, which is revoked or superseded, as of the act (accept)
exit 1
```

#### Case D: an act backdated past the first policy

The script, `case_d.sh`:

```bash
#!/bin/bash
# Case D: an act backdated past a namespace's first policy (D5 (c), LP-8.28).
source "$(dirname "$0")/lib.sh"
R=$SP/attacks/repos/d; OWNER=owner@customer.example; NS=fixture.ledger
mkrepo "$R" $OWNER
D1=$(add_decision "$R" $NS "Money is decimal.")
tty "$R" accept "$D1" | head -1
A0=$(acceptances "$R" | tail -1)
commit "$R" "pre-policy: $A0 unsigned, no grant"
sleep 1.1
GRANT=$(govern "$R" $OWNER $NS)
commit "$R" "policy"
POL=$($HAND list --root "$R" policies | head -1); TP=$(at_of "$R" "$POL")
section "namespace $NS: pre-policy acceptance $A0 at $(at_of "$R" "$A0"); first policy $POL at $TP"
D2=$(add_decision "$R" $NS "Time is UTC.")
section "a hand-filed acceptance of $D2 dated 60s before the policy, no grant, no signature, landed after the policy"
A1=$($HAND accept --root "$R" --decision "$D2" --actor $OWNER --at "$(plus "$TP" -60)")
commit "$R" "backdated $A1"
verify "$R"
```

Its output, today's `ledger` from `918a08d`:

```text
owner@customer.example accepted 4cf6bbbe5287 of dec:fixture.ledger/01M4G8Y5V2414JS2CHFNS7XACZ — the signature names this exact state

## namespace fixture.ledger: pre-policy acceptance acc:01M4G8Y5WE9WDNC8HVKWX5PECS at 2026-10-09T12:06:51.021122038Z; first policy pol:01M4G8Y7417CQ3CTV3AZFYQX1M at 2026-10-09T12:06:52.287357126Z

## a hand-filed acceptance of dec:fixture.ledger/01M4G8Y80WTT950A0J246NB7HF dated 60s before the policy, no grant, no signature, landed after the policy
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 2 finding(s):
  - [L011] acc:01M4G8Y88SGTBTAFN6JG20JZCJ: `fixture.ledger`'s policy requires a `ssh` signature; none is filed
graph stage — 1 finding(s):
  - [A006] acc:01M4G8Y88SGTBTAFN6JG20JZCJ: a governed act names no grant (`under`) — owner@customer.example acts under one
exit 1
```

#### Case E: an act and a close in two pull requests

The script, `case_e.sh`:

```bash
#!/bin/bash
# Case E: an act and a close in two pull requests (LP-8.29).
source "$(dirname "$0")/lib.sh"
OWNER=owner@customer.example; NS=fixture.ledger
# E1: the act's branch is open while the close lands on main (signing.rs).
R=$SP/attacks/repos/e1
mkrepo "$R" $OWNER
GRANT=$(govern "$R" $OWNER $NS)
D1=$(add_decision "$R" $NS "Money is decimal.")
K1=$(bindings "$R" | head -1)
K2=$(keygen "$R" owner-next); tty "$R" identity add --namespace $NS --key-file "$K2.pub" >/dev/null
T0=$(at_of "$R" "$(bindings "$R" | tail -1)")
commit "$R" "base"
git -C "$R" checkout -q -b topic
A1=$($HAND accept --root "$R" --decision "$D1" --actor $OWNER --at "$(plus "$T0" 1)" --under "$GRANT" --key "$R/keys/genesis")
commit "$R" "topic: $A1 signed by K1"
git -C "$R" checkout -q main
sleep 2.1; use_key "$R" "$K2"
tty "$R" identity revoke "$K1" >/dev/null
commit "$R" "main: K1 closed"
git -C "$R" checkout -q topic
section "E1: the topic branch verified against main (the pull request's reading)"
verify "$R" --base main
git -C "$R" checkout -q main; git -C "$R" merge -q --no-ff -m "merge topic" topic
section "E1: the merge"
verify "$R"
# E2: the close's branch is open while the act lands on main.
R=$SP/attacks/repos/e2
mkrepo "$R" $OWNER
GRANT=$(govern "$R" $OWNER $NS)
D1=$(add_decision "$R" $NS "Money is decimal."); D2=$(add_decision "$R" $NS "Time is UTC.")
K1=$(bindings "$R" | head -1)
tty "$R" accept "$D1" >/dev/null; A1=$(acceptances "$R" | tail -1)
K2=$(keygen "$R" owner-next); tty "$R" identity add --namespace $NS --key-file "$K2.pub" >/dev/null
T0=$(at_of "$R" "$(bindings "$R" | tail -1)")
commit "$R" "base: $A1 by K1"
git -C "$R" checkout -q -b close
sleep 2.1; use_key "$R" "$K2"
tty "$R" identity revoke "$K1" >/dev/null
CL=$(bindings "$R" | tail -1)
commit "$R" "close branch: K1 closed at $(at_of "$R" "$CL")"
git -C "$R" checkout -q main
A2=$($HAND accept --root "$R" --decision "$D2" --actor $OWNER --at "$(plus "$T0" 1)" --under "$GRANT" --key "$R/keys/genesis")
commit "$R" "main: $A2 by K1, dated $(plus "$T0" 1), lands first"
section "E2: the close branch verified against main before it merges (the pull request's reading); a2 is in the base"
git -C "$R" checkout -q close
verify "$R" --base main
git -C "$R" checkout -q main; git -C "$R" merge -q --no-ff -m "merge close" close
section "E2: the merge: a2 landed before the close and is dated before it"
verify "$R"
```

Its output, today's `ledger` from `918a08d`:

```text

## E1: the topic branch verified against main (the pull request's reading)
landing computed against base `main` — 1 change-set(s) read from the base
non-conformant — 2 finding(s):
  - [L011] acc:01M4G8Y9TJZDMKEY2MW9B9YXNE: its `ssh` signature does not hold: its key key:01M4G8Y8HPKSEJWR0R29KA2AXY was closed in `fixture.ledger` by key:01M4G8YBYG7X2HRXBAWMZXWSQX, and the act is not dated and landed before the close (D6) — a close ends the key in every namespace, so it signs nothing in `fixture.ledger`
trust-root stage — 1 finding(s):
  - [SIGNERS] .decisions/allowed_signers: does not match the key bindings in the log — regenerate it with `ledger identity sync` — it is never edited by hand
exit 1

## E1: the merge
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 1 finding(s):
  - [L011] acc:01M4G8Y9TJZDMKEY2MW9B9YXNE: its `ssh` signature does not hold: its key key:01M4G8Y8HPKSEJWR0R29KA2AXY was closed in `fixture.ledger` by key:01M4G8YBYG7X2HRXBAWMZXWSQX, and the act is not dated and landed before the close (D6) — a close ends the key in every namespace, so it signs nothing in `fixture.ledger`
exit 1

## E2: the close branch verified against main before it merges (the pull request's reading); a2 is in the base
landing computed against base `main` — 1 change-set(s) read from the base
conformant — 17 entries, 2 decision(s)
2 acceptance(s) under a since-closed key need re-acceptance (L012 after the deadline):
  - acc:01M4G8YDVGP55JD8DRXWVJAYFA (owner@customer.example): key key:01M4G8YCXX731W73XA1F8Z3V41 closed by key:01M4G8YGQQ9HP9JY4NN64HY5YC — re-accept or affirm
  - acc:01M4G8YHBRSXTYBCKRP76EDQ0G (owner@customer.example): key key:01M4G8YCXX731W73XA1F8Z3V41 closed by key:01M4G8YGQQ9HP9JY4NN64HY5YC — re-accept or affirm
exit 0

## E2: the merge: a2 landed before the close and is dated before it
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 17 entries, 2 decision(s)
2 acceptance(s) under a since-closed key need re-acceptance (L012 after the deadline):
  - acc:01M4G8YDVGP55JD8DRXWVJAYFA (owner@customer.example): key key:01M4G8YCXX731W73XA1F8Z3V41 closed by key:01M4G8YGQQ9HP9JY4NN64HY5YC — re-accept or affirm
  - acc:01M4G8YHBRSXTYBCKRP76EDQ0G (owner@customer.example): key key:01M4G8YCXX731W73XA1F8Z3V41 closed by key:01M4G8YGQQ9HP9JY4NN64HY5YC — re-accept or affirm
exit 0
```

#### Case F: the closer omits a legitimate act, against a backdated `at`

The script, `case_f.sh`:

```bash
#!/bin/bash
# Case F: the closer omits a legitimate earlier act, against what a backdated `at` allows today.
source "$(dirname "$0")/lib.sh"
OWNER=owner@customer.example; NS=fixture.ledger
R=$SP/attacks/repos/f
mkrepo "$R" $OWNER
GRANT=$(govern "$R" $OWNER $NS)
D1=$(add_decision "$R" $NS "Money is decimal.")
K1=$(bindings "$R" | head -1)
K2=$(keygen "$R" owner-next); tty "$R" identity add --namespace $NS --key-file "$K2.pub" >/dev/null
sleep 1.1; tty "$R" accept "$D1" >/dev/null; A1=$(acceptances "$R" | tail -1)
commit "$R" "K2 bound, then a1 by K1"
rm -rf "$R-backdated"; cp -r "$R" "$R-backdated"
sleep 2.1; use_key "$R" "$K2"
section "F1: the holder closes K1 with the verb (at = now): a1 is before it"
tty "$R" identity revoke "$K1" >/dev/null
commit "$R" "closed"
verify "$R"
section "F2: the holder's revoke of K1 hand-filed with at = the second a1 is dated in, truncated, so a1 is dated after it (what a backdated at allows today)"
R=$R-backdated; use_key "$R" "$K2"
REV=$($HAND close --root "$R" --act revoke --closes "$K1" --by $OWNER --at "$(plus "$(at_of "$R" "$A1")" 0)" --sign "$K2")
commit "$R" "closed, backdated ($REV)"
verify "$R"
```

Its output, today's `ledger` from `918a08d`:

```text

## F1: the holder closes K1 with the verb (at = now): a1 is before it
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 14 entries, 1 decision(s)
1 acceptance(s) under a since-closed key need re-acceptance (L012 after the deadline):
  - acc:01M4G964QD4PBWXQGAJYGYAHJ2 (owner@customer.example): key key:01M4G962DZEF496FS7TS0FT8MH closed by key:01M4G96777YMD7CBRRMXA9AE0E — re-accept or affirm
exit 0

## F2: the holder's revoke of K1 hand-filed with at = the second a1 is dated in, truncated, so a1 is dated after it (what a backdated at allows today)
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 2 finding(s):
  - [L011] acc:01M4G964QD4PBWXQGAJYGYAHJ2: its `ssh` signature does not hold: its key key:01M4G962DZEF496FS7TS0FT8MH was closed in `fixture.ledger` by key:01M4G9681M12KXD05QB8J0P7EA, and the act is not dated and landed before the close (D6) — a close ends the key in every namespace, so it signs nothing in `fixture.ledger`
trust-root stage — 1 finding(s):
  - [SIGNERS] .decisions/allowed_signers: does not match the key bindings in the log — regenerate it with `ledger identity sync` — it is never edited by hand
exit 1
```

#### Case G: a drawer act

The script, `case_g.sh`:

```bash
#!/bin/bash
# Case G: a drawer act — signed before the close, filed with it (same commit) and after it.
source "$(dirname "$0")/lib.sh"
OWNER=owner@customer.example; NS=fixture.ledger
R=$SP/attacks/repos/g
mkrepo "$R" $OWNER
GRANT=$(govern "$R" $OWNER $NS)
D1=$(add_decision "$R" $NS "Money is decimal."); D2=$(add_decision "$R" $NS "Time is UTC.")
K1=$(bindings "$R" | head -1)
K2=$(keygen "$R" owner-next); tty "$R" identity add --namespace $NS --key-file "$K2.pub" >/dev/null
T0=$(at_of "$R" "$(bindings "$R" | tail -1)")
commit "$R" "base"
section "G1: the drawer act (signed by K1, dated inside the window) is filed in the same commit as the close"
A1=$($HAND accept --root "$R" --decision "$D1" --actor $OWNER --at "$(plus "$T0" 1)" --under "$GRANT" --key "$R/keys/genesis")
sleep 2.1; use_key "$R" "$K2"
tty "$R" identity revoke "$K1" >/dev/null
commit "$R" "close and drawer act $A1 together"
verify "$R"
section "G2: a second drawer act, same key and date, filed one commit later"
A2=$($HAND accept --root "$R" --decision "$D2" --actor $OWNER --at "$(plus "$T0" 1)" --under "$GRANT" --key "$R/keys/genesis")
commit "$R" "drawer act $A2 after the close"
verify "$R"
```

Its output, today's `ledger` from `918a08d`:

```text

## G1: the drawer act (signed by K1, dated inside the window) is filed in the same commit as the close
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 16 entries, 2 decision(s)
1 allocated, awaiting acceptance:
  - dec:fixture.ledger/01M4G8YQ58TH2A0PV6T4XG18Z1
1 acceptance(s) under a since-closed key need re-acceptance (L012 after the deadline):
  - acc:01M4G8YQYTGFWB927KNNJ28EBA (owner@customer.example): key key:01M4G8YPFEV7BVH2AT182W0WC3 closed by key:01M4G8YT22NFDFYYNEJKMJEZ65 — re-accept or affirm
exit 0

## G2: a second drawer act, same key and date, filed one commit later
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 1 finding(s):
  - [L011] acc:01M4G8YTQ9JGCK1AKAJ10X3MNW: its `ssh` signature does not hold: its key key:01M4G8YPFEV7BVH2AT182W0WC3 was closed in `fixture.ledger` by key:01M4G8YT22NFDFYYNEJKMJEZ65, and the act is not dated and landed before the close (D6) — a close ends the key in every namespace, so it signs nothing in `fixture.ledger`
1 acceptance(s) under a since-closed key need re-acceptance (L012 after the deadline):
  - acc:01M4G8YQYTGFWB927KNNJ28EBA (owner@customer.example): key key:01M4G8YPFEV7BVH2AT182W0WC3 closed by key:01M4G8YT22NFDFYYNEJKMJEZ65 — re-accept or affirm
exit 1
```

#### Case H: an impostor's self-bound genesis binding

The script, `case_h.sh`:

```bash
#!/bin/bash
# Case H: an impostor files a self-bound genesis binding while the window is open (LP-4.38, D7).
source "$(dirname "$0")/lib.sh"
OWNER=owner@customer.example; NS=fixture.ledger
R=$SP/attacks/repos/h
mkrepo "$R" $OWNER
section "init --without-key: the window is open"
tty "$R" init --namespace $NS --external-ref "$MANDATE" --without-key | grep -i "warning\|policy" | head -3
commit "$R" "unbound"
verify "$R" | grep -i "notice\|exit"
section "the impostor hand-files a self-bound binding for $OWNER's address, signed by a key nobody vouched for"
sleep 2.1; FK=$(keygen "$R" forged)
F=$($HAND bind --root "$R" --principal $OWNER --by $OWNER --ns $NS --new-key "$FK.pub" --mandate "$MANDATE" --at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" --sign "$FK")
commit "$R" "impostor's self-bound $F"
verify "$R"
section "the real holder then binds their key with the verb"
RK=$(keygen "$R" real); use_key "$R" "$RK"
tty "$R" identity add --namespace $NS --key-file "$RK.pub"; echo "exit $?"
section "the real holder's self-bound binding hand-filed anyway, dated one second before the impostor's (after the genesis grant), landed second"
RB=$($HAND bind --root "$R" --principal $OWNER --by $OWNER --ns $NS --new-key "$RK.pub" --mandate "$MANDATE" --at "$(plus "$(at_of "$R" "$F")" -1)" --sign "$RK")
commit "$R" "real holder's self-bound $RB, landed second"
verify "$R"
```

Its output, today's `ledger` from `918a08d`:

```text

## init --without-key: the window is open
namespace `fixture.ledger` under policy pol:01M4G9276FCQPM0G9954XDTPPV (accept role `acceptor`)
warning: no key bound (no key: `git config user.signingkey` is unset; --without-key) — until `ledger identity add --namespace fixture.ledger`, the first self-bound binding to land for owner@customer.example is the one trusted (D7)
notice: the genesis holder owner@customer.example has no trusted key — governed namespace(s) `fixture.ledger`: the first self-bound binding to land for that address will be the one trusted (D7); bind one with `ledger identity add --namespace <ns>`
exit 0

## the impostor hand-files a self-bound binding for owner@customer.example's address, signed by a key nobody vouched for
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 1 finding(s):
trust-root stage — 1 finding(s):
  - [SIGNERS] .decisions/allowed_signers: the log binds keys but no allowed_signers is committed — regenerate it with `ledger identity sync` — it is never edited by hand
exit 1

## the real holder then binds their key with the verb
refused — $SP/attacks/repos/h/keys/real is not owner@customer.example's live key in `fixture.ledger` — the signature would not verify
exit 0

## the real holder's self-bound binding hand-filed anyway, dated one second before the impostor's (after the genesis grant), landed second
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 1 finding(s):
trust-root stage — 1 finding(s):
  - [SIGNERS] .decisions/allowed_signers: the log binds keys but no allowed_signers is committed — regenerate it with `ledger identity sync` — it is never edited by hand
exit 1
  self_bound: true
  self_bound: true
```

#### Case I: a role landed late; a role edited

The script, `case_i.sh`:

```bash
#!/bin/bash
# Case I: a grant names a role that landed later; a role file edited after landing (LP-6.30, LP-8.27, LP-8.30).
source "$(dirname "$0")/lib.sh"
OWNER=owner@customer.example; NS=fixture.ledger
R=$SP/attacks/repos/i
mkrepo "$R" $OWNER
GRANT=$(govern "$R" $OWNER $NS)
D1=$(add_decision "$R" $NS "Money is decimal.")
tty "$R" accept "$D1" >/dev/null; A1=$(acceptances "$R" | tail -1)
git -C "$R" add -A; git -C "$R" reset -q -- .decisions/roles/acceptor.yml; git -C "$R" commit -q -m "everything but the acceptor role"
commit "$R" "the acceptor role, one commit later"
section "I1: the acceptance $A1 landed one commit before the role its grant names"
verify "$R"
section "I2: the role file edited after it landed (a capability added)"
python3 -c "import sys; p=sys.argv[1]; s=open(p).read().replace('- accept-decision\n','- accept-decision\n- grant-role\n'); open(p,'w').write(s)" "$R/.decisions/roles/acceptor.yml"
cat "$R/.decisions/roles/acceptor.yml"
commit "$R" "edited role"
verify "$R"
```

Its output, today's `ledger` from `918a08d`:

```text

## I1: the acceptance acc:01M4G92AQ9GBEK5GR43F1SD2QB landed one commit before the role its grant names
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 1 finding(s):
graph stage — 1 finding(s):
  - [A006] acc:01M4G92AQ9GBEK5GR43F1SD2QB: owner@customer.example holds no grant of a role that may do this, as of the act (accept)
exit 1

## I2: the role file edited after it landed (a capability added)
format: 6
id: acceptor
title: Accepts decisions
owner: owner@customer.example
may:
- accept-decision
- grant-role
created_at: 2026-10-09
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 2 finding(s):
  - [L007] .decisions/roles/acceptor.yml: changed since it landed in 0689a5521087 — roles are write-once; a new role and new grants supersede
graph stage — 1 finding(s):
  - [A006] acc:01M4G92AQ9GBEK5GR43F1SD2QB: owner@customer.example holds no grant of a role that may do this, as of the act (accept)
exit 1
```

#### Case J: a copy that drops the latest close; a stale clone

The script, `case_j.sh`:

```bash
#!/bin/bash
# Case J: a copy of a namespace that drops its latest close, against a stale clone.
source "$(dirname "$0")/lib.sh"
OWNER=owner@customer.example; NS=fixture.ledger
R=$SP/attacks/repos/j
mkrepo "$R" $OWNER
GRANT=$(govern "$R" $OWNER $NS)
D1=$(add_decision "$R" $NS "Money is decimal.")
K1=$(bindings "$R" | head -1)
tty "$R" accept "$D1" >/dev/null; A1=$(acceptances "$R" | tail -1)
K2=$(keygen "$R" owner-next); tty "$R" identity add --namespace $NS --key-file "$K2.pub" >/dev/null
commit "$R" "a1 by K1"
sleep 2.1; use_key "$R" "$K2"
tty "$R" identity revoke "$K1" >/dev/null
CL=$(bindings "$R" | tail -1)
commit "$R" "K1 closed by $CL"
section "the source: a1 under a closed key"
verify "$R"
CLOSEFILE=$(grep -l "$CL" "$R"/.decisions/log/*.yml)
section "the copy: every file but the change-set holding the close ($(basename "$CLOSEFILE")) and its sidecar, one commit by a copier"
T=$SP/attacks/repos/j-copy; rm -rf "$T"; mkdir -p "$T"; git -C "$T" init -q --initial-branch=main
git -C "$T" config user.name Copier; git -C "$T" config user.email copier@example
mkdir -p "$T/.decisions/log" "$T/.decisions/sig" "$T/.decisions/sets" "$T/.decisions/roles"
cp "$R"/.decisions/sets/* "$T/.decisions/sets/"; cp "$R"/.decisions/roles/* "$T/.decisions/roles/"
for f in "$R"/.decisions/log/*.yml; do [ "$f" = "$CLOSEFILE" ] || cp "$f" "$T/.decisions/log/"; done
for f in "$R"/.decisions/sig/*; do [ "$(basename "$f")" = "${CL#key:}.ssh.sig" ] || cp "$f" "$T/.decisions/sig/"; done
piped "$T" identity sync | head -1
commit "$T" "copied without the close"
verify "$T"
section "a stale clone of the source at the commit before the close (no origin/HEAD, so no base overlay)"
S=$SP/attacks/repos/j-stale; rm -rf "$S"; git clone -q "$R" "$S"; git -C "$S" checkout -q HEAD~1; git -C "$S" remote remove origin
verify "$S"
```

Its output, today's `ledger` from `918a08d`:

```text

## the source: a1 under a closed key
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 14 entries, 1 decision(s)
1 acceptance(s) under a since-closed key need re-acceptance (L012 after the deadline):
  - acc:01M4G92C99Q69845N51SXA1W5Q (owner@customer.example): key key:01M4G92BG6M9XBMB4V1TB5HPS4 closed by key:01M4G92EYVT9YWS4KHBT8FTC84 — re-accept or affirm
exit 0

## the copy: every file but the change-set holding the close (01M4G92EZ4J84HAV183K4NWEPA.yml) and its sidecar, one commit by a copier
allowed_signers rewritten from the log
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 13 entries, 1 decision(s)
exit 0

## a stale clone of the source at the commit before the close (no origin/HEAD, so no base overlay)
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 13 entries, 1 decision(s)
exit 0
```

#### Case K: this repository's no-policy namespaces copied

The script, `case_k.sh`:

```bash
#!/bin/bash
# Case K: this repository's two no-policy namespaces copied into a fresh repository (E1a, E1b, E1c).
source "$(dirname "$0")/lib.sh"
SRC=/home/user/product-cli
for who in copier@example emk@delegate.dk; do
  T=$SP/attacks/repos/k-$who; rm -rf "$T"; mkdir -p "$T"; git -C "$T" init -q --initial-branch=main
  git -C "$T" config user.name Copier; git -C "$T" config user.email "$who"
  mkdir -p "$T/docs"; cp -r "$SRC/.decisions" "$T/"; rm -rf "$T/.decisions/index"; cp -r "$SRC/docs/decisions" "$T/docs/"
  commit "$T" "copied by $who"
  section "copied in one commit by $who, blame on"
  "$L" --root "$T" verify --export 2>&1 | grep -v "^  - \[L009\]" | head -12; echo "L009 findings: $("$L" --root "$T" verify 2>&1 | grep -c "\[L009\]")"
  section "the same, --no-blame"
  verify "$T" --export | tail -4
done
```

Its output, today's `ledger` from `918a08d`:

```text

## copied in one commit by copier@example, blame on
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 91 finding(s):
3 allocated, awaiting acceptance:
  - dec:hafeok.ddd/01KZTGGMEACBFMTC1RJJ8T90GS
  - dec:hafeok.ddd/01KZTGGX5ABSQ2PVTQ32NPKVNE
  - dec:hafeok.ledger/01KZXJX693301CZSY4XNP643XY
notice: namespace `hafeok.ddd` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ddd` opts it in)
notice: namespace `hafeok.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ledger` opts it in)
export: every committed export matches the log byte for byte
L009 findings: 91

## the same, --no-blame
notice: namespace `hafeok.ddd` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ddd` opts it in)
notice: namespace `hafeok.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ledger` opts it in)
export: every committed export matches the log byte for byte
exit 0

## copied in one commit by emk@delegate.dk, blame on
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 284 entries, 93 decision(s)
3 allocated, awaiting acceptance:
  - dec:hafeok.ddd/01KZTGGMEACBFMTC1RJJ8T90GS
  - dec:hafeok.ddd/01KZTGGX5ABSQ2PVTQ32NPKVNE
  - dec:hafeok.ledger/01KZXJX693301CZSY4XNP643XY
notice: namespace `hafeok.ddd` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ddd` opts it in)
notice: namespace `hafeok.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ledger` opts it in)
export: every committed export matches the log byte for byte
L009 findings: 0

## the same, --no-blame
notice: namespace `hafeok.ddd` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ddd` opts it in)
notice: namespace `hafeok.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ledger` opts it in)
export: every committed export matches the log byte for byte
exit 0
```

#### The prototype's output over the stores (`run_d.sh`)

```text

## repos/a: the close names nothing (the act did not exist)
names: {}
acc:01M4G8RXY1RZX2WSW010KC9GC5  at   governed  signed by key:01M4G8RSXT6ES45KKDZW555VQN  close key:01M4G8RX8YTNE4JB999PR45B3H at 2026-10-09T12:03:58Z: AFTER it -> L011

## repos/b: the thief's rotate names nothing; the revoke of K2 names nothing
names: {}
acc:01M4G941RMTWBGN8WJ4A7V0JP1  at 2026-10-09T12:10:03Z  governed  signed by key:01M4G9413H9FE34X0M88RN30A5  close key:01M4G9447039Z33W2RB2T9GNNP at 2026-10-09T12:10:05Z: AFTER it -> L011
acc:01M4G9455BPQTA6623MJCC7G65  at 2026-10-09T12:10:06Z  governed  signed by key:01M4G9447039Z33W2RB2T9GNNP  close key:01M4G947V67N43851K07J3YQWV at 2026-10-09T12:10:06Z: AFTER it -> L011

## repos/b: the thief's rotate names the legit a1; the revoke of K2 names nothing
names: {"key:01M4G9447039Z33W2RB2T9GNNP": ["acc:01M4G941RMTWBGN8WJ4A7V0JP1"]}
acc:01M4G941RMTWBGN8WJ4A7V0JP1  at 2026-10-09T12:10:03Z  governed  signed by key:01M4G9413H9FE34X0M88RN30A5  close key:01M4G9447039Z33W2RB2T9GNNP at 2026-10-09T12:10:05Z: BEFORE it (named, dated earlier) -> review item
acc:01M4G9455BPQTA6623MJCC7G65  at 2026-10-09T12:10:06Z  governed  signed by key:01M4G9447039Z33W2RB2T9GNNP  close key:01M4G947V67N43851K07J3YQWV at 2026-10-09T12:10:06Z: AFTER it -> L011

## repos/c: the revocation names a1
names: {"rev:01M4G8Y4RPJKJKYYXQ6PBA1E97": ["acc:01M4G8Y2B75PJNC8S63P40P1YR"]}
acc:01M4G8Y2B75PJNC8S63P40P1YR  at 2026-10-09T12:06:47Z  governed  grant revoked rev:01M4G8Y4RPJKJKYYXQ6PBA1E97: BEFORE it (named, dated earlier)  signed by key:01M4G8Y1PBHXDT8JE7AGT8Q79R
acc:01M4G8Y5GD8YH7EK1ZEMCK5FRC  at 2026-10-09T12:06:48Z  governed  grant revoked rev:01M4G8Y4RPJKJKYYXQ6PBA1E97: AFTER it -> A006  signed by key:01M4G8Y1PBHXDT8JE7AGT8Q79R

## repos/d: the first policy names a0
names: {"pol:01M4G8Y7417CQ3CTV3AZFYQX1M": ["acc:01M4G8Y5WE9WDNC8HVKWX5PECS"]}
acc:01M4G8Y5WE9WDNC8HVKWX5PECS  at 2026-10-09T12:06:51Z  pre-policy (named by pol:01M4G8Y7417CQ3CTV3AZFYQX1M)
acc:01M4G8Y88SGTBTAFN6JG20JZCJ  at 2026-10-09T12:05:52Z  governed  unsigned

## repos/e1: the close on main names nothing
names: {}
acc:01M4G8Y9TJZDMKEY2MW9B9YXNE  at 2026-10-09T12:06:55Z  governed  signed by key:01M4G8Y8HPKSEJWR0R29KA2AXY  close key:01M4G8YBYG7X2HRXBAWMZXWSQX at 2026-10-09T12:06:57Z: AFTER it -> L011

## repos/e2: the close on the branch names a1 and not a2
names: {"key:01M4G8YGQQ9HP9JY4NN64HY5YC": ["acc:01M4G8YDVGP55JD8DRXWVJAYFA"]}
acc:01M4G8YDVGP55JD8DRXWVJAYFA  at 2026-10-09T12:06:59Z  governed  signed by key:01M4G8YCXX731W73XA1F8Z3V41  close key:01M4G8YGQQ9HP9JY4NN64HY5YC at 2026-10-09T12:07:02Z: BEFORE it (named, dated earlier) -> review item
acc:01M4G8YHBRSXTYBCKRP76EDQ0G  at 2026-10-09T12:07:00Z  governed  signed by key:01M4G8YCXX731W73XA1F8Z3V41  close key:01M4G8YGQQ9HP9JY4NN64HY5YC at 2026-10-09T12:07:02Z: AFTER it -> L011

## repos/f: the close names a1
names: {"key:01M4G96777YMD7CBRRMXA9AE0E": ["acc:01M4G964QD4PBWXQGAJYGYAHJ2"]}
acc:01M4G964QD4PBWXQGAJYGYAHJ2  at 2026-10-09T12:11:11Z  governed  signed by key:01M4G962DZEF496FS7TS0FT8MH  close key:01M4G96777YMD7CBRRMXA9AE0E at 2026-10-09T12:11:14Z: BEFORE it (named, dated earlier) -> review item

## repos/f: the closer omits a1
names: {}
acc:01M4G964QD4PBWXQGAJYGYAHJ2  at 2026-10-09T12:11:11Z  governed  signed by key:01M4G962DZEF496FS7TS0FT8MH  close key:01M4G96777YMD7CBRRMXA9AE0E at 2026-10-09T12:11:14Z: AFTER it -> L011

## repos/g: the close names both drawer acts (both existed as bytes when it was written)
names: {"key:01M4G8YT22NFDFYYNEJKMJEZ65": ["acc:01M4G8YQYTGFWB927KNNJ28EBA", "acc:01M4G8YTQ9JGCK1AKAJ10X3MNW"]}
acc:01M4G8YQYTGFWB927KNNJ28EBA  at 2026-10-09T12:07:09Z  governed  signed by key:01M4G8YPFEV7BVH2AT182W0WC3  close key:01M4G8YT22NFDFYYNEJKMJEZ65 at 2026-10-09T12:07:11Z: BEFORE it (named, dated earlier) -> review item
acc:01M4G8YTQ9JGCK1AKAJ10X3MNW  at 2026-10-09T12:07:09Z  governed  signed by key:01M4G8YPFEV7BVH2AT182W0WC3  close key:01M4G8YT22NFDFYYNEJKMJEZ65 at 2026-10-09T12:07:11Z: BEFORE it (named, dated earlier) -> review item

## repos/g: the close names only the drawer act filed with it
names: {"key:01M4G8YT22NFDFYYNEJKMJEZ65": ["acc:01M4G8YQYTGFWB927KNNJ28EBA"]}
acc:01M4G8YQYTGFWB927KNNJ28EBA  at 2026-10-09T12:07:09Z  governed  signed by key:01M4G8YPFEV7BVH2AT182W0WC3  close key:01M4G8YT22NFDFYYNEJKMJEZ65 at 2026-10-09T12:07:11Z: BEFORE it (named, dated earlier) -> review item
acc:01M4G8YTQ9JGCK1AKAJ10X3MNW  at 2026-10-09T12:07:09Z  governed  signed by key:01M4G8YPFEV7BVH2AT182W0WC3  close key:01M4G8YT22NFDFYYNEJKMJEZ65 at 2026-10-09T12:07:11Z: AFTER it -> L011

## repos/a, rebuilt with the corrected helper: the close names nothing
acc:01M4G97EHAZSHWM4PC0WRMVPD1  at 2026-10-09T12:11:52Z  governed  signed by key:01M4G97APM4C95CZZ3PX6YWYSV  close key:01M4G97E017PXTFJBJ8E3QNSXY at 2026-10-09T12:11:54Z: AFTER it -> L011
```

### Step 2, continued: the verdicts under position D

**By hand**, from the rule of PRD §3.5.1 applied to each store's entries and the names its writer would have had at hand; **checked with the prototype** below for the acceptance-level cases (A to G). The prototype reads each acceptance's signing key from its sidecar's SSHSIG blob, each close of that key, the revocation of the grant the acceptance names, and the namespace's first policy, and applies "named and dated earlier" with a names map supplied by hand. It verifies no signature, judges no filer, reads no git, and decides nothing about enabling entries or roles; its one job is to show that the rule, applied mechanically, gives the verdicts the PRD's table claims.

| Case | Today *(run)* | Under D | How known |
| --- | --- | --- | --- |
| A | `L011` | `L011`: the close names nothing, the act is after it | Hand; prototype |
| B1 (verb revoke, `at` = now) | Conformant; forged and legitimate acceptances both review items | The revoke of K2 names nothing: forged `L011`. The thief's rotate decides K1's acts: named, review; unnamed, `L011` | Hand; prototype, both namings |
| B2 (revoke dated one second after the rotate) | Forged `L011`; legitimate review item | As B1 | Hand; prototype |
| C | `A006` | `A006`: the revocation names only the earlier acceptance | Hand; prototype |
| D | `L011`, `A006`; the pre-policy acceptance stands | The same: the first policy names only the pre-policy acceptance | Hand; prototype |
| E1 | `L011` on the branch with `--base main` and on the merge | `L011`: the close on `main` names nothing | Hand; prototype |
| E2 | Conformant on the branch with `--base main` and on the merge; the act a review item | `L011`: the close was written before the act existed. `main` red after the merge; put right by re-accepting under the live key or by refiling the close naming the act before it merges | Hand; prototype |
| F1 (verb close) | Review item | Named: review item. Omitted: `L011` | Hand; prototype, both |
| F2 (close dated in the act's second) | `L011` | `L011` whether named or not: dated no earlier | Hand |
| G1 (drawer act filed with the close) | Review item | Named by digest: review item | Hand; prototype |
| G2 (drawer act filed after the close) | `L011` | Named by digest: review item; unnamed: `L011`. A name by id alone would be a promise | Hand; prototype, both |
| H | The impostor's binding trusted; the holder's verb refused; the holder's hand-filed binding, dated earlier, **also trusted**: two lines in `allowed_signers`, conformant | Only the binding the genesis grant's `anchor` names is trusted | Hand |
| I1, I2 | `A006`; `L007` | A grant binds its role by `role_hash`; the role's position plays no part; the edit is a schema fault on the grant anywhere, and `L007` in the repository | Hand |
| J | The truncated copy and the stale clone (no base ref) are conformant | Unchanged; the freshness question | Hand |
| K | 91 `L009` by a copier; conformant with `--no-blame`; conformant by the sole acceptor | Unchanged: `L009` is git's and restarts | Hand |

### What was found in passing

Three things about today's code, none of them this session's to fix:

1. **Two self-bound bindings can both be trusted** (case H, O14). `authority::filing::self_bound` asks whether the principal has another binding in `Authority::as_of`, and `as_of` admits an enabling binding only when it is `not_after` the one being judged: landed no later **and dated no later**. A self-bound binding landed second but dated earlier therefore does not see the one landed first, and both pass D7. LP-4.38 says the first self-bound binding to land is the one trusted; the implementation trusts a later-landed one as well when it is backdated. The window is open only after `init --without-key`. Under position D the anchor closes it; under C it is a fix of its own (PRD §7, question 20).
2. **No verb takes a time.** `grep` over `ledger-cli/src` finds no `--at`; every writer stamps `self.now`. D6's remedy for a compromise, "a close with `at` set to the time of compromise", and D7 (5)'s, can only be hand-filed today.
3. **A close cannot be dated before the window it closes.** `authority::filing::closing` looks the closed binding up in `Authority::as_of` at the close's position, and an opening binding is admitted only when `not_after` it; a revoke dated before the binding it closes finds "opens no key window". So the earliest a backdated revoke can be dated is the closed binding's own `at`, which for a thief's `rotate` is the rotate's time (case B2).

### What could not be determined

- Varve's acceptance count under its import. The prompt gives 607 decisions; the audit (`ledger/audits/audit-2026-10.md`) counts 513 acceptances in the interim files. The size table uses 607 as 607 acceptances.
- Whether `ssh-keygen -Y sign` and `-Y verify` behave differently over a 64 KB message than over a 400-byte one. Not measured; nothing in OpenSSH's documentation bounds the message, and the size table is an estimate from field widths, not from a signed file.
- The export literals for the new fields (`ledger:after`, `ledger:anchor`, `ledger:roleHash`) are proposed names; the emitter was not read for how a set-valued payload field is emitted.
- Whether a `rotate`'s names can be overridden by a later act of the genesis holder. No mechanism exists; the PRD states the cost (case B) and does not design one.
- E2a, E2c, E5a and E5c (carried history) were not re-run: `git filter-repo` is not installed here. The PRD cites them from the first session, where they ran on `e20fadc`; no code on that path has changed since, by the fixes session's list.
- The order grid (`order_tests.rs`) was not re-run under D: a prototype over the fixed-landing grid would have to re-implement `Authority::as_of`, which is the implementation issue itself.

### Questions for the principal, in one list

The PRD's §7 holds each with its options and the lean. In order, the ones gating the format first:

1. Position C (as ruled, Appendix A.1) or position D (§3.5).
2. What a name is: ids, digests, or `<id>@sha256:<hash>`.
3. Where the names sit: in the signed payload, or in the entity with a digest in the payload.
4. The field name (`after`).
5. Format 8's content: `after`, `anchor`, `role_hash` in place of the move act (ruling 66).
6. An entry filed below format 8 names nothing, or is placed by D6 as a legacy reading.
7. Successive policies: every earlier act, or since the policy replaced.
8. The founding: the genesis grant names its anchor, the first policy does, or `--without-key` goes; and whether the pin keeps two tokens (ruling 70).
9. #82 before D, or D with grants left on D6 until #82.
10. `role_hash` on grants, or a signed role payload.
11. Strict, or a review window for `rotate`.
12. Stray and dangling names: notices or schema faults.
13. The move's source: whole-namespace removal as a notice, a tombstone, or nothing leaves (ruling 65).
14. `L009`'s scope: every acceptance, or acts no signature covers (ruling 79, LP-8.32).
15. An ungoverned namespace's move: history carried, governed first, or governed in the target (ruling 84).
16. `A007`: unused, or assigned (ruling 77).
17. Ruling 82: as written with a smaller capability, or verify with a notice.
18. `identity revoke --at` for the genesis holder.
19. A close naming an uncommitted act of its family: names it, or refuses.
20. Case H's double trust under C: fix now, or with the anchor.

### What changed in the PRD

- **Header and contents.** The third revision, its base (`918a08d`, #128 merged), and the `(prototype)` mark.
- **§3 intro.** The diagram loses the landing-record node and gains the naming edge.
- **§3.1, §3.1.1.** Three sentences: the landing directory and the departure reader are marked as ruled; under D the readers of flat history shrink and the refusal stands.
- **§3.5** is rewritten: "Order from the acts (position D)", with the direction, the attack table, and the eleven topics of the prompt's step 3, each with its options side by side, costs, and a lean marked as a lean. The second revision's §3.5 moves whole to Appendix A.1.1.
- **§3.9, §3.10, §3.11, §3.13** mark where the two designs differ: the export-only verifier's reach, format 8's content, `A007`, the tests to rewrite, N17 to N19.
- **§4** splits into criteria under either design and position D's: AC-1 rewritten around a copy, one criterion per attack row (AC-D-A to AC-D-K) and one per answer (AC-D-1 to AC-D-11). The ruled AC-1, AC-64, AC-65, AC-83 and AC-84 are in Appendix A.1.2.
- **§5** splits the same way; the ruled move proposals are in Appendix A.1.3. Position D's proposals rewrite LP-8.26, LP-8.28, LP-4.13, LP-6.29 and LP-9.15, add LP-5.23, LP-6.35 to LP-6.37 and LP-8.34, and drop LP-8.35.
- **§6** drops the move-act issue, pulls #82 forward, adds four naming issues and the move-as-copy issue, and re-sizes issue 2 and the export-only verifier.
- **§7** adds "Position C and position D": D6 against D, each of rulings 64, 65, 66, 77, 79, 82, 83 and 84 with what replaces it, rulings 61 to 81 one by one, and twenty questions.
- **Appendix A** gains the third revision's options row and A.1, the ruled move design whole.

### Checks

### Third revision (order from the acts)

- **What changed.** The branch is restarted from `main` at `918a08d`. Against it, the pull request changes two files: `ledger/prd/namespace-independence-prd.md` and this record. Nothing else: no code, test, fixture, protocol text, ruling, store, export or `.ddd/` file. The attack stores, scripts, keys, names and outputs live in the scratchpad and are not committed.
- **Gates**, with the four git-identity variables unset, run on `918a08d`'s code, which the revision does not touch:
  - `cargo build`: exit 0.
  - `cargo clippy -- -D warnings -D clippy::unwrap_used`: exit 0.
  - `cargo t`: exit 0. 2,121 passed, 0 failed and 2 ignored, summed over the 95 `test result:` lines.
  - `dotnet test` was not run, since no .NET code is touched.
- **`ledger verify --export`**, built from `918a08d` and run on this branch and on a worktree of `origin/main` at `918a08d`: exit 0 on both, and byte-identical output:

```text
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 284 entries, 93 decision(s)
3 allocated, awaiting acceptance:
  - dec:hafeok.ddd/01KZTGGMEACBFMTC1RJJ8T90GS
  - dec:hafeok.ddd/01KZTGGX5ABSQ2PVTQ32NPKVNE
  - dec:hafeok.ledger/01KZXJX693301CZSY4XNP643XY
notice: namespace `hafeok.ddd` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ddd` opts it in)
notice: namespace `hafeok.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ledger` opts it in)
export: every committed export matches the log byte for byte
exit 0
```
- **Pushed** to `origin/claude/focused-goodall-qh7sm7` on 9 October 2026, as one commit on top of `918a08d`, for the principal to review and merge. Merging is his.
- **Moved**, later on 9 October 2026, to `claude/zealous-cannon-da2epc-act-ordering`, a new branch from `main` at `fac640d` (the merge of #139, which changes tests only), because `claude/focused-goodall-qh7sm7` is #128's branch and #128 had merged. The commit is carried over unchanged; this note and the two sentences above that said the pull request was on that branch are the only additions. The rulings file is untouched: rulings 82 to 84 stand on `main` until the principal supersedes them. `ledger verify --export`, built from and run on the new branch: exit 0, the output above. The pull request revises the design merged in #128.

## Principal's replies, 2026-10-09

The design is accepted: position D. Every lean of the third revision's §7 is ruled as it was leaned, and one question the PRD did not ask is ruled with them, as rulings 85 to 101 in a new file, `ledger/rulings/order-from-the-acts-rulings-2026-10-09.md`. They supersede D6 of 2 October and rulings 64, 65, 79, 83 and 84; amend rulings 66 and 77, D7's first filer and LP-4.39's "a window closes once"; and leave ruling 82 standing. The earlier rulings files are not edited: the new file states what it supersedes.

### Rulings, by number

| Ruling | Answers | In short |
| --- | --- | --- |
| 85 | Q1, Q11 | Order comes from the acts: before a terminating entry when named by it and dated strictly earlier; unnamed is after, whatever the `at`; an enabling entry covers by signed `at`; landing order decides nothing. Supersedes D6 and ruling 64. |
| 86 | Q2, Q3, Q4 | Names are `<id>@sha256:<hash>`, in a set-valued `after` inside the signed payload; policies too |
| 87 | Q5, Q6 | Format 8 holds `after`, `anchor` and `role_hash`; no move act; an entry below format 8 names nothing. Amends 66. |
| 88 | Q7 | A policy change names only the acts since the policy it replaces; the earliest policy that names an act governs it; unnamed, the tip |
| 89 | Q8 | The genesis grant names its anchor; no "first to land"; `init` refuses without a usable key and `--without-key` is removed; a second genesis is `A005`, cleared by revoking it; the pin's token count is for the pinning design. Amends D7's first filer. |
| 90 | Q9 | Grants, grant acceptances, unavailabilities and availabilities are signed with `at` in their payloads before order from the acts (#82); `A006` judges the grantor as of the grant |
| 91 | Q10 | A grant names its role's content as `role_hash` under `ledger.role.v1`; a role's position plays no part |
| 92 | Q12 | A name outside the family, or resolving to no filed act, is ignored and reported as a notice |
| 93 | Q13 | Whole removal of a namespace and its export in one commit is a notice; part stays `L007`. Supersedes 65. |
| 94 | Q14 | `L009` judges only acts no signature covers. Supersedes 79. |
| 95 | Q15 | A move is a copy; no move act, landing record or freeze; a namespace with no policy moves with its history or is governed and re-accepted first. Supersedes 83 and 84. |
| 96 | Q16 | `A007` returns to unused. Amends 77. |
| 97 | Q17 | Ruling 82 stands as written; the legacy capability shrinks to `L007`, `L009` and the base overlay |
| 98 | Q18 | The genesis holder's revoke takes an `at`, the time of compromise, bounded below by the `at` of the binding it closes |
| 99 | Q19 | A writer names the acts of the family the checkout holds, committed or not; a name that never lands is ignored under 92 |
| 100 | Q20 | Two trusted self-bound bindings: closed by the anchor; no separate fix under D6 |
| 101 | the review | The genesis holder's revoke may close a key a `rotate` already closed; where several closes end one key an act stands only if before each (LP-4.39); a thief's `rotate` naming forged acts does not keep them standing once the revoke at the compromise does not name them. Amends "a window closes once" for this case. |

### Ruling 101, read against case B

Case B's store was read under rulings 85 and 101 by hand; nothing was built or run, and the PRD marks the reading *(inference)* as case L of its attack table.
- The thief's `rotate` is a binding signed by K1. It closes K1, opens K2, and names what the thief chooses. Named by digest, a forged acceptance signed by K1, dated inside K1's window and filed with the rotate, is before that close (case G's mechanism).
- The genesis holder's `revoke`, signed by the genesis holder's own key (case B's `genesis@customer.example`) under the genesis grant, closes K1 again (101), dated at the compromise (98) and no earlier than K1's binding's `at`, and names the holder's legitimate acceptance.
- Under LP-4.39's "every close" reading an act stands only if before each close. The forged acceptance is before the rotate and not before the revoke: `L011`. The legitimate acceptance is before both: a review item, as today.
- The rotate itself is an act K1 signed that the revoke does not name, so it is after the revoke and its signature does not hold; K2 opens no window, and every act K2 signed is refused. Revoking K2 as well (case B) reaches the same verdicts.

One thing the reading turned up, put to the principal in PRD §7: ruling 99 has the writer name every act of the family the checkout holds, and in case L the checkout holds the forged acts, backdatable to before `--at`; ruling 101 has the revoke not name them. Both hold only if that one revoke may name less than the checkout holds, and how is not ruled.

### Case L, built and run

A second session, working from the same rulings, built the store the reading above describes, with the helpers of "Order from the acts": the thief, holding K1, files a forged acceptance backdated inside K1's window, rotates K1 to K2, and accepts with K2; the genesis holder then files a second close of K1 by hand, a `revoke` dated at the compromise. Today the second close is refused: `authority::references::binding_refs` ("already closed — a window closes once") and `authority::filing::closing` ("window is already closed"); the rotate itself falls to the same D7 rule, because the revoke, dated earlier, is in `Authority::as_of` at the rotate's position; and the K2 act is `L011` as signed by a key bound to nobody. That is the "Today" column of row L. The ruled column stays a reading by hand: the prototype does not read several closes of one key.

The script, `case_l.sh`:

```bash
#!/bin/bash
# Case L (ruling 101): the thief's rotate closes K1 and signs forged acts with K1 (backdated) and K2;
# the genesis holder's revoke of K1, dated at the compromise, closes K1 a second time.
source "$(dirname "$0")/lib.sh"
R=$SP/attacks/repos/l; G=genesis@customer.example; H=holder@customer.example; NS=fixture.ledger
mkrepo "$R" $G
GK=$(keygen "$R" genesis); use_key "$R" "$GK"
tty "$R" init --namespace $NS --external-ref "$MANDATE" >/dev/null
GENESIS=$(grants "$R" | head -1)
GRANT=$(tty "$R" grant new acceptor --to $H --scope "ns:$NS" | word grant:)
act_as "$R" $H; tty "$R" grant accept "$GRANT" >/dev/null; act_as "$R" $G
KH=$(keygen "$R" holder)
tty "$R" identity add --namespace $NS --for $H --key-file "$KH.pub" >/dev/null
KHID=$(bindings "$R" | tail -1)
act_as "$R" $H; use_key "$R" "$KH"
D1=$(add_decision "$R" $NS "Money is decimal."); D2=$(add_decision "$R" $NS "Time is UTC."); D3=$(add_decision "$R" $NS "Names are unique.")
sleep 1.1; tty "$R" accept "$D1" >/dev/null; A1=$(acceptances "$R" | tail -1)
commit "$R" "legit: $A1 by K1"
T1=$(at_of "$R" "$A1")
section "holder $H, K1=$KHID; legit $A1 at $T1"
sleep 2.1
section "the thief, with K1: a forged acceptance signed by K1 dated one second after the legit one, then rotate K1 to K2, then an acceptance with K2"
AF=$($HAND accept --root "$R" --decision "$D2" --actor $H --at "$(plus "$T1" 1)" --under "$GRANT" --key "$KH")
K2=$(keygen "$R" thief)
tty "$R" identity rotate "$KHID" --key-file "$K2.pub" | head -1
ROT=$(bindings "$R" | tail -1); TROT=$(at_of "$R" "$ROT")
use_key "$R" "$K2"
tty "$R" accept "$D3" >/dev/null; A3=$(acceptances "$R" | tail -1)
commit "$R" "compromise: forged $AF (K1, backdated), rotate $ROT at $TROT, $A3 (K2)"
verify "$R"
section "the genesis holder's second close of K1, hand-filed: a revoke of $KHID dated at the compromise ($(plus "$T1" 1)), under the genesis grant, signed with the genesis key"
act_as "$R" $G
REV=$($HAND close --root "$R" --act revoke --closes "$KHID" --by $G --at "$(plus "$T1" 1)" --under "$GENESIS" --sign "$GK")
commit "$R" "genesis re-closes K1 ($REV)"
verify "$R"
```

Its output, today's `ledger` from `918a08d`:

```text

## holder holder@customer.example, K1=key:01M4GDF54P26QZTFZ5J1XQJ2M2; legit acc:01M4GDF754AFTHASS4EYADBQ6G at 2026-10-09T13:26:03.631360004Z

## the thief, with K1: a forged acceptance signed by K1 dated one second after the legit one, then rotate K1 to K2, then an acceptance with K2
identity rotate: holder@customer.example in `fixture.ledger` — key:01M4GDF9R0WYFJEJ75TQRK4DY5
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 20 entries, 3 decision(s)
2 acceptance(s) under a since-closed key need re-acceptance (L012 after the deadline):
  - acc:01M4GDF754AFTHASS4EYADBQ6G (holder@customer.example): key key:01M4GDF54P26QZTFZ5J1XQJ2M2 closed by key:01M4GDF9R0WYFJEJ75TQRK4DY5 — re-accept or affirm
  - acc:01M4GDF9P7J3ZDRN87241DMJKG (holder@customer.example): key key:01M4GDF54P26QZTFZ5J1XQJ2M2 closed by key:01M4GDF9R0WYFJEJ75TQRK4DY5 — re-accept or affirm
exit 0

## the genesis holder's second close of K1, hand-filed: a revoke of key:01M4GDF54P26QZTFZ5J1XQJ2M2 dated at the compromise (2026-10-09T13:26:04Z), under the genesis grant, signed with the genesis key
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
non-conformant — 5 finding(s):
  - [SCHEMA] key:01M4GDF9R0WYFJEJ75TQRK4DY5: D7: key:01M4GDF54P26QZTFZ5J1XQJ2M2's window is already closed
  - [SCHEMA] key:01M4GDFB5JJSW9KWATEV6APFAB: key:01M4GDF54P26QZTFZ5J1XQJ2M2 is already closed — a window closes once
  - [SCHEMA] key:01M4GDFB5JJSW9KWATEV6APFAB: D7: key:01M4GDF54P26QZTFZ5J1XQJ2M2's window is already closed
  - [L011] acc:01M4GDFACM6DXCJWQN1DRK54YQ: its `ssh` signature does not hold: signed by SHA256:z+9hGi+zNuvucO6qrEuTpo8XxxjnXASBj2zNkcOb6kg, which is no key bound to holder@customer.example in `fixture.ledger`
trust-root stage — 1 finding(s):
  - [SIGNERS] .decisions/allowed_signers: does not match the key bindings in the log — regenerate it with `ledger identity sync` — it is never edited by hand
exit 1
```

### What changed in the PRD

- **Header and contents.** The fourth revision, the rulings it rests on, the base (`fac640d`), and the contents list.
- **§3 diagram, §3.1, §3.1.1, §3.6, §3.11, §3.13.** Every "as ruled" / "under D" pair collapses to the ruled text: no `landing/` directory, no departure reader, the readers of flat history shrunk to three (97), `--without-key` gone from the tests to rewrite (89), the pin's token count deferred (89), N17 to N19 answered by 85, 94, 93 and 95.
- **§3.5** states the ruled design, citing its ruling at each point: the rule (85, 101), the encoding (86, 87, 92), size and successive policies (88), the unsigned records (90, 91), the founding (89, 100), the writer (98, 99, 101), what git still does (97), the move (93, 94, 95), the export, freshness (open), cost. The attack table gains case L, ruling 101 read against case B. The option tables are gone; what they held is one row each in Appendix A.
- **§3.9 and §3.10** state the export and format 8 as ruled; `A007` is unused (96); the move act's rows are gone.
- **§4** is one list: the layout, authority, export and pin criteria, then order from the acts and the move. AC-D-L is new (101). AC-42 and AC-D-5 carry the pin's token count to the pinning design.
- **§5** is one set of proposals: the shared ones, then order from the acts and the move, each with its ruling. New: the LP-4.39 amendment and LP-6.38 (98, 101). LP-8.30 and LP-8.32 are no longer "if chosen". The superseded design's proposals stay in A.1.3 and are not made.
- **§6** keeps the twenty-two issues, their order and their sizes; 12, 15, 16, 17 and 20 cite the rulings that settled them, and 15 waits on §7's question for the thief's acts.
- **§7** replaces the twenty questions with the rulings that answered them, by number, keeps the side-by-side comparison of C and D as the record of what was weighed, lists what each earlier ruling became, and asks the one question the rulings raise. N-Q2, the freshness question and the pin's token count stay open for the pinning design; N-Q5 is closed.
- **Appendix A** holds every option not chosen in one table, kept short, and A.1 is marked superseded by rulings 85, 93, 94 and 95.

### Checks

#### Fourth revision (rulings 85 to 101)

- **What changed.** The pull request adds `ledger/rulings/order-from-the-acts-rulings-2026-10-09.md` and changes the PRD and this record. Nothing else: no code, test, fixture, protocol text, earlier rulings file, store, export or `.ddd/` file.
- **Gates**, with the four git-identity variables unset, on `fac640d`'s code, which the revision does not touch:
  - `cargo build`: exit 0.
  - `cargo clippy -- -D warnings -D clippy::unwrap_used`: exit 0.
  - `cargo t`: exit 0. 2,125 passed, 0 failed and 2 ignored, summed over the 96 `test result:` lines.
  - `dotnet test` was not run, since no .NET code is touched.
- **`ledger verify --export`**, built from this branch and run on it and on `main` at `fac640d`: exit 0 on both, and byte-identical output, the same as shown under the third revision.

#### The merge with `main` at `17f9656`

The fourth revision was written twice, by two sessions working from the same rulings; the first to push is the one above, and the second was merged into it, keeping the first's three files and adding only the case L run and this paragraph. The merge also brings `main` at `17f9656` (the merge of #141, which changes tests and one protocol note) into the branch.

- **Gates**, with the four git-identity variables unset, on the merged branch's code:
  - `cargo build`: exit 0.
  - `cargo clippy -- -D warnings -D clippy::unwrap_used`: exit 0.
  - `cargo t`: exit 0. 2,126 passed, 0 failed and 2 ignored, summed over the 96 `test result:` lines.
- **`ledger verify --export`**, built from the merged branch and run on it and on a worktree of `origin/main` at `17f9656`: exit 0 on both, and byte-identical output:

```text
landing computed on HEAD's own first-parent line — no base (no `--base`, and no `origin/HEAD` in this clone)
conformant — 284 entries, 93 decision(s)
3 allocated, awaiting acceptance:
  - dec:hafeok.ddd/01KZTGGMEACBFMTC1RJJ8T90GS
  - dec:hafeok.ddd/01KZTGGX5ABSQ2PVTQ32NPKVNE
  - dec:hafeok.ledger/01KZXJX693301CZSY4XNP643XY
notice: namespace `hafeok.ddd` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ddd` opts it in)
notice: namespace `hafeok.ledger` has no policy — nothing in it is role-checked or signature-checked (`ledger init --namespace hafeok.ledger` opts it in)
export: every committed export matches the log byte for byte
exit 0
```
