# `@swc-next/encoder`

Encodes an ESTree-compatible `Program` into binary sections consumed by
SWC Next crates.

**For internal SWC Next use only.** This package's API and binary format do not follow semantic versioning and may change in any release. Use matching Rust and JavaScript versions.

```js
import { encode } from "@swc-next/encoder";

const parts = encode(program);
```

`encode(program)` infers JavaScript or TypeScript from the nodes and reads
source type from `program.sourceType`. `encodeJs` and `encodeTs` are also
available for callers that know whether TypeScript support is needed.

Node spans use UTF-16 offsets. String literal values and cooked template values
preserve lone surrogates through the WTF-8 pool. UTF-8 fields, including
identifiers and raw source text, replace lone surrogates with U+FFFD.
