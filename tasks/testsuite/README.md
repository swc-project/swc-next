# Test suite

Conformance and regression testing for SWC Next against Test262, TypeScript,
Babel, and SWC fixtures, plus project-owned syntax cases.

- Parser suites check parsing and semantic diagnostics against fixture expectations.
- Codegen suites check AST round trips and printing stability.
- Resolver suites compare name resolution with reference output.
- Transform suites compare normalized output or check output syntax and stability.

## Usage

From the workspace root, initialize the pinned fixtures with
[the fixture setup script](../../scripts/clone_fixtures.sh), then run:

```sh
cargo run --locked -p testsuite --bin conformance --profile conformance
```

Review the generated [snapshots](snapshots): ordinary mismatches may not fail
the process. External fixtures are generated inputs; update their pinned
revisions rather than editing them. Fixture setup must run serially across
worktrees sharing a Git common directory.

The [debug runner](src/bin/debug.rs) inspects individual cases without updating
snapshots. [Local syntax cases](fixtures-manual/parser) use Test262 metadata
and have separate totals from upstream Test262 tests.
