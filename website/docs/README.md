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

## Existing documents

The Testing page reads the headers of `tasks/testsuite/snapshots/*.snap` during
the build and renders conformance counts by runner and suite. It uses the
checked-in results without running the suites or requiring external fixtures.
Update snapshots with the commands in the [test suite README](../../tasks/testsuite/README.md),
then rebuild the site. Missing or inconsistent summary counts fail the build.

The flat AST design document is in `docs/architecture/flat-ast.md`. Shared images are in
`docs/public/images/`.
