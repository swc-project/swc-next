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

## Pipeline simulator

```sh
TARGET=next cargo run -p benchmark --release --example rspack_simulator
```

Use `TARGET=oxc` for the Oxc pipeline. Add `--features tracy` to collect a Tracy profile.

## Transformer example

```sh
TARGET=next cargo run -p benchmark --release --example transformer
```

Use `TARGET=oxc` or `TARGET=swc` for the other implementations.
