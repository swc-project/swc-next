---
title: Core
description: Core package exports for Node.js and browser WebAssembly.
---

# Core

The core packages provide synchronous parsing and code generation with the same function signatures and option types in Node.js and browsers.

## Packages

| Package | Runtime | Initialization |
| --- | --- | --- |
| `@swc-next/core` | Node.js | Ready after import |
| `@swc-next/core-browser` | Browser WebAssembly | Await the default `init()` export |

## Exports

| Export | Purpose | Reference |
| --- | --- | --- |
| `parseSync(source, options?)` | Parse source text into a program, comments, and diagnostics | [Parser](/api/javascript/parser) |
| `generateSync(program, options?)` | Generate a `{ code }` result from a program | [Generator](/api/javascript/generator) |
| `walk(root, hooks, state?)` | Traverse and edit an AST synchronously; exported by the Node.js package | [Walk](/api/javascript/walk) |
| `langFromPath(path)` | Infer the parser language from a file path | [Parser helpers](/api/javascript/parser#path-helpers) |
| `sourceTypeFromPath(path)` | Infer the source type from a file path | [Parser helpers](/api/javascript/parser#path-helpers) |

For browser traversal, import `walk` from `@swc-next/walker`. That package is pure JavaScript and requires no initialization. `createWalker` and `visitorKeys` are also available from the walker package directly.

## Browser WebAssembly

The browser package includes the WebAssembly binary and its JavaScript bindings. Its default export initializes the binding:

```ts
export default function init(): Promise<void>;
```

Await `init()` before calling `parseSync` or `generateSync`. It loads the bundled `.wasm` file; after successful initialization, subsequent calls are idempotent no-ops. Serve the package with a bundler that resolves its WebAssembly asset.

Parsing and printing are synchronous after initialization. Run substantial workloads in a worker to keep the interface responsive.

## Type imports

Import types alongside the functions they configure:

```ts
import { parseSync, type ParseApiOptions } from "@swc-next/core";

const options = {
  lang: "ts",
  comments: "all",
} satisfies ParseApiOptions;

const result = parseSync("const answer: number = 42;", options);
```

`satisfies` checks option names and values while preserving their inferred types. The browser package exports the same types from `@swc-next/core-browser`. Both packages also re-export [AST node types](/api/javascript/types), such as `Program` and `Node`.

## Source transformation

`transformSync` compiles source synchronously through Rust's parser, transformer,
and generator. The same API is available from `@swc-next/transformer`.

```ts
import { transformSync, type TransformApiOptions } from "@swc-next/core";

const options = {
  filename: "input.ts",
  parser: { semanticErrors: false },
  transformer: { typescript: {} },
  generator: { minify: true },
  sourceMap: true,
} satisfies TransformApiOptions;

const { code, map, diagnostics } = transformSync("const answer: number = 42;", options);
// code: "const answer=42;"
```

`filename` infers the input language and source type; explicit `parser.lang` and
`parser.sourceType` override it. TypeScript and JSX lowering follow the input
language by default; `transformer.typescript: false` or `transformer.jsx: false`
retains that syntax. `generator.minify` only controls compact printing.
`transformSync` does not perform AST compression or variable mangling.

`parser.semanticErrors` defaults to `false`. Binding analysis required by Rust
transforms still runs. Parser, enabled semantic, and transform errors throw
`SyntaxError` with UTF-16 diagnostics; invalid or unsupported options throw
`TypeError`. Source maps are JSON strings in `map`. This API is currently provided
by the Node.js package with a Wasm fallback.
