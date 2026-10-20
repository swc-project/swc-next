# @swc-next/types

Shared ESTree declarations for SWC Next's parser, decoder, encoder, and codegen.

```ts
import type { Program, Node } from "@swc-next/types";
```

The declarations describe SWC Next's AST, including TypeScript, JSX, and error
recovery nodes. Assignability from other parsers' AST types is not guaranteed.

Types are generated from the Rust AST definitions and `#[estree]` annotations
by [ast_tools](../../tasks/ast_tools). Edit those definitions rather than the
generated declarations.

See the [`estree` attribute documentation](../../crates/swc_ast_macros/src/lib.rs)
for declaration overrides.
