---
title: Install
description: Install SWC Next for Node.js or browser WebAssembly.
---

# Install

Choose the package for your runtime. The following guides show how to parse source, walk the AST, and generate code.

## Node.js

Install the combined package:

```sh
npm install @swc-next/core
```

It provides `parseSync`, `printSync`, and `walk`:

```js
import { parseSync, printSync, walk } from "@swc-next/core";
```

Use ESM imports, for example in an `.mjs` file. See the [Core API](/api/javascript/core) for its exports.

## Browser WebAssembly

Install the browser package and, if you need AST traversal, the standalone walker:

```sh
npm install @swc-next/core-browser @swc-next/walker
```

Call `init()` once before parsing or printing. It loads the WebAssembly binary bundled with the package.

```js
import init, { parseSync, printSync } from "@swc-next/core-browser";
import { walk } from "@swc-next/walker";

await init();

const { program, meta } = parseSync("debugger; const answer = 42;");
walk(program, {
  DebuggerStatement(node, context) {
    context.remove();
  },
});
console.log(printSync(program, {}, meta).code);
```

Use a bundler that serves the package's `.wasm` asset. The walker is pure JavaScript and does not require initialization. See the [Browser API](/api/javascript/core#browser-webassembly) for initialization and runtime details.

## Focused packages

Install these separately when you only need part of the pipeline:

| Package | Purpose | Reference |
| --- | --- | --- |
| `@swc-next/parser` | Parse source in Node.js | [Parser API](/api/javascript/parser) |
| `@swc-next/codegen` | Generate source in Node.js | [Codegen API](/api/javascript/codegen) |
| `@swc-next/walker` | Traverse and edit ASTs in either runtime | [Walk API](/api/javascript/walk) |
| `@swc-next/types` | Shared TypeScript AST declarations | [AST types](/api/javascript/types) |

Continue with [Parser](/guide/parser), [Walk](/guide/walk), and [Codegen](/guide/codegen). Examples use `@swc-next/core`; in the browser, import parsing and printing from `@swc-next/core-browser` after initialization, and walking from `@swc-next/walker`.
