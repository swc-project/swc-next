# `@swc-next/walker2`

Traverse and edit buffer-backed JavaScript / TypeScript AST views with synchronous hooks.
The walker accepts `NodeView` roots from `@swc-next/ast2` (for example
`parseSync(...).program`); plain ESTree objects are not supported.
Pure JavaScript with no native bindings or WASM initialization.
The same ESM package works in Node.js and browsers.

```js
import { walk } from "@swc-next/walker2";

walk(program, {
  DebuggerStatement(ctx) {
    ctx.remove();
  },
});
```

`program` is a view returned by `parseSync`, `readAst`, or `AstView#node`. If
you use `@swc-next/core`, you do not need to install `@swc-next/walker2`
separately. Import the walker APIs directly from `@swc-next/core/ffi2`:

```js
import { parseSync, generateSync, walk } from "@swc-next/core/ffi2";

const { program } = parseSync("debugger; run();");
walk(program, { DebuggerStatement(ctx) { ctx.remove(); } });
const { code } = generateSync(program);
// run();
```

The walker does not parse, print, or orchestrate plugins. It is specialized for
`@swc-next/ast2` views: the root must be a `NodeView`, and structural edits accept
`NodeView` instances or `{ kind, ...fields }` descriptors. Plain ESTree objects
are rejected.

## `walk(root, hooks, state?)`

Walk an existing AST synchronously, mutate it in place, and return the original
root. Parsing and printing are independent. Import from `@swc-next/walker2`:

```js
import { walk } from "@swc-next/walker2";

const dependencies = [];
walk(program, {
  ImportDeclaration(ctx) {
    ctx.state.push(ctx.node.source.value);
  },
}, dependencies);
```

Hooks are keyed by native Rust node names, such as `Function`, `TsFunction`,
and `IdentifierReference`. A function is an enter hook; an object may have
`enter` and `leave` functions. Global `enter` and `leave` hooks apply to all nodes.

```js
walk(program, {
  CallExpression: {
    enter(ctx) {},
    leave(ctx) {},
  },
  Function(ctx) {},
  enter(ctx) {},
  leave(ctx) {},
});
```

The order is global enter, concrete enter, children, concrete leave, and
global leave. Unknown
hook names and node types throw. Hooks must be synchronous and return undefined;
use context methods for structural edits. Async hooks and other return values
throw. Hook exceptions propagate; edits already made are not rolled back.

Child fields come from `visitorKeys`, generated alongside the codecs from this
repository's native AST schema. Their array order
is the traversal order (not necessarily source order). Metadata, comments, raw
text, and arbitrary plugin properties are not traversed. The frozen key table is
also exported. Trees must be acyclic; shared node objects are visited once per
position, not deduplicated.

### Reuse hooks across files

```js
import { createWalker } from "@swc-next/walker2";

const walker = createWalker({
  DebuggerStatement(ctx) {
    if (ctx.index !== null) ctx.remove();
  },
});
for (const program of programs) walker.walk(program);
```

`createWalker(hooks)` validates and snapshots the hook functions once. Later
changes to the hooks object do not affect it. Each `walker.walk(root, state)`
initializes its own traversal state, supports nested calls, and clears positions,
node references and caller state on return or failure. Cleared contexts and range
cursors are reused in a bounded pool. Captured callback closures remain alive with the compiled
walker. Ordinary `walk(root, hooks, state)` continues to compile on every call;
there is no implicit cache of mutable hook objects.

### Context and edits

| API | Semantics |
| --- | --- |
| `node` | Current `NodeView`, created on first access and updated immediately by `replace`. Hooks that only need numeric metadata never create one. |
| `nodeId`, `parentId` | Numeric node IDs; `parentId` is `NONE` at the root. |
| `type`, `kind`, `ast` | Live native type, native kind, and backing `AstView`; these getters do not allocate a view. |
| `parent`, `key`, `index` | Parent view and its child field; index is the current array index or null (also null after removal). All three are null at the root. |
| `state` | The supplied state object; it may be mutated. |
| `ancestors()` | A new array of views from the root to the parent, excluding the current node. Only the returned snapshot is allocated on demand. |
| `ancestorIds()` | Parent IDs from the root to the parent, excluding the current node. |
| `replace(node)` | Replace a non-root node immediately and end the remaining hooks in the current phase. `node` is a view or a `{ kind, ...fields }` descriptor. Enter does not run again on the replacement. After an enter replacement, walk its children and run leave hooks for its type. A leave replacement is not traversed. Missing `start` / `end` on the replacement root inherit the old values. |
| `remove()` | Splice the current array element. During enter, skip its children and all leave hooks; during leave, stop remaining leave hooks. Later siblings are still visited. Root and scalar-field removal throw. |
| `insertBefore(nodeOrNodes)` | Insert siblings in an array, in supplied/call order; each is a view or a descriptor. They are not visited during this walk. |
| `insertAfter(nodeOrNodes)` | Insert siblings in an array, in supplied/call order; each is a view or a descriptor. They are visited later in this walk. |
| `skip()` | Enter only: stop remaining enter hooks and skip children, but run the current node's leave hooks. |
| `stop()` | Finish the current callback, then end the walk, without pending leave hooks. |

**One context is reused across synchronous hooks and may be reused by a later
walk. Do not retain it.** A saved reference aliases the position of later hooks, so stale-reference
use during another hook cannot be detected. Getters and edit methods throw when
no hook is active, including after completion or an exception. A nested walk has
its own context and does not change the outer callback's position. Node, state,
and ancestor snapshot references may be retained; they are ordinary objects.
Call methods through `ctx` (for example `ctx.remove()`); they are not bound
functions and must not be detached from their receiver.

The walker generates direct child accessors and separate scalar/list traversal.
Node types without hooks use generated traversal functions; unobserved leaf nodes
return immediately. Hooked nodes retain the editing checks below. Context storage
is populated only for phases with hooks and reused within each walk. An ancestor
stack scales with depth; `ancestors()` allocates a snapshot only when requested.
No runtime code generation or `eval` is used.

Replace followed by skip traverses no replacement children. Remove or stop
prevents further hooks for that node; removal makes subsequent structural edits
in the same callback invalid. Inserting before/after then removing the current
node is supported. An insertion does not end the current hook phase.

Edit scalar node data (such as an identifier's `name`) directly. Use the context
for changes to the current position and sibling list. Do not splice, reorder,
replace, or reparent an active ancestor's child container directly while walking.
The walker checks the current position before context edits but does not detect
every external mutation. To edit a scalar child, replace it with a valid node;
to delete an optional child, set the parent's field to null in a parent hook.

This is structural traversal, without Babel's grammar repair, bindings, rename,
reference tracking, or full schema validation. Replacements and inserted nodes
must be valid for their destination. Removing the last required array item can
produce an invalid AST. Source positions on synthetic descendants remain the
caller's responsibility; copying a span does not provide source-map generation.
Babel `NodePath` plugins are not directly compatible.

### TypeScript

`NodeType` is generated from the native node kinds. Default hooks
narrow `node.type`; node fields are typed through `AstNode` from
`@swc-next/ast2`. Consumers can supply a discriminated node union with
`WalkHooks<State, Nodes>` and `walk<Root, State, Nodes>(root, hooks, state)` for
field-level checking.

## Development

```sh
# Walker tests, including core parse/edit/print integration.
pnpm --filter @swc-next/core build
pnpm --filter @swc-next/core test:ffi2
```

Tests use Rstest and live in `npm/core/test/walk2`. Root `pnpm test` and
`pnpm test:wasm` include core and walker tests. Tests are excluded from the published package.

Buffer-backed `@swc-next/ast2` nodes are compared by their AST and node ID when
validating edits. Their wrappers may differ across reads; use
`a.isSame(b)` for logical node equality.

Buffer-backed lists are traversed through internal ranges until their public
array is accessed or a context edit needs it. This applies to every visitor
configuration and preserves callback order and structural editing behavior.
