---
title: "Minifier"
description: "Optimize a valid JavaScript program AST before printing."
---

# Minifier

Optimize a valid JavaScript program AST before printing.

## Compression

- Boolean and global constant shortening, safe typeof simplification, and unused local removal.

## Current boundaries

- Not currently exposed through the JavaScript API. Name mangling is not implemented.
- Compact printing is a separate code generator option.

## Related

[Choosing packages](/guide/packages) · [API reference](/api/)
