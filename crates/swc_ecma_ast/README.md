# `swc_next_ecma_ast`

Flat, arena-backed JavaScript, TypeScript and JSX AST storage for SWC Next.
Node IDs, generated builders, property accessors and visitors operate on shared
node and extra-data tables.

`ffi` provides the existing AST transfer format. `ffi2::serialize` produces a
self-contained buffer for lazy access through `@swc-next/ast`, retaining UTF-8
spans and preserving native table layout and node IDs. Protocol
details and constraints are documented in the Rust modules.
`ffi2::deserialize` validates original/extension buffers and exposes a read-only
native AST. Aligned little-endian base tables are borrowed directly; extensions
and conversions use owned arena storage. The unchecked reader follows the
trusted parser/encoder contract documented in Rust.

AST declarations and generated APIs are maintained with `ast_tools`. Node handles
must only be used with their owning `Ast`.

```sh
cargo test -p swc_next_ecma_ast
```
