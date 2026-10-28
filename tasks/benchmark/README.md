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

See the [Gungraun workflow](../../.github/workflows/gungraun.yml) for comparisons.
For the JavaScript N-API boundary, use [JavaScript API Gungraun](js/README.md).

## Pipeline simulator

```sh
TARGET=next cargo run -p benchmark --release --example rspack_simulator
```

Use `TARGET=oxc` for the Oxc pipeline. Add `--features tracy` to collect a Tracy profile.
