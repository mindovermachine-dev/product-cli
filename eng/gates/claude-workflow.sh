#!/usr/bin/env bash
# The issue-session gate: .github/workflows/claude.yml is the bridge ADR-0014
# states, and it agrees with the routine's prompt, eng/routines/issue-session.md.
#
# The workflow fails when it:
#   - is triggered by anything but issues and issue_comment (D2);
#   - grants any permission but contents: read, or a job declares its own (D2);
#   - has more than one job, or the job's if: lacks the label check, the
#     sender check, the not-a-pull-request check, the comment-author check,
#     the typed-by-hand check (performed_via_github_app == null) or the
#     prefix check (D2);
#   - uses any action but actions/checkout pinned to a SHA with its version;
#   - names a secret but ROUTINE_FIRE_URL and ROUTINE_FIRE_TOKEN, lacks
#     either, or uses the job token (D3, D4);
#   - has not exactly one curl, posting to "$ROUTINE_FIRE_URL" with the bearer
#     token, anthropic-version and Content-Type, or has a curl that retries
#     (D3, D7);
#   - puts an expression inside a run: block, or issue or comment text in
#     the steps (D3);
#   - has no timeout-minutes, or one over 10, or lacks the per-issue
#     concurrency group (D7).
#
# The workflow and the prompt agree when every login in the if: is the
# prompt's Actor, the label is its Label, the prefix is its Start prefix, its
# Approval prefix starts with the Start prefix, and the prompt has the rule
# sections and names the values in them. The values are compared, not the text.
#
# Comment lines of the workflow are not read, so a check cannot be met by one.
#
# Usage: eng/gates/claude-workflow.sh [workflow] [prompt]
#   (defaults: .github/workflows/claude.yml eng/routines/issue-session.md)
set -euo pipefail

file=${1:-.github/workflows/claude.yml}
prompt=${2:-eng/routines/issue-session.md}
for f in "$file" "$prompt"; do
  if [ ! -f "$f" ]; then
    echo "claude-workflow: no file $f" >&2
    exit 1
  fi
done

failed=0
report() {
  echo "claude-workflow: $1"
  failed=1
}

# The workflow without its comment lines, each line kept with its number.
body=$(awk '$0 !~ /^[[:space:]]*#/ { printf "%d\t%s\n", NR, $0 }' "$file")
lines() { cut -f2- <<<"$body"; }
has() { lines | grep -qF -- "$1"; }
numbered() { grep -E "$1" <<<"$body" || true; }

# --- triggers and permissions ------------------------------------------------

while IFS=$'\t' read -r n line; do
  [ -n "$n" ] || continue
  report "line $n: a trigger beyond issues and issue_comment: ${line#"${line%%[![:space:]]*}"}"
done < <(numbered $'^[0-9]+\t  (pull_request|pull_request_target|pull_request_review|pull_request_review_comment|workflow_run|workflow_dispatch|workflow_call|repository_dispatch|schedule|push|discussion|discussion_comment):')

top=$(lines | awk '/^permissions:/ {in_p=1; next} in_p && /^[^[:space:]]/ {in_p=0} in_p && NF {gsub(/^[[:space:]]+|[[:space:]]+$/, ""); print}')
if [ "$top" != "contents: read" ]; then
  report "the top-level permissions are not exactly contents: read"
fi
if lines | grep -qE '^[[:space:]]+permissions:'; then
  report "a job declares its own permissions"
fi

# --- the job and its if: -----------------------------------------------------

jobs=$(lines | grep -cE '^    runs-on:' || true)
if [ "$jobs" -ne 1 ]; then
  report "$jobs jobs, not one"
fi
cond=$(lines | awk '/^    if:/ {in_if=1; print; next} in_if && /^      / {print; next} {in_if=0}')
if [ -z "$cond" ]; then
  report "the job has no if:"
fi
for check in \
  "github.event.label.name == '" \
  "github.event.sender.login == '" \
  "!github.event.issue.pull_request" \
  "github.event.comment.user.login == '" \
  "github.event.comment.performed_via_github_app == null" \
  "startsWith(github.event.comment.body, '"; do
  if ! grep -qF -- "$check" <<<"$cond"; then
    report "the job's if: lacks the check on ${check%% ==*}"
  fi
done

# --- the steps ---------------------------------------------------------------

steps=$(lines | awk '/^    steps:/ {in_s=1; next} in_s && /^    [^[:space:]]/ {in_s=0} in_s')

while IFS=$'\t' read -r n line; do
  [ -n "$n" ] || continue
  if [[ ! "$line" =~ ^[[:space:]]*(-[[:space:]]+)?uses:[[:space:]]+actions/checkout@[0-9a-f]{40}[[:space:]]+#[[:space:]]+v[0-9][^[:space:]]*[[:space:]]*$ ]]; then
    report "line $n: a uses: other than actions/checkout pinned to a SHA with its version: ${line#"${line%%[![:space:]]*}"}"
  fi
done < <(numbered $'^[0-9]+\t[[:space:]]*(-[[:space:]]+)?uses:')

secrets=$(lines | grep -oE 'secrets\.[A-Za-z0-9_]+' | sort -u || true)
while IFS= read -r s; do
  [ -n "$s" ] || continue
  case "$s" in
    secrets.ROUTINE_FIRE_URL | secrets.ROUTINE_FIRE_TOKEN) ;;
    *) report "a secret beyond ROUTINE_FIRE_URL and ROUTINE_FIRE_TOKEN: $s" ;;
  esac
done <<<"$secrets"
for s in ROUTINE_FIRE_URL ROUTINE_FIRE_TOKEN; do
  if ! has "$s: \${{ secrets.$s }}"; then
    report "no '$s: \${{ secrets.$s }}'"
  fi
done
if lines | grep -qE 'github\.token|GITHUB_TOKEN'; then
  report "the job token is used"
fi

curls=$(grep -cE '(^|[[:space:]])curl[[:space:]]' <<<"$steps" || true)
if [ "$curls" -ne 1 ]; then
  report "$curls curl commands, not exactly one"
fi
for required in \
  '-X POST "$ROUTINE_FIRE_URL"' \
  '-H "Authorization: Bearer $ROUTINE_FIRE_TOKEN"' \
  '-H "anthropic-version: ' \
  '-H "Content-Type: application/json"'; do
  if ! grep -qF -- "$required" <<<"$steps"; then
    report "the curl lacks ${required%' '}"
  fi
done
if grep -qE -- '--retry' <<<"$steps"; then
  report "the curl retries, and the fire API has no idempotency"
fi

# No expression inside a run: block: values reach the script through env:.
run_text=$(lines | awk '
  /^[[:space:]]*(- )?run:/ { in_r=1; match($0, /^[[:space:]]*/); ind=RLENGTH; print; next }
  in_r { match($0, /^[[:space:]]*/); if (NF && RLENGTH <= ind) in_r=0; else { print; next } }')
if grep -qF -- '${{' <<<"$run_text"; then
  report "an expression inside a run: block; pass it through env:"
fi
if grep -qE 'github\.event\.(comment|issue)\.(body|title)' <<<"$steps"; then
  report "issue or comment text in the steps; the payload names the start only"
fi

# --- limits ------------------------------------------------------------------

timeout=$(lines | sed -nE 's/^    timeout-minutes: ([0-9]+)$/\1/p')
if [ -z "$timeout" ]; then
  report "the job has no timeout-minutes"
elif [ "$timeout" -gt 10 ]; then
  report "timeout-minutes is $timeout, over 10"
fi
for required in \
  'group: claude-issue-${{ github.event.issue.number }}' \
  'cancel-in-progress: false'; do
  if ! has "$required"; then
    report "no '$required'"
  fi
done

# --- agreement with the prompt -----------------------------------------------

contract() {  # contract <row>: the value of a row of the prompt's contract table
  sed -nE "s/^\| $1 \| \`?([^\`|]*)\`? \|$/\1/p" "$prompt" | head -n 1
}
actor=$(contract Actor)
label=$(contract Label)
start=$(contract 'Start prefix')
approval=$(contract 'Approval prefix')
heading=$(contract 'Plan heading')
for v in actor label start approval heading; do
  if [ -z "${!v}" ]; then
    report "$prompt: no contract value for $v"
  fi
done

while IFS= read -r login; do
  [ -n "$login" ] || continue
  if [ "$login" != "$actor" ]; then
    report "the if: checks the login '$login', and the prompt's Actor is '$actor'"
  fi
done < <(grep -oE "login == '[^']*'" <<<"$cond" | sed -E "s/login == '(.*)'/\1/")
if ! grep -qF -- "github.event.label.name == '$label'" <<<"$cond"; then
  report "the if: does not check the prompt's Label '$label'"
fi
if ! grep -qF -- "startsWith(github.event.comment.body, '$start')" <<<"$cond"; then
  report "the if: does not check the prompt's Start prefix '$start'"
fi
if [ -n "$start" ] && [[ "$approval" != "$start"* ]]; then
  report "the prompt's Approval prefix '$approval' does not start with its Start prefix '$start'"
fi

# The rule sections, and the values named in them outside the table.
section() {  # section <title>: the prompt's section, without its heading
  awk -v h="## $1" '$0 == h {in_s=1; next} in_s && /^## / {in_s=0} in_s' "$prompt"
}
for title in \
  'Check the start before anything else' \
  'Who you take instructions from' \
  'Plan first' \
  'One session per start' \
  'Every run'; do
  if ! grep -qxF -- "## $title" "$prompt"; then
    report "$prompt: no section '$title'"
  fi
done
check_start=$(section 'Check the start before anything else')
if ! grep -qF -- "\`$actor\`" <<<"$check_start" || ! grep -qF -- 'performed_via_github_app' <<<"$check_start"; then
  report "$prompt: 'Check the start' does not check the Actor and performed_via_github_app"
fi
if ! grep -qF -- "\`$start\`" <<<"$check_start"; then
  report "$prompt: 'Check the start' does not check the Start prefix"
fi
if ! grep -qF -- "\`$approval\`" <<<"$(section 'Plan first')"; then
  report "$prompt: 'Plan first' does not name the Approval prefix"
fi
if ! grep -qF -- "\`$heading\`" <<<"$(section 'Plan first')"; then
  report "$prompt: 'Plan first' does not name the Plan heading"
fi
if ! grep -qF -- 'performed_via_github_app' <<<"$(section 'Who you take instructions from')"; then
  report "$prompt: 'Who you take instructions from' does not name performed_via_github_app"
fi

if [ "$failed" -ne 0 ]; then
  exit 1
fi
echo "claude-workflow: $file is the bridge ADR-0014 states, and agrees with $prompt"
