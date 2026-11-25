# swc_ecma_transformer_v1

JavaScript transform passes ported from SWC to the [SWC Next AST](../swc_ecma_ast).

## Testing

```sh
cargo test -p swc_next_ecma_transformer_v1 --locked
```

Conformance tests run through the unified [testsuite](../../tasks/testsuite) (`transform_v1`).
