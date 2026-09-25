# `@swc-next/decoder`

Decodes the binary output of an SWC Next parser crate into an ESTree-compatible program, comments, and parser diagnostics.

The binary format is internal and versioned with the Rust crates. Depend on the exact package version that matches the `swc_next_*` crate versions used by your native binding.

```js
import { decode } from "@swc-next/decoder";
import { parseSync as __parseSync } from "./binding.js";

export function parseSync(source, options) {
  const sourceBuffer = new TextEncoder().encode(source);
  const buffer = __parseSync(sourceBuffer, source.length, options);
  return decode(buffer, source, sourceBuffer);
}
```

`decode` selects the JavaScript or TypeScript decoder from the buffer header. `decodeJs` and `decodeTs` are also exported for callers that already know the language.
