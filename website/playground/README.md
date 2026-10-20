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

The development server builds Wasm at startup. Rebuild it after Rust changes
using the playground package's `build:wasm` script.

## Deployment

The [deployment workflow](../../.github/workflows/deploy-playground.yml) builds
the static site and publishes it to Cloudflare Pages. It requires the repository
secrets `CLOUDFLARE_ACCOUNT_ID` and `CLOUDFLARE_API_TOKEN`.

Every deployment publishes the selected revision to production, including
manual runs from other branches.
