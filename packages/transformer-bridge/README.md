# `@swc-next/transformer-bridge`

JavaScript glue for SWC Next's native and WebAssembly source transformation
bindings. It encodes transform configuration, decodes code, source maps, and
diagnostics, and wraps a binding as a synchronous source transformation API.

The package depends on `@swc-next/encoder` and `@swc-next/decoder` to reuse
`encodeGeneratorConfig` and `encodeParserConfig`. Those two packages remain
independent. `@swc-next/transformer`, `@swc-next/core`, and
`@swc-next/core-browser` use this bridge.

```js
import { createTransformSync } from "@swc-next/transformer-bridge";
import { transformSync as nativeTransformSync } from "./binding.js";

export const transformSync = createTransformSync(nativeTransformSync);
```

`encodeJsTransformApiOptions(options)` and `decodeJsTransformOutput(output)` are
also available for direct configuration and result conversion. Encoded
configuration uses reusable scratch storage and must be consumed before the next
encoding call. Option getters and Proxy traps must not reenter the active encoder.

This package is intended for internal SWC Next use. Use the exact version matching
the native or WebAssembly binding. For application code, use
`@swc-next/transformer` or `@swc-next/core`.

Future JavaScript plugin runner code will live in this package alongside the
transformation bridge. Plugin execution is not implemented yet.
