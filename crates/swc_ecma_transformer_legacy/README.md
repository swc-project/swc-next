# swc_ecma_transformer_legacy

Legacy SWC JavaScript transforms for the [SWC Next AST](../swc_ecma_ast), with
options grouped like [`swc_ecma_transformer`](../swc_ecma_transformer).
`Transformer::transform` accepts a mutable AST and options without requiring Semantic.

Supports classic and automatic JSX runtimes, development source/self metadata,
pragma comments, entity decoding, and JSX purity annotations. Pragmas accept dotted
paths. TypeScript supports type erasure, enums, namespaces, parameter properties,
import/export assignments, and TSX factory import retention. TypeScript runs before
JSX; hygiene runs once after all selected transforms.

Cross-body namespace binding resolution, React Refresh, display names, purity
annotations for existing React calls, and edition transforms are not supported.

## Testing

```sh
cargo test -p swc_next_ecma_transformer_legacy --locked
cargo test -p swc_next_ecma_transformer_legacy --locked -- --include-ignored
```

The ignored [conformance test](tests/conformance.rs) requires pinned SWC fixtures
and compares output with SWC snapshots.
