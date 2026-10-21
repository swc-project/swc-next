---
title: "Node.js"
description: "Start with @swc-next/core to parse, edit, and print a program."
---

# Node.js

Start with @swc-next/core to parse, edit, and print a program.

## Installation

- Install @swc-next/core with your package manager.

## Your first program

- Parse with parseSync, edit with walk, then emit code with printSync.

```sh
npm install @swc-next/core
```

```js
import { parseSync, printSync } from "@swc-next/core";

const { program, diagnostics, meta } = parseSync("const answer = 42;");
console.log(diagnostics);
console.log(printSync(program, {}, meta).code);
```

## Related

[Tool overview](/tools/) · [API reference](/api/)
