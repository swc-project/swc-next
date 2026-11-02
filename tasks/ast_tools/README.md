# AST tools

Code generation for SWC Next's flat AST and JavaScript bindings. The tool
reads the [AST declarations](../../crates/swc_ecma_ast/src/ast) and their
`#[ast]` and `#[estree]` annotations to generate Rust storage and traversal
APIs, ESTree types and legacy codecs, plus the independent layout tables,
projection metadata and node getters for `@swc-next/ast2`. Generated field validators protect the
base/extension buffer boundary in Rust.

Run from the workspace root:

```sh
cargo run -p ast_tools
```

Change the AST declarations or [generators](src/generator) rather than editing
generated files. See the [AST design](../../website/docs/docs/architecture/flat-ast.md) for
the storage model.

Attribute syntax and semantics are documented on the
[`ast` and `estree` macros](../../crates/swc_ast_macros/src/lib.rs).
