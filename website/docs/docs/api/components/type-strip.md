---
title: "Type strip"
description: "Erase the supported TypeScript syntax while preserving source positions."
---

# Type strip

Erase the supported TypeScript syntax while preserving source positions.

## Behavior

- Output preserves UTF-16 length and line layout.
- Inspect diagnostics before using recovery output.

## Current boundaries

- Available through Rust for the erasableSyntaxOnly subset.
- Type stripping is distinct from general TypeScript lowering.

## Related

[Choosing packages](/guide/packages) · [API reference](/api/)
