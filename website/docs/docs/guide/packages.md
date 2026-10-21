---
title: "Choosing packages"
description: "Choose the smallest public interface that fits your tool."
---

# Choosing packages

Choose the smallest public interface that fits your tool.

## JavaScript

- @swc-next/core combines parsing, printing, and walk.
- @swc-next/parser and @swc-next/codegen provide focused native packages.
- @swc-next/walker and @swc-next/types provide traversal and shared declarations.
- @swc-next/core-browser provides browser WebAssembly bindings.

## Rust

- Use individual swc_next_* crates for the components in your pipeline.

## Related

[Tool overview](/tools/) · [API reference](/api/)
