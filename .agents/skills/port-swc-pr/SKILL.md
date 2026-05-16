---
name: port-swc-pr
description: Port a PR from swc-project/swc to this repository, including tests and code changes.
---

# Port SWC PR Skill

This skill guides you through the process of porting a Pull Request from the upstream `swc-project/swc` repository to the local `swc-experimental` repository.

## Input

- **PR ID**: The ID of the pull request in `swc-project/swc` (e.g., `11462`).

## Workflow

### 1. Fetch & Analyze Diff

First, retrieve the changes from the upstream PR.

```bash
gh pr diff <PR_ID> -R swc-project/swc
```

**Goal**: Identify:

1.  **Test Cases**: Look for changes in `crates/swc_ecma_parser/tests`. Upstream uses snapshot tests.
2.  **Source Code**: Look for changes in `crates/swc_ecma_parser/src`.

Create a git branch named like `02-01-feat/port-1234`, where `feat` should be respected to the type of change in the source PR.

### 2. Port Test Cases (First Step)

**Rule**: Always match the test failure _before_ fixing the code.

1.  **Locate Target Directory**:
    Tests in this repo are stored in `tasks/testsuite/fixtures/misc-parser`.
    - **Passing Tests** (Should parse successfully): `tasks/testsuite/fixtures/misc-parser/pass`
    - **Failing Tests** (Should fail parsing): `tasks/testsuite/fixtures/misc-parser/fail`

2.  **Create Test File**:
    - **Naming**: Create a single file in `tasks/testsuite/fixtures/misc-parser/{pass,fail}`.
    - **Filename**: Derive the filename from the upstream test path to be descriptive (e.g., if upstream is `tests/foo/bar.js`, name it `bar.js` or `foo-bar.js`).
    - **Content**: Copy the JavaScript code from the upstream snapshot/test.

3.  **Run Tests (Expect Failure)**:
    Run the test suite to confirm the tests fail (or behave incorrectly) without the fix.
    ```bash
    cargo run -p testsuite
    ```
    _Verify that your new test case is failing._

### 3. Port Source Code

Apply the logic changes from the diff to the local codebase.

**File Mapping Guide**:
The local repository structure differs slightly from upstream. Use this mapping as a reference:

- Upstream `crates/swc_ecma_parser/src/parser/*.rs` -> Local `crates/swc_ecma_parser/src/parser/js/*.rs`
  - _Example_: `src/parser/expr.rs` -> `src/parser/js/expr.rs`
  - _Example_: `src/parser/stmt.rs` -> `src/parser/js/stmt.rs`

**Instructions**:

1.  Read the upstream code changes carefully.
2.  Find the equivalent code in the mapped local file.
3.  Apply the changes, adapting variable names and internal struct logic if necessary.

### 4. Verify Fix

Run the test suite again to verify the fix.

```bash
cargo run -p testsuite
```

- **Pass**: The test case should now pass (or fail gracefully if it's a negative test).
- **Fail**: If it still fails, debug the local implementation.

### 5. Submit PR

Once the tests pass and the code is verified:

1.  **Get Upstream Title**:

    ```bash
    gh pr view <PR_ID> -R swc-project/swc --json title --jq .title
    ```

2.  **Create PR**:
    Create the PR using the upstream title and referencing the upstream PR ID in the body.
    ```bash
    gh pr create --title "<UPSTREAM_TITLE>" --body "Port of <PR_ID>"
    ```

## Tips

- **File Structure**: Use `find_by_name` if you can't locate a file.
- **Run Specific Tests**: The `testsuite` might run all tests. If it's too slow, check if `cargo run -p testsuite -- --help` offers filtering options (e.g., matching the filename).
