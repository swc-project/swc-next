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

- Supports the erasableSyntaxOnly subset. Not currently exposed through the JavaScript API.
- Type stripping is distinct from general TypeScript lowering.
