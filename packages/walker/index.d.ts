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
 * Current traversal position and editing methods. Reused within each walk;
 * do not retain it beyond the receiving synchronous hook or detach its methods.
 * Use context methods for structural edits; assign scalar properties directly.
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
  /** A new snapshot ordered from root to parent, excluding the current node. */
  ancestors(): AstNode[];
  /** Enter only: skip children and remaining enter hooks, but still run leave hooks. */
  skip(): void;
  /** End the entire walk after this callback finishes, without pending leave hooks. */
  stop(): void;
  /**
   * Replace a non-root node and end remaining hooks in the current phase.
   * During enter, visit the replacement's children and leave hooks, without
   * re-entering it. During leave, do not traverse it. Missing `start` / `end`
   * inherit the old values. Passing the current node is a no-op.
   */
  replace(node: AstNode): void;
  /**
   * Remove the current array element, skipping its remaining hooks and children.
   * Root and scalar-field removal, and further structural edits, throw.
   */
  remove(): void;
  /** Insert array siblings in order, without visiting the inserted nodes. */
  insertBefore(nodes: AstNode | readonly AstNode[]): void;
  /** Insert array siblings in order; they are visited later unless the walk stops. */
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
 * Node-type hooks, `Function` / `Class` aliases, and global `enter` / `leave` hooks.
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
 *
 * Hooks must return `undefined`. Use {@link WalkContext} for structural edits;
 * the root cannot be replaced or removed. Child traversal follows {@link visitorKeys}.
 * Use {@link createWalker} to reuse compiled hooks across trees.
 *
 * @param root - An acyclic supported node or subtree.
 * @param hooks - Node-type, alias, and/or global callbacks.
 * @param state - Optional value exposed as `ctx.state`.
 * @returns The original root, mutated in place.
 * @throws {TypeError} For invalid hooks, unsupported nodes, or non-undefined returns.
 * @throws {Error} For invalid context use or structural edits.
 *
 * @example
 * ```ts
 * import { parseSync, walk } from "@swc-next/core";
 *
 * const { program } = parseSync("debugger; run();");
 * walk(program, {
 *   DebuggerStatement(node, ctx) { ctx.remove(); },
 * });
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
   * Like {@link walk}, using captured hooks and an independent context per call.
   * @param root - An acyclic supported node or subtree.
   * @param state - Per-call value exposed as `ctx.state`.
   * @returns The original root, mutated in place.
   */
  walk<Root extends AstNode>(root: Root, state?: State): Root;
}

/**
 * Validate and snapshot hooks once for repeated synchronous tree traversal.
 * Later hook changes do not affect the snapshot; callback closures remain shared.
 *
 * @param hooks - Callbacks with the same rules as {@link walk}.
 * @returns A frozen walker with a `walk(root, state?)` method.
 * @throws {TypeError} For invalid hook names or callback definitions.
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
 * }
 * ```
 */
export declare function createWalker<
  State = unknown,
  Node extends AstNode = AstNode,
>(hooks: WalkHooks<State, Node>): CompiledWalker<State>;
