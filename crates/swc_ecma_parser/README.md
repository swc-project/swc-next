# swc_ecma_parser

JavaScript and TypeScript parser for SWC Next. It supports JSX and TSX and
writes directly into the flat, ESTree-based AST.

The public API provides parsing options, diagnostics, comments, and optional
token collection. See the [parser API](src/parser.rs) and
[configuration types](src/config.rs).

## Usage

Parse a JavaScript or TypeScript file from the workspace root:

```sh
cargo run -p swc_next_ecma_parser --example parser -- input.ts
```

Use `--preserve-parens` to retain explicit parenthesis nodes. The input file's
extension selects JavaScript, TypeScript, or JSX/TSX.

## Testing

Unit tests live beside the lexer and parser implementations.

```sh
cargo test -p swc_next_ecma_parser --locked
```
