# @swc-next/formatter-prettier-plugin

JavaScript and TypeScript formatting for Prettier, powered by SWC Next. Supports JSX, TSX, and parser selection by file extension.

**Experimental and unstable.** Syntax support is incomplete, and output can differ from Prettier's built-in formatters.

## Usage

```js
import * as prettier from "prettier";
import * as swc from "@swc-next/formatter-prettier-plugin";

const code = await prettier.format("const answer:number=42", {
  plugins: [swc],
  parser: "swc-ts",
});
```

Use `swc` for JavaScript and JSX, `swc-ts` for TypeScript, and `swc-tsx` for TSX. Set `filepath` instead of `parser` to infer the language, including declaration files and CommonJS extensions.

## Limitations

Whole-file formatting is supported in Node.js through native or WebAssembly bindings. Browser standalone is not packaged.

Range formatting, pragma options, and embedded-language formatting are unsupported. The plugin also rejects `objectWrap: "collapse"`, `experimentalTernaries`, and `experimentalOperatorPosition: "start"`.

## Testing

From the workspace root, run `pnpm --filter @swc-next/formatter-prettier-plugin test`, or `pnpm --filter @swc-next/formatter-prettier-plugin test:wasm` with WebAssembly bindings built.

Both commands generate the [conformance report](test/prettier-plugin-conformance.json), comparing SWC and Prettier printers and the SWC plugin with Prettier's built-in formatters.
