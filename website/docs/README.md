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

Preview the production build with `pnpm --filter docs preview`.
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
counts and process-local warmups. Both JavaScript groups compare SWC Next, SWC,
and Babel. `parse-walk` parses a fresh AST and counts Identifiers without scope
analysis; it uses JavaScript inputs because SWC's JS Visitor does not implement
TypeScript type traversal. The MDX page renders tables and bar charts
from this JSON during the static build, using SWC Next as the 1× baseline for
each input and metric. Wall time and peak RSS describe whole
processes, not individual operations. The first order is the primary comparison;
reverse-order statistics are available separately.

## Existing documents

The Testing page reads the headers of `tasks/testsuite/snapshots/*.snap` during
the build and renders conformance counts by runner and suite. It uses the
checked-in results without running the suites or requiring external fixtures.
Update snapshots with the commands in the [test suite README](../../tasks/testsuite/README.md),
then rebuild the site. Missing or inconsistent summary counts fail the build.

The Benchmark page is in `docs/guide/benchmark.mdx`. The flat AST design
document is in `docs/architecture/flat-ast.md`. Shared images are in
`docs/public/images/`.
