#!/usr/bin/env bash

set -euo pipefail

current_worktree="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
common_git_dir="$(git -C "$current_worktree" rev-parse --path-format=absolute --git-common-dir)"
repository_root="$common_git_dir/swc-next/testsuite-fixture-cache/repositories"
fixtures_root="$current_worktree/tasks/testsuite/fixtures"
conformance_dir="${SWC_CONFORMANCE_DIR:-$fixtures_root/swc-conformance}"
ref="851d2e7ea79638af25da0cf9ac87c7ffd3f4dc12"
repository_path="$repository_root/swc-conformance"
pin_ref="refs/fixture-pins/$ref"

git init --quiet --bare --object-format=sha1 "$repository_path"
if ! git --git-dir="$repository_path" rev-parse --verify --quiet "${pin_ref}^{commit}" > /dev/null; then
    git --git-dir="$repository_path" fetch --quiet --no-tags --depth 1 \
        "${SWC_CONFORMANCE_DIR:-https://github.com/CPunisher/swc-conformance.git}" "${ref}:${pin_ref}"
fi

rm -rf -- "$fixtures_root/swc-conformance"
mkdir -p "$fixtures_root/swc-conformance"
git --git-dir="$repository_path" archive "$pin_ref" | tar -xf - -C "$fixtures_root/swc-conformance"

bash "$conformance_dir/scripts/clone_fixtures.sh" "$fixtures_root" "$repository_root"
