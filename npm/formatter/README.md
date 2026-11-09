# @swc-next/formatter

Language-independent document encoding and Rust layout printing for native and
Wasm runtimes. The package consumes SWCD documents using the shared formatter
core. For JavaScript and TypeScript source formatting, use
`@swc-next/ecma-formatter` or the SWCFMT Prettier plugin.

```js
import { encodeDoc } from "@swc-next/formatter/document";
import { printDocBufferSync } from "@swc-next/formatter";

const { buffer } = encodeDoc(["hello", { type: "line", hard: true }, "world"]);
const text = printDocBufferSync(buffer, { printWidth: 80 });
```

The document subpath is pure JavaScript and can be imported without loading a
native or Wasm binding. Label values remain in the encoder's caller-owned
sidecar. The printing API also supports cursor ranges, indentation, and output
line endings.

The version-0 API and wire format may change. Document printing uses the native
printer's existing resource behavior.

Build native: `pnpm --filter @swc-next/formatter build`.
Build Wasm: `pnpm --filter @swc-next/formatter build:wasm`.
