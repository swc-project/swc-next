# `@swc-next/transform`

Transform JavaScript and TypeScript with SWC Next through a standalone native
binding, with a WebAssembly fallback. JavaScript plugins can inspect and edit
the complete ESTree AST before or after Rust transforms.

```js
import { transformSync } from "@swc-next/transform";

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

Each plugin completes its walk in array order within its phase. The default
phase is `"before"`; use `phase: "after"` to edit the transformed AST. Hooks use
the [walker API](../../packages/walker/README.md), must be synchronous, and must
return `undefined`. A `Program` hook receives the entire file's AST. Custom
node properties do not cross the Rust boundary.

Stages without plugins stay in Rust: no plugins means no AST transfers; one
plugin phase needs one AST round trip, and both phases need two.

`filename` determines the language and source type unless overridden by
`parser`; without a filename, input is parsed as a JavaScript module.
TypeScript types are removed by default; set `transform: { typescript: false }`
to retain them. Static block lowering is available through
`transform: { staticBlocks: true }`. JSX is preserved. Comments, JSX pragmas,
source maps, and general syntax target selection are not supported.

Syntax and semantic errors throw `SyntaxError` with a `diagnostics` array
containing UTF-16 source offsets. Plugin exceptions propagate unchanged.

## Testing

API and FFI tests are maintained in `@swc-next/core`. Run from the workspace root:

```sh
pnpm build
pnpm test
```

Build the WebAssembly bindings with `pnpm build:wasm`, then run the same tests
with `pnpm test:wasm`. To build only this package, use
`pnpm --filter @swc-next/transform build` or
`pnpm --filter @swc-next/transform build:wasm`.
