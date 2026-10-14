# `@swc-next/core-browser`

The browser WebAssembly API for SWC Next. The package includes its WebAssembly
binary and browser JavaScript bindings.

```js
import init, { parseSync, printSync } from "@swc-next/core-browser";

await init();

const { program } = parseSync("const value = 1;");
program.body[0].declarations[0].init.value = 2;
console.log(printSync(program).code);
```

Call `init()` before parsing or printing. It loads the bundled `.wasm` file;
subsequent calls are idempotent.

Parser options use strings, for example
`{ lang: "ts", sourceType: "module", comments: "flat" }`.
`Lang`, `SourceType`, and `CommentMode` are available only as TypeScript types.
