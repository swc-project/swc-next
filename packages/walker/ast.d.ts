import type { AstList, AstNode, AstView, NodeView } from "@swc-next/ast";
import type { NodeType } from "./visitor-keys.js";

export { visitorKeys } from "./visitor-keys.js";
export type { NodeType };
export declare const NONE: 0xffffffff;

/** Current numeric position. Valid only during the receiving synchronous hook. */
export interface AstWalkContext<State = unknown> {
  /** Current ID; replace may change it after the callback began. */
  readonly nodeId: number;
  /** Parent ID, or NONE at the root. */
  readonly parentId: number;
  readonly ast: AstView;
  /** Live projected type and native kind; neither getter creates a view. */
  readonly type: NodeType;
  readonly kind: number;
  readonly key: string | null;
  readonly index: number | null;
  readonly state: State;
  /** Fresh views on access. Compare logical identity with NodeView.isSame. */
  readonly node: AstNode;
  readonly parent: AstNode | null;
  ancestors(): AstNode[];
  ancestorIds(): number[];
  /** Enter only: omit children and remaining enter hooks, then run leave. */
  skip(): void;
  /** End traversal without pending leave hooks. */
  stop(): void;
  /** Replace a non-root node; the ID argument of the current hook stays unchanged. */
  replace(node: NodeView): void;
  remove(): void;
  insertBefore(nodes: NodeView | readonly NodeView[] | AstList<NodeView>): void;
  insertAfter(nodes: NodeView | readonly NodeView[] | AstList<NodeView>): void;
}

export type AstWalkHandler<State = unknown> = (
  nodeId: number,
  context: AstWalkContext<State>,
) => void;

export type AstWalkHook<State = unknown> =
  | AstWalkHandler<State>
  | { enter?: AstWalkHandler<State>; leave?: AstWalkHandler<State> };

export type AstWalkHooks<State = unknown> = {
  [Type in NodeType]?: AstWalkHook<State>;
} & {
  enter?: AstWalkHandler<State>;
  leave?: AstWalkHandler<State>;
  Function?: AstWalkHook<State>;
  Class?: AstWalkHook<State>;
};

/** Traverse a binary AST by ID. Views are created only when requested by a hook. */
export declare function walk<State = unknown>(
  ast: AstView,
  rootId: number,
  hooks: AstWalkHooks<State>,
  state?: State,
): number;

export interface CompiledAstWalker<State = unknown> {
  walk(ast: AstView, rootId: number, state?: State): number;
}

/** Capture hooks once; nested calls acquire independent traversal contexts. */
export declare function createWalker<State = unknown>(
  hooks: AstWalkHooks<State>,
): CompiledAstWalker<State>;
