#!/usr/bin/env bash

set -euo pipefail

common_git_dir="$(git rev-parse --path-format=absolute --git-common-dir)"
repository_root="$common_git_dir/swc-next/testsuite-fixture-cache/repositories"

# Clone a fixture's bare repository and export its pinned tree.
prepare_fixture() {
    local repo="$1"
    local path="$2"
    local ref="$3"
    local name="$4"
    local repository_path="$repository_root/$name"
    local repository_url="https://github.com/$repo.git"
    local pin_ref="refs/fixture-pins/$ref"
    local fixture_root

    fixture_root="$(dirname "$path")"

    echo "Preparing $name..."

    mkdir -p "$repository_root" "$fixture_root"
    git init --quiet --bare "$repository_path"
    git --git-dir="$repository_path" fetch --quiet --no-tags --depth 1 \
        "$repository_url" "${ref}:${pin_ref}"

    rm -rf -- "$path"
    mkdir "$path"
    git --git-dir="$repository_path" archive "$pin_ref" | tar -xf - -C "$path"

    echo "✓ Prepared $name"
}

prepare_fixture "tc39/test262" "tasks/testsuite/fixtures/test262" "d1d583db95a521218f3eb8341a887fd63eda8ff1" "test262"
prepare_fixture "microsoft/TypeScript" "tasks/testsuite/fixtures/typescript" "637d5746b70257028fb95aad32ddec6b26ab0a14" "typescript"
prepare_fixture "babel/babel" "tasks/testsuite/fixtures/babel" "1554d587b31460f135adf65812d199f963a34e61" "babel"
prepare_fixture "oxc-project/estree-conformance" "tasks/testsuite/fixtures/estree-conformance" "079d13881372c0316b0e0b6552fe5f0ae5528d24" "estree-conformance"
