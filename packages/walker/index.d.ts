import type { NodeType } from "./visitor-keys.js";
export { visitorKeys } from "./visitor-keys.js";
export type { NodeType };

/** Structural ESTree node. Supply a discriminated node union to WalkHooks for field-level typing. */
export interface AstNode {
  type: NodeType;
  start?: number;
  end?: number;
  [field: string]: any;
}

/**
 * The current traversal position and editing methods supplied to every hook.
 *
 * One context is reused within each walk. Use it only in the receiving
 * synchronous hook: a saved reference aliases later positions, and access
 * outside a hook throws. Nested walks have independent contexts. Node, state,
 * and ancestor snapshot references may be retained.
 *
 * Call methods through the context (for example, `ctx.remove()`); they are not
 * bound to their receiver. Use these methods for structural edits instead of
 * directly changing an active ancestor's child container. Edits do not validate
 * grammar, repair required fields, or update bindings or source maps.
 */
export interface WalkContext<State = unknown> {
  /** Current node, updated by `replace`; the hook's node argument is unchanged. */
  readonly node: AstNode;
  /** Parent node, or `null` at the root. */
  readonly parent: AstNode | null;
  /** Child field on the parent, or `null` at the root. */
  readonly key: string | null;
  /** Current array index, or `null` at the root, in scalar fields, or after removal. */
  readonly index: number | null;
  /** State passed to this walk, without cloning; `undefined` when omitted. */
  readonly state: State;
  /**
   * Take a snapshot of the current ancestors.
   * @returns A new array ordered from root to parent, excluding the current node.
   */
  ancestors(): AstNode[];
  /**
   * Skip children and remaining enter hooks after this callback finishes.
   * The current node's leave hooks still run.
   * @throws {Error} If called from a leave hook.
   */
  skip(): void;
  /** End the entire walk after this callback finishes, without pending leave hooks. */
  stop(): void;
  /**
   * Replace the current non-root node immediately, ending remaining hooks in
   * the current phase after this callback. Passing the current node is a no-op.
   *
   * During enter, the replacement's children and leave hooks run, but its enter
   * hooks do not. During leave, the replacement is not traversed. Missing
   * `start` / `end` values on the replacement root inherit the old values.
   *
   * @param node - A supported node valid for the destination field.
   * @throws {Error} If the current node is the root or was removed.
   * @throws {TypeError} If the replacement has an unsupported node type.
   */
  replace(node: AstNode): void;
  /**
   * Remove the current array element immediately. After this callback, skip any
   * remaining hooks and children for this node; later siblings are still visited.
   * Further structural edits in the same callback throw after removal.
   * @throws {Error} If the current node is the root, a scalar child, or already removed.
   */
  remove(): void;
  /**
   * Insert siblings before the current array element, in supplied/call order.
   * Inserted nodes are not visited during this walk; remaining hooks still run.
   * @param nodes - One node or an array of nodes valid for the destination list.
   * @throws {Error} If the current node is the root, a scalar child, or removed.
   * @throws {TypeError} If an inserted node has an unsupported node type.
   */
  insertBefore(nodes: AstNode | readonly AstNode[]): void;
  /**
   * Insert siblings after the current array element, in supplied/call order.
   * Inserted nodes are visited later unless the walk stops; remaining hooks
   * still run. Inserting siblings and then removing the current node is allowed.
   * @param nodes - One node or an array of nodes valid for the destination list.
   * @throws {Error} If the current node is the root, a scalar child, or removed.
   * @throws {TypeError} If an inserted node has an unsupported node type.
   */
  insertAfter(nodes: AstNode | readonly AstNode[]): void;
}

/** A synchronous callback that must return `undefined`; edit through the context. */
export type WalkHandler<Node extends AstNode = AstNode, State = unknown> = (
  node: Node,
  context: WalkContext<State>,
) => void;
/** A function runs on enter; an object can register enter and/or leave callbacks. */
export type WalkHook<Node extends AstNode = AstNode, State = unknown> =
  | WalkHandler<Node, State>
  | { enter?: WalkHandler<Node, State>; leave?: WalkHandler<Node, State> };
type NodeOfType<
  Node extends AstNode,
  Type extends NodeType,
> = Node extends unknown
  ? Type extends Node["type"]
    ? Node & { type: Type }
    : never
  : never;
/**
 * Hooks keyed by supported node types, with global `enter` / `leave` callbacks.
 * `Function` matches function declarations, expressions, arrows, declare
 * functions, and empty-body function expressions; `Class` matches declarations
 * and expressions. These aliases group syntax and do not track scopes.
 * Supply a discriminated `Node` union for field-level typing.
 */
export type WalkHooks<State = unknown, Node extends AstNode = AstNode> = {
  [Type in Node["type"]]?: WalkHook<NodeOfType<Node, Type>, State>;
} & {
  enter?: WalkHandler<Node, State>;
  leave?: WalkHandler<Node, State>;
  Function?: WalkHook<
    NodeOfType<
      Node,
      | "FunctionDeclaration"
      | "FunctionExpression"
      | "ArrowFunctionExpression"
      | "TSDeclareFunction"
      | "TSEmptyBodyFunctionExpression"
    >,
    State
  >;
  Class?: WalkHook<
    NodeOfType<Node, "ClassDeclaration" | "ClassExpression">,
    State
  >;
};

/**
 * Traverse and edit an existing ESTree / TS-ESTree tree synchronously in place.
 * Available from both `@swc-next/walker` and `@swc-next/core`.
 *
 * Hooks run in this order: global enter, alias enters in registration order,
 * concrete enter, children, concrete leave, alias leaves in reverse order,
 * global leave. A function-valued hook is an enter hook. Every callback must
 * return `undefined`; use {@link WalkContext} methods to control traversal and
 * edit structure, or assign scalar node properties directly.
 *
 * Child fields follow {@link visitorKeys}, which may differ from source order.
 * Comments, metadata, and arbitrary extra properties are not traversed. Trees
 * must be acyclic; shared nodes are visited once per position. The root cannot
 * be replaced or removed. No parsing, printing, or grammar repair is performed.
 *
 * Hooks are validated and snapshotted on every call. Use {@link createWalker}
 * to reuse a snapshot across trees. Each call has an independent context, which
 * must not be retained beyond its receiving hook. Exceptions propagate without
 * rolling back edits already made.
 *
 * @param root - The supported node or subtree to visit, including the root itself.
 * @param hooks - Node-type, alias, and/or global synchronous callbacks.
 * @param state - Optional value exposed unchanged as `ctx.state` in every hook.
 * @returns The original `root` object, including when stopped with `ctx.stop()`.
 * @throws {TypeError} For invalid hooks, unsupported node types, or callbacks
 * that return a promise or any value other than `undefined`.
 * @throws {Error} For invalid context use or structural edits.
 *
 * @example
 * ```ts
 * import { parseSync, printSync, walk } from "@swc-next/core";
 *
 * const { program, meta } = parseSync("debugger; run();");
 * const names: string[] = [];
 * walk(program, {
 *   Identifier(node, ctx) { ctx.state.push(node.name); },
 *   DebuggerStatement(node, ctx) { ctx.remove(); },
 * }, names);
 * // names is ["run"]; debugger statements have been removed.
 * const { code } = printSync(program, {}, meta);
 * ```
 */
export declare function walk<
  Root extends AstNode,
  State = unknown,
  Node extends AstNode = AstNode,
>(root: Root, hooks: WalkHooks<State, Node>, state?: State): Root;

/** A snapshot of hook functions, reusable across independent or nested walks. */
export interface CompiledWalker<State = unknown> {
  /**
   * Traverse a tree with the hooks captured by {@link createWalker}.
   * Uses the same traversal order, editing rules, and errors as {@link walk}.
   * Each call creates an independent context and clears its traversal positions
   * on completion or failure, so nested calls and reuse after errors are supported.
   *
   * @param root - An acyclic supported node or subtree to traverse in place.
   * @param state - Per-call state exposed as `ctx.state`; omitted means `undefined`.
   * @returns The original `root` object, including after `ctx.stop()`.
   */
  walk<Root extends AstNode>(root: Root, state?: State): Root;
}

/**
 * Validate and snapshot hooks once for repeated synchronous tree traversal.
 * Available from both `@swc-next/walker` and `@swc-next/core`.
 *
 * The returned walk method follows {@link walk}'s traversal and editing rules.
 * Later changes to the hooks object or its enter/leave properties do not affect
 * the snapshot. Callback closures remain shared; captured values are not cloned.
 * Pass per-tree state to `walker.walk(root, state)` for independent invocations.
 *
 * @param hooks - Node-type, alias, and/or global hooks to validate and capture.
 * @returns A frozen walker with a `walk(root, state?)` method that returns its root.
 * @throws {TypeError} If hook names or callback definitions are invalid. Callback
 * return values and AST nodes are checked when the returned walker runs.
 *
 * @example
 * ```ts
 * import { createWalker, parseSync } from "@swc-next/core";
 *
 * const collectNames = createWalker<string[]>({
 *   Identifier(node, ctx) { ctx.state.push(node.name); },
 * });
 * for (const source of ["first();", "second();"]) {
 *   const { program } = parseSync(source);
 *   const names: string[] = [];
 *   collectNames.walk(program, names);
 *   // Each tree has its own names array and traversal context.
 * }
 * ```
 */
export declare function createWalker<
  State = unknown,
  Node extends AstNode = AstNode,
>(hooks: WalkHooks<State, Node>): CompiledWalker<State>;
