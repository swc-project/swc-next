---
title: Walk
description: Inspect and edit ASTs with synchronous visitor hooks.
---

# Walk

Use `walk` to visit AST nodes and edit them in place. It is exported by `@swc-next/core` and the standalone `@swc-next/walker` package.

## Remove debugger statements

Hooks are named after node types. A function runs when the walker enters that node:

```js
import { parseSync, printSync, walk } from "@swc-next/core";

const { program } = parseSync("debugger; const answer = 42;");
walk(program, {
  DebuggerStatement(node, context) {
    context.remove();
  },
});
console.log(printSync(program).code); // const answer = 42;
```

Use context methods for structural edits. Simple values, such as an identifier's `name`, can be edited directly. Hooks must be synchronous.

## Collect information

Pass state as the third argument to share a result across hooks:

```js
import { parseSync, walk } from "@swc-next/core";

const { program } = parseSync('import value from "example";');
const dependencies = [];
walk(program, {
  ImportDeclaration(node, context) {
    context.state.push(node.source.value);
  },
}, dependencies);
console.log(dependencies); // ["example"]
```

## Next steps

[Generate code](/guide/codegen) from the edited AST.

See the [Walk API](/api/javascript/walk) for inputs and return values, enter/leave hooks, context methods, and reusable walkers.
