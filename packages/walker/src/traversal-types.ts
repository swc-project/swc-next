import type { AstNode } from "./index.js";

export type NodeList = (AstNode | null | undefined)[];
export type ChildVisitor = (node: AstNode, walker: ChildWalker) => boolean;

export interface ChildWalker {
  visitList(nodes: NodeList, parent: AstNode, key: string): boolean;
  visitListKnown(nodes: NodeList, parent: AstNode, key: string, dispatch: DispatchNode): boolean;
  visitScalar(node: AstNode, parent: AstNode, key: string): boolean;
  leaveChanged(node: AstNode, parent: AstNode | null, key: string | null, list: NodeList | null, index: number): number;
  visitHooks(node: AstNode, parent: AstNode | null, key: string | null, list: NodeList | null, index: number): number;
  visitLeaveHooks(node: AstNode, parent: AstNode | null, key: string | null, list: NodeList | null, index: number): number;
}

export type NodeWalker = (
  node: AstNode, walker: ChildWalker, parent: AstNode | null,
  key: string | null, list: NodeList | null, index: number,
) => number;
export type WalkerTable = Record<string, NodeWalker | null>;
export type DispatchNode = (...args: [...Parameters<NodeWalker>, WalkerTable]) => number;
