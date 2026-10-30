---
title: Parser
description: Parse JavaScript and TypeScript into an AST.
---

# Parser

Use `parseSync` to turn source text into an AST compatible with [ESTree](https://github.com/estree/estree). TypeScript and JSX nodes follow typescript-eslint's AST format. [Install](/guide/install) the package for your runtime first.

## Parse a program

```js
import { parseSync } from "@swc-next/core";

const { program, diagnostics } = parseSync("const answer = 42;");
console.log(program.type); // Program
console.log(diagnostics); // []
```

By default, `parseSync` throws the first diagnostic if parsing reports an error. Set `errorRecovery: true` to receive diagnostics and a recovered AST; inspect the diagnostics before transforming or printing it.

## Parse TypeScript or JSX

Choose the language explicitly:

```js
import { parseSync } from "@swc-next/core";

const result = parseSync("const answer: number = 42;", { lang: "ts" });
console.log(result.program.body[0].type); // VariableDeclaration
```

Use `jsx` or `tsx` for JSX input, and `dts` for declaration files. Parsing TypeScript preserves its types; it does not compile them away.

## Next steps

[Walk the AST](/guide/walk) or [generate code](/guide/codegen).

See the [Parser API](/api/javascript/parser) for input and output types, language and source type options, comments, diagnostics, and source positions.
