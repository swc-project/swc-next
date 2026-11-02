# `@swc-next/ast2`

Read and edit a SWC Next AST through TypeScript views over binary storage.
The package is ESM for Node.js and browsers and has no native runtime
dependency. Node fields follow `@swc-next/types`; list fields use `AstList`.

```js
import { generateSync, parseSync, NodeKind } from '@swc-next/core/ffi2';

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
diagnostic unless error recovery is enabled. `readAst(buffer, source?)` opens
existing parser bytes; pass the original source so source-backed strings slice
it instead of decoding UTF-8. Views read the current storage on access; setters
write existing fields directly.

`new_node(kind, start, end)` allocates a native node; spans are byte offsets.
Fill required fields before passing the buffers to Rust decoding. Use
`get(field)`/`set(field, value)` when a native wrapper differs from its
projected ESTree shape. References from another AST are cloned into this AST;
`clone(node)` performs the same copy explicitly. The AST must remain a tree:
each child has one incoming reference. Remove a subtree's previous reference
before attaching it elsewhere. Spans are copied only when both views share the
same source text, otherwise cloned nodes start at offset 0. Node setters also
accept `{ kind, ...fields }` descriptors, allocated in the current AST;
`start`/`end` are UTF-16 positions. Ordinary object importing is unsupported.

`AstList` reads live positions through `get`/`length`. Use `set`, `push`,
`splice`, `clear` or `replace` to modify. Numeric index assignment is
unsupported. Iterators and callback methods such as `map` and `forEach`
capture the list range when called; structural edits during iteration are not
guaranteed to be reflected. Capture the elements first or use `get`/`length`
for live positions.

`encode()` returns `[original, extension]` without traversing the tree; changes
that fit in the original storage produce an empty extension. Copy both buffers
to retain a snapshot. There is no rollback, and appended storage is retained
for the life of the view; release and reparse for unbounded editing sessions.
Construction and `encode()` do not validate AST kinds, reference roles or tree
structure. Checked Rust decoding validates untrusted plugin output; unchecked
decoding requires trusted, valid output. JS setters enforce encoding types,
buffer ownership and storage limits. The checked decoder borrows its input for
the duration of the native call: pass immutable buffers. If the backing store
is shared or can change concurrently, such as a `SharedArrayBuffer`, copy the
bytes into a non-shared `Uint8Array` first.

Each buffer must have one editing view. Other views over the same bytes have
independent list positions; reopen a copy for independent edits or snapshots.
Node views cache their native kind until their own view's edit version changes,
so edits made through another view over the same bytes are not observed.

Use `AstNode<T>` for field-bearing views and `NodeView<T>` for native handles
with `get`/`set`. Node reads create fresh wrappers; compare logical nodes with
`a.isSame(b)` or `AstView.isSame(a, b)`. `cloneNode` makes a shallow ordinary copy;
`materialize` copies an independent ESTree tree. Clear optional fields with
`null` or `undefined`. Parent and ancestor relationships come from the walker's
visitor context; views do not store parents.

```js
import { generateBuffersSync } from '@swc-next/core/ffi2';

const [original, extension] = ast.encode();
console.log(generateBuffersSync(original, extension).code);
```

Use `walk` from `@swc-next/core` to traverse projected nodes. Collect nodes
first when edits must operate on a fixed set. `encode({ sourceType, lang })`
overrides output metadata; language changes must stay within the JavaScript
(`js`/`jsx`) or TypeScript (`ts`/`tsx`/`dts`) family.
