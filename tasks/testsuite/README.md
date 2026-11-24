# Test suite

Conformance testing for SWC Next against Test262, TypeScript, Babel, SWC,
Prettier, and terser fixtures.

- Parser suites check parsing and semantic diagnostics against fixture expectations.
- Codegen suites check AST round trips and printing stability.
- Semantic suites compare identifier end offsets and semantic binding identities
  with reference output.
- Transform suites compare normalized output or check output syntax and stability.
- Minifier suites compare normalized output with the reference output of SWC's
  minifier fixtures and its terser port, and check compression stability.
  Fixture options are mapped to the supported compress options. Negative cases
  expect the input unchanged. Fixtures without a reference output or with
  mangling are skipped, and code is not executed.

SWC transform cases use `crates/swc/tests/compile`, selecting the nearest `.swcrc`
for each input file. Unsupported compiler settings and snapshots requiring
diagnostics, source maps, or declaration output are skipped.

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

Install the workspace JavaScript dependencies before running the Prettier
compatibility matrix.

```sh
cargo run -p testsuite --bin formatter --profile conformance
```

The runner checks the pinned Prettier version, reads physical JS/JSX/TS/TSX
inputs, discovers options from `runFormatTest`, and
compares output against committed Jest snapshots. Parser-specific error snapshots
identify expected rejections for each option set. Shared JSX inputs exercise both
parser partitions. The reports list unsupported options separately.

Successful formatting checks output parsing, comment retention, source/AST
entry-point parity, and second-pass idempotency. These checks do not establish
full semantic equivalence. Formatting and parser failures remain visible as
syntax support grows.

All output mismatches count toward the raw snapshot match rate. The formatter
suites track verified differences by fixture and option set in
[the divergence list](src/formatter/divergences.rs). A full run fails if a
registered option set no longer produces an output mismatch. Output that fails
a correctness check remains a failure.

The Markdown suite includes CommonMark and GFM examples from the upstream
snapshots. SWC formats the document and nested Markdown fences. Other fenced
languages and front matter use the same plugins as Prettier. Rust combines their
Docs with the surrounding Markdown. Unlabeled fences, unknown languages, and
disabled embedding preserve the fenced body, including trailing spaces. The
suite compares output from the source and AST entry points and formats the
result a second time. Registered Markdown differences also require an exact
match with the recorded SWC output.

Use `--markdown` to run only the Markdown suite.

```sh
cargo run -p testsuite --bin formatter --profile conformance -- --markdown
```

The default run writes `snapshots/formatter_prettier_{js,typescript,markdown}.snap`.
CI checks the generated reports against the committed snapshots.

Inspect matching inputs and detailed diffs without changing reports:

```sh
PRETTIER_FILTER='js/comments/jsx.js' \
  cargo run -p testsuite --bin formatter --profile conformance
```

## Debug one case

The debug runner inspects a uniquely matched case without updating snapshots:

```sh
cargo run -p testsuite --bin debug -- --runner parser --case test262 --filter '<unique substring>'
```

Parser and codegen runners accept `test262`, `typescript`, or `babel`.
Transform accepts `babel`, `typescript`, or `swc`; semantic accepts `swc_resolver`;
minifier accepts `terser` or `swc`.
See `--help` for additional options.

## JavaScript conformance

After fixture setup, run from the workspace root:

```sh
pnpm install
pnpm test:conformance
pnpm test:codec
```

ESTree conformance checks the N-API AST against reference fixtures. Codec
conformance encodes parsed programs into buffers, then checks byte-for-byte
stability after decoding and re-encoding with the original source. Each command
builds its native bindings. Both are separate from the Rust conformance runner.
