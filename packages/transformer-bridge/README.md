# `@swc-next/transformer-bridge`

JavaScript glue for SWC Next's native and WebAssembly source transformation
bindings. It encodes source and transform configuration, and decodes code,
source maps, and diagnostics.

The package depends on `@swc-next/encoder` and `@swc-next/decoder` to reuse
`encodeGeneratorConfig` and `encodeParserConfig`. Those two packages remain
independent. `@swc-next/transformer`, `@swc-next/core`, and
`@swc-next/core-browser` use this bridge.

```js
import {
  encodeSource,
  encodeTransformApiOptions,
  decodeJsOutput,
} from "@swc-next/transformer-bridge";
import { transformSync as nativeTransformSync } from "./binding.js";

export function transformSync(source, options) {
  const config = encodeTransformApiOptions(options);
  const output = nativeTransformSync(encodeSource(source), source.length, config);
  return decodeJsOutput(output);
}
```

Source and configuration encoders each reuse scratch storage. Consume returned
buffers synchronously before calling the same encoder again; do not retain or
modify them. Configuration encoding shares state across API entry points. Option
getters and Proxy traps must not reenter it, including via any `transformSync`.

This package is intended for internal SWC Next use. Use the exact version matching
the native or WebAssembly binding. For application code, use
`@swc-next/transformer` or `@swc-next/core`.

Future JavaScript plugin runner code will live in this package alongside the
transformation bridge. Plugin execution is not implemented yet.
