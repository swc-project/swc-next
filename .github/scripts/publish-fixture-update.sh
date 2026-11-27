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

body='Advance the tracked conformance fixtures to their upstream heads.'
if [[ "$REPORTS_OUTCOME" == success ]]; then
    body+=' Includes the regenerated conformance reports.'
else
    run_url="$GITHUB_SERVER_URL/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID"
    body+=" Report generation failed. Apply any runner fixes and commit the regenerated reports on this branch. [Update workflow log]($run_url) · [Regeneration commands](https://github.com/swc-project/swc-next/blob/main/tasks/testsuite/README.md)"
fi
gh pr create --base main --head "$branch" --title "$title" --body "$body"
