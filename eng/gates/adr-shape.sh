#!/usr/bin/env bash
# The ADR shape gate: every file under docs/decisions/ is an ADR of the required shape (ADR-0001 D1).
#
# An ADR is ADR-NNNN-<slug>.md with:
#   - an H1 "# ADR-NNNN: <title>" whose number is the file's;
#   - a "**Status:** <state>" line;
#   - "## Context", "## Decision", "## Alternatives considered" and
#     "## Consequences".
# No two ADRs share a number. The only other files allowed are README.md and
# the collected source, collected-YYYY-MM-DD.md; anything else fails, so no
# file escapes the check by its name.
#
# Usage: eng/gates/adr-shape.sh [dir]   (default: docs/decisions)
set -euo pipefail

dir=${1:-docs/decisions}
if [ ! -d "$dir" ]; then
  echo "adr-shape: no directory $dir" >&2
  exit 1
fi

failed=0
report() {
  echo "adr-shape: $1: $2"
  failed=1
}

declare -A seen=()
adrs=0
while IFS= read -r -d '' path; do
  name=${path#"$dir"/}
  if [ ! -f "$path" ]; then
    report "$name" "not a regular file"
    continue
  fi
  case "$name" in
    README.md) continue ;;
  esac
  if [[ "$name" =~ ^collected-[0-9]{4}-[0-9]{2}-[0-9]{2}\.md$ ]]; then
    continue
  fi
  if [[ ! "$name" =~ ^ADR-([0-9]{4})-[a-z0-9]+(-[a-z0-9]+)*\.md$ ]]; then
    report "$name" "not an ADR file name (ADR-NNNN-<slug>.md)"
    continue
  fi
  n=${BASH_REMATCH[1]}
  adrs=$((adrs + 1))
  if [ -n "${seen[$n]:-}" ]; then
    report "$name" "number $n is also ${seen[$n]}"
  fi
  seen[$n]=$name

  if ! head -n 1 "$path" | grep -qE "^# ADR-$n: .+"; then
    report "$name" "first line is not '# ADR-$n: <title>'"
  fi
  if ! grep -qE '^\*\*Status:\*\* (Proposed|Accepted|Superseded|Withdrawn|Deprecated)' "$path"; then
    report "$name" "no '**Status:**' line with a status"
  fi
  for section in 'Context' 'Decision' 'Alternatives considered' 'Consequences'; do
    if ! grep -qxF "## $section" "$path"; then
      report "$name" "no '## $section' section"
    fi
  done
done < <(find "$dir" -mindepth 1 -maxdepth 1 -print0 | sort -z)

if [ "$adrs" -eq 0 ]; then
  report "$dir" "holds no ADR"
fi
if [ "$failed" -ne 0 ]; then
  exit 1
fi
echo "adr-shape: $adrs ADR(s) in $dir have the shape"
