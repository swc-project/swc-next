#!/usr/bin/env bash

set -euo pipefail

current_worktree="$(git rev-parse --show-toplevel)"
common_git_dir="$(git rev-parse --path-format=absolute --git-common-dir)"
main_worktree="$(dirname "$common_git_dir")"
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
    git init --quiet --bare "$repository_path"

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

# Copy the main worktree's fixture using copy-on-write when supported.
copy_fixture() {
    local path="$1"
    local name="$2"
    local source_path="$main_worktree/tasks/testsuite/fixtures/$name"

    rm -rf -- "$path"
    mkdir -p "$path"

    case "$(uname -s)" in
        Darwin)
            cp -a -c "$source_path/." "$path/"
            ;;
        Linux)
            cp -a --reflink=auto "$source_path/." "$path/"
            ;;
        *)
            cp -a "$source_path/." "$path/"
            ;;
    esac
}

prepare_fixture() {
    local repo="$1"
    local relative_path="$2"
    local ref="$3"
    local name="$4"
    local path="$current_worktree/$relative_path"
    local repository_path="$repository_root/$name"
    local source_path="$main_worktree/tasks/testsuite/fixtures/$name"
    local cache_head

    echo "Preparing $name..."

    update_fixture_cache "$repo" "$ref" "$name"
    cache_head="$(git --git-dir="$repository_path" rev-parse --verify HEAD 2> /dev/null || true)"

    if [[ "$current_worktree" == "$main_worktree" ]]; then
        if [[ "$cache_head" != "$ref" || ! -d "$path" ]]; then
            restore_fixture "$path" "$ref" "$name"
            git --git-dir="$repository_path" update-ref HEAD "$ref"
        fi
    else
        if [[ "$cache_head" == "$ref" && -d "$source_path" ]]; then
            copy_fixture "$path" "$name"
        else
            restore_fixture "$path" "$ref" "$name"
        fi
    fi

    echo "✓ Prepared $name"
}

prepare_fixture "tc39/test262" "tasks/testsuite/fixtures/test262" "d1d583db95a521218f3eb8341a887fd63eda8ff1" "test262"
prepare_fixture "microsoft/TypeScript" "tasks/testsuite/fixtures/typescript" "637d5746b70257028fb95aad32ddec6b26ab0a14" "typescript"
prepare_fixture "babel/babel" "tasks/testsuite/fixtures/babel" "1554d587b31460f135adf65812d199f963a34e61" "babel"
prepare_fixture "oxc-project/estree-conformance" "tasks/testsuite/fixtures/estree-conformance" "079d13881372c0316b0e0b6552fe5f0ae5528d24" "estree-conformance"
