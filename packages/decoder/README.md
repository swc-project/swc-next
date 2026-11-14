# `@swc-next/decoder`

Decodes the binary output of an SWC Next parser crate into an ESTree-compatible program, comments, and parser diagnostics.

**For internal SWC Next use only.** This package's API and binary format do not follow semantic versioning and may change in any release. Use matching Rust and JavaScript versions.

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
