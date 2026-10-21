---
title: Compiler components
description: Analysis, transformations, and optimization in the compiler pipeline.
---

# Compiler components

Name resolution, syntax lowering, optimization, and declaration generation share the same flat AST. These stages prepare a program for further transformation or output.

| Component | Capability |
| --- | --- |
| [Semantic analysis](/api/components/semantic) | Scopes, symbols, references, and module records. |
| [Transformer](/api/components/transformer) | Supported syntax lowering and AST transformations. |
| [Minifier](/api/components/minifier) | Program compression and optimization. |
| [Type strip](/api/components/type-strip) | Erase supported TypeScript syntax while preserving source positions. |
| [Isolated declarations](/api/components/isolated-declarations) | Generate TypeScript declaration surfaces. |

JavaScript APIs for these components are not yet available.
