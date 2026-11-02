# AST tools

Code generation for SWC Next's flat AST and JavaScript bindings. The tool
reads the [AST declarations](../../crates/swc_ecma_ast/src/ast) and their
`#[ast]` and `#[estree]` annotations to generate Rust storage and traversal
APIs, ESTree types, and JavaScript encoders and decoders.
It also generates the FFI2 layout tables, structural reference readers and node
getters used by `@swc-next/ast`.
The same ESTree encoder generator supplies incremental edit encoders, while
FFI2's generated field validators protect the two-buffer Rust decoding boundary.

Run from the workspace root:

```sh
cargo run -p ast_tools
```

Change the AST declarations or [generators](src/generator) rather than editing
generated files. See the [AST design](../../website/docs/docs/architecture/flat-ast.md) for
the storage model.

Attribute syntax and semantics are documented on the
[`ast` and `estree` macros](../../crates/swc_ast_macros/src/lib.rs).
