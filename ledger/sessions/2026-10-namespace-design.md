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

## Questions for the principal

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

## Checks

- **What changed.** Two files were added: the PRD and this record. Nothing else in the repository changed. The experiment's repositories, scripts, keys and outputs live in the scratchpad and are not committed.
- **Gates**, with the four git-identity variables unset, as `CLAUDE.md` requires before any commit: 

  - `cargo build`: exit 0.
  - `cargo clippy -- -D warnings -D clippy::unwrap_used`: exit 0.
  - `cargo t`: exit 0. 2,083 passed, 0 failed and 2 ignored, summed over the 90 `test result:` lines.
  - `dotnet test` was not run, since no .NET code is touched.
- **`ledger verify --export`** on this branch and on `main` at `e20fadc` (the unshallowed clone): exit 0 on both, and byte-identical output after the first line. The output:

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
