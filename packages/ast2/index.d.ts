import type { Lang, SourceType } from "@swc-next/types";
import type { Node, Program } from "./native-types.d.ts";
import type { NodeKind as NativeNodeKind } from "./node-kinds.d.ts";

export declare const NodeKind: typeof NativeNodeKind;

export type * from "./native-types.d.ts";
export type { Lang, SourceType } from "@swc-next/types";

export type AstValue<T> = T extends readonly (infer Element)[] ? AstList<AstValue<Element>>
  : T extends Node ? AstNode<T> : T;
/**
 * Shorthand for allocating and filling a node in a node-valued setter.
 * `kind` selects the native node type. For nodes with their own `kind` field,
 * assign that field after the node is created. List mutators accept
 * descriptors directly; use `NodeView.set` for a typed descriptor field write.
 */
export interface NodeDescriptor {
  /** Native node kind, for example `NodeKind.IdentifierReference`. */
  readonly kind: number;
  /** Optional UTF-16 span in this AST's source. */
  readonly start?: number;
  readonly end?: number;
  readonly [field: string]: unknown;
}
type NodeInputValue<T> = Extract<AstValue<T>, NodeView> extends never
  ? AstValue<T> extends AstList<infer Element> ? readonly (Element | NodeDescriptor)[] : never
  : NodeDescriptor;
export type AstNode<T extends Node = Node> = T extends unknown ? { [K in keyof T]: AstValue<T[K]> } & NodeView<T> : never;
export type AstNodeInput<T extends Node> = { [K in keyof T]: T[K] | AstValue<T[K]> | NodeInputValue<T[K]> };
export type Materialized<T> = T extends AstList<infer Element> ? Materialized<Element>[]
  : T extends RegExp ? T
  : T extends readonly (infer Element)[] ? Materialized<Element>[]
  : T extends NodeView<infer Shape> ? Shape
  : T extends object ? { [K in keyof T]: Materialized<T[K]> } : T;
export type ShallowNode<T extends NodeView> = T extends NodeView<infer Shape> ? AstNodeInput<Shape> : never;

/** Live list positions over AST storage. Mutations publish bytes before returning. */
export declare class AstList<T = AstNode> implements Iterable<T> {
  private constructor();
  readonly length: number;
  get(index: number): T | undefined;
  nodeId(index: number): number;
  set(index: number, value: T | NodeDescriptor): void;
  splice(start: number, deleteCount?: number, ...values: (T | NodeDescriptor)[]): T[];
  push(...values: (T | NodeDescriptor)[]): number;
  pop(): T | undefined;
  unshift(...values: (T | NodeDescriptor)[]): number;
  shift(): T | undefined;
  clear(): void;
  replace(values: Iterable<T | NodeDescriptor>): void;
  reverse(): this;
  sort(compare?: (a: T, b: T) => number): this;
  slice(start?: number, end?: number): T[];
  toArray(): T[];
  toJSON(): T[];
  map<R>(callback: (value: T, index: number, list: this) => R): R[];
  filter(callback: (value: T, index: number, list: this) => unknown): T[];
  forEach(callback: (value: T, index: number, list: this) => void): void;
  some(callback: (value: T, index: number, list: this) => unknown): boolean;
  every(callback: (value: T, index: number, list: this) => unknown): boolean;
  find(callback: (value: T, index: number, list: this) => unknown): T | undefined;
  findIndex(callback: (value: T, index: number, list: this) => unknown): number;
  [Symbol.iterator](): IterableIterator<T>;
}

export declare function sameList(left: unknown, right: unknown): boolean;

/** A small facade, allocated only when a node is requested. */
export declare class NodeView<T extends Node = Node> {
  private constructor();
  private readonly nodeShape: T;
  /** Compare logical buffer nodes without requiring wrapper identity. */
  isSame(other: unknown): boolean;
  readonly ast: AstView;
  readonly nodeId: number;
  readonly type: T["type"];
  start: number;
  end: number;
  /** Read a native field by name. */
  get(field: string): unknown;
  /** Write a native field in the current buffer. */
  set(field: string, value: unknown): void;
  /** Materialize this node for JSON serialization. */
  toJSON(): unknown;
}

export interface AstComment {
  readonly type: "Line" | "Block";
  readonly value: string;
  /** UTF-16 offsets, computed from the stored UTF-8 span on access. */
  readonly start: number;
  readonly end: number;
}

export interface AstDiagnosticLabel {
  readonly message: string;
  readonly start: number;
  readonly end: number;
}

export interface AstDiagnostic extends AstDiagnosticLabel {
  readonly severity: "error" | "warning" | "hint" | "info";
  readonly help?: string;
  readonly labels: readonly AstDiagnosticLabel[];
}

export declare class AstView {
  constructor(buffer: ArrayBuffer | ArrayBufferView);
  /** Compare logical buffer nodes without requiring wrapper identity. */
  static isSame(left: unknown, right: unknown): boolean;
  /** Original storage, updated in place by node setters. Its identity and length remain fixed. */
  readonly buffer: Uint8Array;
  readonly sourceBytes: Uint8Array;
  /** Decoded only when requested. */
  readonly source: string;
  readonly sourceUtf16Length: number;
  readonly meta: { readonly lang: Lang; readonly sourceType: SourceType };
  /** Original native table slots, including unused slots. */
  readonly nodeCount: number;
  readonly rootId: number;
  /** Number of node wrappers created, including repeated access to the same ID. */
  readonly materializedNodeCount: number;
  /** Current appended node/list/string storage, or null when no new data exists. Do not mutate its bytes directly. */
  readonly editBuffer: Uint8Array | null;
  /** Includes newly allocated logical nodes in addition to original native slots. */
  readonly totalNodeCount: number;
  /** Allocate a native node slot; fill required fields before checked Rust decoding. Spans use byte offsets. */
  new_node<T extends Node = Node>(kind: (typeof NodeKind)[keyof typeof NodeKind], start?: number, end?: number): AstNode<T>;
  /** Splice a public node list and publish its references before returning. */
  spliceList(node: AstNode | number, key: string, start: number, deleteCount: number, ...values: (AstNode | NodeDescriptor | null | undefined)[]): (AstNode | null)[];
  /**
   * Expose the current storage; list edits are already committed.
   * Both views are valid until another edit; copy both buffers to retain a snapshot.
   * No whole-tree encoder runs. Language overrides must stay within the
   * original JavaScript (js/jsx) or TypeScript (ts/tsx/dts) family.
   */
  encode(meta?: {
    lang?: Lang;
    sourceType?: SourceType;
  }): [buffer: Uint8Array, extension: Uint8Array];
  /** A shallow, editable copy; child node references are preserved. */
  cloneNode<T extends Node>(node: T): T;
  cloneNode<T extends NodeView>(node: T): ShallowNode<T>;
  /**
   * Deep-clone a native node from any AST view into this AST. Spans are copied
   * only when both views share the same source text; otherwise cloned nodes
   * start at offset 0. AST metadata such as the source type stays with the
   * destination.
   */
  clone<T extends Node>(node: AstNode<T>): AstNode<T>;
  readonly root: AstNode;
  /** Throws if the serialized root is a subtree instead of a program. */
  readonly program: AstNode<Program>;
  readonly comments: AstComment[];
  readonly diagnostics: readonly AstDiagnostic[];
  node(id: number): AstNode | null;
  kind(id: number): (typeof NodeKind)[keyof typeof NodeKind];
  byteStart(id: number): number;
  byteEnd(id: number): number;
  /** Query the Rust-built non-ASCII source position index. */
  utf16Offset(byteOffset: number): number;
  /** Convert a modified UTF-16 position into a UTF-8 offset in the original source. */
  utf8Offset(utf16Offset: number): number;
  /** Original source bytes for a node, sharing the AST buffer. */
  textBytes(id: number): Uint8Array;
  /** Original source text for a node; rejects spans outside the source. */
  text(id: number): string;
  /** UTF-8/WTF-8 bytes for a native string field, sharing its base or delta buffer. */
  bytesForString(id: number, field: string): Uint8Array | null;
  /** Explicit copying boundary for consumers that need ordinary own properties. */
  materialize<T>(node: T): Materialized<T>;
  materialize(): Node;
}

export declare function readAst(buffer: ArrayBuffer | ArrayBufferView, source?: string): AstView;
