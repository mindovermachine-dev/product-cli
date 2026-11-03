#!/usr/bin/env bash
# The ADR text gate: ADR-0001 to ADR-0012 equal their sections of the collected file, byte for byte.
#
# The collected file is docs/decisions/collected-2026-11-03.md. For each
# "## ADR-NNNN:" section this rebuilds the file the ADR must be:
#   - the section, up to the "---" separator before the next one;
#   - every heading promoted one level ("## " becomes "# ", "### " becomes
#     "## "), outside code fences;
#   - for each ADR the preamble's "Rulings changed since acceptance" lists,
#     its note, inserted as a blockquote before "## Context":
#       > **Note (2026-11-03, Draft 4)**
#       >
#       > <the bullet's text, its "**ADR-NNNN.**" lead removed>
#       >
#       > - <each sub-bullet, de-indented>   (only when it has sub-bullets)
# and compares it with the committed file. Any byte of difference fails, and
# so does a missing note, a note on an ADR the preamble does not list, a
# section with no file, or a section with two.
#
# Usage: eng/gates/adr-text.sh [dir]   (default: docs/decisions)
set -euo pipefail

dir=${1:-docs/decisions}
collected="$dir/collected-2026-11-03.md"
if [ ! -f "$collected" ]; then
  echo "adr-text: no collected file $collected" >&2
  exit 1
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

awk -v out="$work" '
function flush(   i, line, fence, k) {
  if (n == "") return
  # Drop the separator between sections: trailing blanks, "---", blanks.
  while (cnt > 0 && buf[cnt] == "") cnt--
  if (cnt > 0 && buf[cnt] == "---") cnt--
  while (cnt > 0 && buf[cnt] == "") cnt--
  file = out "/" n ".md"
  fence = 0
  for (i = 1; i <= cnt; i++) {
    line = buf[i]
    if (line ~ /^```/) fence = !fence
    if (!fence && line ~ /^##+ /) line = substr(line, 2)
    if (!fence && line == "## Context" && (n in note)) {
      printf "%s\n\n", note[n] > file
    }
    printf "%s\n", line > file
  }
  close(file)
  print n > (out "/sections")
  n = ""
  cnt = 0
}
/^## Rulings changed since acceptance$/ { pre = 1; next }
/^## Index$/ { pre = 0; next }
pre && /^- \*\*ADR-[0-9][0-9][0-9][0-9]\.\*\* / {
  cur = substr($0, 9, 4)
  text = $0
  sub(/^- \*\*ADR-[0-9][0-9][0-9][0-9]\.\*\* /, "", text)
  note[cur] = "> **Note (2026-11-03, Draft 4)**\n>\n> " text
  subs[cur] = 0
  next
}
pre && /^  - / && cur != "" {
  if (!subs[cur]) { note[cur] = note[cur] "\n>"; subs[cur] = 1 }
  note[cur] = note[cur] "\n> - " substr($0, 5)
  next
}
/^## ADR-[0-9][0-9][0-9][0-9]: / { flush(); n = substr($0, 8, 4); cnt = 0 }
n != "" { buf[++cnt] = $0 }
END { flush() }
' "$collected"

if [ ! -s "$work/sections" ]; then
  echo "adr-text: no ADR sections in $collected" >&2
  exit 1
fi

failed=0
checked=0
while IFS= read -r n; do
  shopt -s nullglob
  files=("$dir"/ADR-"$n"-*.md)
  shopt -u nullglob
  if [ "${#files[@]}" -ne 1 ]; then
    echo "adr-text: ADR-$n: expected one file, found ${#files[@]}"
    failed=1
    continue
  fi
  if cmp -s "$work/$n.md" "${files[0]}"; then
    checked=$((checked + 1))
  else
    echo "adr-text: ${files[0]} differs from its section of $collected:"
    diff -u --label "expected ADR-$n" --label "${files[0]}" "$work/$n.md" "${files[0]}" || true
    failed=1
  fi
done < "$work/sections"

if [ "$failed" -ne 0 ]; then
  exit 1
fi
echo "adr-text: $checked ADR(s) equal their sections of $collected"
