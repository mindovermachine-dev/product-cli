#!/usr/bin/env bash
# The smoke test of a published binary (ADR-0003 D3): what it writes, byte for byte, and how it exits.
#
#   <binary> --version   writes exactly "product <version> (commit <commit>)\n"
#                        to stdout, nothing to stderr, and exits 0;
#   <binary> domain      writes nothing to stdout and exits 2.
#
# The expected line is built here, from the arguments, and compared with cmp,
# so a CRLF, a trailing space or a missing line feed fails.
#
# Usage: eng/gates/smoke.sh <binary> <version> <commit>
set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "usage: eng/gates/smoke.sh <binary> <version> <commit>" >&2
  exit 1
fi
binary=$1 version=$2 commit=$3

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

failed=0
fail() {
  echo "smoke: $1"
  failed=1
}

printf 'product %s (commit %s)\n' "$version" "$commit" > "$work/expected"

status=0
"$binary" --version > "$work/stdout" 2> "$work/stderr" || status=$?
if [ "$status" -ne 0 ]; then
  fail "--version exited $status, not 0"
fi
if ! cmp -s "$work/expected" "$work/stdout"; then
  fail "--version wrote other bytes than expected:"
  echo "  expected: $(od -An -c "$work/expected" | tr -s ' ' | tr -d '\n')"
  echo "  actual:   $(od -An -c "$work/stdout" | tr -s ' ' | tr -d '\n')"
fi
if [ -s "$work/stderr" ]; then
  fail "--version wrote to stderr: $(head -c 200 "$work/stderr")"
fi

status=0
"$binary" domain > "$work/stdout" 2> "$work/stderr" || status=$?
if [ "$status" -ne 2 ]; then
  fail "'domain' exited $status, not 2"
fi
if [ -s "$work/stdout" ]; then
  fail "'domain' wrote to stdout: $(head -c 200 "$work/stdout")"
fi

if [ "$failed" -ne 0 ]; then
  exit 1
fi
echo "smoke: $binary prints 'product $version (commit $commit)' and exits 0; 'domain' exits 2"
