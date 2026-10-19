#!/usr/bin/env bash

set -euo pipefail

current_worktree="$(git rev-parse --show-toplevel)"
common_git_dir="$(git rev-parse --path-format=absolute --git-common-dir)"
repository_root="$common_git_dir/swc-next/testsuite-fixture-cache/repositories"

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
    local repo="$1"
    local relative_path="$2"
    local ref="$3"
    local name="$4"
    local path="$current_worktree/$relative_path"

    echo "Preparing $name..."

    update_fixture_cache "$repo" "$ref" "$name"
    restore_fixture "$path" "$ref" "$name"
    echo "✓ Prepared $name"
}

prepare_fixture "tc39/test262" "tasks/testsuite/fixtures/test262" "d53ebc8b1931d9f1541e43d59650776e4b766248" "test262"
prepare_fixture "microsoft/TypeScript" "tasks/testsuite/fixtures/typescript" "5237fd46890956e4bb8177a1a05dcd69e4166560" "typescript"
prepare_fixture "babel/babel" "tasks/testsuite/fixtures/babel" "93e39e4d0b657efe7f569f9d25acbff96ddb89dc" "babel"
prepare_fixture "oxc-project/estree-conformance" "tasks/testsuite/fixtures/estree-conformance" "eb050aeacb0d0588f84829da0a5cbf0c63676637" "estree-conformance"
prepare_fixture "swc-project/swc" "tasks/testsuite/fixtures/swc" "4768dbe091a6706a009d2e9c08950cacba107d55" "swc"
prepare_fixture "CPunisher/swc-conformance" "tasks/testsuite/fixtures/swc-conformance" "528c158a89553e5ab6c19354acd8479239e8b676" "swc-conformance"
