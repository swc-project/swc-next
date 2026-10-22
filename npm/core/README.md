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
[`@swc-next/transform`](../transform/README.md). It accepts a strict subset of
`@swc/core` configuration, plus `plugins` and explicit `swcNext` extensions:

```js
transformSync("const answer: number = 42;", {
  filename: "input.ts",
  isModule: true,
  jsc: {
    parser: { syntax: "typescript" },
    target: "esnext",
    transform: { verbatimModuleSyntax: true },
  },
});
```

`filename` infers the input language unless `jsc.parser` is provided. Module
mode defaults to `isModule: true` regardless of the filename; use `false` for
scripts or `"commonjs"` for CommonJS. The default target is `"esnext"`, which is
also the only supported target. This differs from SWC's default of ES5.
Top-level `minify` only compacts printing. JSX is preserved; comments, JSX
pragmas, source maps, and configuration file discovery are not supported.
Unknown fields, unsupported values, and the former `parser`, `transform`, and
`codegen` options throw an error identifying the configuration path.

Syntax and semantic errors throw `SyntaxError` with a `diagnostics` array
containing UTF-16 source offsets. The input and AST edited by `before` plugins
are checked; edits from `after` plugins are printed without another semantic
check. Plugin exceptions propagate to the caller.

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
