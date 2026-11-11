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

# A root with the .NET pin and a project tree, which every plant starts from.
toolchain_root() {  # toolchain_root <name>
  local r="$work/$1"
  rm -rf "$r"
  mkdir -p "$r/src/ProductCli.Probe" "$r/tests"
  cp "$repo/global.json" "$r/global.json"
  : > "$r/src/ProductCli.Probe/ProductCli.Probe.csproj"
  echo "$r"
}

r=$(toolchain_root root-clean)
expect_pass "no-toolchain" "$gates/no-toolchain.sh" "$r"

# bin/, obj/ and .git/ are not the tree: a file in them is not a toolchain.
r=$(toolchain_root root-ignored)
mkdir -p "$r/src/ProductCli.Probe/obj" "$r/.git"
: > "$r/src/ProductCli.Probe/obj/package.json"
: > "$r/.git/global.json"
expect_pass "no-toolchain: bin, obj and .git are not the tree" "$gates/no-toolchain.sh" "$r"

for f in Cargo.toml rust-toolchain.toml package.json pnpm-lock.yaml pyproject.toml requirements.txt .python-version; do
  r=$(toolchain_root "root-$f")
  : > "$r/$f"
  expect_fail "no-toolchain: $f" "no-toolchain: $f is a non-.NET toolchain file" "$gates/no-toolchain.sh" "$r"
done
r=$(toolchain_root root-cargo-dir)
mkdir -p "$r/.cargo"
expect_fail "no-toolchain: .cargo/" "no-toolchain: .cargo is a non-.NET toolchain file" "$gates/no-toolchain.sh" "$r"

r=$(toolchain_root root-nested-node)
: > "$r/src/ProductCli.Probe/package.json"
expect_fail "no-toolchain: package.json under src/" "no-toolchain: src/ProductCli.Probe/package.json is a non-.NET toolchain file" "$gates/no-toolchain.sh" "$r"

r=$(toolchain_root root-no-pin)
rm "$r/global.json"
expect_fail "no-toolchain: no global.json" "no-toolchain: global.json is not at the root" "$gates/no-toolchain.sh" "$r"

r=$(toolchain_root root-wrong-sdk)
jq '.sdk.version = "9.0.100"' "$repo/global.json" > "$r/global.json"
expect_fail "no-toolchain: another SDK version" "global.json .sdk.version is 9.0.100, not 10.0.401" "$gates/no-toolchain.sh" "$r"

r=$(toolchain_root root-wrong-rollforward)
jq '.sdk.rollForward = "latestMajor"' "$repo/global.json" > "$r/global.json"
expect_fail "no-toolchain: another rollForward" "global.json .sdk.rollForward is latestMajor, not latestFeature" "$gates/no-toolchain.sh" "$r"

r=$(toolchain_root root-no-runner)
jq 'del(.test)' "$repo/global.json" > "$r/global.json"
expect_fail "no-toolchain: no test runner" "global.json .test.runner is (absent), not Microsoft.Testing.Platform" "$gates/no-toolchain.sh" "$r"

r=$(toolchain_root root-second-pin)
cp "$repo/global.json" "$r/tests/global.json"
expect_fail "no-toolchain: a second global.json" "no-toolchain: tests/global.json is a second global.json" "$gates/no-toolchain.sh" "$r"

# --- smoke -------------------------------------------------------------------

# Each stub stands in for a published binary. The clean one behaves as the
# host must; each plant breaks one thing the smoke test reads.
sha=0123456789abcdef0123456789abcdef01234567
stub() {  # stub <name> <body>: an executable script
  local f="$work/stub-$1"
  printf '#!/usr/bin/env bash\n%s\n' "$2" > "$f"
  chmod +x "$f"
  echo "$f"
}
good_version="printf 'product 1.2.3 (commit $sha)\\n'"
good_domain="echo \"product: 'domain' arrives in step 4\" >&2; exit 2"
body() {  # body <version branch> <domain branch>
  printf 'if [ "$1" = --version ]; then %s; else %s; fi' "$1" "$2"
}

expect_pass "smoke" "$gates/smoke.sh" "$(stub clean "$(body "$good_version" "$good_domain")")" 1.2.3 "$sha"
expect_fail "smoke: another version" "smoke: --version wrote other bytes than expected" \
  "$gates/smoke.sh" "$(stub version "$(body "printf 'product 1.2.4 (commit $sha)\\n'" "$good_domain")")" 1.2.3 "$sha"
expect_fail "smoke: another commit" "smoke: --version wrote other bytes than expected" \
  "$gates/smoke.sh" "$(stub commit "$(body "printf 'product 1.2.3 (commit ffff)\\n'" "$good_domain")")" 1.2.3 "$sha"
expect_fail "smoke: a CRLF line end" "smoke: --version wrote other bytes than expected" \
  "$gates/smoke.sh" "$(stub crlf "$(body "printf 'product 1.2.3 (commit $sha)\\r\\n'" "$good_domain")")" 1.2.3 "$sha"
expect_fail "smoke: --version exits 1" "smoke: --version exited 1, not 0" \
  "$gates/smoke.sh" "$(stub exit1 "$(body "$good_version; exit 1" "$good_domain")")" 1.2.3 "$sha"
expect_fail "smoke: --version writes to stderr" "smoke: --version wrote to stderr" \
  "$gates/smoke.sh" "$(stub stderr "$(body "$good_version; echo warning >&2" "$good_domain")")" 1.2.3 "$sha"
expect_fail "smoke: a verb exits 0" "smoke: 'domain' exited 0, not 2" \
  "$gates/smoke.sh" "$(stub verb0 "$(body "$good_version" "exit 0")")" 1.2.3 "$sha"
expect_fail "smoke: a verb writes to stdout" "smoke: 'domain' wrote to stdout" \
  "$gates/smoke.sh" "$(stub verbout "$(body "$good_version" "echo nodes; exit 2")")" 1.2.3 "$sha"

# --- claude-workflow ---------------------------------------------------------

# Each plant is a copy of the workflow or of the prompt with one thing changed;
# the other file is the real one.
wf="$repo/.github/workflows/claude.yml"
rp="$repo/eng/routines/issue-session.md"
plant_copy() {  # plant_copy <source> <name> <sed script>: a planted copy
  local f="$work/claude-$2.${1##*.}"
  sed -E "$3" "$1" > "$f"
  if cmp -s "$1" "$f"; then
    echo "selftest: plant  claude-workflow: $2: the plant changed nothing" >&2
    failed=1
  fi
  echo "$f"
}
gate_wf() {  # gate_wf <label> <finding> <name> <sed script>: a planted workflow
  expect_fail "claude-workflow: $1" "$2" "$gates/claude-workflow.sh" "$(plant_copy "$wf" "$3" "$4")" "$rp"
}
gate_rp() {  # gate_rp <label> <finding> <name> <sed script>: a planted prompt
  expect_fail "claude-workflow: $1" "$2" "$gates/claude-workflow.sh" "$wf" "$(plant_copy "$rp" "$3" "$4")"
}

expect_pass "claude-workflow" "$gates/claude-workflow.sh" "$wf" "$rp"

gate_wf "the old action" "a uses: other than actions/checkout pinned" old-action \
  's|^    steps:$|    steps:\n      - uses: anthropics/claude-code-action@c0949c50a78c1ec8ce2647db36629b870e715d0c # v1.0.268|'
gate_wf "a third secret" "a secret beyond ROUTINE_FIRE_URL and ROUTINE_FIRE_TOKEN: secrets.CLAUDE_CODE_OAUTH_TOKEN" third-secret \
  's|^(          ROUTINE_FIRE_TOKEN: .*)$|\1\n          OAUTH: ${{ secrets.CLAUDE_CODE_OAUTH_TOKEN }}|'
gate_wf "checkout pinned to a tag" "a uses: other than actions/checkout pinned" checkout-tag \
  's|^    steps:$|    steps:\n      - uses: actions/checkout@v7|'
gate_wf "a second curl" "2 curl commands, not exactly one" second-curl \
  's|^(          echo "claude: .*)$|\1\n          curl -X POST "$ROUTINE_FIRE_URL"|'
gate_wf "curl to a hard-coded URL" 'the curl lacks -X POST "$ROUTINE_FIRE_URL"' hard-url \
  's|-X POST "\$ROUTINE_FIRE_URL"|-X POST https://api.anthropic.com/v1/claude_code/routines/trig_x/fire|'
gate_wf "a curl that retries" "the curl retries" retry \
  's|--max-time 30|--max-time 30 --retry 3|'
gate_wf "contents: write" "the top-level permissions are not exactly contents: read" write \
  's|^  contents: read$|  contents: write|'
gate_wf "no actor check on the label" "lacks the check on github.event.sender.login" no-sender \
  '/github.event.sender.login == .Hafeok./d; s|&& github.event.label.name == .claude.$|&)|'
gate_wf "no actor check on the comment" "lacks the check on github.event.comment.user.login" no-commenter \
  '/github.event.comment.user.login == .Hafeok./d'
gate_wf "an actor check only in a comment" "lacks the check on github.event.sender.login" comment-only \
  's|^( +)&& github.event.sender.login == .Hafeok.\)$|\1)\n      # \&\& github.event.sender.login == '"'"'Hafeok'"'"'|'
gate_wf "a comment made through an app admitted" "lacks the check on github.event.comment.performed_via_github_app" app-comment \
  '/performed_via_github_app == null/d'
gate_wf "an expression inside run:" "an expression inside a run: block" run-expression \
  's|"issue: \$ISSUE"|"issue: ${{ github.event.issue.number }}"|'
gate_wf "comment text in the payload" "issue or comment text in the steps" comment-body \
  's|^(          COMMENT_AUTHOR: .*)$|\1\n          COMMENT_BODY: ${{ github.event.comment.body }}|'
gate_wf "a pull_request_target trigger" "a trigger beyond issues and issue_comment" pr-target \
  's|^  issue_comment:$|  pull_request_target:\n  issue_comment:|'
gate_wf "no timeout" "the job has no timeout-minutes" timeout \
  '/timeout-minutes/d'
gate_rp "the prompt's Actor is another login" "and the prompt's Actor is 'someone-else'" actor \
  's/^\| Actor \| `Hafeok` \|$/| Actor | `someone-else` |/'
gate_rp "the prompt without Plan first" "no section 'Plan first'" no-plan-first \
  's/^## Plan first$/## Planning/'
gate_rp "the prompt's start check without the app check" "'Check the start' does not check the Actor and performed_via_github_app" no-app-check \
  '/^## Check the start/,/^## /s/ and no `performed_via_github_app`|, it has no `performed_via_github_app`//'

if [ "$failed" -ne 0 ]; then
  echo "selftest: a check did not behave as its plant requires"
  exit 1
fi
echo "selftest: every check passes its clean case and fails every plant"
