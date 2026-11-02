# `@swc-next/ast`

Read and edit a SWC Next AST through TypeScript views over binary storage.
The package is ESM for Node.js and browsers and has no native runtime
dependency. Node fields follow `@swc-next/types`; list fields use `AstList`.

```js
import { generateSync, parseSync, NodeKind } from '@swc-next/core';

const ast = parseSync('const greeting = "你好";');
ast.program.body.get(0).declarations.get(0).id.name = 'renamed';

const value = ast.new_node(NodeKind.NumericLiteral);
value.value = 42;
const statement = ast.new_node(NodeKind.ExpressionStatement);
statement.expression = value;
ast.program.body.push(statement);
console.log(generateSync(ast.program).code);
```

`parseSync(source, options)` parses and opens a view, throwing the first
diagnostic unless error recovery is enabled. `readAst(buffer)` opens existing
parser bytes. Views read the current storage on access; setters write existing
fields directly.

`new_node(kind, start, end)` allocates a native node; spans are byte offsets.
Fill required fields before passing the buffers to Rust decoding. Use
`get(field)`/`set(field, value)` when a native wrapper differs from its
projected ESTree shape. References must belong to the same AST, and the AST
must remain a tree: each child has one incoming reference. Remove a subtree's
previous reference before attaching it elsewhere. Ordinary object importing is
unsupported.

`AstList` reads live positions. Use `get`/`at` to read and `set`, `push`,
`splice`, `clear` or `replace` to modify. Numeric index assignment is
unsupported.

`encode()` returns `[original, extension]` without traversing the tree; changes
that fit in the original storage produce an empty extension. Copy both buffers
to retain a snapshot. There is no rollback, and appended storage is retained
for the life of the view; release and reparse for unbounded editing sessions.
Construction and `encode()` do not validate AST kinds, reference roles or tree
structure. Checked Rust decoding validates untrusted plugin output; unchecked
decoding requires trusted, valid output. JS setters enforce encoding types,
buffer ownership and storage limits.

Each buffer must have one editing view. Other views over the same bytes have
independent list positions; reopen a copy for independent edits or snapshots.

Use `AstNode<T>` for field-bearing views and `NodeView<T>` for native handles
with `get`/`set`. Node reads create fresh wrappers; compare logical nodes with
`NodeView.isSame(a, b)`. `cloneNode` makes a shallow ordinary copy;
`materialize` copies an independent ESTree tree. Clear optional fields with
`null` or `undefined`. Parent and ancestor relationships come from the walker's
visitor context; views do not store parents.

```js
import { generateBuffersSync } from '@swc-next/core';

const [original, extension] = ast.encode();
console.log(generateBuffersSync(original, extension).code);
```

Use `walk` from `@swc-next/core` to traverse projected nodes. Collect nodes
first when edits must operate on a fixed set. `encode({ sourceType, lang })`
overrides output metadata; language changes must stay within the JavaScript
(`js`/`jsx`) or TypeScript (`ts`/`tsx`/`dts`) family.
