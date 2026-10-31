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

`@swc-next/core` exposes the same `transformSync` API. Pass options use the
Rust structures directly, with Rust handling defaults and validation:

```js
transformSync("const answer: number = 42;", {
  filename: "input.ts",
  parser: { sourceType: "module" },
  typescript: {},
  es2022: { staticBlocks: true },
  codegen: { asciiOnly: true, minify: true },
});
// code: "const answer=42;export{};"
```

`filename` infers JavaScript, JSX, TypeScript, TSX, or declaration syntax unless
`parser.lang` is provided. `parser.sourceType` defaults to `"module"` regardless
of the filename. Parser and codegen field names match `parseSync` and
`printSync`; transform requires semantic checks and omits comments.

TypeScript conversion is enabled by default for TypeScript input. Use
`typescript: {}` for Rust defaults or `typescript: false` to retain TS syntax.
The options follow [TypeScriptOptions](../../crates/swc_ecma_transformer/src/typescript/options.rs),
[ES2022Options](../../crates/swc_ecma_transformer/src/es2022/mod.rs), and
[ProposalOptions](../../crates/swc_ecma_transformer/src/proposal.rs).
`typescript.jsx` controls retention of JSX factory imports; JSX itself is
preserved for a later transform.

General target lowering, comment pragmas, source maps, and configuration file
discovery are not supported. `codegen.minify` compacts printing without
compression or mangling. Unsupported options throw an error.

Syntax and semantic errors throw `SyntaxError` with a `diagnostics` array
containing UTF-16 source offsets. The input is parsed and checked; ASTs edited by
`before` plugins are checked again for semantic errors. Parser-only context
checks, such as the placement of `new.target` and `import.meta`, are not rerun
on edited ASTs. Edits from `after` plugins are printed without another semantic
check. Plugin exceptions propagate unchanged.

Known limitation: moving declarations in a `before` plugin can produce incorrect
enum constant folding because Rust transforms still use original source positions
for some semantic decisions. See [#1259](https://github.com/swc-project/swc-next/issues/1259).

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
