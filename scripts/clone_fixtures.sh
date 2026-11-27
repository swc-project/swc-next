#!/usr/bin/env bash

set -euo pipefail

current_worktree="$(git rev-parse --show-toplevel)"
common_git_dir="$(git rev-parse --path-format=absolute --git-common-dir)"
repository_root="$common_git_dir/swc-next/testsuite-fixture-cache/repositories"
revisions="${1:-$current_worktree/scripts/fixture-revisions.txt}"

# Ensure the shared bare repository contains the fixture's pinned commit.
update_fixture_cache() {
    local repo="$1"
    local ref="$2"
    local name="$3"
    local repository_path="$repository_root/$name"
    local repository_url="https://github.com/$repo.git"
    local pin_ref="refs/fixture-pins/$ref"

    mkdir -p "$repository_root"
    git init --quiet --bare --object-format=sha1 "$repository_path"

    if ! git --git-dir="$repository_path" rev-parse --verify --quiet "${pin_ref}^{commit}" > /dev/null; then
        git --git-dir="$repository_path" fetch --quiet --no-tags --depth 1 \
            "$repository_url" "${ref}:${pin_ref}"
    fi
}

# Replace a fixture with the tree stored at its pinned commit.
restore_fixture() {
    local path="$1"
    local ref="$2"
    local name="$3"
    local repository_path="$repository_root/$name"
    local pin_ref="refs/fixture-pins/$ref"

    rm -rf -- "$path"
    mkdir -p "$path"
    git --git-dir="$repository_path" archive "$pin_ref" | tar -xf - -C "$path"
}

prepare_fixture() {
    local name="$1"
    local repo="$2"
    local ref="$3"
    local path="$current_worktree/tasks/testsuite/fixtures/$name"

    echo "Preparing $name..."

    update_fixture_cache "$repo" "$ref" "$name"
    restore_fixture "$path" "$ref" "$name"
    echo "✓ Prepared $name"
}

while read -r name repo ref _; do
    if [[ ! "$name" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ || ! "$ref" =~ ^[0-9a-f]{40}$ ]]; then
        echo "Invalid fixture revision: $name $repo $ref" >&2
        exit 1
    fi
    prepare_fixture "$name" "$repo" "$ref"
done < "$revisions"
