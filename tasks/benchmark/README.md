# Benchmarks

Performance benchmarks for SWC Next's parser, AST traversal, semantic analysis,
type stripping, minification, and compiler pipelines. Run commands from the workspace root.

## Wall time

```sh
cargo bench -p benchmark --bench parser
```

Select `visit`, `semantic`, `type_strip`, or `minifier` instead of `parser` for those
components.

The [CodSpeed workflow](../../.github/workflows/codspeed.yml) measures these
benchmarks on CodSpeed Macro Runners for every push to `main`. Pull requests run
only when they have the `Codspeed` label, including when the label is added.
CodSpeed must be enabled for the repository, and the organization's runner group
must allow this public repository to use Macro Runners.

The [Profile Performance workflow](../../.github/workflows/profile-perf.yml)
uses the same triggers for paired wall time on an x86_64 GitHub runner. It builds
the base and head revisions and runs each benchmark with the base binary and the
head binary in an even number of rounds in the same job. Base runs first in odd
rounds and head runs first in even rounds, so the run order does not bias the
change. It measures only the targets that use a changed file, as determined from
the workspace dependency graph in `cargo metadata`. It writes the change to the
job summary and comments on the pull request or the pushed `main` commit. It does
not comment on pull requests from forks. The summary also records the runner's CPU
and system. To compare any two revisions, run the workflow manually with `base`
and `head`. Set `targets` to `all` or to a list of targets to measure other targets.
To run the wall-time comparison locally:

```sh
node .github/scripts/paired-walltime.mjs build target/paired/base   # on the base revision
node .github/scripts/paired-walltime.mjs build target/paired/head   # on the head revision
node .github/scripts/paired-walltime.mjs run target/paired/base target/paired/head target/paired/results.json
node .github/scripts/paired-walltime.mjs render target/paired/results.json
```

## Instructions and profiles

On Linux with Valgrind and the `gungraun-runner` version matching the workspace's
Gungraun dependency:

```sh
cargo bench -p benchmark --bench gungraun
```

The Profile Performance workflow also compares Gungraun instruction counts for
every pull request and manual run, in the same job and from the same build as the
wall-time benchmarks. Use `suite` to run only one measurement manually.

The `transform` group measures the TypeScript, JSX, React Refresh, and ES2022
static block pipeline. Parsing, normalization, and initial semantic analysis run
in setup. Run it on Linux with Valgrind, or smoke-test it without Valgrind:

```sh
cargo bench --locked -p benchmark --bench gungraun -- --allow-aslr=yes '*::transform::*'
cargo test --locked -p benchmark --bench gungraun -- --test '*::transform::*'
```

For the JavaScript N-API boundary, use [JavaScript API Gungraun](js/README.md).

## JavaScript plugins

Build the native core and JavaScript packages, then run the Criterion plugin
benchmarks with Node.js 22 or later:

```sh
pnpm --filter @swc-next/core build --release
pnpm build:js
cargo bench -p benchmark --bench js-plugin
```

The plugins in `files/js-plugin/` process every JavaScript, JSX, TypeScript and
TSX fixture in `files/` in a persistent Node process. Each iteration reports
`parse` (Rust parse, AST encode, root/comments decode), `plugin` (JS walker and
transform, including lazy field reads) and `print` (JS encode, unchecked Rust
decode and print). `count-functions` is a read-only walker pass with no encode or
print, so it has only `parse` and `plugin`.

The worker warms the dataset once before timing (`JS_PLUGIN_WARMUP` overrides the
count). Module loading, plugin construction, validation, context construction,
IPC and the first cold pass are outside the phase clocks; `total` includes
context construction and clock reads, so it can exceed the phase sum. The cold
pass is reported separately as `coldPluginNs`. Timing uses the internal unchecked
`generateSyncUnchecked`; AST legality checking belongs to the Rust checked
path. Keep Node, fixtures and plugin sources identical across revisions;
validation normalizes only RegExp flag order. `JS_PLUGIN_NODE`,
`JS_PLUGIN_REPORT_DIR` and `JS_PLUGIN_ROOT` select the executable, report
directory and runner checkout.

Criterion uses 20 flat samples, 10 ms warmup and 200 ms measurement targets per
phase; each invocation still runs the whole dataset. The usual `--warm-up-time`,
`--measurement-time` and `--sample-size` flags override them. Node lowers
feedback/Maglev/TurboFan/OSR thresholds so both revisions optimize early; reports
record the flags in `nodeArgs`, and unsupported flags fall back to ordinary Node.

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
