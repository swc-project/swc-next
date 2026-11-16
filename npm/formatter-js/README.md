# @swc-next/formatter-js

JavaScript, JSX, TypeScript, and TSX formatting document production for SWC Next, with native and WebAssembly bindings for Node.js. Accepts parser and SWC formatter configuration objects and returns an unprinted document.

**Experimental and unstable.** APIs, document formats, and output may change. Document consumers require matching package versions.

## Usage

```js
import { formatDocBufferSync } from "@swc-next/formatter-js";
import { printDocBufferSync } from "@swc-next/formatter";

const options = { lineWidth: 100, indentStyle: "space" };
const buffer = formatDocBufferSync("const answer=42;", { lang: "js" }, options);
console.log(printDocBufferSync(buffer, options));
```

Source errors throw `SyntaxError` with UTF-16 `start` and `end` offsets. See the [configuration API](src-js/index.d.ts). For formatting through Prettier, use the [Prettier plugin](../formatter-prettier-plugin).

## Testing

From the workspace root, run `pnpm --filter @swc-next/formatter-js test`, or `pnpm --filter @swc-next/formatter-js test:wasm` with WebAssembly bindings built. API tests cover configuration forwarding, validation, and source error offsets.
