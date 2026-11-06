#!/usr/bin/env bash
# Checks that every non-merge commit in <base>..<head> says which model helped write it.
#
# A commit passes when its trailers include either
#   Co-Authored-By: Claude <Family> <version> <noreply@anthropic.com>
#     e.g. "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>" (one per model used), or
#   AI-Assisted: none
#     for a commit written without any model.
# A Co-Authored-By trailer with an Anthropic address that doesn't name the model, such as
# "Claude <noreply@anthropic.com>", fails: the point is to record which model it was.
#
# Usage: check-attribution.sh <base> <head>
#
# With ATTRIBUTION_REPORT set to a path, also appends one Markdown list item per failing
# commit to that file, for the PR comment. It holds only the commit hash and a fixed reason,
# never text from the commit, so a commit message can't put mentions or markup in the comment.
set -euo pipefail

if [ $# -ne 2 ]; then
  echo "usage: $0 <base> <head>" >&2
  exit 2
fi
base=$1
head=$2

model_re='^Claude (Fable|Opus|Sonnet|Haiku) [0-9]+(\.[0-9]+)* <noreply@anthropic\.com>$'

failed=0
checked=0

fail() {
  local commit=$1 subject=$2 reason=$3 summary=$4
  failed=$((failed + 1))
  echo "::error title=Missing model attribution::$(git rev-parse --short "$commit") \"$subject\": $reason"
  if [ -n "${ATTRIBUTION_REPORT:-}" ]; then
    echo "- $commit: $summary" >> "$ATTRIBUTION_REPORT"
  fi
}

for commit in $(git rev-list --no-merges --reverse "$base..$head"); do
  checked=$((checked + 1))
  subject=$(git log -1 --format=%s "$commit")
  coauthors=$(git log -1 --format='%(trailers:key=Co-Authored-By,valueonly)' "$commit")
  assisted=$(git log -1 --format='%(trailers:key=AI-Assisted,valueonly)' "$commit")

  models=0
  bad=""
  while IFS= read -r value; do
    value=$(printf '%s' "$value" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')
    [ -z "$value" ] && continue
    if printf '%s\n' "$value" | grep -Eq "$model_re"; then
      models=$((models + 1))
    elif printf '%s\n' "$value" | grep -Eiq 'anthropic\.com|claude'; then
      bad="$value"
    fi
  done <<< "$coauthors"

  none=0
  if printf '%s\n' "$assisted" | grep -Eiq '^[[:space:]]*none[[:space:]]*$'; then
    none=1
  fi

  if [ -n "$bad" ]; then
    fail "$commit" "$subject" "\"Co-Authored-By: $bad\" doesn't name the model; use e.g. \"Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>\"" \
      "a \`Co-Authored-By\` trailer for Claude doesn't name the model"
  elif [ "$models" -gt 0 ] && [ "$none" -eq 1 ]; then
    fail "$commit" "$subject" "has both a model Co-Authored-By trailer and \"AI-Assisted: none\"" \
      "has both a model \`Co-Authored-By\` trailer and \`AI-Assisted: none\`"
  elif [ "$models" -eq 0 ] && [ "$none" -eq 0 ]; then
    fail "$commit" "$subject" "no model attribution; add \"Co-Authored-By: Claude <Model> <noreply@anthropic.com>\" for each model used, or \"AI-Assisted: none\" if none was" \
      "no model attribution"
  fi
done

if [ "$failed" -gt 0 ]; then
  echo "$failed of $checked commit(s) don't attribute the models used."
  echo "Fix with an interactive rebase (reword), then force-push the branch."
  exit 1
fi
echo "All $checked commit(s) attribute the models used."
