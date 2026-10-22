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

`@swc-next/core` exposes the same `transformSync` API. Both accept a strict
subset of `@swc/core` configuration, plus `plugins` and explicit `swcNext`
extensions:

```js
transformSync("const answer: number = 42;", {
  filename: "input.ts",
  isModule: true,
  jsc: {
    parser: { syntax: "typescript" },
    target: "esnext",
    transform: { verbatimModuleSyntax: true },
    output: { charset: "ascii" },
  },
  minify: true,
});
// code: "const answer=42;export{};"
```

`filename` infers JavaScript, JSX, TypeScript, or TSX when `jsc.parser` is
omitted; without either, input is JavaScript. An explicit parser selects
`{ syntax: "ecmascript", jsx?: boolean }` or
`{ syntax: "typescript", tsx?: boolean }` and overrides filename inference.
TypeScript declaration filenames select ambient parsing with TypeScript syntax.
These filename rules follow SWC Next. Module mode defaults to
`isModule: true` regardless of the filename. Use `false` for scripts or
`"commonjs"` for CommonJS; `"unknown"` is not supported.

The default target is `"esnext"`, which is also the only supported target.
This differs from SWC's default of ES5. Top-level `minify` compacts printing;
it does not compress or mangle. `jsc.output.charset` accepts `"ascii"` or
`"utf8"` and defaults to `"utf8"`.

TypeScript types are removed by default. `jsc.transform.verbatimModuleSyntax`
preserves imports and exports without explicit type modifiers. In SWC Next it
also preserves const enum declarations and disables their inlining, so its
const enum behavior differs from `@swc/core`.

The `swcNext` extensions expose `typescript: false` to retain TypeScript,
`staticBlocks: true` to lower static blocks, `preserveParens` to control
parentheses in plugin ASTs, and `inlineScript` to escape closing script tags.
`swcNext.preserveConstEnums: true` retains
const enum declarations while allowing safe local uses to be inlined; it is
not equivalent to SWC's `treatConstEnumAsEnum`.

Unknown fields and unsupported values throw an error identifying the
configuration path. This includes the former top-level `parser`, `transform`,
and `codegen` options, as well as unsupported SWC options such as `env`,
`module`, `jsc.minify`, and `jsc.transform.react`. JSX is preserved. Comments,
JSX pragmas, source maps, and configuration file discovery are not supported.
Matching a configuration field does not imply full SWC behavior compatibility.

Syntax and semantic errors throw `SyntaxError` with a `diagnostics` array
containing UTF-16 source offsets. The input and AST edited by `before` plugins
are checked; edits from `after` plugins are printed without another semantic
check. Plugin exceptions propagate unchanged.

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
