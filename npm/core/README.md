# `@swc-next/core`

Parse, transform, and print JavaScript and TypeScript with SWC Next. JavaScript
plugins can inspect and edit the complete ESTree AST before or after Rust
transforms.

```js
import { transformSync } from "@swc-next/core";

const { code, diagnostics } = transformSync("debugger; run();", {
  filename: "input.js",
  plugins: [{
    name: "remove-debugger",
    visitors: {
      DebuggerStatement(node, ctx) {
        if (ctx.index !== null) ctx.remove();
        else ctx.replace({ type: "EmptyStatement" });
      },
    },
  }],
});
// code: "run();\n"
```

Each plugin runs a complete walk in list order within its phase. The default
phase is `"before"`; use `phase: "after"` to edit the transformed AST. Hooks use
the [walker API](../../packages/walker/README.md), must be synchronous, and must
return `undefined`. A `Program` hook receives the entire file's AST.
The Rust boundary reconstructs the AST; custom node properties do not cross it.
Stages without plugins stay in Rust: a before-only or after-only `plugins`
array needs one AST round trip, while using both phases needs two. When
`plugins` is omitted or empty, the entire pipeline stays in Rust.

`filename` determines the language and source type unless overridden by
`parser`; without a filename, input is parsed as a JavaScript module.
TypeScript inputs have their types removed by default; set
`transform: { typescript: false }` to retain them. Static block lowering is
available with `transform: { staticBlocks: true }`. Use `codegen` for printing
options. JSX is preserved, but comments and JSX pragmas are not retained.
Source maps and general syntax target selection are not supported.

Syntax and semantic errors throw `SyntaxError` with a `diagnostics` array
containing UTF-16 source offsets. Plugin exceptions propagate to the caller.

For separate parsing, editing, and printing, use `parseSync`, `walk`, and
`printSync`. Pass the parser result's `meta` to `printSync` to retain its language
and source type.

## Testing

Run from the workspace root:

```sh
pnpm build
pnpm test
```

Build the WebAssembly bindings with `pnpm build:wasm`, then run the same tests
with `pnpm test:wasm`.
