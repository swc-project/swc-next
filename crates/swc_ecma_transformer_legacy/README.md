# swc_ecma_transformer_legacy

Legacy SWC JavaScript transforms for the [SWC Next AST](../swc_ecma_ast)

## Testing

```sh
cargo test -p swc_next_ecma_transformer_legacy --locked
cargo test -p swc_next_ecma_transformer_legacy --locked -- --include-ignored
```

The ignored [conformance test](tests/conformance.rs) requires pinned SWC fixtures
and compares output with SWC snapshots.
