# AST tools

`ast_tools` is the code generator for the flattening ECMAScript AST. It reads
the hand-written AST declarations, builds a schema from the types marked with
`#[ast]`, and regenerates the repetitive storage and traversal APIs used by
`swc_next_ecma_ast`.

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
also records field types, enum variants, `Option<T>`/`SubRange<T>` wrappers,
and integer `#[repr(uN)]` sizes needed to lay values out in the compact node
payload. After parsing all types, it assigns every struct field a `Layout`
location: a byte offset and size in the 16-byte payload, or an index in the
extra-data table. Rust and JavaScript generators read these same locations.
Field declarations use their actual storage types; the generator does not
translate collection types.

## Generated outputs

All output is written under
[`crates/swc_ecma_ast/src/generated`](../../crates/swc_ecma_ast/src/generated):

| File                   | Generated API                                                                              |
| ---------------------- | ------------------------------------------------------------------------------------------ |
| `ast_builder.rs`       | `Ast` constructors that build typed payloads and extra-data storage                       |
| `ast_payload.rs`       | Concrete payload layouts and their 16-byte union                                          |
| `ast_property.rs`      | Typed field getters/setters and enum variant accessors                                     |
| `ast_node.rs`          | AST enum data, constructors, `NodeKind`, typed/untyped node conversion, node IDs, and spans |
| `ast_clone_in.rs`      | Arena-aware `CloneIn` implementations for AST nodes                                        |
| `ast_visitor.rs`       | Immutable, mutable, and transforming visitor traits and child traversal                    |

The generator formats Rust output with `rustfmt` and avoids rewriting a file
when its bytes have not changed.

## Field invalidation

Use `#[ast(invalidates(...))]` when changing an authoritative field makes one
or more optional metadata fields stale:

```rust
#[ast]
pub struct NumericLiteral {
    #[ast(invalidates(raw))]
    value: f64,
    raw: Option<AstUtf8>,
}
```

The generated setter compares the old and new compact field representations.
If they differ, it writes the new value and clears every listed field to
`None`. Comparing compact bits preserves source metadata for an unchanged
value while still distinguishing values such as `0.0` and `-0.0`.

Invalidation targets must be `Option<T>` fields on the same struct. Unknown
fields, duplicate targets, self-invalidation, non-optional targets, and
dependency cycles are rejected while building the schema. Builders and clone
implementations initialize all fields together and do not trigger field
invalidation.

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
   cargo check -p swc_next_ecma_ast
   ```

Generated files should not be edited by hand: those edits will be replaced the
next time `ast_tools` runs. Change the AST declarations or the relevant
generator in `tasks/ast_tools/src`, then regenerate.
