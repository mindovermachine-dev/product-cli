#!/usr/bin/env bash
# The signature gate: every commit of a pull request is verified by GitHub (ADR-0007 D1).
#
# Reads the pull request's commits as the REST API returns them
# (GET /repos/{owner}/{repo}/pulls/{n}/commits, all pages, as one JSON
# array) and fails on any commit whose verification is not "verified".
#
# Usage: eng/gates/signatures.sh <commits.json>
set -euo pipefail

if [ $# -ne 1 ]; then
  echo "usage: $0 <commits.json>" >&2
  exit 2
fi

count=$(jq 'length' "$1")
if [ "$count" -eq 0 ]; then
  echo "signatures: no commits to check" >&2
  exit 1
fi

failed=0
while IFS=$'\t' read -r sha verified reason; do
  if [ "$verified" = "true" ]; then
    echo "signatures: ${sha:0:12} verified ($reason)"
  else
    echo "signatures: ${sha:0:12} NOT verified ($reason)"
    failed=1
  fi
done < <(jq -r '.[] | [.sha, (.commit.verification.verified | tostring), (.commit.verification.reason // "none")] | @tsv' "$1")

if [ "$failed" -ne 0 ]; then
  exit 1
fi
echo "signatures: $count commit(s) verified"
