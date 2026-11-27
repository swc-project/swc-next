#!/usr/bin/env bash

set -euo pipefail

branch=automation/update-conformance-fixtures
remote_ref="refs/heads/$branch"
title="chore(testsuite): update conformance fixtures"

# Keep the PR branch available for fixes while its checks are being resolved.
pr_url="$(gh pr list --base main --head "$branch" --state open \
    --json url,isCrossRepository --jq 'map(select(.isCrossRepository | not))[0].url // empty')"
if [[ -n "$pr_url" ]]; then
    echo "Fixture update PR: $pr_url"
    exit 0
fi

git apply --cached "$1"
if ! git diff --cached --no-renames --quiet -- \
    ':!scripts/fixture-revisions.txt' \
    ':(exclude,glob)tasks/testsuite/snapshots/*.snap' \
    ':!npm/formatter-prettier-plugin/test/prettier-plugin-conformance.json'; then
    echo "Fixture update contains unexpected paths." >&2
    exit 1
fi

if git diff --cached --quiet -- scripts/fixture-revisions.txt; then
    echo "No fixture revisions to publish."
    exit 0
fi

fixture_updates="$(
    git diff --cached --no-color --unified=0 -- scripts/fixture-revisions.txt |
        awk '
            /^-[^-]/ { previous[substr($1, 2)] = $3 }
            /^\+[^+]/ {
                name = substr($1, 2)
                if (name in previous && $3 != previous[name]) {
                    printf "- `%s` `%s` → `%s`\n", name, substr(previous[name], 1, 7), substr($3, 1, 7)
                }
            }
        '
)"

remote_tip="$(git ls-remote --refs origin "$remote_ref")"
remote_sha="${remote_tip%%[[:space:]]*}"
if [[ -n "$remote_sha" ]]; then
    git fetch --no-tags --depth=1 origin "$remote_sha"
fi

if [[ -z "$remote_sha" ]] || ! git diff --cached --quiet "$remote_sha" --; then
    app_user_id="$(gh api "/users/${APP_SLUG}[bot]" --jq .id)"
    git -c user.name="${APP_SLUG}[bot]" \
        -c user.email="${app_user_id}+${APP_SLUG}[bot]@users.noreply.github.com" \
        commit -m "$title"
    git push --force-with-lease="$remote_ref:$remote_sha" origin "HEAD:$remote_ref"
fi

if [[ "$REPORTS_OUTCOME" == success ]]; then
    status='✅ Conformance reports regenerated.'
else
    run_url="$GITHUB_SERVER_URL/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID"
    status="❌ Conformance report generation failed. [Workflow log]($run_url)"
fi

body_file="$(mktemp)"
trap 'rm -- "$body_file"' EXIT
printf '%s\n\n%s\n' "$status" "$fixture_updates" > "$body_file"
gh pr create --base main --head "$branch" --title "$title" --body-file "$body_file"
