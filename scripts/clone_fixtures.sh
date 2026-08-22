#!/usr/bin/env bash

set -euo pipefail

# Clone a fixture repository at its pinned commit.
clone_repo() {
    local repo="$1"
    local path="$2"
    local ref="$3"
    local name="$4"

    echo "Starting clone of $name..."

    mkdir -p "$(dirname "$path")"
    git clone --quiet --no-progress --depth 1 --revision="$ref" \
        "https://github.com/$repo.git" "$path"

    echo "✓ Completed clone of $name"
}

clone_repo "tc39/test262" "tasks/testsuite/fixtures/test262" "d1d583db95a521218f3eb8341a887fd63eda8ff1" "test262"
clone_repo "microsoft/TypeScript" "tasks/testsuite/fixtures/typescript" "637d5746b70257028fb95aad32ddec6b26ab0a14" "typescript"
clone_repo "babel/babel" "tasks/testsuite/fixtures/babel" "1554d587b31460f135adf65812d199f963a34e61" "babel"
clone_repo "oxc-project/estree-conformance" "tasks/testsuite/fixtures/estree-conformance" "079d13881372c0316b0e0b6552fe5f0ae5528d24" "estree-conformance"
