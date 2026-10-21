---
title: Project status
description: Implementation status across the JavaScript packages and native compiler components.
---

# Project status

Parsing, AST editing, and printing are available in the JavaScript packages. Other stages of the compiler have different levels of implementation and JavaScript exposure.

The [Roadmap](https://github.com/swc-project/swc-next/issues/181) tracks upcoming work. [Conformance results](/guide/testing#conformance) report how the current implementation performs against upstream test suites.

## Feature status

The JavaScript column covers exported package APIs; Rust covers the native compiler components.

- **Implemented**: usable for the scope listed in the row.
- **Partial**: a working subset with missing functionality.
- **Not started**: no implementation in that interface yet.

These labels describe feature availability, not conformance pass rates or API stability.

| Feature | JavaScript | Rust | Scope and limitations |
| --- | --- | --- | --- |
| [Parsing](/api/javascript/parser) | Implemented | Implemented | JavaScript, TypeScript, JSX, TSX, and declaration files. |
| [AST traversal and editing](/api/javascript/walk) | Implemented | Implemented | Structural traversal, visitor hooks, and node edits. |
| [Code generation](/api/javascript/codegen) | Implemented | Implemented | Readable or compact output, ASCII-only output, and inline-script escaping. |
| Parser diagnostics and comments | Implemented | Implemented | Syntax recovery, diagnostic spans, and comment collection. |
| Semantic early errors | Implemented | Implemented | Enable `semanticErrors` in the JavaScript parser options. |
| [Scopes, symbols, and references](/api/components/semantic) | Not started | Implemented | The full semantic model is separate from parser early-error diagnostics. |
| [Syntax transformations](/api/components/transformer) | Not started | Partial | TypeScript lowering and class static blocks; no general browser-target compilation pipeline. |
| [Minification](/api/components/minifier) | Not started | Partial | Selected compression passes. Name mangling is not implemented; the printer's `minify` option only controls formatting. |
| [Type stripping](/api/components/type-strip) | Not started | Implemented | The `erasableSyntaxOnly` subset, preserving UTF-16 length and line layout. |
| [Isolated declarations](/api/components/isolated-declarations) | Not started | Implemented | Declaration-surface generation, without full TypeScript type checking. |
| Comment and source-map emission | Not started | Implemented | JavaScript `printSync` currently returns code only. |
| SWC configuration compatibility | Not started | Not started | Selected `.swcrc` options are planned. |
