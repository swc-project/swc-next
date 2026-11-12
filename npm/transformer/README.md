# @swc-next/transformer

Transform JavaScript, TypeScript, and JSX through a synchronous native API, with
a WebAssembly fallback. Parsing, built-in transforms and printing
run in Rust.

```js
import { transformSync } from "@swc-next/transformer";
// The same API is available from "@swc-next/core".

const result = transformSync("const answer: number = 42;", {
  filename: "input.ts",
  transformer: { typescript: {} },
});
// { code: "const answer = 42;\n", diagnostics: [] }
```

Configuration uses `TransformApiOptions`: `parser: ParserConfig`,
`transformer: TransformerConfig`, and `generator: GeneratorConfig` configure each stage. `filename` infers the input language and source
type; explicit `parser.lang` and `parser.sourceType` take precedence. Without a
filename, the defaults are JavaScript and module source type.

TypeScript and JSX lowering are enabled for their input languages by default.
Use `transformer.typescript: false` or `transformer.jsx: false` to preserve that syntax.
Source comments default to `"all"`; `comments: "none"` removes
them. Set `sourceMap: true` to return a JSON source map in `result.map`. `generator.minify` controls compact printing; `transformSync` does not accept
`minify` or run compression and mangling. The independent package does not depend
on the minifier.

Unsupported options throw `TypeError`. JavaScript plugins and general target
lowering are not supported by this package yet. Additional semantic error checks
are controlled by `parser.semanticErrors` and default to `false`, as in
`parseSync`; the binding analysis needed by Rust transforms still runs. Parser,
enabled semantic, and transform errors throw `SyntaxError` with a `diagnostics`
array containing UTF-16 source offsets.

## Testing

API and FFI tests live in `@swc-next/core`. From the workspace root:

```sh
pnpm build
pnpm test
pnpm build:wasm
pnpm test:wasm
```
