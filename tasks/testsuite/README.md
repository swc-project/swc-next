# Test suite

Conformance testing for SWC Next against Test262, TypeScript, Babel, and SWC
fixtures.

- Parser suites check parsing and semantic diagnostics against fixture expectations.
  Negative Test262 and TypeScript cases use V8 and tsrs reference spans when
  available. Every reported diagnostic span must fit entirely inside a reference
  span. TypeScript references are combined across configurations for the same
  virtual file; existing diagnostic-code filters still apply. Cases without
  usable reference spans retain the error-presence check.
- Codegen suites check AST round trips and printing stability.
- Semantic suites compare identifier end offsets and semantic binding identities
  with reference output.
- Transform suites compare normalized output or check output syntax and stability.

Semantic fixtures use the resolver JSON snapshots and prepared inputs from
`swc-conformance` as their reference.

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

Fixture repositories and revisions are managed by `swc-conformance`. To
initialize from a local checkout:

```sh
SWC_CONFORMANCE_DIR=/path/to/swc-conformance ./scripts/clone_fixtures.sh
```

Test262 metadata and TypeScript virtual files, options, and baseline diagnostic
codes are read from `swc-conformance/fixtures-json/`. SWC Next retains its own
case and diagnostic filters. Legacy Test262 sources and generated metadata are
also managed by `swc-conformance`.

To read fixture JSON and resolver snapshots directly from a local checkout:

```sh
SWC_CONFORMANCE_DIR=/path/to/swc-conformance cargo run --locked -p testsuite --bin conformance --profile conformance
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
