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
