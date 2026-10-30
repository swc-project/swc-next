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

The workflow also compares the base and compare revisions on one Macro Runner,
using the compare revision's benchmark harness, fixtures, and Rust toolchain.
Five independent process pairs alternate execution order. Each revision's time
is the median of its process medians, rather than its fastest sample. The report
shows paired bootstrap 95% intervals and flags candidates only when the entire
interval exceeds a 1% change. With five pairs, intervals are approximate and are
not corrected for multiple comparisons; candidates require confirmation and do
not fail CI. The suite summary is the equally weighted geometric mean of the
individual time ratios. Keep individual results when assessing regressions.

Comparisons appear in the Actions summary and, for same-repository pull requests,
in a PR comment. Raw samples and results are retained as artifacts for 90 days.
Pushes compare against the previous `main` revision; pull requests compare their
merge revision against the PR base. CodSpeed uploads and profiles remain available
with CodSpeed's own comparison statistics.

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
