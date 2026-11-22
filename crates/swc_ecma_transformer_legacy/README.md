# swc_ecma_transformer_legacy

[`swc_ecma_transformer`](../swc_ecma_transformer).

## Testing

```sh
cargo test -p swc_next_ecma_transformer_legacy --locked
cargo test -p swc_next_ecma_transformer_legacy --test conformance --locked -- --include-ignored
```

The [SWC conformance test](tests/conformance.rs) reuses pinned SWC fixtures and records
the current pass/fail baseline in a snapshot.
