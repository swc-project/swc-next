# swc_ecma_semantic

Semantic analysis for SWC Next's flat AST. It builds scopes, symbols,
and references and reports early errors.

TypeScript analysis resolves declarations, names, and visibility; it does not
perform full type checking or expression type inference.

See the [public semantic API](src/lib.rs). Recompute analysis after AST changes
that invalidate bindings or references.

## Usage

Inspect a file's scope tree from the workspace root:

```sh
cargo run -p swc_next_ecma_semantic --example semantic -- input.ts
```

## Testing

Snapshots in `tests/semantic/` record all symbols, references, and scopes for each input.

```sh
cargo test -p swc_next_ecma_semantic --locked
```
