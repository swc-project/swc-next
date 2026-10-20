# AST tools

Code generation for SWC Next's flat AST and JavaScript bindings. The tool
reads the [AST declarations](../../crates/swc_ecma_ast/src/ast) and their
`#[ast]` and `#[estree]` annotations to generate Rust storage and traversal
APIs, ESTree types, and JavaScript encoders and decoders.

Run from the workspace root:

```sh
cargo run -p ast_tools
```

Change the AST declarations or [generators](src/generator) rather than editing
generated files. See the [AST design](../../website/docs/FlatteningAST.md) for
the storage model.

Attribute syntax and semantics are documented on the
[`ast` and `estree` macros](../../crates/swc_ast_macros/src/lib.rs).
