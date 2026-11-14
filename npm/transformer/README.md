# @swc-next/transformer

A JavaScript and TypeScript source transformer written in Rust. Lower TypeScript
and JSX with SWC Next.

```js
import { transformSync } from "@swc-next/transformer";

const { code, diagnostics } = transformSync("const answer: number = 42;", {
  filename: "input.ts",
});
console.log(code); // const answer = 42;
```

The filename determines the language and source type unless `parser` overrides
them. TypeScript and JSX transforms are enabled for their respective languages.
Transformation settings are not configurable through the JavaScript API.
Other JavaScript syntax and module formats are preserved. Syntax errors are
returned in `diagnostics`; their
positions use UTF-16 offsets in the original source.

Set `sourceMap: true` to receive a JSON source map in `map`. Comments are preserved
by default. See the [Transformer API documentation](https://next.swc.rs/api/javascript/transformer)
for options.

## Building

```sh
pnpm --filter @swc-next/transformer build
pnpm --filter @swc-next/transformer build:wasm
```

The Wasm build requires the `wasm32-unknown-unknown` Rust target. Both builds use
the same synchronous API in Node.js. Set `NAPI_RS_FORCE_WASI=error` to require the
Wasm binding.
