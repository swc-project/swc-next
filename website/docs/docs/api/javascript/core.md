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

### JavaScript plugins

`transformSync` in `@swc-next/core` and `@swc-next/transformer` supports `plugins`.
`@swc-next/core-browser` does not support this option. Synchronous visitors edit
the ESTree / TS-ESTree Program. Same-stage visitors share one traversal;
callbacks run in plugin array order, so later plugins see earlier edits.

```js
transformSync('import value from "pkg"; debugger;', {
  plugins: [{
    name: "remove-debugger",
    stage: "afterParse",
    visitor: {
      DebuggerStatement(_node, ctx) {
        if (ctx.index !== null) ctx.remove();
        else ctx.replace({ type: "EmptyStatement" });
      },
    },
  }],
});
```

`afterParse` (the default) runs before built-in transforms, including TypeScript
erasure and JSX lowering. `beforeCodegen` receives the transformed program and
runs just before printing. An absent or empty list keeps the AST in Rust.
When `parser.semanticErrors` is enabled, input is checked before callbacks and
the AST is checked again after `afterParse` visitors. `beforeCodegen` edits are
printed without another semantic check.

Hooks must return `undefined` synchronously. Plugin exceptions propagate;
invalid input, enabled semantic errors, and built-in transform errors throw
`SyntaxError` before later hooks execute. Diagnostic spans are UTF-16 offsets;
the error message uses one-based line and column numbers. Without plugins,
errors are returned in `diagnostics`.

Traversal controls are shared within a stage. `skip()` suppresses remaining
enter callbacks and children, `remove()` suppresses the node's remaining hooks,
and `stop()` ends the stage's walk. `replace()` suppresses remaining callbacks
in the current phase. An enter replacement visits the replacement's children
and leave hooks without re-entering it. A leave replacement is not traversed.
The next stage starts an independent walk.

Nonempty plugins do not support source maps or preserved source comments.
Use `comments: "none"` to suppress output comments. `parser.comments` controls
collection during parsing; the built-in transforms can still use collected pragmas.
Parser-only context checks are not rerun after edits.
Source spans remain the plugin author's responsibility.
