# SWC Next Playground

A Rsbuild + React workspace styled with Tailwind CSS, using Jotai for state,
and an independent Rust crate compiled to
WebAssembly. All parsing, semantic checks, transforms, and code generation run
locally in a Web Worker using the workspace's Rust crates.

Rust sources live in `src/`, while the React frontend and compiler Worker live
in `src-js/`. Generated WASM bindings and binaries live in `wasm/`.

From the repository root:

```sh
pnpm install
rustup target add wasm32-unknown-unknown
pnpm dev:playground
```

`dev` builds the playground WASM binary before starting Rsbuild at
`http://localhost:3000`. After changing Rust code, rebuild the binary in a
second terminal:

```sh
pnpm --filter playground build:wasm
```

Build and preview the standalone static site:

```sh
pnpm --filter playground check
pnpm build:playground
pnpm --filter playground preview
```

The production build uses the `wasm-release` profile and includes its WASM
binary, Monaco editor, and Workers in `website/playground/dist`. It does not
require a server API or CDN. The binary and JavaScript bindings generated under
`wasm/` are ignored; the browser API declaration is checked in. The workspace's
`bridge_cli` builds the Rust crate and generates its bindings, including when a
custom Cargo target directory or profile is used.

The three columns contain compiler options, editable source, and Code / AST
results. The input/output divider supports pointer dragging and arrow keys.
Source and options persist locally; the header resets both, while the sidebar
reset button resets only options.

- **Parser:** JavaScript, JSX, TypeScript, TSX, and declaration files; source
  type, parentheses preservation, and semantic errors.
- **Transform:** TypeScript lowering and ES2022 class static blocks. JSX is
  preserved. Required scope checks run before transformations even when the
  optional semantic errors setting is off.
- **Codegen:** compact whitespace (`minify`), ASCII escaping, and inline-script
  escaping. `minify` is a printing option, not optimizer passes.

AST shows the original parsed ESTree with UTF-16 source ranges. Expanding nodes
reveals their properties; the cursor highlights the smallest containing node,
and selecting a node selects its source. Parser errors still expose a recovered
AST; erroneous input is not transformed or printed.

`src-js/state.ts` holds Jotai atoms for source, compiler options, theme, compilation
results, and UI state. Source and options use a validated storage atom; the theme
is initialized synchronously in the HTML head to avoid flashing on refresh.
Components use Tailwind utilities; `src-js/styles.css` contains theme tokens and
base rules.

The Rust `#[bridge] run(source, options)` API accepts both inputs as strings.
The Worker serializes options with `JSON.stringify`; Rust decodes them directly
with `serde_json::from_str`. Bridge handles string transport, memory ownership,
and error propagation. The compilation payload contains a little-endian `u32`
AST length, the repository's binary AST with diagnostics padded to four bytes,
then UTF-8 output code. The Worker decodes the AST with `@swc-next/decoder` and
ignores responses superseded by newer edits.

Bridge is enabled only for WASM targets. Native builds expose the ordinary Rust
function without generating N-API bindings; the playground has no `napi-build`
dependency or build script.

## Production deployment

The production site is https://play.next.swc.rs, hosted on Cloudflare Pages.
The `Deploy Playground` GitHub Actions workflow builds the WASM binary and static
site, checks TypeScript types, and uploads `dist` with Wrangler. It runs daily
at 00:00 Asia/Shanghai (16:00 UTC) and can also be started manually. Scheduled
runs use the latest commit on the default branch. Pushes and pull requests do
not deploy the site.

One-time setup:

1. In the Cloudflare account that manages `swc.rs`, create a Direct Upload Pages
   project named `swc-next-playground` with production branch `main`.
2. Add `play.next.swc.rs` to the project's custom domains, then create or confirm
   its CNAME record in the `swc.rs` zone. Point it to the project's actual Pages
   hostname (`swc-next-playground.pages.dev`). Wait for the custom domain and
   HTTPS certificate to become active.
3. Add `CLOUDFLARE_ACCOUNT_ID` and `CLOUDFLARE_API_TOKEN` to the repository's
   GitHub Actions secrets. Limit the token to this account with
   `Account > Cloudflare Pages > Edit` permission. DNS permissions are not
   required for routine deployments.

After the workflow is merged into the default branch, open **Actions > Deploy
Playground > Run workflow**, leave the branch set to `main`, and start the run.
The equivalent command from the repository root is:

```sh
gh workflow run deploy-playground.yml --ref main
```

The selected workflow branch supplies the source revision. Every run publishes
to production (`--branch=main`); selecting another branch also replaces the
production site and does not create a preview. Concurrent runs share a deployment
group and do not cancel an active run. GitHub may replace an older pending run
with a newer pending run.

The run summary records the commit, deployment URL, and production URL.
Dependency installation, type checks, and the production build must all succeed
before upload starts. A failed build leaves the existing deployment in place.
After deployment, check Code and AST results in the browser and confirm that
the WASM binary and Workers load without errors.
