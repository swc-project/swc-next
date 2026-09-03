# `@swc-next/encoder`

Encodes an ESTree-compatible `Program` into the binary format consumed by SWC Next crates.

The binary format is internal and versioned with the Rust crates. Depend on the exact package version that matches the `swc_next_*` crate versions used by your native binding.

```js
import { encode } from "@swc-next/encoder";
import { printSync as __printSync } from "./binding.js";

export function printSync(program, options) {
  return __printSync(encode(program), options);
}
```
