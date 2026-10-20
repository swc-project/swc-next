# SWC Next Playground

An interactive playground for SWC Next's parser, semantic analysis,
TypeScript transforms, and code generator. Edit JavaScript, TypeScript, JSX,
or TSX and inspect the generated code, ESTree AST, and diagnostics.

Try it at [play.next.swc.rs](https://play.next.swc.rs). Compilation runs locally
in a Web Worker through WebAssembly. Source and compiler options persist locally.

## Development

From the workspace root:

```sh
pnpm install
rustup target add wasm32-unknown-unknown
pnpm dev:playground
```

The development server builds Wasm at startup. After Rust changes, rebuild it:

```sh
pnpm --filter playground build:wasm
```

To check types and build the static site:

```sh
pnpm --filter playground check
pnpm build:playground
```

The output is `website/playground/dist`; use `pnpm --filter playground preview`
to serve it locally.

## Deployment

The [deployment workflow](../../.github/workflows/deploy-playground.yml) builds
the static site and publishes it to Cloudflare Pages. It requires the repository
secrets `CLOUDFLARE_ACCOUNT_ID` and `CLOUDFLARE_API_TOKEN`.

It runs daily at 00:00 Asia/Shanghai and can also be dispatched manually:

```sh
gh workflow run deploy-playground.yml --ref main
```

Every deployment publishes the selected revision to production, including
manual runs from other branches.
