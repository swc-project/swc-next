#!/usr/bin/env bash

set -euo pipefail

branch=automation/update-conformance-fixtures
remote_ref="refs/heads/$branch"
title="chore(testsuite): update conformance fixtures"

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

pr_url="$(gh pr list --base main --head "$branch" --state open \
    --json url,isCrossRepository --jq 'map(select(.isCrossRepository | not))[0].url // empty')"
if [[ -n "$pr_url" ]]; then
    echo "$pr_url"
else
    gh pr create --base main --head "$branch" --title "$title" --body \
        'Advance the conformance fixture revisions to their upstream heads and regenerate the Rust, JavaScript, and formatter reports.'
fi
