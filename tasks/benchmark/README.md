# Benchmarks

Performance benchmarks for SWC Next's parser, AST traversal, semantic analysis,
and compiler pipelines. Run commands from the workspace root.

## Wall time

```sh
cargo bench -p benchmark --bench parser
```

Select `visit` or `semantic` instead of `parser` for those components.

## JavaScript parser batches

```sh
node tasks/benchmark/bin/babel.js tasks/benchmark/files/typescript.js 10
node tasks/benchmark/bin/babel.js tasks/benchmark/files/typescript.js 10 parse-walk
```

Use `swc.js` or `swc-next.js` for the other parsers. Each command validates the
input, warms up five times, then runs the requested number of parses. Replace
the iteration count with `--calibrate` to report milliseconds per parse.

## Instructions and profiles

On Linux with Valgrind and the `gungraun-runner` version matching the workspace's
Gungraun dependency:

```sh
cargo bench -p benchmark --bench gungraun
```

For a comparison in Actions without a PR, push both revisions and dispatch
[the Gungraun workflow](../../.github/workflows/gungraun.yml):

```sh
gh workflow run gungraun.yml --ref <workflow-branch> -f base_ref=<base-sha> -f head_ref=<head-sha> -f filter='<benchmark-filter>'
```

The workflow must exist on the default branch. Omit `head_ref` to compare against
the dispatch revision, or `filter` to run all cases. PR runs compare the base
with the merge revision. Check the resolved SHAs in the run summary; artifacts
contain raw profiles. Instruction counts are not wall time.

For the JavaScript N-API boundary, use [JavaScript API Gungraun](js/README.md).

## Pipeline simulator

```sh
TARGET=next cargo run -p benchmark --release --example rspack_simulator
```

Use `TARGET=oxc` for the Oxc pipeline. Add `--features tracy` to collect a Tracy profile.
