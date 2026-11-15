# @swc-next/formatter-prettier-plugin

**Experimental and unstable.** APIs, document formats, and formatting output may
change without notice.

JavaScript and TypeScript formatting through SWC and Prettier.
Rust constructs the formatting document; a compact binary transport carries it
through `@swc-next/formatter-js`, and Prettier performs the final layout. The
parser/printer format identifier is **SWCFMT**.

```js
import * as prettier from "prettier";
import * as swc from "@swc-next/formatter-prettier-plugin";

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
printer. The transport maps the formatter's layout operations to public
Prettier Docs. Prettier normalizes input line endings before parsing; the
resolved output `endOfLine` also reaches Rust document construction. Literal
line Docs preserve source whitespace, including Markdown hard breaks in
comments. Errors carry source locations for Prettier's code frames. See the
[design notes](../../website/docs/PrettierPlugin.md) for details.

The binary document format and `formatDocBufferSync` are internal, version-matched
interfaces. Consumers should use the Prettier plugin API.

Sources live in `src-js`; Rslib bundles the plugin into `dist`. Run
`pnpm build:js` from the workspace root to build the JavaScript entry points.
The plugin test commands rebuild these entry points before running.

`test/prettier-plugin-conformance.js` runs two comparisons for each local
formatter fixture and option set:

- SWC plugin + SWC formatter versus SWC plugin + Prettier formatter: compare
  the printers using SWC-produced documents.
- SWC plugin + Prettier formatter versus native Prettier + Prettier formatter:
  compare the language formatters using Prettier's printer.

Each run writes two corresponding sections to
[`prettier-plugin-conformance.json`](test/prettier-plugin-conformance.json).
Both sections group differences by fixture and show the options and both compared
outputs, or their first error lines. Matching cases are omitted.
CI checks report changes with `git diff`.

Run `pnpm --filter @swc-next/formatter-prettier-plugin test` for native bindings,
or `pnpm --filter @swc-next/formatter-prettier-plugin test:wasm` after building
the Wasm bindings. Both commands regenerate the report. Rust tests check
formatting, idempotency, and document serialization.
