---
title: Compiler components
description: Semantic analysis, syntax transformation, compression, and declaration generation.
---

# Compiler components

These components analyze and transform programs inside the compiler. Their JavaScript APIs are not yet available; the examples show what each component does.

| Component | Purpose |
| --- | --- |
| [Semantic analysis](/api/components/semantic) | Resolve scopes, declarations, and references. |
| [Transformer](/api/components/transformer) | Lower supported TypeScript and JavaScript syntax. |
| [Minifier](/api/components/minifier) | Shorten expressions and remove unused local variables. |
| [Type strip](/api/components/type-strip) | Remove erasable TypeScript syntax while preserving source positions. |
| [Isolated declarations](/api/components/isolated-declarations) | Generate `.d.ts` declarations from TypeScript. |
