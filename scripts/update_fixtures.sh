#!/usr/bin/env bash

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

revisions=scripts/fixture-revisions.txt
updated_revisions="$(mktemp "$revisions.XXXXXX")"
trap 'rm -f -- "$updated_revisions"' EXIT

while read -r name repo ref policy; do
    if [[ "$policy" == fixed ]]; then
        printf '%s %s %s %s\n' "$name" "$repo" "$ref" "$policy"
        continue
    fi

    remote_head="$(git ls-remote --exit-code "https://github.com/$repo.git" HEAD)"
    read -r latest _ <<< "$remote_head"
    if [[ ! "$latest" =~ ^[0-9a-f]{40}$ ]]; then
        echo "Invalid upstream HEAD for $repo: $remote_head" >&2
        exit 1
    fi
    if [[ "$ref" != "$latest" ]]; then
        echo "Updating $name: $ref -> $latest" >&2
    fi
    printf '%s %s %s\n' "$name" "$repo" "$latest"
done < "$revisions" > "$updated_revisions"

if cmp -s "$revisions" "$updated_revisions"; then
    echo "Fixture revisions are up to date."
    exit 0
fi

chmod 644 "$updated_revisions"
mv -- "$updated_revisions" "$revisions"
