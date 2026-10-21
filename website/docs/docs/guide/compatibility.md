---
title: "Compatibility with SWC"
description: "SWC Next is a ground-up rewrite with separate packages and a new AST representation."
---

# Compatibility with SWC

SWC Next is a ground-up rewrite with separate packages and a new AST representation.

## API and AST

- The JavaScript API differs from the existing SWC API.
- The JavaScript API produces an AST compatible with [ESTree](https://github.com/estree/estree) and typescript-eslint's TypeScript and JSX node format, rather than the legacy SWC AST.

## Configuration

- Selected .swcrc options are planned. Configuration compatibility is not implemented yet.

## Related

[Compiler components](/api/components/) · [API reference](/api/)
