# `@swc-next/ast`

Read and edit a SWC Next AST through views over binary storage. The package is
plain ESM JavaScript, works in Node.js and browsers, and uses the AST interfaces
from `@swc-next/types`. It has no native runtime dependency.

```js
import { parseAstSync, printSync } from '@swc-next/core';

const ast = parseAstSync('const greeting = "你好";');
ast.program.body[0].declarations[0].id.name = 'renamed';
console.log(printSync(ast.program).code);
```

`parseAstSync(source, options)` combines parsing and opening the view, following
`parseSync`'s diagnostic/error-recovery behavior. To open existing parser bytes,
use `readAst(buffer)` from this package. It accepts `ArrayBuffer` and
`ArrayBufferView`, including Node.js `Buffer` and unaligned subviews.
`parseBufferSync` from `@swc-next/core` always returns the buffer, including parser
diagnostics.

## Views and editing

Opening a buffer validates its format without copying its storage or creating
node wrappers. `ast.program` implements `Program` from `@swc-next/types`. Child
nodes are wrapped on access and strings are decoded on access. Direct node
lookups create fresh wrappers; compare them with `NodeView.isSame(a, b)`, which
checks the owning AST and logical node ID. Changes through one view remain
visible through another view of that ID, including after delta-buffer growth.

Keep `ast.buffer` alive and immutable. Node setters write into a separate SWE3
delta buffer before returning. Existing nodes retain their logical IDs through
override rows, so ordinary leaf edits do not copy their ancestor paths. New
nodes, list entries and strings are stored in the delta.

Use `createNode` to import an ordinary node immediately, and continue editing
through the returned view. `spliceList` and walker context methods such as
`replace`, `insertBefore` and `remove` also commit their changes before returning.

```js
const statement = ast.createNode({
  type: 'ExpressionStatement',
  expression: { type: 'Literal', value: 42, raw: '42' },
});
ast.spliceList(ast.program, 'body', ast.program.body.length, 0, statement);
```

Public list fields remain ordinary arrays, without proxies. `push`, `splice`,
index assignment and deletion update their retained array mirrors; `encode()`
or printing the root view synchronizes those changes to bytes. Mutable literal
metadata objects, such as a template element's `value` or a literal's `regex`,
are checked at the same boundary. This compatibility scan costs time proportional
to the exposed mirror contents, plus any required imports and list rewrites.
Assigning a whole list through a node setter commits it immediately.

After a list is synchronized, its existing array is filled with the resulting
NodeViews, preserving array identity. An imported plain object is no longer the
node's backing storage: later edits to that original object do not update the
AST. Edit the returned view or read the inserted node from its parent instead.
`editBuffer` exposes the current published delta bytes for inspection; treat
them as read-only and use `encode()` when retaining a snapshot.

Accessor fields live on generated prototypes, so spreading a view does not copy
its AST fields. `ast.cloneNode(node)` makes a shallow ordinary copy that preserves
child references. Clear optional fields with `null` or `undefined`, rather than
deleting an accessor. `ast.materialize()` explicitly creates an independent
ordinary ESTree tree, including current edits, for use with the ordinary
`printSync(program, options, meta)` path.

## Encoding and printing

```js
import { printBuffersSync } from '@swc-next/core';

const [original, delta] = ast.encode();
console.log(printBuffersSync(original, delta).code);
```

The original buffer keeps its identity, length and contents. `encode()` flushes
changed public mirrors and seals the already encoded delta; it does not run a
whole-tree encoder. An untouched AST with unchanged metadata returns an empty
delta. Repeated calls may return the same sealed delta. A later edit copies its
used bytes into writable storage, preserving previously returned buffer pairs.
Snapshots include spare arena capacity; there is no compact/rebase API.

Printing a root NodeView uses the two-buffer path and borrows the synchronized
delta for the synchronous native call. Rust merges override rows and appended
data into the existing contiguous AST tables before code generation. This merge
copies data when a delta is present; it is part of printing's cost.

`encode({ sourceType, lang })` overrides metadata for that call. Assigning
`ast.program.sourceType` changes the default for future output, while `ast.meta`
remains parser metadata. Language overrides are supported within `js`/`jsx` or
within `ts`/`tsx`/`dts`. Switching between those families throws; encoding does not
convert a TypeScript AST to JavaScript. Some projected fields cannot represent
arbitrary writes, including `optional: true` on variable/catch binding identifiers
or parameters with initializers.

## Positions and traversal

Storage uses UTF-8 byte spans. Node `start` and `end`, comment positions and
diagnostic positions are exposed as UTF-16 offsets. Conversion indexes the
requested source prefix lazily. `byteStart(id)` and `byteEnd(id)` read current
stored byte positions; setter and import operations convert explicit UTF-16
positions immediately. Omitted positions default to zero. Importing a NodeView
from another AST resets its spans to zero. Explicit plain-object positions must
fit the destination's original source.

Editing nodes does not change `ast.source` or shift source positions.
`sourceBytes` and `textBytes(id)` share original source storage;
`bytesForString(id, field)` can refer to base or delta storage.

```js
import { NodeKind } from '@swc-next/ast';

ast.traverse((id, kind) => {
  if (kind === NodeKind.CallExpression) console.log(ast.node(id).callee.type);
});
```

Native traversal uses lazily derived occurrence, first-child and next-sibling
indices, includes internal parameter wrappers, and follows schema order. Its
callback may return `'skip'` or `'stop'`. Native navigation describes the original
structural snapshot; use `walk(ast.program, hooks)` for the current projected
tree after structural edits. Shared base nodes receive separate occurrence IDs
so an edit to one occurrence leaves the others unchanged.

`nodeCount` counts original serialized slots, including unused slots.
`totalNodeCount` includes subsequently allocated logical nodes and occurrences.
Missing links use `0xffffffff`. `materializedNodeCount` counts wrapper allocations,
including repeated lookups. Comments and diagnostics remain read-only parser
metadata. Each AstView retains the original source and string pool.

## Format, validation and tests

The immutable parser buffer uses SWA2 version 2; the incremental buffer uses SWE3
version 3. Both carry the generated schema fingerprint. Producer and reader must
match; these formats are not stable on-disk caches. Protocol details and Rust
entry-point contracts are documented in
[`swc_next_ecma_ast::ffi2`](../../crates/swc_ecma_ast/src/ffi2/mod.rs).

The JavaScript reader checks section bounds and references on access. Rust's
checked two-buffer decoder also validates typed references, lists, strings,
enum/boolean representations and cycles. `printBuffersSync` and printing roots
opened with `readAst` use checked native decoding. Root views from
`parseAstSync` use the trusted reader while their graph invariants remain known.
Changing a native kind or moving/sharing existing subtrees switches that view to
checked readback, including subsequent prints; this validation walks the graph.
Printing rejects invalid typed children and cycles. Print-option getters run
before borrowing the edit buffers. Aligned little-endian base-only tables can be borrowed directly;
deltas and alignment/endian conversions require copied tables. Always keep a
delta paired with its original base and leave both inputs unchanged during
printing. Printing does not preserve comments.

JS API and FFI tests live in `@swc-next/core`. Build the native binding with
`pnpm --filter @swc-next/core build`; the Wasm suite requires
`pnpm --filter @swc-next/core build:wasm`. Run the corresponding suite with:

```sh
pnpm --filter @swc-next/core test
pnpm --filter @swc-next/core test:wasm
```

See [`index.d.ts`](./index.d.ts) for the API contracts.

The format and lazy-view approach draw on the
[TypeScript encoder](https://github.com/microsoft/TypeScript/blob/main/tsc/internal/api/encoder/encoder.go),
[TypeScript node views](https://github.com/microsoft/TypeScript/blob/main/packages/typescript/src/api/node/node.ts),
and Marvin Hagemeister's [Rust and JavaScript plugins article](https://marvinh.dev/blog/speeding-up-javascript-ecosystem-part-11/).
