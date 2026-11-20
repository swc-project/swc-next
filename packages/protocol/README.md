# `@swc-next/protocol`

Shared JavaScript source for SWC Next's internal native and WebAssembly protocols. Includes document encoders, parser and generator configuration encoders, source transport, transform options and output, and formatter configuration and results.

```js
import { encodeParserConfig } from "@swc-next/protocol/parser";
import { encodeFormatterConfig } from "@swc-next/protocol/formatter";
```

APIs and wire formats do not follow semantic versioning. JavaScript and native/Wasm code must use matching versions. Source and transform option encoders reuse scratch storage; consume borrowed buffers before calling the same encoder again. Transform option encoding are not reentrant. Formatter and document encoders return owned buffers. Conversion from SWCD documents to Prettier Docs belongs to the Prettier plugin.

## Testing

From the workspace root, `pnpm build:js` builds the API JavaScript entry points. `pnpm test` and `pnpm test:wasm` cover the protocols through the core and formatter APIs; `pnpm test:codec` covers AST codecs.

AST storage and generated projections belong to `@swc-next/ast`.
