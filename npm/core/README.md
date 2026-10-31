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

`transformSync` shares its options and behavior with
[`@swc-next/transform`](../transform/README.md). Pass options use the Rust
structures directly, with Rust handling defaults and validation:

```js
transformSync("const answer: number = 42;", {
  filename: "input.ts",
  typescript: { verbatimModuleSyntax: true },
  es2022: { staticBlocks: true },
  codegen: { minify: true },
});
```

TypeScript conversion is enabled by default for TypeScript input. Use
`typescript: {}` for Rust defaults or `typescript: false` to retain TS syntax.
`parser` and `codegen` use the same field names as `parseSync` and `printSync`.
`parser.lang` overrides filename inference; `parser.sourceType` defaults to
`"module"`. Transform requires semantic checks and omits comments.

JSX is preserved. General target lowering, comment pragmas, source maps, and
configuration file discovery are not supported. `codegen.minify` compacts
printing without compression or mangling. Unsupported options throw an error.

Syntax and semantic errors throw `SyntaxError` with a `diagnostics` array
containing UTF-16 source offsets. The input is parsed and checked; ASTs edited by
`before` plugins are checked again for semantic errors. Parser-only context
checks, such as the placement of `new.target` and `import.meta`, are not rerun
on edited ASTs. Edits from `after` plugins are printed without another semantic
check. Plugin exceptions propagate to the caller.

Known limitation: moving declarations in a `before` plugin can produce incorrect
enum constant folding because Rust transforms still use original source positions
for some semantic decisions. See [#1259](https://github.com/swc-project/swc-next/issues/1259).

For separate parsing, editing, and printing, use `parseSync`, `walk`, and
`printSync`. Pass the parser result's `meta` to `printSync` to retain its language
and source type.

`@swc-next/core-browser` does not expose `transformSync`.

## Testing

Run from the workspace root:

```sh
pnpm build
pnpm test
```

Build the WebAssembly bindings with `pnpm build:wasm`, then run the same tests
with `pnpm test:wasm`.
