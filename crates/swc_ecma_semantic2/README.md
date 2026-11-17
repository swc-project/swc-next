# swc_ecma_semantic2

Semantic analysis for SWC Next's flat AST. It builds scopes, symbols,
and references and reports early errors.

TypeScript analysis resolves lexical declarations and references. It does not
merge namespace or enum members across bodies, resolve alias targets, or perform
type checking or expression type inference.

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
