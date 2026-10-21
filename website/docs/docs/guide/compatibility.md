---
title: "Compatibility with SWC"
description: "SWC Next is a ground-up rewrite with separate packages and a new AST representation."
---

# Compatibility with SWC

SWC Next is a ground-up rewrite with separate packages and a new AST representation.

## API and AST

- Rust and JavaScript APIs differ from the existing SWC APIs.
- The JavaScript AST is ESTree-based; compatibility with arbitrary third-party ASTs is not guaranteed.

## Configuration

- Selected .swcrc options are planned. Configuration compatibility is not implemented yet.

## Related

[Component APIs](/api/components/) · [API reference](/api/)
