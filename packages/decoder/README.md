# `@swc-next/decoder`

Decodes the binary output of an SWC Next parser crate into an ESTree-compatible program and comments.

The binary format is internal and versioned with the Rust crates. Depend on the exact package version that matches the `swc_next_*` crate versions used by your native binding.

```js
import { decode } from "@swc-next/decoder";

const result = decode(nativeParse(source), source);
```
