import type { AstList, AstNode, AstView, NodeDescriptor, NodeView } from "#protocol/index";
import type { NodeType } from "./visitor-keys.js";

export { visitorKeys } from "./visitor-keys.js";
export type { NodeType };

/** Sentinel parent ID returned when a node has no parent. */
export declare const NONE: 0xffffffff;

/**
 * Current traversal position and editing methods. Reused within each walk;
 * do not retain it beyond the receiving synchronous hook or detach its methods.
 * Use context methods for structural edits; assign scalar properties directly.
 */
export interface WalkContext<State = unknown, Node extends AstNode = AstNode> {
  /** Current node ID; `replace` may change it after the callback began. */
  readonly nodeId: number;
  /** Parent node ID, or {@link NONE} at the root. */
  readonly parentId: number;
  /** Backing AST view. */
  readonly ast: AstView;
  /** Live native type and kind; neither getter creates a view. */
  readonly type: NodeType;
  readonly kind: number;
  /** Current node, materialized on first access and updated by `replace`. */
  readonly node: Node;
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
  /** Parent IDs ordered from root to parent, excluding the current node. */
  ancestorIds(): number[];
  /** Enter only: skip children and remaining enter hooks, but still run leave hooks. */
  skip(): void;
  /** End the entire walk after this callback finishes, without pending leave hooks. */
  stop(): void;
  /**
   * Replace a non-root node and end remaining hooks in the current phase.
   * During enter, visit the replacement's children and leave hooks, without
   * re-entering it. During leave, do not traverse it. Missing `start` / `end`
   * inherit the old values. Passing the current node is a no-op.
   * The replacement is a view or a `{ kind, ...fields }` descriptor.
   */
  replace(node: AstNode | NodeDescriptor): void;
  /**
   * Remove the current array element, skipping its remaining hooks and children.
   * Root and scalar-field removal, and further structural edits, throw.
   */
  remove(): void;
  /** Insert array siblings in order, without visiting the inserted nodes. */
  insertBefore(nodes: AstNode | NodeDescriptor | readonly (AstNode | NodeDescriptor)[] | AstList<AstNode>): void;
  /** Insert array siblings in order; they are visited later unless the walk stops. */
  insertAfter(nodes: AstNode | NodeDescriptor | readonly (AstNode | NodeDescriptor)[] | AstList<AstNode>): void;
}

/** A synchronous callback that must return `undefined`; edit through the context. */
export type WalkHandler<Node extends AstNode = AstNode, State = unknown> = (
  context: WalkContext<State, Node>,
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
 * Native node-type hooks and global `enter` / `leave` hooks.
 * Supply a discriminated node union for field-level typing. The walker only
 * accepts native AST views; plain ESTree objects are not supported.
 */
export type WalkHooks<State = unknown, Node extends AstNode = AstNode> = {
  [Type in Node["type"]]?: WalkHook<NodeOfType<Node, Type>, State>;
} & {
  enter?: WalkHandler<Node, State>;
  leave?: WalkHandler<Node, State>;
};

/**
 * Traverse a buffer-backed AST synchronously from a {@link NodeView}.
 *
 * Hooks must return `undefined`. Use {@link WalkContext} for structural edits;
 * the root cannot be replaced or removed. Child traversal follows {@link visitorKeys}.
 * Use {@link createWalker} to reuse compiled hooks across trees.
 *
 * @param root - A `NodeView` from an `AstView` (`ast.program` or `ast.node(id)`).
 * @param hooks - Native node-type and/or global callbacks.
 * @param state - Optional value exposed as `ctx.state`.
 * @returns The original root view.
 * @throws {TypeError} For invalid hooks, unsupported roots, or non-undefined returns.
 * @throws {Error} For invalid context use or structural edits.
 *
 * @example
 * ```ts
 * import { walk } from "@swc-next/core-ffi2";
 *
 * walk(program, {
 *   DebuggerStatement(ctx) { ctx.remove(); },
 * });
 * ```
 */
export declare function walk<
  Root extends NodeView,
  State = unknown,
  Node extends AstNode = AstNode,
>(root: Root, hooks: WalkHooks<State, Node>, state?: State): Root;

/** A snapshot of hook functions, reusable across independent or nested walks. */
export interface CompiledWalker<State = unknown> {
  /**
   * Like {@link walk}, using captured hooks and an independent context per call.
   * @param root - A `NodeView` from an `AstView`.
   * @param state - Per-call value exposed as `ctx.state`.
   * @returns The original root view.
   */
  walk<Root extends NodeView>(root: Root, state?: State): Root;
}

/**
 * Validate and snapshot hooks once for repeated synchronous AST traversal.
 * Later hook changes do not affect the snapshot; callback closures remain shared.
 *
 * @param hooks - Callbacks with the same rules as {@link walk}.
 * @returns A frozen walker with a `walk(root, state?)` method.
 * @throws {TypeError} For invalid hook names or callback definitions.
 */
export declare function createWalker<
  State = unknown,
  Node extends AstNode = AstNode,
>(hooks: WalkHooks<State, Node>): CompiledWalker<State>;
