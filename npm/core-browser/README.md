# `@swc-next/core-browser`

The browser WebAssembly API for SWC Next. The package includes its WebAssembly
binary and browser JavaScript bindings. Running the binary requires WebAssembly
SIMD support.

```js
import init, { parseSync, generateSync } from "@swc-next/core-browser";

await init();

const { program } = parseSync("const value = 1;");
program.body[0].declarations[0].init.value = 2;
console.log(generateSync(program).code);
```

Call `init()` before parsing or printing. It loads the bundled `.wasm` file;
subsequent calls are idempotent.

Parser options use strings, for example
`{ lang: "ts", sourceType: "module", comments: "all" }`.
`Lang`, `SourceType`, and `CommentMode` are available only as TypeScript types.

## Building

Run from the workspace root:

```sh
pnpm install
rustup target add wasm32-unknown-unknown
pnpm build:browser --profile wasm-release
pnpm prepare:browser
pnpm --filter @swc-next/core-browser build:js
```

These commands build the Wasm binding, assemble its browser assets, and build
the JavaScript API in this package's `dist` directory. Omit
`--profile wasm-release` for a debug build. To assemble from an existing Node.js
core Wasm build instead:

```sh
pnpm prepare:browser --source-dir ../core/dist
pnpm --filter @swc-next/core-browser build:js
```

`--source-dir` is relative to `npm/core-browser`; it defaults to `dist`.

## Testing

After building and assembling the package:

```sh
pnpm exec playwright install --with-deps --only-shell chromium
pnpm test:browser
```
