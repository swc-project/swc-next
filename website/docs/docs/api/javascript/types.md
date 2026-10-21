---
title: AST types
description: ESTree and typescript-eslint compatible AST declarations and source positions.
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

The JavaScript API produces ASTs compatible with the [ESTree specification](https://github.com/estree/estree), using [typescript-eslint's AST format](https://typescript-eslint.io/packages/typescript-estree/ast-spec/) for TypeScript and JSX nodes. The exported declarations describe these node shapes, including error recovery nodes.

Set the parser's [`preserveParens`](/api/javascript/parser#parseroptions) option to `false` to match typescript-eslint's representation of parentheses. The default preserves explicit parenthesized expression and type nodes.
