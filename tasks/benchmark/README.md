# Benchmarks

Performance benchmarks for SWC Next's parser, AST traversal, semantic analysis,
and compiler pipelines. Run commands from the workspace root.

## Wall time

```sh
cargo bench -p benchmark --bench parser
```

Select `visit` or `semantic` instead of `parser` for those components.

The [CodSpeed workflow](../../.github/workflows/codspeed.yml) measures these
benchmarks on CodSpeed Macro Runners for every push to `main`. Pull requests run
only when they have the `Codspeed` label, including when the label is added.
CodSpeed must be enabled for the repository, and the organization's runner group
must allow this public repository to use Macro Runners.

## Instructions and profiles

On Linux with Valgrind and the `gungraun-runner` version matching the workspace's
Gungraun dependency:

```sh
cargo bench -p benchmark --bench gungraun
```

The `jsx_lowering` group measures classic, automatic, and development JSX
lowering. Parsing, TypeScript erasure, normalization, and semantic analysis run
in setup; output validation and cleanup run in teardown. The `jsx_transform`
group measures the combined TypeScript, JSX, and optional React Refresh pipeline.
Run the isolated pass on Linux with Valgrind, or smoke-test it without Valgrind:

```sh
cargo bench --locked -p benchmark --bench gungraun -- --allow-aslr=yes '*::jsx_lowering::*'
cargo test --locked -p benchmark --bench gungraun -- --test '*::jsx_lowering::*'
```

See the [Gungraun workflow](../../.github/workflows/gungraun.yml) for comparisons.
For the JavaScript N-API boundary, use [JavaScript API Gungraun](js/README.md).

## Pipeline simulator

```sh
TARGET=next cargo run -p benchmark --release --example rspack_simulator
```

Use `TARGET=oxc` for the Oxc pipeline. Add `--features tracy` to collect a Tracy profile.
