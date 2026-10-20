# Test suite

Conformance testing for SWC Next against Test262, TypeScript, Babel, and SWC
fixtures.

- Parser suites check parsing and semantic diagnostics against fixture expectations.
- Codegen suites check AST round trips and printing stability.
- Resolver suites compare name resolution with reference output.
- Transform suites compare normalized output or check output syntax and stability.

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

## Debug one case

The debug runner inspects a uniquely matched case without updating snapshots:

```sh
cargo run -p testsuite --bin debug -- --runner parser --case test262 --filter '<unique substring>'
```

Parser and codegen runners accept `test262`, `typescript`, or `babel`.
Transform accepts `babel`, `typescript`, or `swc`; resolver accepts `swc_resolver`.
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
