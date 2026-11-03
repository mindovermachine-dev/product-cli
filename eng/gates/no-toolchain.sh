#!/usr/bin/env bash
# The toolchain gate: no Rust, Node or Python toolchain file exists at the repository root (chapter 7).
#
# Usage: eng/gates/no-toolchain.sh [root]   (default: .)
set -euo pipefail

root=${1:-.}

rust=(Cargo.toml Cargo.lock rust-toolchain rust-toolchain.toml .cargo)
node=(package.json package-lock.json npm-shrinkwrap.json yarn.lock .yarnrc.yml
      pnpm-lock.yaml pnpm-workspace.yaml bun.lockb bun.lock .nvmrc .node-version
      deno.json deno.jsonc)
python=(pyproject.toml setup.py setup.cfg Pipfile Pipfile.lock poetry.lock uv.lock
        .python-version tox.ini)
shared=(.tool-versions)

failed=0
for f in "${rust[@]}" "${node[@]}" "${python[@]}" "${shared[@]}"; do
  if [ -e "$root/$f" ]; then
    echo "no-toolchain: $f is at the root"
    failed=1
  fi
done
shopt -s nullglob
for f in "$root"/requirements*.txt; do
  echo "no-toolchain: ${f#"$root"/} is at the root"
  failed=1
done
shopt -u nullglob

if [ "$failed" -ne 0 ]; then
  exit 1
fi
echo "no-toolchain: no Rust, Node or Python toolchain file at the root"
