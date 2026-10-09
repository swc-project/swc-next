# `@swc-next/core`

Parse, inspect, transform, and print JavaScript / TypeScript ESTree ASTs.

```js
import { transformSync } from "@swc-next/core";

function removeDebugger() {
  return {
    name: "remove-debugger",
    visitor: {
      DebuggerStatement(node, ctx) {
        ctx.remove();
      },
    },
  };
}

const { code } = transformSync("debugger; run();", {
  plugins: [removeDebugger()],
});
// run();
```

## `walk(root, hooks, state?)`

Walk an existing AST synchronously, mutate it in place, and return the original
root. Parsing and printing are independent. Import from `@swc-next/core/ast` to
use the walker without loading the native or WASM binding:

```js
import { walk } from "@swc-next/core/ast";

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
| `ancestors()` | A new array from the root to the parent, excluding the current node. No ancestor array is maintained eagerly. |
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

The walker reuses traversal frames by depth and constructs context methods once
on the prototype, rather than allocating a context and closures per hook. Frame
allocation scales with maximum depth, not the number of visited nodes. Ancestor
arrays are allocated only when `ancestors()` is requested.

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

## `transformSync(source, options?)`

Options are `{ parser?: ParserOptions, codegen?: CodegenOptions, plugins?: [] }`.
The result contains `{ code, program, diagnostics, meta }`.

1. Parse once, using `parser` options.
2. Throw `SyntaxError` with a `diagnostics` property on parse errors, before any
   plugin runs. Warnings do not prevent transforms.
3. Walk once per plugin, in array order. Each plugin sees all previous plugins'
   changes, including nodes inserted before their traversal position.
4. Print once, preserving parser metadata and applying `codegen` options.

Plugins are objects `{ name?, visitor, state? }`. Invoke factories explicitly,
for example `plugins: [myPlugin(options)]`. `name` is descriptive; there is no
plugin registry or implicit factory calling. State belongs to the caller: use a
fresh plugin instance for per-file state. Plugin hooks are not merged into one
pass. There are no pre/post lifecycle callbacks or async transform hooks yet.

`transformSync` uses the existing printer: comments are not automatically
reattached, and it does not provide Babel presets, target lowering, semantic
analysis of edits, or source maps. Its AST hook shape is informed by Babel and
[`yuku-ast`](https://github.com/yuku-toolchain/yuku/tree/main/npm/yuku-ast), while
edit semantics above are specific to SWC Next.

The browser package exposes the same hooks and transform API after `await init()`;
its `/ast` export needs no WASM initialization. The Node WASM fallback runs the
same JS walker and transform implementation.
