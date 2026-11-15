# @swc-next/formatter

Document encoding and printing for SWC Next, with native and WebAssembly bindings for Node.js. Supports Prettier Docs, layout options, and cursor positions.

**Experimental and unstable.** APIs, document formats, and output may change.

## Usage

```js
import { encodeDoc } from "@swc-next/formatter/document";
import { printDocBufferSync } from "@swc-next/formatter";

const { buffer } = encodeDoc(["hello", { type: "line", hard: true }, "world"]);
console.log(printDocBufferSync(buffer));
```

Document buffers require matching package versions. The document encoder can be used without loading a native or WebAssembly binding. See the [printing API](src-js/index.d.ts) and [encoder API](src-js/document.d.ts).

For JavaScript and TypeScript source formatting, use the [Prettier plugin](../formatter-prettier-plugin).

## Testing

API tests cover document encoding, layout options, string handling, errors, and cursor positions.

From the workspace root, run `pnpm --filter @swc-next/formatter test` for native API tests, or `pnpm --filter @swc-next/formatter test:wasm` with WebAssembly bindings built.
