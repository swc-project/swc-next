# swc_ecma_transformer_v1

JavaScript transform passes ported from SWC to the [SWC Next AST](../swc_ecma_ast).

## Testing

```sh
cargo test -p swc_next_ecma_transformer_v1 --locked
cargo test -p swc_next_ecma_transformer_v1 --locked -- --include-ignored
```

The ignored [conformance test](tests/conformance.rs) requires pinned SWC fixtures
and compares output with SWC snapshots.
