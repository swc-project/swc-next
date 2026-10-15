# JavaScript API instruction profiles

The `JavaScript API Gungraun` workflow compares exact base/head revisions on
Ubuntu 24.04 with Node 24.16.0 and Gungraun 0.20.0. It measures the public
`@swc-next/core` N-API entry point with JIT enabled, including synchronous JS,
encoding/decoding and Rust work. WASM is not measured.

`parse` returns the lazy result; `parse_full` also reads every enumerable AST
property. `print` uses an already materialized AST. `lang` and `source_type`
exercise the path helpers. Each case gets 100 warmup calls and three fresh
processes. Each measured batch has 10 AST calls or 10,000 path calls. Results
are retained until collection ends. Setup, explicit GC, warmup and validation
are outside the measured region. GC or JIT work triggered on the main thread
during that region remains part of the measurement. Background V8 threads
are not part of this synchronous API metric.

Callgrind collection is controlled by `measure.c`, a benchmark-only N-API
addon. The callback and its loop have a small shared measurement cost. No
production code is instrumented. The harness uses a standalone Cargo manifest
and lockfile so an older base can be measured without replacing its manifests,
lockfiles or product code. Both revisions use the head harness and fixtures.

## Parallel CI

Every PR runs five representative cases: `parse`, `parse_full`, and `print` on
`underscore-1.5.2.js`, plus `lang/tsx` and `source_type/mjs`. Manual runs execute
all 24 cases unless a `filter` is supplied. Cases run in separate jobs, up to
24 at once subject to the account's runner concurrency. One preparation job builds both N-API revisions,
the benchmark driver and the profiling helper. Jobs download that frozen runtime
bundle and compare base/head on the same runner, with three fresh processes per
side. CPU models are recorded per case. The same-SHA check runs independently
in parallel. A final job rejects missing, duplicate or failed shards before
publishing the combined comparison. Logs stream during measurement.

## Run on Linux

Install Valgrind, a C compiler, Node 24.16.0 with its headers, pnpm, the repository
Rust toolchain, and `cargo install --locked --version 0.20.0 gungraun-runner`.
Commit the harness before using the comparison script; it reads benchmark files
from the selected head revision. From the repository root:

```sh
BASE_REF=main HEAD_REF=HEAD JS_BENCH_FILTER=parse/underscore \
  node .github/scripts/run-gungraun-js.mjs
```

To run all cases manually in Actions after this workflow is on the default
branch:

```sh
gh workflow run gungraun-js.yml -f base=<base-sha> -f head=<head-sha>
```

Use fresh `JS_BENCH_PREPARED` and `JS_BENCH_OUTPUT` directories for every comparison.
The defaults are `target/gungraun-js-prepared` and `target/gungraun-js-ci`. Omit `JS_BENCH_FILTER` to run all cases. It is a literal
substring of `operation/fixture`; an unmatched filter is an error. To check
repeatability, set both refs to the same SHA. The script creates detached
worktrees beneath the prepared directory and leaves them for inspection; remove
them with `git worktree remove` when no longer needed.

Use `--prepare` to build inputs only, then `--measure` to run the frozen driver
without Cargo. `JS_BENCH_CASE` selects one exact `operation/fixture` identity;
`--same-sha` measures the prepared head on both sides. CI transports the runtime
bundle as a tar archive to preserve workspace package symlinks and executable
permissions. Measurement jobs retain the preparation path and pinned Node
version, verify executable hashes, and do not rebuild product code.

The workspace also exposes `cargo bench -p benchmark --bench gungraun_js`.
For direct use, set absolute `JS_BENCH_HARNESS`, `JS_BENCH_ENTRY`, and
`JS_BENCH_NODE` paths. The harness directory must contain `runner.mjs`,
`measure.node`, and `files/` with the four fixtures. The comparison script
prepares these automatically and uses the standalone manifest.

## Interpret results

Artifacts retain raw profiles, flamegraphs, Gungraun JSON, exact trial counts,
median/range summaries, logs and metadata for 90 days. Metadata records SHAs,
tool versions, Node flags, build settings and SHA-256 hashes of the harness and
fixtures. `base-summaries.json` preserves baseline summaries before the head run.
The release build retains symbols and debug information on both sides.

Ir is an instruction count under Valgrind, not wall time. JIT remains enabled;
three samples do not guarantee statistical significance. Overlapping ranges
are inconclusive. Inspect profiles and repeat comparisons near an optimization
threshold. JIT machine code can appear without useful JavaScript source names;
retained native symbols still expose synchronous N-API and Rust costs.
Do not apply the optimization skill's acceptance thresholds to an inconclusive
result. Performance differences never fail CI; missing or invalid comparisons,
failed API assertions and build failures do. The same-SHA control must have
overlapping ranges. Disjoint control ranges fail the repeatability job and
prevent publication of a directional comparison; inspect the retained trial
counts and profiles before rerunning.

## Harness checks

```sh
node --test tasks/benchmark/js/*.test.mjs
cargo check --locked --manifest-path tasks/benchmark/js/Cargo.toml --benches
```

Run the normal Node API tests after building the N-API packages. Linux CI is
required to validate the actual Callgrind boundary and binary summary schema.
