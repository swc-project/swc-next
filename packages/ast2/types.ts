import type { Lang, SourceType } from "@swc-next/types";
import type { Node } from "./native-types.d.ts";
import type { NodeView } from "./node.ts";
import type { AstView } from "./index.ts";
import type { AstList, ListEntry } from "./list.ts";

export interface AstMeta {
  readonly lang: Lang;
  readonly sourceType: SourceType;
}

export type BufferInput = ArrayBuffer | ArrayBufferView;
export type EnumValue = string;
export type FieldType =
  | "node"
  | "list"
  | "string"
  | "bool"
  | "u8"
  | "i8"
  | "u16"
  | "i16"
  | "u32"
  | "i32"
  | "f64"
  | `enum:${string}`;
export type FieldDescriptor = readonly [
  extra: boolean,
  offset: number,
  type: FieldType,
  optional: boolean,
];
export interface NodeLayout {
  readonly name: string;
  readonly extra: number | null;
  readonly fields: readonly (readonly [field: number, descriptorId: number])[];
}

export interface FieldOffset {
  view: DataView;
  offset: number;
  type: FieldType;
  optional: boolean;
}

type UnionKeys<T> = T extends unknown ? keyof T : never;
type UnionField<T, K extends PropertyKey> = T extends unknown
  ? K extends keyof T
    ? T[K]
    : never
  : never;

export type NodeField<K extends string> =
  K extends UnionKeys<Node> ? ViewValue<UnionField<Node, K>> : unknown;

type ViewValue<T> = T extends readonly unknown[] ? AstList<ListEntry>
  : T extends Node ? NodeView : T;

/**
 * Shorthand for allocating and filling a node in a node-valued setter.
 * `kind` selects the native node type. For nodes with their own `kind` field,
 * assign that field after the node is created.
 */
export interface NodeDescriptor {
  /** Native node kind, for example `NodeKind.IdentifierReference`. */
  readonly kind: number;
  /** Optional UTF-16 span in this AST's source. */
  readonly start?: number;
  readonly end?: number;
  readonly [field: string]: unknown;
}

export type AstValue<T> = T extends readonly (infer Element)[] ? AstList<AstValue<Element>>
  : T extends Node ? AstNode<T> : T;
export type AstNodeFields<T extends Node> = { [K in keyof T]: AstValue<T[K]> };
type NodeInputValue<T> = Extract<AstValue<T>, NodeIdentity> extends never
  ? AstValue<T> extends AstList<infer Element> ? readonly (Element | NodeDescriptor)[] : never
  : NodeDescriptor;
export type AstNodeInput<T extends Node> = { [K in keyof T]: T[K] | AstValue<T[K]> | NodeInputValue<T[K]> };
export interface NodeIdentity {
  readonly ast: AstView;
  readonly nodeId: number;
  get(field: string): unknown;
  set(field: string, value: unknown): void;
}
export type AstNode<T extends Node = Node> = T extends unknown ? AstNodeFields<T> & NodeIdentity : never;
export type FieldInput<K extends string> = K extends UnionKeys<Node>
  ? NodeField<K>
    | (NodeField<K> extends AstList<infer Element> ? readonly Element[] : never)
    | NodeInputValue<UnionField<Node, K>>
  : unknown;

export type NodeList = AstList<ListEntry>;
