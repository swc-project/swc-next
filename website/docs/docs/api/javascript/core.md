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
| `parseSync(source, options?)` | Parse source text into a program, comments, diagnostics, and metadata | [Parser](/api/javascript/parser) |
| `printSync(program, options?, meta?)` | Generate a `{ code }` result from a program | [Codegen](/api/javascript/codegen) |
| `walk(root, hooks, state?)` | Traverse and edit an AST synchronously; exported by the Node.js package | [Walk](/api/javascript/walk) |
| `langFromPath(path)` | Infer the parser language from a file path | [Parser helpers](/api/javascript/parser#path-helpers) |
| `sourceTypeFromPath(path)` | Infer the source type from a file path | [Parser helpers](/api/javascript/parser#path-helpers) |

For browser traversal, import `walk` from `@swc-next/walker`. That package is pure JavaScript and requires no initialization. `createWalker` and `visitorKeys` are also available from the walker package directly.

## Browser WebAssembly

The browser package includes the WebAssembly binary and its JavaScript bindings. Its default export initializes the binding:

```ts
export default function init(): Promise<void>;
```

Await `init()` before calling `parseSync` or `printSync`. It loads the bundled `.wasm` file; after successful initialization, subsequent calls are idempotent no-ops. Serve the package with a bundler that resolves its WebAssembly asset.

Parsing and printing are synchronous after initialization. Run substantial workloads in a worker to keep the interface responsive.

## Type imports

Import types alongside the functions they configure:

```ts
import { parseSync, type ParserOptions } from "@swc-next/core";

const options = {
  lang: "ts",
  comments: "flat",
} satisfies ParserOptions;

const result = parseSync("const answer: number = 42;", options);
```

`satisfies` checks option names and values while preserving their inferred types. The browser package exports the same types from `@swc-next/core-browser`. Both packages also re-export [AST node types](/api/javascript/types), such as `Program` and `Node`.
