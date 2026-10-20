# swc_ecma_codegen

JavaScript and TypeScript code generation for SWC Next's flat AST, including
JSX and TSX.

The printer supports readable or compact output, ASCII-only escaping, and
inline-script escaping. Compact output controls formatting; AST optimizations
are provided separately by [the minifier](../swc_ecma_minifier).

See the [printer API and options](src/printer.rs).

## Usage

Parse and print a file from the workspace root:

```sh
cargo run -p swc_next_ecma_codegen --example codegen -- input.ts
```

The example accepts `--minify`, `--ascii-only`, `--inline-script`, and
`--preserve-parens`; use `--help` for details.

## Testing

Local integration tests live in `tests/codegen/js.rs` and `tests/codegen/ts.rs`.
Keep project-specific cases there; `tests/codegen/esbuild/` mirrors upstream tests.

```sh
cargo test -p swc_next_ecma_codegen --test codegen --locked
```
