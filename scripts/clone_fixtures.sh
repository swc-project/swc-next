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

prepare_fixture "tc39/test262" "tasks/testsuite/fixtures/test262" "d1d583db95a521218f3eb8341a887fd63eda8ff1" "test262"
prepare_fixture "microsoft/TypeScript" "tasks/testsuite/fixtures/typescript" "637d5746b70257028fb95aad32ddec6b26ab0a14" "typescript"
prepare_fixture "babel/babel" "tasks/testsuite/fixtures/babel" "0915316cf4a66e85af77ff9043d2f9c8a439418b" "babel"
prepare_fixture "oxc-project/estree-conformance" "tasks/testsuite/fixtures/estree-conformance" "079d13881372c0316b0e0b6552fe5f0ae5528d24" "estree-conformance"
prepare_fixture "swc-project/swc" "tasks/testsuite/fixtures/swc" "e4f588ee8744a1a4e3323dfc0f5e1f2f98afb67f" "swc"
prepare_fixture "CPunisher/swc-conformance" "tasks/testsuite/fixtures/swc-conformance" "88e085fc5e96b68c4c7d5a6fc97d8de6396ab823" "swc-conformance"
prepare_fixture "prettier/prettier" "tasks/testsuite/fixtures/prettier" "cdd17f2288b28b170a76416c72dac56e3ea5daff" "prettier"
