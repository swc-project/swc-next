# Test suite

Conformance testing for SWC Next against Test262, TypeScript, Babel, SWC, and Prettier
fixtures.

- Parser suites check parsing and semantic diagnostics against fixture expectations.
- Codegen suites check AST round trips and printing stability.
- Semantic suites compare identifier end offsets and semantic binding identities
  with reference output.
- Transform suites compare normalized output or check output syntax and stability.

Semantic fixtures use SWC's legacy resolver as a reference, including its parser
and identifier-marking model.

## Usage

Run from the workspace root. Initialize the pinned external fixtures before
the first conformance run:

```sh
./scripts/clone_fixtures.sh
cargo run --locked -p testsuite --bin conformance --profile conformance
```

Review the generated [snapshots](snapshots): ordinary mismatches may not fail
the process. External fixtures are generated inputs; update their pinned
revisions rather than editing them. Fixture setup must run serially across
worktrees sharing a Git common directory.

## Formatter conformance

Run the Prettier compatibility matrix:

```sh
cargo run -p testsuite --bin formatter_conformance --profile conformance
```

The fixture is pinned to Prettier 3.9.9 at commit
`cdd17f2288b28b170a76416c72dac56e3ea5daff`. The runner verifies the version,
reads physical JS/JSX/TS/TSX inputs, discovers options from `runFormatTest`, and
compares output against committed Jest snapshots. Unsupported options are
reported explicitly. Shared JSX inputs exercise both parser partitions.

Successful formatting checks output parsing, comment retention, source/AST
entry-point parity, and second-pass idempotency. These checks do not establish
full semantic equivalence. Formatting and parser failures remain visible as
syntax support grows.

Summaries are written to `snapshots/formatter_prettier_{js,typescript}.snap`.
A normal run can succeed with known mismatches or unsupported syntax; CI checks
the generated reports against the committed snapshots.

Inspect matching inputs and detailed diffs without changing reports:

```sh
PRETTIER_FILTER='js/comments/jsx.js' \
  cargo run -p testsuite --bin formatter_conformance --profile conformance
```

## Debug one case

The debug runner inspects a uniquely matched case without updating snapshots:

```sh
cargo run -p testsuite --bin debug -- --runner parser --case test262 --filter '<unique substring>'
```

Parser and codegen runners accept `test262`, `typescript`, or `babel`.
Transform accepts `babel`, `typescript`, or `swc`; semantic accepts `swc_resolver`.
See `--help` for additional options.

## JavaScript conformance

After fixture setup, run from the workspace root:

```sh
pnpm install
pnpm test:conformance
pnpm test:codec
```

ESTree conformance checks the N-API AST against reference fixtures. Codec
conformance checks that JavaScript decode/encode output agrees with native
parse/codegen. Each command builds its required native bindings. Both are
separate from the Rust conformance runner.
