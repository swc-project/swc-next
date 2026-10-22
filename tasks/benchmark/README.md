# Benchmarks

Performance benchmarks for SWC Next's parser, AST traversal, semantic analysis,
and compiler pipelines. Run commands from the workspace root.

## Wall time

```sh
cargo bench -p benchmark --bench parser
```

Select `visit` or `semantic` instead of `parser` for those components.

## Parser comparisons

Compare native operations with SWC and JavaScript parsing and traversal with
SWC and Babel. Requires Hyperfine 2, Rust, and the workspace dependencies.
On Linux, install `taskset` and choose an available CPU with `BENCH_CPU`.

```sh
BENCH_CPU=0 node website/docs/scripts/benchmark.js
```

Results: [Benchmark page](../../website/docs/docs/guide/benchmark.mdx).

## Instructions and profiles

On Linux with Valgrind and the `gungraun-runner` version matching the workspace's
Gungraun dependency:

```sh
cargo bench -p benchmark --bench gungraun
```

See the [Gungraun workflow](../../.github/workflows/gungraun.yml) for comparisons.
For the JavaScript N-API boundary, use [JavaScript API Gungraun](js/README.md).

## Pipeline simulator

```sh
TARGET=next cargo run -p benchmark --release --example rspack_simulator
```

Use `TARGET=oxc` for the Oxc pipeline. Add `--features tracy` to collect a Tracy profile.
