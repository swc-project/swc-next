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

## TypeScript exports

Both packages export AST declarations from `@swc-next/types`, plus `ParserOptions`, `ParseResult`, `Comment`, `CommentMode`, `Diagnostic`, `DiagnosticLabel`, `CodegenOptions`, `CodegenOutput`, and `AstMeta`.

`Lang`, `SourceType`, and `CommentMode` are types, not runtime enum objects. Use string values such as `{ lang: "ts", sourceType: "module", comments: "flat" }`.

## Focused packages

Use `@swc-next/parser` for parsing only, `@swc-next/codegen` for printing only, and `@swc-next/walker` for traversal. The parser and printer use the same function signatures as the core packages.
