---
title: AST types
description: Shared ESTree-based declarations and source positions.
---

# AST types

`@swc-next/types` provides TypeScript declarations for the AST, including TypeScript, JSX, and error recovery nodes. These types are also re-exported by the core, parser, codegen, and browser packages.

```ts
import type { Node, Program, Lang, SourceType } from "@swc-next/types";
```

## Program and nodes

```ts
interface BaseNode {
  start: number;
  end: number;
}

interface Program extends BaseNode {
  type: "Program";
  body: Array<Directive | Statement>;
  hashbang: Hashbang | null;
  sourceType: SourceType;
}
```

`Node` is the union of supported node declarations, discriminated by `type`. Positions use UTF-16 code units with exclusive end offsets. Source positions on synthetic nodes are the caller's responsibility.

## Language and source type

```ts
type Lang = "js" | "ts" | "jsx" | "tsx" | "dts";
type SourceType = "module" | "script" | "commonjs" | "unambiguous";
```

These are type aliases, not runtime enums. Use string values with the [Parser API](/api/javascript/parser#parseroptions).

## Compatibility

The declarations describe SWC Next's own ESTree projection. Assignability from another parser's AST is not guaranteed. Use the [Walk API](/api/javascript/walk) for structural traversal and the [Codegen API](/api/javascript/codegen) for accepted program inputs.
