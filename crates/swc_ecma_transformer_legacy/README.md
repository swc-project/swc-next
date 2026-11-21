# swc_ecma_transformer_legacy

Home for legacy SWC JavaScript transforms ported to the SWC Next AST.
The crate defines [transform options](src/lib.rs) for ES2016–ES2022, RegExp,
TypeScript, and React, using SWC Next's grouping and legacy SWC's configuration
fields and defaults. It also contains a
[transform inventory](docs/transform-inventory.md).

Transform execution is not implemented yet. The crate has no runtime dependencies.

The Cargo package is `swc_next_ecma_transformer_legacy`, following the workspace
naming convention. Existing transforms remain available through
[`swc_ecma_transformer`](../swc_ecma_transformer).

## Testing

```sh
cargo test -p swc_next_ecma_transformer_legacy --locked
```
