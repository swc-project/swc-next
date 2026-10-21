---
title: API reference
description: JavaScript interfaces for parsing, AST traversal, and code generation.
---

# API reference

Parse source text into an AST compatible with [ESTree](https://github.com/estree/estree), inspect or edit its nodes, and generate code with SWC Next's JavaScript packages.

## JavaScript

| API | Functions and types |
| --- | --- |
| [Core](/api/javascript/core) | Node.js and browser packages, exports, and WebAssembly initialization |
| [Parser](/api/javascript/parser) | `parseSync`, parser options, results, and diagnostics |
| [Walk](/api/javascript/walk) | `walk`, visitor hooks, traversal context, and `createWalker` |
| [Codegen](/api/javascript/codegen) | `printSync`, formatting options, and generated output |
| [AST types](/api/javascript/types) | Node declarations and source positions |

## Compiler components

[Compiler components](/api/components/) operate on the flat AST to resolve names, lower syntax, optimize programs, and generate declarations.
