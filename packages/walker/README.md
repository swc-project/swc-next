# `@swc-next/walker`

Traverse and edit JavaScript / TypeScript ESTree ASTs with synchronous hooks.
Pure JavaScript with no runtime dependencies, native bindings, or WASM initialization.
The same ESM package works in Node.js and browsers.

```js
import { walk } from "@swc-next/walker";

walk(program, {
  DebuggerStatement(node, ctx) {
    ctx.remove();
  },
});
```

`program` is an existing AST. For example, compose the walker with SWC Next:

```js
import { parseSync, printSync } from "@swc-next/core";
import { walk } from "@swc-next/walker";

const { program, meta } = parseSync("debugger; run();");
walk(program, { DebuggerStatement(node, ctx) { ctx.remove(); } });
const { code } = printSync(program, {}, meta);
// run();
```

The walker does not parse, print, or orchestrate plugins. Parser-produced ASTs
must use the supported ESTree node names and child fields described below.

## `walk(root, hooks, state?)`

Walk an existing AST synchronously, mutate it in place, and return the original
root. Parsing and printing are independent. Import from `@swc-next/walker`:

```js
import { walk } from "@swc-next/walker";

const dependencies = [];
walk(program, {
  ImportDeclaration(node, ctx) {
    ctx.state.push(node.source.value);
  },
}, dependencies);
```

Hooks are keyed by ESTree node type. A function is an enter hook; an object may
have `enter` and `leave` functions. Global `enter` / `leave` hooks apply to all
nodes. `Function` groups function declarations, expressions, arrows,
`TSDeclareFunction`, and `TSEmptyBodyFunctionExpression`; `Class` groups class
declarations and expressions. These are syntax groups, not scope-aware queries.

```js
walk(program, {
  CallExpression: {
    enter(node, ctx) {},
    leave(node, ctx) {},
  },
  Function(node, ctx) {},
  enter(node, ctx) {},
  leave(node, ctx) {},
});
```

The order is global enter, alias enters in registration order, concrete enter,
children, concrete leave, alias leaves in reverse order, global leave. Unknown
hook names and node types throw. Hooks must be synchronous and return undefined;
use context methods for structural edits. Async hooks and other return values
throw. Hook exceptions propagate; edits already made are not rolled back.

Child fields come from `visitorKeys`, generated alongside the codecs from this
repository's AST schema with explicit ESTree projection rules. Their array order
is the traversal order (not necessarily source order). Metadata, comments, raw
text, and arbitrary plugin properties are not traversed. The frozen key table is
also exported. Trees must be acyclic; shared node objects are visited once per
position, not deduplicated.

### Context and edits

| API | Semantics |
| --- | --- |
| `node` | Current node, updated immediately by `replace`. The callback's `node` argument still refers to the original object. |
| `parent`, `key`, `index` | Parent node and its child field; index is the current array index or null (also null after removal). All three are null at the root. |
| `state` | The supplied state object; it may be mutated. |
| `ancestors()` | A new array from the root to the parent, excluding the current node. Only the returned snapshot is allocated on demand. |
| `replace(node)` | Replace a non-root node immediately and end the remaining hooks in the current phase. Enter does not run again on the replacement. After an enter replacement, walk its children and run leave hooks for its type. A leave replacement is not traversed. Missing `start` / `end` on the replacement root inherit the old values. |
| `remove()` | Splice the current array element. During enter, skip its children and all leave hooks; during leave, stop remaining leave hooks. Later siblings are still visited. Root and scalar-field removal throw. |
| `insertBefore(nodeOrNodes)` | Insert siblings in an array, in supplied/call order. They are not visited during this walk. |
| `insertAfter(nodeOrNodes)` | Insert siblings in an array, in supplied/call order. They are visited later in this walk. |
| `skip()` | Enter only: stop remaining enter hooks and skip children, but run the current node's leave hooks. |
| `stop()` | Finish the current callback, then end the walk, without pending leave hooks. |

**One context is reused across the synchronous hooks of each walk. Do not retain
it.** A saved reference aliases the position of later hooks, so stale-reference
use during another hook cannot be detected. Getters and edit methods throw when
no hook is active, including after completion or an exception. A nested walk has
its own context and does not change the outer callback's position. Node, state,
and ancestor snapshot references may be retained; they are ordinary objects.
Call methods through `ctx` (for example `ctx.remove()`); they are not bound
functions and must not be detached from their receiver.

The walker uses generated child accessors (for example `node.left`) and a shared
per-type dispatch table. It only populates context storage for phases with hooks;
recursive positions stay in local variables. One mutable position record is
reused for the entire walk. An ancestor stack scales with depth; `ancestors()`
allocates a snapshot only when requested. The traversal engine and context
methods live on the prototype, avoiding per-walk traversal closures and per-hook
allocations. No runtime code generation or `eval` is used.

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

`NodeType` is generated from the supported ESTree projections. Default hooks
narrow `node.type`; other fields use the structural `AstNode` interface. This
first version does not generate complete per-node field declarations. Consumers
can supply their own discriminated union with `WalkHooks<State, Nodes>` and
`walk<Root, State, Nodes>(root, hooks, state)` for field-level checking.
