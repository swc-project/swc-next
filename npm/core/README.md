# @swc-next/core

The Node.js API for SWC Next, a fast JavaScript and TypeScript toolchain written in Rust. Parse source, traverse and edit ESTree-based ASTs, and generate code.

## Documentation

See the [Core API documentation](https://next.swc.rs/api/javascript/core) for details.

`transformSync` compiles source through Rust core's `parser`, `transformer`, and
`generator` configuration. The focused `@swc-next/transformer` package exposes the
same API.

```js
import { transformSync } from "@swc-next/core";

const { code } = transformSync("const value: number = 1;", {
  filename: "input.ts",
  transformer: { typescript: {} },
});
```
