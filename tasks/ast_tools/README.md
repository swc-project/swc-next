# AST tools

`ast_tools` is the code generator for the flattening ECMAScript AST. It reads
the hand-written AST declarations, builds a schema from the types marked with
`#[ast]`, and regenerates the repetitive storage and traversal APIs used by
`swc_experimental_ecma_ast`.

Run the tool from the workspace root because its input and output paths are
workspace-relative:

```sh
cargo run -p ast_tools
```

## Inputs

The input list is defined in `src/main.rs` and currently contains:

- [`js.rs`](../../crates/swc_ecma_ast/src/ast/js.rs)
- [`ts.rs`](../../crates/swc_ecma_ast/src/ast/ts.rs)
- [`jsx.rs`](../../crates/swc_ecma_ast/src/ast/jsx.rs)

Only structs and enums marked with `#[ast]` become AST schema types. The schema
also records field types, enum variants, `Option<T>`/`Vec<T>` wrappers, and
integer `#[repr(uN)]` sizes needed to lay values out in the compact node
payload.

## Generated outputs

All output is written under
[`crates/swc_ecma_ast/src/generated`](../../crates/swc_ecma_ast/src/generated):

| File                   | Generated API                                                                              |
| ---------------------- | ------------------------------------------------------------------------------------------ |
| `ast_builder.rs`       | `Ast` constructors that encode node fields into inline payload bits and extra-data storage |
| `ast_property.rs`      | Typed field getters/setters and enum variant accessors                                     |
| `ast_node.rs`          | `NodeKind`, typed/untyped node conversion, node IDs, spans, and kind data                  |
| `ast_clone_in.rs`      | Arena-aware `CloneIn` implementations for AST nodes                                        |
| `ast_visitor.rs`       | Immutable, mutable, and transforming visitor traits and child traversal                    |
| `ast_extra_compact.rs` | `ExtraDataCompact` conversions used by out-of-line compact storage                         |

The generator formats Rust output with `rustfmt` and avoids rewriting a file
when its bytes have not changed.

## Development workflow

1. Edit the AST declaration in `crates/swc_ecma_ast/src/ast/{js,ts,jsx}.rs`.
   Add `#[ast]` to every new schema node and keep field types within the shapes
   understood by the schema parser.
2. Regenerate all derived APIs:

   ```sh
   cargo run -p ast_tools
   ```

3. Review the generated diff:

   ```sh
   git diff -- crates/swc_ecma_ast/src/generated
   ```

4. Compile both the generator and its consumer:

   ```sh
   cargo check -p ast_tools
   cargo check -p swc_experimental_ecma_ast
   ```

Generated files should not be edited by hand: those edits will be replaced the
next time `ast_tools` runs. Change the AST declarations or the relevant
generator in `tasks/ast_tools/src`, then regenerate.
