#!/usr/bin/env bash
# The toolchain gate: the only toolchain is .NET, pinned at the root (chapter 7; ADR-0013, #167).
#
# Fails when:
#   1. a Rust, Node or Python toolchain file, or a requirements*.txt, is
#      anywhere in the tree (outside .git/, bin/, obj/ and artifacts/);
#   2. the root has no global.json, or its sdk.version is not 10.0.401, its
#      sdk.rollForward is not latestFeature, or its test.runner is not
#      Microsoft.Testing.Platform;
#   3. a global.json exists below the root, a second pin.
#
# Usage: eng/gates/no-toolchain.sh [root]   (default: .)
set -euo pipefail

root=${1:-.}
sdk_version=10.0.401
roll_forward=latestFeature
test_runner=Microsoft.Testing.Platform

rust=(Cargo.toml Cargo.lock rust-toolchain rust-toolchain.toml .cargo)
node=(package.json package-lock.json npm-shrinkwrap.json yarn.lock .yarnrc.yml
      pnpm-lock.yaml pnpm-workspace.yaml bun.lockb bun.lock .nvmrc .node-version
      deno.json deno.jsonc)
python=(pyproject.toml setup.py setup.cfg Pipfile Pipfile.lock poetry.lock uv.lock
        .python-version tox.ini)
shared=(.tool-versions)

failed=0

# 1. No other toolchain, anywhere.
names=()
for f in "${rust[@]}" "${node[@]}" "${python[@]}" "${shared[@]}"; do
  names+=(-o -name "$f")
done
while IFS= read -r -d '' path; do
  echo "no-toolchain: ${path#"$root"/} is a non-.NET toolchain file"
  failed=1
done < <(find "$root" \( -name .git -o -name bin -o -name obj -o -name artifacts \) -prune -o \
              \( -name 'requirements*.txt' "${names[@]}" \) -print0 | sort -z)

# 2. The .NET pin, at the root.
pin="$root/global.json"
if [ ! -f "$pin" ]; then
  echo "no-toolchain: global.json is not at the root"
  failed=1
else
  check() {  # check <jq path> <expected>
    local actual
    actual=$(jq -r "$1 // \"(absent)\"" "$pin" 2>/dev/null || echo "(unreadable)")
    if [ "$actual" != "$2" ]; then
      echo "no-toolchain: global.json $1 is $actual, not $2"
      failed=1
    fi
  }
  check .sdk.version "$sdk_version"
  check .sdk.rollForward "$roll_forward"
  check .test.runner "$test_runner"
fi

# 3. One pin only.
while IFS= read -r -d '' path; do
  echo "no-toolchain: ${path#"$root"/} is a second global.json"
  failed=1
done < <(find "$root" \( -name .git -o -name bin -o -name obj -o -name artifacts \) -prune -o \
              -name global.json ! -path "$root/global.json" -print0 | sort -z)

if [ "$failed" -ne 0 ]; then
  exit 1
fi
echo "no-toolchain: the only toolchain is .NET, SDK $sdk_version pinned in global.json"
