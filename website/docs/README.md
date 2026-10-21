# SWC Next documentation

The Rspress documentation site for SWC Next. Markdown and MDX pages live in
`docs/`, navigation is defined in `navigation.ts`, and the custom theme follows
the Playground's visual language.

## Development

Run from the workspace root:

```sh
pnpm install
pnpm dev:docs
```

Check types and build the static site:

```sh
pnpm check:docs
pnpm build:docs
```

Preview the production build with `pnpm --filter @swc-next/website preview`.
Static output is written to `website/docs/doc_build`.

## Performance results

Run from the workspace root with Hyperfine 2, Rust, and the workspace dependencies
installed (Linux also requires `taskset`):

```sh
BENCH_CPU=0 node website/docs/scripts/benchmark.js
pnpm build:docs
```

Choose an available CPU with `BENCH_CPU` on Linux. The script builds the native
benchmarks and Node.js package, then writes `data/benchmarks.json`. Use
`--output <path>` to save a separate report. Failed runs leave the previous
report intact.

The JSON groups Hyperfine exports by operation and input. Both command orders
retain their full exports, including samples, summaries, units, and metadata.
Native batches run ten iterations; JavaScript batches use calibrated iteration
counts and process-local warmups. The MDX page renders tables and bar charts
from this JSON during the static build. Wall time and peak RSS describe whole
processes, not individual operations. The first order is the primary comparison;
reverse-order statistics are available separately.

The checked-in initial JSON preserves the previous report's rounded summaries;
it has no individual samples or reverse-order measurements. A new benchmark run
replaces it with full Hyperfine exports.

## Existing documents

The Performance page is in `docs/guide/publish-results.mdx`. The flat AST design
document is in `docs/architecture/flat-ast.md`. Shared images are in
`docs/public/images/`.
