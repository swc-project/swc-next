# swc_ecma_semantic

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
cargo run -p swc_next_ecma_semantic --example semantic -- input.ts
```

## Testing

Integration tests in `tests/semantic/` cover public semantic-model behavior and
project-owned early-error regressions.

```sh
cargo test -p swc_next_ecma_semantic --test semantic --locked
```
