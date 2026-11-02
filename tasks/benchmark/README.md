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

The [paired wall-time workflow](../../.github/workflows/paired-walltime.yml)
uses the same triggers on an x86_64 GitHub runner. It builds the base and head
revisions and runs each benchmark with the base binary and then the head binary,
in several rounds in the same job. It writes the change to the job summary and
comments on the pull request or the pushed `main` commit. It does not comment on
pull requests from forks. The summary also records the runner's CPU and system.
To compare any two revisions, run the workflow manually with `base` and `head`.
To run it locally after building both revisions:

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

See the [Gungraun workflow](../../.github/workflows/gungraun.yml) for comparisons.
For the JavaScript N-API boundary, use [JavaScript API Gungraun](js/README.md).

## Pipeline simulator

```sh
TARGET=next cargo run -p benchmark --release --example rspack_simulator
```

Use `TARGET=oxc` for the Oxc pipeline. Add `--features tracy` to collect a Tracy profile.
