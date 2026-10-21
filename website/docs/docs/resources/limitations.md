---
title: "Known limitations"
description: "Understand the boundaries of the current implementation."
---

# Known limitations

Understand the boundaries of the current implementation.

## Interfaces

- Rust AST handles are not tied to their owning AST and have known soundness issues.
- Not every Rust component has a public JavaScript API.

## Compatibility

- SWC configuration compatibility and minifier name mangling are not implemented.
- Semantic analysis does not perform complete TypeScript type checking.

## Related

[Project status](/guide/status) · [Flat AST](/resources/architecture/flat-ast)
