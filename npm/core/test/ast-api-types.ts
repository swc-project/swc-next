import type { Node, Program } from "@swc-next/types";
import type { AstView, AstList, NodeView, NodeKind, AstNode, AstNodeInput } from "@swc-next/ast";
import type { AstView as RuntimeView } from "../../../packages/ast/index.ts";
import type { AstList as RuntimeList } from "../../../packages/ast/list.ts";
import type { NodeView as RuntimeNode } from "../../../packages/ast/node.ts";
import type { NodeKind as RuntimeKind } from "../../../packages/ast/schema.ts";

type NoMissingMembers<T extends never> = T;
type IsTrue<T extends true> = T;

// Keep the published facade aligned with the TypeScript implementation.
export type ViewMembers = NoMissingMembers<Exclude<keyof AstView, keyof RuntimeView>>;
export type ListMembers = NoMissingMembers<Exclude<keyof AstList, keyof RuntimeList>>;
export type NodeMembers = NoMissingMembers<Exclude<keyof NodeView, keyof RuntimeNode>>;
export type Kinds = IsTrue<typeof NodeKind extends typeof RuntimeKind ? true : false>;
export type EncodeOptions = IsTrue<Parameters<AstView["encode"]> extends Parameters<RuntimeView["encode"]> ? true : false>;
export type ListSplice = IsTrue<Parameters<AstList<number>["splice"]> extends Parameters<RuntimeList<number>["splice"]> ? true : false>;

// Check the field-bearing API as well as the deliberately narrow native handle.
type UnionKeys<T> = T extends unknown ? keyof T : never;
export type ViewFields = NoMissingMembers<Exclude<UnionKeys<AstNode>, keyof RuntimeNode>>;
export type ListJSON = IsTrue<ReturnType<AstList<AstNode>["toJSON"]> extends AstNode[] ? true : false>;
export type NodeJSON = IsTrue<ReturnType<NodeView["toJSON"]> extends ReturnType<RuntimeNode["toJSON"]> ? true : false>;
export type CloneShape = IsTrue<ReturnType<AstView["cloneNode"]> extends AstNodeInput<Node> ? true : false>;
export type ProgramMaterialized = IsTrue<ReturnType<AstView["materialize"]> extends Node ? true : false>;
export type SpliceElision = IsTrue<null extends ReturnType<AstView["spliceList"]>[number] ? true : false>;

// Native handles retain their declared shape when shallow-cloned.
declare const view: AstView;
declare const handle: NodeView<Program>;
const copied: AstNodeInput<Program> = view.cloneNode(handle);
void copied;
