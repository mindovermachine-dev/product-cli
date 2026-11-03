#!/usr/bin/env bash
# The commit-message gate: every commit in <base>..<head> is checked (ADR-0007 D1).
#
# Each commit must carry:
#   - a "Refs #N" line;
#   - a "Signed-off-by: Name <email>" naming a person, never a no-reply,
#     bot or agent address;
#   - no model identifier, in the message (trailers included) or in the
#     author or committer name and email.
#
# Usage: eng/gates/commits.sh <base> <head>
set -euo pipefail

if [ $# -ne 2 ]; then
  echo "usage: $0 <base> <head>" >&2
  exit 2
fi
base=$1
head=$2

# A model identifier is the string an API takes to select a model, or a model
# family named with its version. Matched case-insensitively. The agent
# identity "Claude (Emil)" and the "Claude-Session:" trailer do not match.
model_re='(^|[^a-z0-9])('
model_re+='claude-(opus|sonnet|haiku|fable|instant|[0-9])'
model_re+='|claude[ ]+[0-9]'
model_re+='|(opus|sonnet|haiku|fable)[ -]?[0-9]'
model_re+='|chatgpt|gpt-?[0-9]'
model_re+='|gemini[ -]?[0-9]|gemini-(pro|flash|ultra|nano)'
model_re+='|o[1-9](-mini|-pro|-preview)?'
model_re+='|llama[ -]?[0-9]|mistral-|mixtral|codestral|deepseek|qwen[0-9-]'
model_re+=')([^a-z0-9]|$)'

# A co-author trailer naming a model vendor's address names a model.
coauthor_re='^co-authored-by:.*@([a-z0-9.-]*\.)?(anthropic\.com|openai\.com|deepmind\.com|mistral\.ai)'

# A sign-off names a person: not a no-reply, bot or agent address.
dco_re='^Signed-off-by: [^<>]+ <[^<>@ ]+@[^<>@ ]+>$'
not_person_re='no-?reply|\[bot\]|anthropic\.com|openai\.com|claude\+'

commits=$(git rev-list --reverse "$base..$head")
if [ -z "$commits" ]; then
  echo "commits: no commits in $base..$head" >&2
  exit 1
fi

failed=0
report() {
  echo "commits: $1: $2"
  failed=1
}

for c in $commits; do
  short=$(git rev-parse --short "$c")
  msg=$(git log -1 --format=%B "$c")
  people=$(git log -1 --format='%an <%ae>%n%cn <%ce>' "$c")

  if ! grep -qE '^Refs #[0-9]+([^0-9]|$)' <<<"$msg"; then
    report "$short" "no 'Refs #N' line"
  fi

  signoffs=$(grep -E '^Signed-off-by:' <<<"$msg" || true)
  if [ -z "$signoffs" ]; then
    report "$short" "no 'Signed-off-by:' line"
  else
    while IFS= read -r s; do
      if ! grep -qE "$dco_re" <<<"$s"; then
        report "$short" "malformed sign-off: $s"
      elif grep -qiE "$not_person_re" <<<"$s"; then
        report "$short" "sign-off does not name a person: $s"
      fi
    done <<<"$signoffs"
  fi

  if hit=$(grep -inE "$model_re" <<<"$msg"); then
    report "$short" "message names a model: $hit"
  fi
  if hit=$(grep -inE "$coauthor_re" <<<"$msg"); then
    report "$short" "co-author trailer names a model vendor: $hit"
  fi
  if hit=$(grep -iE "$model_re" <<<"$people"); then
    report "$short" "author or committer names a model: $hit"
  fi
done

if [ "$failed" -ne 0 ]; then
  exit 1
fi
echo "commits: $(wc -w <<<"$commits" | tr -d ' ') commit(s) in $base..$head pass"
