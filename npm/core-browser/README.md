# `@swc-next/core-browser`

The browser WebAssembly API for SWC Next. The package includes its WebAssembly
binary and browser JavaScript bindings.

```js
import init, { parseSync, printSync } from "@swc-next/core-browser";

await init();

const { program } = parseSync("const value = 1;");
program.body[0].declarations[0].init.value = 2;
console.log(printSync(program).code);
```

Call `init()` before parsing or printing. It loads the bundled `.wasm` file;
subsequent calls are idempotent.

Parser options use strings, for example
`{ lang: "ts", sourceType: "module", comments: "flat" }`.
`Lang`, `SourceType`, and `CommentMode` are available only as TypeScript types.

## Building

Run from the workspace root:

```sh
pnpm install
rustup target add wasm32-unknown-unknown
pnpm build:browser --profile wasm-release
node scripts/wasm-package/cli.mjs prepare-browser --source-dir target/wasm-package/core
```

The build produces core Wasm artifacts; the assembly step copies them into
this package. Omit `--profile wasm-release` for a debug build. To assemble
from an existing Node.js core Wasm build instead:

```sh
node scripts/wasm-package/cli.mjs prepare-browser --source-dir npm/core/src-js
```

## Testing

After building and assembling the package:

```sh
pnpm exec playwright install --with-deps --only-shell chromium
pnpm test:browser
```
