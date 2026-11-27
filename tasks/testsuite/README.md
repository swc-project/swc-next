# Test suite

The suite checks SWC Next against Test262, TypeScript, Babel, SWC, Prettier, and
terser fixtures.

## Rust conformance

Run all commands from the workspace root. Set up the pinned fixtures before
running conformance.

```sh
./scripts/clone_fixtures.sh
cargo run --locked -p testsuite --bin conformance --profile conformance
```

The runner checks parsing, AST round trips, identifier resolution, transforms,
and minification. Transform checks skip unsupported SWC settings and cases that
require diagnostics, source maps, or declarations. Minifier checks skip cases
that use mangling or lack reference output.

Check for mismatches even when the command succeeds by reviewing the generated
[snapshots](snapshots).

## Formatter conformance

Run `pnpm install` before running the formatter or JavaScript suites.

```sh
cargo run --locked -p testsuite --bin formatter --profile conformance
```

The runner compares JavaScript, TypeScript, and Markdown output against the
pinned Prettier snapshots. It checks parsing, comment retention, source and AST
output parity, and idempotency. Verified differences are tracked in the
[divergence list](src/formatter/divergences.rs).

Append `-- --markdown` to run only Markdown. Set `PRETTIER_FILTER` to a fixture
path substring to inspect detailed diffs without rewriting reports.

## Debug one case

Use `--filter` to select a single case.

```sh
cargo run -p testsuite --bin debug -- --runner parser --case test262 --filter '<unique substring>'
```

Run with `--help` for the supported runners and fixture suites.

## JavaScript conformance

```sh
pnpm test:conformance
pnpm test:codec
```

ESTree checks the JavaScript AST against reference fixtures. Codec checks that
AST buffers round-trip byte for byte. Both commands build the native bindings.
