---
title: "Browser & WebAssembly"
description: "Use @swc-next/core-browser to run the parser and printer in the browser."
---

# Browser & WebAssembly

Use @swc-next/core-browser to run the parser and printer in the browser.

## Initialization

- Call init() before parsing or printing.

## Runtime considerations

- The package includes its WebAssembly binary. Use a worker for work that should not block the interface.

```js
import init, { parseSync, printSync } from "@swc-next/core-browser";

await init();
const { program, meta } = parseSync("const answer = 42;");
console.log(printSync(program, {}, meta).code);
```

## Related

[Tool overview](/tools/) · [API reference](/api/)
