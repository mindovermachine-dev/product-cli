#!/usr/bin/env bash
# The issue-session workflow gate: .github/workflows/claude.yml is as ADR-0014 states it.
#
# It fails when the file:
#   - uses an action not pinned to a 40-hex commit SHA with its version in a
#     comment (D1);
#   - grants any permission but contents, issues and pull-requests: write, at
#     the top, or declares permissions on a job, or names a workflows or
#     id-token permission anywhere (D6);
#   - has more than one job, or a job whose if: lacks the label check, the
#     actor checks on the label's sender and the comment's author, the check
#     that the comment is on an issue, or the '@claude' prefix (D2);
#   - is triggered by anything but issues and issue_comment (D2);
#   - sets an input ADR-0014 refuses: allowed_non_write_users, allowed_bots,
#     use_commit_signing, additional_permissions, prompt, or a --model (D2,
#     D3, D5, D7);
#   - lacks the signing key, the token, the attribution setting, the timeout
#     or the per-issue concurrency group (D3, D4, D6, D7).
#
# Comment lines are not read, so a check cannot be met by a comment.
#
# Usage: eng/gates/claude-workflow.sh [file]   (default: .github/workflows/claude.yml)
set -euo pipefail

file=${1:-.github/workflows/claude.yml}
if [ ! -f "$file" ]; then
  echo "claude-workflow: no file $file" >&2
  exit 1
fi

failed=0
report() {
  echo "claude-workflow: $1"
  failed=1
}

# The file without its comment lines, each line kept with its number.
body=$(awk '$0 !~ /^[[:space:]]*#/ { printf "%d\t%s\n", NR, $0 }' "$file")
lines() { cut -f2- <<<"$body"; }
has() { lines | grep -qF -- "$1"; }

# D1: every action pinned by SHA, its version in a comment.
while IFS=$'\t' read -r n line; do
  if [[ ! "$line" =~ ^[[:space:]]*(-[[:space:]]+)?uses:[[:space:]]+[A-Za-z0-9_.-]+/[A-Za-z0-9_./-]+@[0-9a-f]{40}[[:space:]]+#[[:space:]]+v[0-9][^[:space:]]*[[:space:]]*$ ]]; then
    report "line $n: an action not pinned to a commit SHA with its version: ${line#"${line%%[![:space:]]*}"}"
  fi
done < <(grep -E $'^[0-9]+\t[[:space:]]*(-[[:space:]]+)?uses:' <<<"$body" || true)

# D6: the permissions, exactly three, at the top only.
while IFS=$'\t' read -r n line; do
  key=$(sed -E 's/^[[:space:]]*([a-z-]+):.*/\1/' <<<"$line")
  report "line $n: the $key permission is refused by ADR-0014 D6"
done < <(grep -E $'^[0-9]+\t[[:space:]]*(workflows|id-token):' <<<"$body" || true)
top=$(lines | awk '/^permissions:/ {in_p=1; next} in_p && /^[^[:space:]]/ {in_p=0} in_p && NF {gsub(/^[[:space:]]+|[[:space:]]+$/, ""); print}' | sort)
want=$(printf '%s\n' 'contents: write' 'issues: write' 'pull-requests: write')
if [ "$top" != "$want" ]; then
  report "the top-level permissions are not exactly contents, issues and pull-requests: write"
fi
if lines | grep -qE '^[[:space:]]+permissions:'; then
  report "a job declares its own permissions"
fi

# D2: the triggers.
while IFS=$'\t' read -r n line; do
  report "line $n: a trigger beyond issues and issue_comment: ${line#"${line%%[![:space:]]*}"}"
done < <(grep -E $'^[0-9]+\t  (pull_request|pull_request_target|pull_request_review|pull_request_review_comment|workflow_run|workflow_dispatch|workflow_call|repository_dispatch|schedule|push|discussion|discussion_comment):' <<<"$body" || true)

# D2: one job, and its if: holds every check.
jobs=$(lines | grep -cE '^    runs-on:' || true)
if [ "$jobs" -ne 1 ]; then
  report "$jobs jobs, not one"
fi
cond=$(lines | awk '/^    if:/ {in_if=1; print; next} in_if && /^      / {print; next} {in_if=0}')
if [ -z "$cond" ]; then
  report "the job has no if:"
fi
for check in \
  "github.event.label.name == 'claude'" \
  "github.event.sender.login == 'Hafeok'" \
  "!github.event.issue.pull_request" \
  "github.event.comment.user.login == 'Hafeok'" \
  "startsWith(github.event.comment.body, '@claude')"; do
  if ! grep -qF -- "$check" <<<"$cond"; then
    report "the job's if: lacks the check $check"
  fi
done

# D2, D3, D5, D7: inputs ADR-0014 refuses.
for refused in allowed_non_write_users allowed_bots use_commit_signing additional_permissions; do
  while IFS=$'\t' read -r n _; do
    report "line $n: $refused is refused by ADR-0014"
  done < <(grep -F -- "$refused" <<<"$body" || true)
done
while IFS=$'\t' read -r n _; do
  report "line $n: a prompt input is refused by ADR-0014 D5 (it leaves tag mode)"
done < <(grep -E $'^[0-9]+\t[[:space:]]+prompt:' <<<"$body" || true)
while IFS=$'\t' read -r n _; do
  report "line $n: --model is refused by ADR-0014 D7"
done < <(grep -E -- '--model([[:space:]=]|$)' <<<"$body" || true)

# D3, D4, D6, D7: what must be there.
for required in \
  'ssh_signing_key: ${{ secrets.AGENT_SSH_SIGNING_KEY }}' \
  'claude_code_oauth_token: ${{ secrets.CLAUDE_CODE_OAUTH_TOKEN }}' \
  'github_token: ${{ github.token }}' \
  '"attribution": { "commit": "", "pr": "" }' \
  'group: claude-issue-${{ github.event.issue.number }}' \
  'cancel-in-progress: false'; do
  if ! has "$required"; then
    report "no '$required'"
  fi
done
if ! lines | grep -qE '^    timeout-minutes: [0-9]+$'; then
  report "the job has no timeout-minutes"
fi

if [ "$failed" -ne 0 ]; then
  exit 1
fi
echo "claude-workflow: $file is as ADR-0014 states it"
