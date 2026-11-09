import type {
  FormalParameters,
  Lang,
  Node,
  Program,
  SourceType,
} from "@swc-next/types";
import { NodeKind } from "./schema.js";

export { NodeKind } from "./schema.js";
export declare const FORMAT_VERSION: number;

export type * from "@swc-next/types";

/** A small facade, allocated only when a node is requested. */
export declare class NodeView {
  private constructor();
  /** Compare logical buffer nodes without requiring wrapper identity. */
  static isSame(left: unknown, right: unknown): boolean;
  readonly ast: AstView;
  readonly nodeId: number;
  readonly parent: Node | null;
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
  /** A view of the original storage. Keep its bytes immutable while AST views exist. */
  readonly buffer: Uint8Array;
  readonly sourceBytes: Uint8Array;
  /** Decoded only when requested. */
  readonly source: string;
  readonly sourceUtf16Length: number;
  readonly meta: { readonly lang: Lang; readonly sourceType: SourceType };
  /** Native table slots, including unused slots. Derived occurrence IDs can exceed this count. */
  readonly nodeCount: number;
  readonly rootId: number;
  /** Number of node wrappers created, including repeated access to the same ID. */
  readonly materializedNodeCount: number;
  /** Current published binary edit storage, or null before the first edit. Do not mutate its bytes. */
  readonly editBuffer: Uint8Array | null;
  /** Includes newly allocated logical nodes in addition to original native slots. */
  readonly totalNodeCount: number;
  /** Import a node into binary storage immediately. Continue editing through the returned view. */
  createNode<T extends Node>(node: T, kind?: (typeof NodeKind)[keyof typeof NodeKind] | keyof typeof NodeKind): T & NodeView;
  /** Splice a public node list and publish its references before returning. */
  spliceList(node: Node | number, key: string, start: number, deleteCount: number, ...values: Node[]): Node[];
  /**
   * Synchronize public arrays/literal objects and seal the current extension.
   * Later edits copy sealed storage so previously returned pairs stay valid.
   * No whole-tree encoder runs. Language overrides must stay within the
   * original JavaScript (js/jsx) or TypeScript (ts/tsx/dts) family.
   */
  encode(meta?: {
    lang?: Lang;
    sourceType?: SourceType;
  }): [buffer: Uint8Array, extension: Uint8Array];
  /** A shallow, editable copy; child node references are preserved. */
  cloneNode<T extends Node>(node: T): T;
  readonly root: Node | FormalParameters;
  /** Throws if the serialized root is a subtree instead of a program. */
  readonly program: Program;
  readonly comments: AstComment[];
  readonly diagnostics: readonly AstDiagnostic[];
  node(id: number): Node | FormalParameters | null;
  kind(id: number): (typeof NodeKind)[keyof typeof NodeKind];
  parentId(id: number): number;
  firstChildId(id: number): number;
  nextSiblingId(id: number): number;
  byteStart(id: number): number;
  byteEnd(id: number): number;
  /** Convert an offset lazily, indexing only the necessary UTF-8 source prefix. */
  utf16Offset(byteOffset: number): number;
  /** Convert a modified ESTree position into a UTF-8 offset in the original source. */
  utf8Offset(utf16Offset: number): number;
  /** Original source bytes for a node, sharing the AST buffer. */
  textBytes(id: number): Uint8Array;
  /** Bytes for a generated native string field, sharing the buffer: UTF-8/WTF-8 in base
   * storage, UTF-16 code units in delta storage. */
  bytesForString(id: number, field: string): Uint8Array | null;
  /**
   * Traverse native structural occurrences without allocating node facades.
   * Ownership is indexed lazily; shared nodes may receive derived occurrence IDs.
   * Internal parameter wrappers are included. Missing edges use 0xffffffff.
   */
  traverse(
    visitor: (
      id: number,
      kind: (typeof NodeKind)[keyof typeof NodeKind],
    ) => void | "skip" | "stop",
    root?: number,
  ): void;
  /** Explicit copying boundary for consumers that need ordinary own properties. */
  materialize<T extends Node>(node: T): T;
  materialize(): Node;
}

export declare function readAst(buffer: ArrayBuffer | ArrayBufferView): AstView;
