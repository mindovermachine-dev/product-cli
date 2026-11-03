#!/usr/bin/env bash
# The gates' own test: each check passes its clean case and fails every case planted for it.
#
# Every plant is built in a scratch directory on the machine running this,
# and nothing is committed to, or pushed from, this repository. The
# model-identifier plants are assembled at run time from fragments, so no
# model identifier is written into this repository, even here.
#
# Usage: eng/gates/selftest.sh   (run from the repository root)
set -euo pipefail

gates=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$gates/../.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

failed=0
expect_pass() {
  local label=$1; shift
  if out=$("$@" 2>&1); then
    echo "selftest: clean  $label: passes"
  else
    echo "selftest: clean  $label: FAILED, and it should pass:"
    sed 's/^/    /' <<<"$out"
    failed=1
  fi
}
# expect_fail <label> <finding> <command...>: the command must fail, and its
# output must carry the finding the plant is for, so that a check failing for
# another reason (a wrong path, a crash) does not count as the plant caught.
expect_fail() {
  local label=$1 finding=$2; shift 2
  if out=$("$@" 2>&1); then
    echo "selftest: plant  $label: PASSED, and it should fail"
    failed=1
  elif ! grep -qF -- "$finding" <<<"$out"; then
    echo "selftest: plant  $label: failed without the finding '$finding':"
    sed 's/^/    /' <<<"$out"
    failed=1
  else
    echo "selftest: plant  $label: fails as it should"
    sed 's/^/    /' <<<"$out"
  fi
}
in_g() { (cd "$g" && "$@"); }

# --- commits -----------------------------------------------------------------

g="$work/git"
git init -q "$g"
# The identity is set through the environment, which overrides any the
# machine carries (a session's GIT_AUTHOR_* would otherwise win over -c).
gitc() {
  GIT_AUTHOR_NAME=${PLANT_NAME:-Planted} GIT_AUTHOR_EMAIL=planted@example.invalid \
  GIT_COMMITTER_NAME=${PLANT_NAME:-Planted} GIT_COMMITTER_EMAIL=planted@example.invalid \
    git -C "$g" -c commit.gpgsign=false "$@"
}
gitc commit -q --allow-empty -m "chore: base"
base=$(git -C "$g" rev-parse HEAD)

signoff='Signed-off-by: Emil Okkels Klein <emil@okkels-klein.dk>'
plant_commit() {  # plant_commit <message> [extra git options...]
  local msg=$1; shift
  gitc "$@" commit -q --allow-empty -m "$msg"
  git -C "$g" rev-parse HEAD
}

model="$(printf '%s-%s' gpt 9)"              # matches the gpt-<version> family
family="$(printf '%s %s' Opus 9.9)"           # a family named with a version
vendor="$(printf 'noreply@%s.com' anthropic)" # a model vendor's address

clean=$(plant_commit "$(printf 'docs: a clean commit\n\nRefs #1\n\n%s\nClaude-Session: https://claude.ai/code/session_x' "$signoff")")
expect_pass "commits" in_g "$gates/commits.sh" "$base" "$clean"

declare -a plants=(
  "no Refs line|no 'Refs #N' line|$(printf 'docs: x\n\n%s' "$signoff")"
  "no sign-off|no 'Signed-off-by:' line|$(printf 'docs: x\n\nRefs #1')"
  "malformed sign-off|malformed sign-off|$(printf 'docs: x\n\nRefs #1\n\nSigned-off-by: Emil')"
  "sign-off by an agent address|does not name a person|$(printf 'docs: x\n\nRefs #1\n\nSigned-off-by: Claude <%s>' "$vendor")"
  "model id in the message|message names a model|$(printf 'docs: written with %s\n\nRefs #1\n\n%s' "$model" "$signoff")"
  "model family in a trailer|message names a model|$(printf 'docs: x\n\nRefs #1\n\n%s\nCo-Authored-By: Claude %s <someone@example.invalid>' "$signoff" "$family")"
  "co-author at a model vendor|co-author trailer names a model vendor|$(printf 'docs: x\n\nRefs #1\n\n%s\nCo-Authored-By: Claude <%s>' "$signoff" "$vendor")"
)
for p in "${plants[@]}"; do
  label=${p%%|*}
  rest=${p#*|}
  finding=${rest%%|*}
  msg=${rest#*|}
  gitc reset -q --hard "$clean"
  sha=$(plant_commit "$msg")
  expect_fail "commits: $label" "$finding" in_g "$gates/commits.sh" "$clean" "$sha"
done

gitc reset -q --hard "$clean"
sha=$(PLANT_NAME="Bot $model" plant_commit "$(printf 'docs: x\n\nRefs #1\n\n%s' "$signoff")")
expect_fail "commits: model id in the author name" "author or committer names a model" in_g "$gates/commits.sh" "$clean" "$sha"

expect_fail "commits: an empty range" "no commits in" in_g "$gates/commits.sh" "$clean" "$clean"

# --- signatures --------------------------------------------------------------

commit_json() {  # commit_json <sha> <verified> <reason>
  printf '{"sha":"%s","commit":{"verification":{"verified":%s,"reason":"%s"}}}' "$1" "$2" "$3"
}
printf '[%s,%s]\n' "$(commit_json aaaaaaaaaaaa true valid)" "$(commit_json bbbbbbbbbbbb true valid)" > "$work/ok.json"
expect_pass "signatures" "$gates/signatures.sh" "$work/ok.json"
printf '[%s,%s]\n' "$(commit_json aaaaaaaaaaaa true valid)" "$(commit_json cccccccccccc false unsigned)" > "$work/unsigned.json"
expect_fail "signatures: an unsigned commit" "cccccccccccc NOT verified (unsigned)" "$gates/signatures.sh" "$work/unsigned.json"
printf '[%s]\n' "$(commit_json dddddddddddd false bad_email)" > "$work/bad.json"
expect_fail "signatures: a signature GitHub cannot verify" "dddddddddddd NOT verified (bad_email)" "$gates/signatures.sh" "$work/bad.json"
printf '[]\n' > "$work/empty.json"
expect_fail "signatures: no commits" "no commits to check" "$gates/signatures.sh" "$work/empty.json"

# --- adr-shape and adr-text --------------------------------------------------

fresh() {  # fresh <name>: a copy of docs/decisions to plant in
  rm -rf "$work/$1"
  cp -R "$repo/docs/decisions" "$work/$1"
  echo "$work/$1"
}
drop_line() {  # drop_line <file> <exact line>
  grep -vxF "$2" "$1" > "$1.tmp" || true
  mv "$1.tmp" "$1"
}

d=$(fresh shape-clean)
expect_pass "adr-shape" "$gates/adr-shape.sh" "$d"

d=$(fresh shape-status)
sed -i '/^\*\*Status:\*\*/d' "$d/ADR-0003-native-aot-single-binary.md"
expect_fail "adr-shape: no status" "ADR-0003-native-aot-single-binary.md: no '**Status:**'" "$gates/adr-shape.sh" "$d"

d=$(fresh shape-alternatives)
drop_line "$d/ADR-0004-package-policy.md" "## Alternatives considered"
expect_fail "adr-shape: no alternatives" "ADR-0004-package-policy.md: no '## Alternatives considered'" "$gates/adr-shape.sh" "$d"

d=$(fresh shape-consequences)
drop_line "$d/ADR-0005-node-kind-source-generator.md" "## Consequences"
expect_fail "adr-shape: no consequences" "ADR-0005-node-kind-source-generator.md: no '## Consequences'" "$gates/adr-shape.sh" "$d"

d=$(fresh shape-number)
sed -i '1s/^# ADR-0006:/# ADR-0060:/' "$d/ADR-0006-tests-observe-bytes.md"
expect_fail "adr-shape: a number that is not the file's" "first line is not '# ADR-0006: <title>'" "$gates/adr-shape.sh" "$d"

d=$(fresh shape-duplicate)
cp "$d/ADR-0007-commit-discipline.md" "$d/ADR-0007-commit-discipline-again.md"
expect_fail "adr-shape: two ADRs with one number" "number 0007 is also" "$gates/adr-shape.sh" "$d"

d=$(fresh shape-stray)
printf 'Not an ADR.\n' > "$d/notes.md"
expect_fail "adr-shape: a file that is not an ADR" "notes.md: not an ADR file name" "$gates/adr-shape.sh" "$d"

d=$(fresh text-clean)
expect_pass "adr-text" "$gates/adr-text.sh" "$d"

d=$(fresh text-byte)
sed -i '0,/strictly downward/s//strictly downwards/' "$d/ADR-0002-layers.md"
expect_fail "adr-text: one word edited" "ADR-0002-layers.md differs" "$gates/adr-text.sh" "$d"

d=$(fresh text-newline)
printf '\n' >> "$d/ADR-0010-workbench-blazor-webassembly.md"
expect_fail "adr-text: a trailing newline added" "ADR-0010-workbench-blazor-webassembly.md differs" "$gates/adr-text.sh" "$d"

d=$(fresh text-no-note)
f="$d/ADR-0001-decisions-in-embedded-writ.md"
awk '/^> \*\*Note \(2026-11-03, Draft 4\)\*\*$/ {skip=1} skip && /^$/ {skip=0; next} !skip' "$f" > "$f.tmp" && mv "$f.tmp" "$f"
expect_fail "adr-text: a note removed" "ADR-0001-decisions-in-embedded-writ.md differs" "$gates/adr-text.sh" "$d"

d=$(fresh text-extra-note)
f="$d/ADR-0002-layers.md"
awk '$0 == "## Context" {print "> **Note (2026-11-03, Draft 4)**"; print ""} {print}' "$f" > "$f.tmp" && mv "$f.tmp" "$f"
expect_fail "adr-text: a note on an ADR the preamble does not list" "ADR-0002-layers.md differs" "$gates/adr-text.sh" "$d"

d=$(fresh text-missing)
rm "$d/ADR-0012-adopting-on-the-fly.md"
expect_fail "adr-text: an ADR file missing" "ADR-0012: expected one file, found 0" "$gates/adr-text.sh" "$d"

# --- no-toolchain ------------------------------------------------------------

mkdir -p "$work/root-clean"
cp -R "$repo/docs" "$work/root-clean/" 2>/dev/null || true
expect_pass "no-toolchain" "$gates/no-toolchain.sh" "$work/root-clean"
for f in Cargo.toml rust-toolchain.toml package.json pnpm-lock.yaml pyproject.toml requirements.txt .python-version; do
  r="$work/root-$f"
  mkdir -p "$r"
  : > "$r/$f"
  expect_fail "no-toolchain: $f" "no-toolchain: $f is at the root" "$gates/no-toolchain.sh" "$r"
done
mkdir -p "$work/root-cargo-dir/.cargo"
expect_fail "no-toolchain: .cargo/" "no-toolchain: .cargo is at the root" "$gates/no-toolchain.sh" "$work/root-cargo-dir"

if [ "$failed" -ne 0 ]; then
  echo "selftest: a check did not behave as its plant requires"
  exit 1
fi
echo "selftest: every check passes its clean case and fails every plant"
