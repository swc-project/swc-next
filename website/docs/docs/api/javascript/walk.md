---
title: Walk
description: Walk inputs, hooks, context methods, and reusable walkers.
---

# Walk

`@swc-next/walker` provides pure JavaScript traversal in Node.js and browsers. `@swc-next/core` also exports `walk`.

## Inputs and return value

```ts
declare function walk<
  Root extends AstNode,
  State = unknown,
  Node extends AstNode = AstNode,
>(root: Root, hooks: WalkHooks<State, Node>, state?: State): Root;
```

| Input | Meaning |
| --- | --- |
| `root` | An acyclic AST using the node types and child fields supported by SWC Next. |
| `hooks` | Synchronous callbacks keyed by node type, supported alias, or global `enter` / `leave`. |
| `state` | Optional data shared through `context.state` for this walk. |

The walk mutates the tree in place and returns the original root object. The root cannot be replaced or removed. Traversal is structural; it does not provide bindings, scope-aware renaming, or grammar repair.

## Hooks

```ts
type WalkHandler<Node extends AstNode, State> = (
  node: Node,
  context: WalkContext<State>,
) => void;
```

A node-type hook may be a function (enter only) or an object with `enter` and `leave` callbacks. Global `enter` and `leave` callbacks run for every node. `Function` and `Class` aliases group their supported syntax node types.

The order is global enter, alias enters in registration order, concrete enter, children, concrete leave, alias leaves in reverse order, and global leave. Hooks must return `undefined`. Async callbacks, unknown hook names, and unsupported node types throw. Exceptions propagate and edits are not rolled back.

## WalkContext

| Field or method | Behavior |
| --- | --- |
| `node` | Current node, updated after replacement. |
| `parent`, `key`, `index` | Parent, child field, and current array index. All are `null` at the root; `index` is also `null` for scalar fields. |
| `state` | The state supplied to `walk`. |
| `ancestors()` | Return a new array from the root to the parent. |
| `skip()` | Enter only: skip children and remaining enter hooks, while still running leave hooks. |
| `stop()` | Stop after the current callback, without pending leave hooks. |
| `replace(node)` | Replace a non-root node. Enter hooks do not run again for the replacement; an enter replacement's children and leave hooks are visited. |
| `remove()` | Remove the current array element. Root and scalar-field removal throw. |
| `insertBefore(nodeOrNodes)` | Insert array siblings without visiting them in this walk. |
| `insertAfter(nodeOrNodes)` | Insert array siblings that will be visited later in this walk. |

Use the context only inside its receiving synchronous hook; it is reused and must not be retained. Call methods through the context object. Edit scalar values directly, and use context methods for structural changes to the current position. Replacement and inserted nodes must be valid for their destination.

## createWalker

```ts
declare function createWalker<State = unknown, Node extends AstNode = AstNode>(
  hooks: WalkHooks<State, Node>,
): CompiledWalker<State>;

interface CompiledWalker<State = unknown> {
  walk<Root extends AstNode>(root: Root, state?: State): Root;
}
```

Import `createWalker` from `@swc-next/walker`. It validates and snapshots hooks once for reuse across trees. Each call to its `walk` method has a fresh context; later changes to the hooks object do not affect the compiled walker.

## visitorKeys and types

`visitorKeys` is the frozen child-field table used by the walker, exported from `@swc-next/walker`. It skips metadata, comments, and arbitrary custom properties.

The package exports `AstNode`, `NodeType`, `WalkHooks`, `WalkContext`, and `CompiledWalker` types. Default hooks use a structural node type; supply a discriminated node union through `WalkHooks<State, Node>` for field-level typing. See [AST types](/api/javascript/types).
