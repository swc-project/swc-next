# swc_ecma_semantic2

Semantic analysis for SWC Next's flat AST. It builds scopes, symbols,
and references and reports early errors.

TypeScript analysis tracks lexical bindings, merged namespace and enum members,
and import aliases for the TypeScript-to-JavaScript transformer. Dotted `=`
assignments to function declarations add value properties to the container.

See the [public semantic API](src/lib.rs). Recompute analysis after AST changes
that invalidate bindings or references.

## Usage

Inspect a file's scope tree from the workspace root:

```sh
cargo run -p swc_next_ecma_semantic2 --example semantic2 -- input.ts
```

## Testing

Integration tests in `tests/` cover syntax checks, symbol links, and
symbol/reference snapshots. Run the crate tests from the workspace root:

```sh
cargo test -p swc_next_ecma_semantic2 --locked
```

## Limitations

- **Type checking and inference.** For `const n: string = 1`, analysis records
  the binding for `n` and leaves the type mismatch to a type checker.
- **Computed expando properties.** Bracket notation, including `fn["name"] = value`,
  is outside the expando model.
- **Global augmentations.** In `declare global { interface X {} } export default X;`,
  the declaration of `X` stays in the block, so the export reference is unresolved.
- **Type parameters in merged declarations.** In `interface Box<T> { value: T }`
  and `interface Box<T> { extra: T }`, the interfaces share a `Box` symbol and
  each `T` has its own symbol.
