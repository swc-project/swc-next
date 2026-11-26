# Benchmarks

Performance benchmarks for SWC Next's parser, AST traversal, semantic analysis,
type stripping, minification, and compiler pipelines. Run commands from the workspace root.

## Wall time

```sh
cargo bench -p benchmark --bench parser
```

Select `visit`, `semantic`, `type_strip`, or `minifier` instead of `parser` for those
components.

The [CodSpeed workflow](../../.github/workflows/codspeed.yml) measures the
`parser`, `visit`, and `semantic` benchmarks on CodSpeed Macro Runners for every
push to `main`. It runs on an AMD Ryzen 9950X (x86_64) runner and on an Arm
Cortex-A72 (arm64) runner. A manual dispatch measures the selected branch. Pull
requests run only when they have the `Codspeed`
label, including when the label is added. CodSpeed must be enabled for the
repository, and the organization's runner group must allow this public
repository to use Macro Runners.

The CodSpeed benchmark IDs end with the target architecture, such as
`jquery/parser/new/x86_64`, so the two runners report separate benchmarks.

## Instructions and profiles

On Linux with Valgrind and the `gungraun-runner` version matching the workspace's
Gungraun dependency:

```sh
cargo bench -p benchmark --bench gungraun
```

The Profile Performance workflow also compares Gungraun instruction counts for
every pull request and manual run.

The `transform` group measures the TypeScript, JSX, React Refresh, and ES2022
static block pipeline. Parsing, normalization, and initial semantic analysis run
in setup. Run it on Linux with Valgrind, or smoke-test it without Valgrind:

```sh
cargo bench --locked -p benchmark --bench gungraun -- --allow-aslr=yes '*::transform::*'
cargo test --locked -p benchmark --bench gungraun -- --test '*::transform::*'
```

For the JavaScript N-API boundary, use [JavaScript API Gungraun](js/README.md).

The Markdown parser and formatter use the same three [documents](fixtures/markdown).
The formatter receives a parsed document and preserves fenced code as plain text.

```sh
cargo bench --locked -p benchmark --bench gungraun -- --allow-aslr=yes '*::bench_markdown_*'
```

## JavaScript plugins

Build the native core and JavaScript packages, then run the Criterion plugin
benchmarks with Node.js 22 or later:

```sh
pnpm --filter @swc-next/core build --release
pnpm build:js
cargo bench -p benchmark --bench js-plugin
```

The `js-plugin` group runs `files/js-plugin/` with FFI1 via `@swc-next/core`.
The `js-plugin2` group runs `files/js-plugin2/` with FFI2 via
`@swc-next/core/ffi2` and `@swc-next/walker2` using native AST fields. Both groups process the same
JavaScript, JSX, TypeScript and TSX fixtures in persistent Node processes.
Each iteration measures `parse` (Rust parse, AST encode, root/comments decode),
`plugin` (JS traversal and transform), and `print` (JS encode, native decode
and print). `ident-length` has no `print` phase.

The worker warms the dataset three times before timing (`JS_PLUGIN_WARMUP`
overrides the count), so the default V8 tier-up thresholds reach the hot path.
Module loading, plugin construction, validation, context construction,
IPC and the first cold pass are outside the phase clocks; `total` includes
context construction and clock reads, so it can exceed the phase sum. The cold
pass is reported separately as `coldPluginNs`. The `js-plugin2` group uses the
internal unchecked `generateSync2Unchecked` for printing. Keep Node, fixtures
and plugin sources identical across revisions;
validation normalizes only RegExp flag order. `JS_PLUGIN_NODE`,
`JS_PLUGIN_REPORT_DIR` and `JS_PLUGIN_ROOT` select the executable, report
directory and runner checkout.

Criterion uses 20 flat samples, 10 ms warmup and 200 ms measurement targets per
phase; each invocation still runs the whole dataset. The usual `--warm-up-time`,
`--measurement-time` and `--sample-size` flags override them. The worker uses
Node's default tier-up thresholds; its warmup passes ensure the measured phases
run on optimized code.

## Pipeline simulator

```sh
TARGET=next cargo run -p benchmark --release --example rspack_simulator
```

Use `TARGET=oxc` for the Oxc pipeline. Add `--features tracy` to collect a Tracy profile.

## Transformer example

```sh
TARGET=next cargo run -p benchmark --release --example transformer_comparison
```

Use `TARGET=oxc` or `TARGET=swc` for the other implementations.
