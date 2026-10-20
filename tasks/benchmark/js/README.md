# JavaScript API instruction profiles

Gungraun benchmarks for the public `@swc-next/core` N-API entry point,
including JavaScript, AST encoding/decoding, and Rust work. They measure
synchronous executed instructions with JIT disabled; Wasm is not measured.

`parse` reads the complete AST, `print` uses a materialized AST, and `lang`
and `source_type` exercise path helpers. Setup, warmup, and explicit GC are
outside the measured region; GC triggered during measurement remains included.

## Usage

The [JavaScript API Gungraun workflow](../../../.github/workflows/gungraun-js.yml)
compares selected base/head revisions and publishes profiles and summaries.
After pushing both revisions, dispatch it from the workspace root:

```sh
gh workflow run gungraun-js.yml -f base=<base-sha> -f head=<head-sha> -f filter='parse/underscore'
```

The workflow must exist on the default branch. Omit `filter` for all cases;
PRs run four representative cases.

## Local profiling

On Linux, install Valgrind, a C compiler, Node with its headers, pnpm, and the
`gungraun-runner` version specified in the workflow. Run from the workspace root:

```sh
BASE_REF=main HEAD_REF=HEAD node .github/scripts/run-gungraun-js.mjs
```

Both revisions use the committed head harness and fixtures. Use fresh output
and preparation directories for each comparison; the script leaves detached
worktrees for inspection. `JS_BENCH_OUTPUT` and `JS_BENCH_PREPARED` override those
directories; `JS_BENCH_FILTER=parse/underscore` selects a case substring.

## Results

Results report instruction counts, not normal JIT-enabled wall time.
Overlapping trial ranges are inconclusive, and a failed same-SHA repeatability
check prevents publication of a directional comparison.
