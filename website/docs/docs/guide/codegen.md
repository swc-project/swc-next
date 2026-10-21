---
title: Codegen
description: Generate JavaScript and TypeScript source from an AST.
---

# Codegen

Use `printSync` to turn a program AST back into source. Import it from `@swc-next/core`, the focused `@swc-next/codegen` package, or the initialized browser package.

## Print an edited program

```js
import { parseSync, printSync } from "@swc-next/core";

const { program } = parseSync("const answer = 1;");
program.body[0].declarations[0].init.value = 42;

console.log(printSync(program).code); // const answer = 42;
```

## Compact output

```js
import { parseSync, printSync } from "@swc-next/core";

const { program } = parseSync("const answer = 42;");
console.log(printSync(program, { minify: true }).code); // const answer=42;
```

`minify` controls printing and whitespace. It does not perform the optimizations described under [Minifier](/api/components/minifier). Printing TypeScript also preserves types.

## Reference

See the [Codegen API](/api/javascript/codegen) for AST and metadata inputs, the output shape, and all formatting options. For a complete workflow, start with [Parser](/guide/parser) and [Walk](/guide/walk).
