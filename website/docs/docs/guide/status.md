---
title: "Project status"
description: "A toolchain in active development, with different capabilities available through Rust, Node.js, and the browser."
---

# Project status

A toolchain in active development, with different capabilities available through Rust, Node.js, and the browser.

## Available today

- Parse and print JavaScript, TypeScript, JSX, and TSX.
- Traverse and edit JavaScript ASTs with synchronous hooks.
- Use Rust components for semantic analysis, transformations, minification, type stripping, and isolated declarations.

## Current boundaries

- Rust APIs are at an early stage and have known AST handle soundness issues. Use the existing SWC Rust API for production.
- The JavaScript API does not yet expose every Rust component.
- SWC configuration compatibility has not been implemented.

## Related

[Tool overview](/tools/) · [API reference](/api/)
