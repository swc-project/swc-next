# `@swc-next/encoder`

Encodes an ESTree-compatible `Program` into the binary format consumed by
SWC Next crates. Use the exact package version matching your native binding's
`swc_next_*` crates.

```js
import { encode } from "@swc-next/encoder";

const buffer = encode(program, meta);
```

Pass the decoder result's `meta: { lang, sourceType }` to preserve source
metadata. `encode(program)` supports TypeScript by default; `encodeJs` and
`encodeTs` are available when the language is known.

Node spans use UTF-16 offsets. String literal values and cooked template values
preserve lone surrogates through the WTF-8 pool. UTF-8 fields, including
identifiers and raw source text, replace lone surrogates with U+FFFD.
