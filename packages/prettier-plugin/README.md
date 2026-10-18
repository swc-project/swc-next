# @swc-next/prettier-plugin-swcfmt

Experimental JavaScript and TypeScript formatting through SWC and Prettier.
Rust constructs the formatting document; a compact binary transport carries it
through `@swc-next/formatter`, and Prettier performs the final layout. The
parser/printer format identifier is **SWCFMT**.

```js
import * as prettier from "prettier";
import * as swc from "@swc-next/prettier-plugin-swcfmt";

const code = await prettier.format(source, {
  plugins: [swc],
  parser: "swc-ts",
  printWidth: 100,
});
```

Parsers: `swc` (JS/JSX), `swc-ts` (TS), `swc-tsx` (TSX).
With no explicit parser, `filepath` selects the language. Declaration files and
CommonJS extensions are recognized. Built-in parser names are not overridden.

For project-wide use, add the plugin to your Prettier configuration and use
`overrides` to select these parsers for the corresponding extensions. Standard
Prettier formatting options and defaults apply. Prettier 3.9.9 is currently pinned.

The name `@swc-next/prettier-plugin-swc-parser` is reserved for a possible future
parser-only integration that supplies an AST to Prettier's AST printer. That
package is not implemented.

## Current scope

Whole-file formatting through Node.js native bindings or the existing Node Wasm
loader. Set `NAPI_RS_FORCE_WASI=error` to test the Wasm path after building it.
The transport package has no dependency on `@swc-next/core`.

Range formatting, pragma options, `objectWrap: collapse`, experimental ternaries,
and operators at the start of a line are rejected. `quoteProps: consistent` and
`singleAttributePerLine` are supported. Embedded languages remain as source;
this plugin does not yet implement `embed`. Cursor tracking uses Prettier's
whole-document fallback. Browser standalone is not yet packaged.

Output inherits SWC formatter's documented differences and existing idempotency
limitations; it is not a byte-identical replacement for Prettier's built-in JS
printer. Native hints for canonical measurement have no direct public Prettier
equivalent, so some width boundaries also differ from the native printer.
Forced-flat logical groups are folded into conditional mode bindings. Prettier normalizes input line endings before parsing;
the resolved output `endOfLine` also reaches Rust document construction. Raw
trailing spaces, including Markdown hard breaks in comments, are preserved
through literal line Docs. Errors carry source locations for Prettier's
code frames. See the [design notes](../../website/docs/PrettierPlugin.md) for details.

The binary document format and `formatDocBufferSync` are internal, version-matched
interfaces. Consumers should use the Prettier plugin API.

Tests compare every local formatter fixture and option set directly with the
native Rust printer on the same host-normalized input. A test-only N-API addon
calls `format()` and returns strings directly, preserving CR, LF, and EOF
whitespace. The test commands build this addon with the `test-native` Cargo
feature and use it as the native oracle for both native and Wasm plugin bindings.
