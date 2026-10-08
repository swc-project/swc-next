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

/** Valid only during the synchronous callback receiving it, including getters. */
export interface WalkContext<State = unknown> {
  readonly node: AstNode;
  readonly parent: AstNode | null;
  readonly key: string | null;
  readonly index: number | null;
  readonly state: State;
  /** Snapshot ordered from root to parent; allocated only when requested. */
  ancestors(): AstNode[];
  /** Enter only. Skip children and remaining enter hooks, but still run leave hooks. */
  skip(): void;
  /** Stop immediately after this callback, without pending leave hooks. */
  stop(): void;
  /** Replace a non-root node; enter does not run again for the replacement. */
  replace(node: AstNode): void;
  /** Remove the current array element. Scalar fields and the root are rejected. */
  remove(): void;
  /** Insert array siblings in order, without visiting the inserted nodes. */
  insertBefore(nodes: AstNode | readonly AstNode[]): void;
  /** Insert array siblings in order; the inserted nodes will be visited. */
  insertAfter(nodes: AstNode | readonly AstNode[]): void;
}

export type WalkHandler<Node extends AstNode = AstNode, State = unknown> = (
  node: Node,
  context: WalkContext<State>,
) => void;
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

/** Mutates an acyclic AST in place. The root cannot be replaced or removed. */
export declare function walk<
  Root extends AstNode,
  State = unknown,
  Node extends AstNode = AstNode,
>(root: Root, hooks: WalkHooks<State, Node>, state?: State): Root;

export interface TransformPlugin<State = any> {
  name?: string;
  visitor: WalkHooks<State>;
  state?: State;
}
