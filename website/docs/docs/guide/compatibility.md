---
title: Compatibility
description: Compatibility with SWC APIs, configuration, plugins, and ESTree tooling.
---

# Compatibility

SWC Next has its own JavaScript packages and API surface. Integrations built for SWC need to account for differences in the available APIs, configuration, and AST format.

## SWC

### JavaScript APIs

`@swc-next/core` is a separate package from [`@swc/core`](https://swc.rs/docs/usage/core). Its public APIs cover parsing, AST traversal and editing, and code generation.

| Capability | SWC Next API |
| --- | --- |
| Parsing | [`parseSync`](/api/javascript/parser) parses JavaScript and TypeScript into an AST. |
| AST traversal and editing | [`walk`](/api/javascript/walk) visits and modifies AST nodes. |
| Code generation | [`printSync`](/api/javascript/codegen) prints an AST as source code. |
| Compilation and minification | The JavaScript packages do not currently expose SWC's `transform`, `transformSync`, `minify`, or `minifySync` APIs. |

### Configuration

SWC Next uses separate parser and code generation options and does not load [`.swcrc` files](https://swc.rs/docs/configuration/swcrc). Compatibility with selected SWC configuration options is planned.

Available options are documented in the [Parser](/api/javascript/parser) and [Codegen](/api/javascript/codegen) references.

### Plugins

JavaScript plugins can use [`walk`](/api/javascript/walk) and its visitor hooks to inspect and transform ASTs. SWC's WebAssembly plugins cannot be loaded yet; support is planned for a future release.

## AST format

The JavaScript API produces ASTs compatible with the [ESTree specification](https://github.com/estree/estree), using [typescript-eslint's AST format](https://typescript-eslint.io/packages/typescript-estree/ast-spec/) for TypeScript and JSX nodes.

This differs from SWC's AST format, so existing AST visitors and transformations need adaptation. The JavaScript packages export [AST types](/api/javascript/types) for working with these nodes.
