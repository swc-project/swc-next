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

## Existing documents

The original benchmark report is in `docs/benchmarks/results.md`; its script
remains in `scripts/benchmark.js`. The flat AST design document is in
`docs/resources/architecture/flat-ast.md`. Shared images are in `docs/public/images/`.

The remaining pages provide the initial documentation structure and concise
summaries. They do not yet provide a complete API reference.
