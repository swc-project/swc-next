# @swc-next/types

Shared ESTree declarations for SWC Next's parser, decoder, encoder, and codegen.

```ts
import type { Program, Node } from "@swc-next/types";
```

`cargo run -p ast_tools` reads Rust AST definitions and `#[estree(...)]`
annotations to generate `index.d.ts`. Type generation uses generic rules;
node-specific public shapes are described beside the AST definitions.
Encoder and decoder generation retain their existing implementation.

Rust structs become interfaces extending `BaseNode`, with a `type` discriminator.
`BaseNode` contains only `start` and `end`, matching the populated node fields.
Field names become camelCase, `SubRange<T>` becomes `Array<T>`, and `Option<T>`
becomes `T | null`. Acronym prefixes use `TS`, `JSX`, and `JS`.
Rust AST enums become unions of their variants' public types. Scalar enums
marked with `#[estree]` become unions of string literals from their variant
names. `rename` changes those names; `ts_type` supplies other types such as
`null` or `true`. Rust discriminants do not affect declarations.

On an enum, `rename` selects the declaration name and updates references to it.
For example, `Stmt` generates `type Statement = ...` directly. On a struct,
`rename` selects the `type` discriminator; its interface keeps the Rust-derived
name so multiple node shapes can share a discriminator such as `Literal`.

The annotations each control one part of a declaration:

| Annotation | Meaning |
| --- | --- |
| `rename = "..."` | Set a struct's discriminator, an enum's declaration name, a stored field name, or an enum variant name. Unit variants become string literals; payload variants select a public type reference. |
| `ts_type = "..."` | Override a field or enum variant type; on a struct, override the discriminator type. |
| `skip` | Hide a stored field from declarations. |
| `optional` | Allow a property to be absent, independently of whether its value may be `null`. |
| `ts_alias = "..."` | Replace the generated interface or union with a type alias. |
| `fields(name(...))` | Add public properties absent from Rust storage, using `ts_type` and optionally `optional`. |
| `add_ts_def = "..."` | Add declarations next to their owning AST definition. |

For example, computed properties specify their types directly:

```rust
#[ast]
#[estree(fields(sourceType(ts_type = "SourceType")))]
pub struct Program {
    // ...
}
```

The same rename rule describes scalar names and contextual AST variants:

```rust
#[estree]
pub enum ImportPhase {
    #[estree(rename = "source")]
    Source,
    #[estree(rename = "defer")]
    Defer,
}

#[ast(repr)]
#[estree(rename = "Expression")]
pub enum Expr {
    #[estree(rename = "FunctionExpression")]
    Function(Function),
    // ...
}
```

Single discriminator names use `rename`, while `ts_type` can describe a wider
discriminator union:

```rust
#[ast]
#[estree(
    rename = "Literal",
    fields(value(ts_type = "bigint | null"), bigint(ts_type = "string"))
)]
pub struct BigIntLiteral {
    raw: AstUtf8,
}
```

This generates `interface BigIntLiteral` with `type: "Literal"`.

Generation rejects duplicate projected fields, including collisions with
`BaseNode` properties. Bare `skip` affects declarations. The existing
reason-valued `skip = "..."` form also preserves its codec behavior.

Public types must include decoder output and stay within the structures that
can be encoded into the corresponding Rust AST types. Grammar constraints may
be looser than other parsers' declarations when Rust can represent the data.
Cross-field constraints, such as computed keys and arrow function body forms,
are intentionally left independent to keep annotations and generation simple.
These annotations never change Rust storage or codec behavior. Assignability
from other parsers' AST declarations is not guaranteed.

Alignment with `@typescript-eslint/types` 8.71.2 retains decoder-specific differences,
including nullable TS fields, CommonJS programs, preserved parentheses,
JSDoc types, source-phase imports, BigInt property names, and error recovery
nodes. Optional TS fields accept omission only where the encoder can represent
it safely. Fixed Rust child types remain required even when typescript-eslint
allows omission or a wider node union.

Some grammar restrictions would exclude current output: chain expressions can
contain `TSInstantiationExpression`, `TSImportType.options` can be any expression,
and parameter properties can contain destructuring. Error recovery preserves
private names outside classes and delegated yields without an argument. These
shapes stay accepted rather than copying narrower declarations.

The decoder currently emits abstract methods with function bodies. These
remain in the declarations to cover its output. The encoder selects the Rust
method structure from the function value's type and preserves the abstract flag
independently. Directive encoding checks for a string literal before selecting
the Rust directive structure.
