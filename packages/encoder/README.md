# `@swc-next/encoder`

Encodes an ESTree-compatible `Program` into the binary format consumed by SWC Next crates.

The binary format is internal and versioned with the Rust crates. Depend on the exact package version that matches the `swc_next_*` crate versions used by your native binding.

```js
import { encode } from "@swc-next/encoder";
import { printSync as __printSync } from "./binding.js";

export function printSync(program, options, meta) {
  return __printSync(encode(program, meta), options);
}
```

The generator emits separate JavaScript/JSX and TypeScript/TSX encoders.
Pass the decoder result's `meta: { lang, sourceType }` to `encode(program, meta)`
to preserve source metadata. `encode(program)` retains full TypeScript support.
Use `encodeJs` or `encodeTs` when the AST language is already known; both accept
the same metadata. Regenerate with `cargo run -p ast_tools`.

String values are appended without deduplication to a UTF-8 field source
section. Lone surrogates are replaced with U+FFFD, matching `Buffer.write`
and `TextEncoder`.
The field source is not the original source text; node spans still describe
the original UTF-16 positions.
