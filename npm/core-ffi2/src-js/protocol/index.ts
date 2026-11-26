import type { Node, Program } from "./native-types.d.ts";
import type { DeltaArena } from "./storage.ts";
import type { AstMeta, BufferInput, NodeList, FieldOffset, FieldDescriptor, NodeField, AstNode, AstNodeInput, EnumValue, NodeDescriptor } from "./types.ts";
export type * from "./native-types.d.ts";
export type { Lang, SourceType } from "@swc-next/types";
import { layouts, enums, nativeField, NodeKind } from "./schema.ts";
import { NodeView, createNodeView, createNumericNodeView, NUMERIC_NODE_KIND } from "./node.ts";
import { cloneNode as cloneNodeInto } from "./clone.ts";
import { writeType, writeSpan, writeNativeField, finishEdits, writeStringAt, writeNodeAt, writeBoolAt, writeNumberAt, writeEnumAt } from "./writer.ts";
import { fieldLocation, isWellFormed, newNode } from "./storage.ts";
import { AstList, isList, listAt } from "./list.ts";
import type { ListEntry } from "./list.ts";
import { fieldId, fieldNames, parserMeta } from "./protocol.ts";
import { NONE, SPAN_SIZE, PAYLOAD_SIZE } from "./layout.ts";
import { PositionCache } from "./position.ts";

export { NodeKind } from "./schema.ts";

const EMPTY_POSITION_CACHE = new PositionCache(new DataView(new ArrayBuffer(8)), 0, 0);
export { NodeView } from "./node.ts";
export { AstList, sameList } from "./list.ts";
export type { AstNode, AstNodeFields, AstNodeInput } from "./types.ts";
const LAYOUT_TEMPLATE = Symbol("AST view layout");
const decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });

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

export class AstView {
  // An empty-buffer template keeps the completed layout reachable across GC.
  // Initialize fields before parsing so the constructor only updates slots.
  static #shape = new this(LAYOUT_TEMPLATE);
  /** Compare logical buffer nodes without requiring wrapper identity. */
  static isSame(left: unknown, right: unknown): boolean {
    return left === right || (left instanceof NodeView && left.isSame(right));
  }
  #nodeCount = 0;
  #rootId = 0;
  #meta: AstMeta;
  #source: string | undefined = undefined;
  _bytes: Uint8Array;
  _flags = 0;
  _diagnosticLength = 0;
  _view: DataView;
  _nodes = 0;
  _kinds = 0;
  _payloads = 0;
  _extra = 0;
  _extraCount = 0;
  _strings = 0;
  _sourceLength = 0;
  _stringLength = 0;
  _comments = 0;
  _commentCount = 0;
  _diagnostics = 0;
  _diagnosticCount = 0;
  _createdNodeCount = 0;
  _listVersion = 0;
  _positions: PositionCache = EMPTY_POSITION_CACHE;
  _arena: DeltaArena | null = null;
  _editor: { flags: number } | null = null;
  _editVersion = 0;
  _listCapacities?: Map<number, Map<number, [start: number, capacity: number]>>;
  constructor(input: BufferInput | typeof LAYOUT_TEMPLATE, source?: string) {
    if (input === LAYOUT_TEMPLATE) {
      this._bytes = new Uint8Array();
      this._view = new DataView(this._bytes.buffer);
      this.#meta = parserMeta[0];
      return;
    }
    const bytes = inputBytes(input);
    if (!bytes || bytes.byteLength < 8) { throw new RangeError("Invalid AST buffer"); }
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const nodeCount = view.getUint32(0, true), nodes = 8;
    const encoding = view.getUint32(4, true);
    if (encoding !== 0) { throw new RangeError("Invalid AST string encoding"); }
    const kinds = nodes + nodeCount * SPAN_SIZE;
    const payloads = Math.ceil((kinds + nodeCount) / 8) * 8;
    const extraInfo = payloads + nodeCount * PAYLOAD_SIZE;
    const extraCount = view.getUint32(extraInfo, true), extra = extraInfo + 4;
    const stringsInfo = extra + extraCount * 4;
    const sourceLength = view.getUint32(stringsInfo, true),
      stringLength = view.getUint32(stringsInfo + 4, true), strings = stringsInfo + 8;
    const commentsInfo = Math.ceil((strings + stringLength) / 4) * 4;
    const commentCount = view.getUint32(commentsInfo, true), comments = commentsInfo + 4;
    const diagnosticsInfo = comments + commentCount * 8;
    const diagnosticCount = view.getUint32(diagnosticsInfo, true),
      diagnosticBytes = view.getUint32(diagnosticsInfo + 4, true), diagnostics = diagnosticsInfo + 8;
    const positionsInfo = Math.ceil((diagnostics + diagnosticBytes) / 4) * 4;
    const positions = new PositionCache(view, positionsInfo, sourceLength);
    const metadata = positions.runs + positions.count * 16;
    const rootId = view.getUint32(metadata, true), flags = view.getUint32(metadata + 4, true);
    if (!nodeCount || rootId >= nodeCount || flags > 31 || (flags >> 2) > 4 ||
      sourceLength > stringLength || metadata + 8 !== bytes.byteLength ||
      positions.excess > sourceLength || (positions.count === 0) !== (positions.excess === 0) ||
      diagnosticCount * 18 > diagnosticBytes)
      { throw new RangeError("Invalid AST array layout"); }
    this._flags = flags;
    this._diagnosticLength = diagnosticBytes;
    this._bytes = bytes;
    this._view = view;
    this._nodes = nodes;
    this._kinds = kinds;
    this._payloads = payloads;
    this._extra = extra;
    this._extraCount = extraCount;
    this._strings = strings;
    this._sourceLength = sourceLength;
    this._stringLength = stringLength;
    this._comments = comments;
    this._commentCount = commentCount;
    this._diagnostics = diagnostics;
    this._diagnosticCount = diagnosticCount;
    this._positions = positions;
    if (source !== undefined) {
      if (typeof source !== "string") { throw new TypeError("source must be a string"); }
      if (source.length !== this.sourceUtf16Length) {
        throw new RangeError("Source length does not match the AST buffer");
      }
      // Unpaired surrogates are normalized to U+FFFD by the UTF-8 transport;
      // keep the buffer's source in that case so slices match the AST strings.
      if (isWellFormed(source)) { this.#source = source; }
    }
    this.#nodeCount = nodeCount;
    this.#rootId = rootId;
    this.#meta = parserMeta[flags];
    this.kind(rootId);
  }
  get nodeCount() {
    return this.#nodeCount;
  }
  get rootId() {
    return this._arena?.rootId ?? this.#rootId;
  }
  get totalNodeCount() { return this._arena?.nodeCount ?? this.nodeCount; }
  get extraDataCount() { return this._arena?.extraCount ?? this._extraCount; }
  get stringLength() { return this._arena?.stringLength ?? this._stringLength; }
  row(id: number): number { return this._arena === null ? -1 : this._arena.row(id); }
  new_node<T extends Node = Node>(kind: number, start = 0, end = start): AstNode<T> {
    const id = newNode(this, kind, start, end);
    this._createdNodeCount++;
    return createNodeView(this, id, kind) as unknown as AstNode<T>;
  }
  get editBuffer() {
    const arena = this._arena;
    return arena === null || !arena.rowCount && !arena.extraUsed && !arena.stringUsed
      ? null : arena.borrow(this.rootId, this._editor?.flags ?? this._flags);
  }
  spliceList(node: NodeView | number, key: string, start: number, deleteCount: number, ...values: (ListEntry | NodeDescriptor)[]) {
    if (node instanceof NodeView && node.ast !== this) { throw new TypeError("Node belongs to another AST buffer"); }
    const id = node instanceof NodeView ? node.nodeId : node;
    const list = this.value(id, fieldId(key));
    if (!(list instanceof AstList)) { throw new TypeError("Expected an AST list"); }
    const removed = list.splice(start, deleteCount, ...values);
    return removed;
  }
  get meta() {
    return this._editor === null ? this.#meta : parserMeta[this._editor.flags];
  }
  get buffer() {
    return this._bytes;
  }
  get sourceBytes() {
    return this._bytes.subarray(
      this._strings,
      this._strings + this._sourceLength,
    );
  }
  get source() {
    return this.#getSource();
  }
  get sourceUtf16Length() {
    return this._positions.sourceUnits;
  }
  get root(): NodeView | null {
    return this.node(this.rootId);
  }
  get program(): AstNode<Program> {
    if (this.kind(this.rootId) !== NodeKind.Program)
      { throw new TypeError("AST root is not a Program"); }
    return this.node(this.rootId) as AstNode<Program>;
  }
  /** Number of NodeView objects created so far, including internal conversions. */
  get materializedNodeCount() {
    return this._createdNodeCount;
  }
  getField<K extends string>(id: number, key: K, kind?: number): NodeField<K> {
    return this.value(id, fieldId(key), kind) as NodeField<K>;
  }
  writeNativeField(id: number, field: number, value: unknown) { writeNativeField(this, id, field, value); }
  _setType(id: number, value: string): void { writeType(this, id, value); }
  _setSpan(id: number, isStart: boolean, value: number): void { writeSpan(this, id, isStart, value); }
  /** Internal fast paths for generated field setters with constant offsets. */
  _writeStringAt(id: number, offset: number, optional: boolean, value: unknown): void {
    writeStringAt(this, id, offset, optional, value);
  }
  _writeNodeAt(id: number, offset: number, value: unknown): void {
    writeNodeAt(this, id, offset, value);
  }
  _writeBoolAt(id: number, offset: number, value: unknown): void {
    writeBoolAt(this, id, offset, value);
  }
  _writeNumberAt(id: number, offset: number, type: string, value: unknown): void {
    writeNumberAt(this, id, offset, type, value);
  }
  _writeEnumAt(id: number, offset: number, optional: boolean, table: readonly EnumValue[], value: unknown): void {
    writeEnumAt(this, id, offset, optional, table, value);
  }
  cloneNode<T extends Node>(node: T): T;
  cloneNode<T extends Node>(node: AstNode<T>): AstNodeInput<T>;
  cloneNode(node: NodeView): Record<string, unknown>;
  cloneNode(node: object): Record<string, unknown> {
    if (!(node instanceof NodeView)) { return { ...node }; }
    const ast = node.ast;
    const keys = ast.#keys(node.nodeId);
    return Object.fromEntries([...keys].map((key) => [key, (node as unknown as Record<string, unknown>)[key]]));
  }
  /**
   * Deep-clone a native node from any AST view into this AST. Spans are copied
   * only when both views share the same source text; otherwise cloned nodes
   * start at offset 0.
   */
  clone<T extends Node>(node: AstNode<T>): AstNode<T>;
  clone(node: NodeView): NodeView;
  clone(node: unknown): NodeView {
    if (!(node instanceof NodeView)) { throw new TypeError("Expected a NodeView"); }
    return cloneNodeInto(this, node);
  }
  encode(meta?: Partial<AstMeta>) {
    return finishEdits(this, meta);
  }
  validateId(id: number): void {
    if (!Number.isInteger(id) || id < 0) { throw new RangeError("Invalid AST node ID"); }
    if (id < this.nodeCount || this._arena?.has(id)) { return; }
    throw new RangeError("Invalid AST node ID");
  }
  kind(id: number): number {
    this.validateId(id);
    return this._kindAt(id);
  }
  /** Views already hold a validated logical ID; original slots read directly. */
  _kindAt(id: number): number {
    const kind = id < this.#nodeCount ? this._bytes[this._kinds + id] : this._arena!.kind(id);
    if (kind === undefined || kind >= layouts.length) { throw new RangeError("Unknown AST node kind"); }
    return kind;
  }
  /** Internal callers already validated the logical ID and resolved its row. */
  _payloadAt(id: number, row: number): number {
    if (row !== -1) { return row + 16; }
    return this._payloads + id * PAYLOAD_SIZE;
  }
  _typeAt(id: number, kind = this._kindAt(id)): string {
    return layouts[kind].name;
  }
  byteStart(id: number): number {
    this.validateId(id);
    return id < this.#nodeCount
      ? this._view.getUint32(this._nodes + id * SPAN_SIZE, true)
      : this._arena!.spanStart(id);
  }
  byteEnd(id: number): number {
    this.validateId(id);
    return id < this.#nodeCount
      ? this._view.getUint32(this._nodes + id * SPAN_SIZE + 4, true)
      : this._arena!.spanEnd(id);
  }
  start(id: number): number {
    return this.utf16Offset(this.byteStart(id));
  }
  end(id: number): number {
    return this.utf16Offset(this.byteEnd(id));
  }
  /** Query the Rust-built source index without scanning or decoding source bytes. */
  utf16Offset(byte: number): number {
    return this._positions.utf16Offset(byte);
  }
  /** Convert modified UTF-16 positions back to the original source's UTF-8 offsets. */
  utf8Offset(position: number): number {
    return this._positions.utf8Offset(position);
  }
  node(id: number): NodeView | null {
    if (id === NONE) { return null; }
    if (id < this.#nodeCount) {
      if (!Number.isInteger(id) || id < 0) { throw new RangeError("Invalid AST node ID"); }
    } else {
      this.validateId(id);
    }
    return this._nodeAt(id);
  }
  /** Child slots have already checked their numeric IDs. */
  _nodeAt(id: number): NodeView | null {
    if (id === NONE) { return null; }
    const kind = id < this.#nodeCount
      ? this._view.getUint8(this._kinds + id)
      : this.kind(id);
    if (kind >= layouts.length) { throw new RangeError("Unknown AST node kind"); }
    this._createdNodeCount++;
    return kind === NUMERIC_NODE_KIND ? createNumericNodeView(this, id, kind) : new NodeView(this, id, kind);
  }
  extraOffset(payload: number, pointer: number, slot: number, words = 1): number {
    const index = this._view.getUint32(payload + pointer, true) + slot;
    if (index + words > this._extraCount) { throw new RangeError("AST extra-data reference out of bounds"); }
    return this._extra + index * 4;
  }
  stringAt(offset: number, optional: boolean, view = this._view) {
    const start = view.getUint32(offset, true), end = view.getUint32(offset + 4, true);
    return optional && start === NONE && end === 0 ? null : this.string(start, end);
  }
  enumAt(offset: number, name: string, optional: boolean, view = this._view): import("./types.ts").EnumValue | null {
    const value = view.getUint8(offset);
    if (optional && value === 255) { return null; }
    const values = enums[name];
    if (value >= values.length) { throw new RangeError("Unknown AST enum value"); }
    return values[value];
  }
  fieldOffset(id: number, field: number, kind = this.kind(id)): FieldOffset | null {
    const row = this.row(id);
    if (row !== -1) { kind = this._arena!.kind(id); }
    const descriptor = nativeField(kind, field);
    if (descriptor === undefined) { return null; }
    const { view, offset } = fieldLocation(this, id, descriptor, kind);
    return { view, offset, type: descriptor[2], optional: descriptor[3] };
  }
  childAt(id: number, offset: number, optional: boolean, view = this._view): number {
    const child = view.getUint32(offset, true);
    if (child === NONE) {
      if (!optional) { throw new RangeError("Missing required AST child"); }
      return NONE;
    }
    if (child >= this.totalNodeCount) { throw new RangeError("AST child out of bounds"); }
    return child;
  }
  ref(id: number, key: string, allowMissing = false): number {
    return this._ref(id, fieldId(key), allowMissing);
  }
  _ref(id: number, slot: number, allowMissing = false): number {
    const field = this.fieldOffset(id, slot);
    if (!field) { return NONE; }
    if (field.type !== "node") { throw new TypeError("Expected a node reference"); }
    return this.childAt(id, field.offset, field.optional || allowMissing, field.view);
  }
  range(id: number, key: string): [number, number] {
    return this._range(id, fieldId(key));
  }
  _range(id: number, slot: number): [number, number] {
    const field = this.fieldOffset(id, slot);
    if (!field || field.type !== "list")
      { throw new TypeError("Expected an AST list"); }
    const start = field.view.getUint32(field.offset, true),
      end = field.view.getUint32(field.offset + 4, true);
    if (start > end || end > this.extraDataCount)
      { throw new RangeError("AST list out of bounds"); }
    return [start, end];
  }
  extraChild(owner: number, index: number, optional = true): number {
    if (index < this._extraCount)
      { return this.childAt(owner, this._extra + index * 4, optional); }
    return this.childAt(owner, this._arena!.extraAddress(index), optional, this._arena!.view!);
  }
  list(owner: number, field: number): NodeList { return new AstList(this, owner, field); }

  value(id: number, field: number, kind = this.kind(id)): unknown {
    const row = this.row(id);
    if (row !== -1) { kind = this._arena!.kind(id); }
    const descriptor = nativeField(kind, field);
    if (descriptor === undefined) { return undefined; }
    return this._valueAt(id, field, descriptor, kind);
  }
  /** Read a field with an already-resolved layout descriptor. */
  _valueAt(id: number, field: number, descriptor: FieldDescriptor, kind: number): unknown {
    const { view, offset } = fieldLocation(this, id, descriptor, kind);
    const type = descriptor[2], optional = descriptor[3];
    if (type === "node") {
      const child = this.childAt(id, offset, optional, view);
      return this._nodeAt(child);
    }
    if (type === "list") { return this.list(id, field); }
    if (type === "string") { return this.stringAt(offset, optional, view); }
    if (type.startsWith("enum:")) { return this.enumAt(offset, type.slice(5), optional, view); }
    switch (type) {
      case "bool":
        return view.getUint8(offset) !== 0;
      case "u8":
        return view.getUint8(offset);
      case "i8":
        return view.getInt8(offset);
      case "u16":
        return view.getUint16(offset, true);
      case "i16":
        return view.getInt16(offset, true);
      case "u32":
        return view.getUint32(offset, true);
      case "i32":
        return view.getInt32(offset, true);
      case "f64":
        return view.getFloat64(offset, true);
      default:
        throw new RangeError(`Unsupported AST field ${type}`);
    }
  }
  stringBytes(start: number, end: number) {
    if (
      !Number.isInteger(start) ||
      !Number.isInteger(end) ||
      start < 0 ||
      start > end ||
      end > this.stringLength
    )
      { throw new RangeError("AST string out of bounds"); }
    // Both pools transfer UTF-8/WTF-8 bytes.
    if (end <= this._stringLength) { return this._bytes.subarray(this._strings + start, this._strings + end); }
    if (start < this._stringLength) { throw new RangeError("AST string crosses storage regions"); }
    const offset = this._arena!.stringsOffset - this._stringLength;
    return this._arena!.bytes!.subarray(offset + start, offset + end);
  }
  /**
   * Decode a delta UTF-8/WTF-8 range, preserving unpaired surrogates.
   */
  deltaString(start: number, end: number): string {
    const arena = this._arena!;
    const offset = arena.stringsOffset - this._stringLength;
    return this._utf8String(arena.bytes!, offset + start, offset + end);
  }
  string(start: number, end: number): string {
    if (!Number.isInteger(start) || !Number.isInteger(end) || start < 0 || end < start)
      { throw new RangeError("AST string out of bounds"); }
    if (end > this.stringLength) { throw new RangeError("AST string out of bounds"); }
    if (start < this._sourceLength && end > this._sourceLength) { throw new RangeError("AST string crosses source boundary"); }
    if (start === end) { return ""; }
    // Source slices avoid decoding the UTF-8 bytes for the common identifier
    // and literal case. `utf16Offset` is the identity for ASCII-only sources.
    const source = this.#source;
    if (source !== undefined && end <= this._sourceLength) {
      return this._positions.count === 0
        ? source.slice(start, end)
        : source.slice(this.utf16Offset(start), this.utf16Offset(end));
    }
    return start >= this._stringLength ? this.deltaString(start, end) : this._baseString(start, end);
  }
  #getSource(): string {
    let source = this.#source;
    if (source === undefined) {
      source = this._sourceLength === 0 ? "" : this._baseString(0, this._sourceLength);
      this.#source = source;
    }
    return source;
  }
  _baseString(start: number, end: number): string {
    if (end > this._stringLength) { throw new RangeError("AST string crosses storage regions"); }
    return this._utf8String(this._bytes, this._strings + start, this._strings + end);
  }
  _utf8String(bytes: Uint8Array, start: number, end: number): string {
    const length = end - start;
    if (length <= 8) {
      const offset = start;
      let mask = 0;
      for (let i = 0; i < length; i++) { mask |= bytes[offset + i]; }
      if (mask < 0x80) {
        const a = bytes[offset], b = length >= 2 ? bytes[offset + 1] : 0,
          c = length >= 3 ? bytes[offset + 2] : 0, d = length >= 4 ? bytes[offset + 3] : 0;
        switch (length) {
          case 1: return String.fromCharCode(a);
          case 2: return String.fromCharCode(a, b);
          case 3: return String.fromCharCode(a, b, c);
          case 4: return String.fromCharCode(a, b, c, d);
          case 5: return String.fromCharCode(a, b, c, d, bytes[offset + 4]);
          case 6: return String.fromCharCode(a, b, c, d, bytes[offset + 4], bytes[offset + 5]);
          case 7: return String.fromCharCode(a, b, c, d, bytes[offset + 4], bytes[offset + 5], bytes[offset + 6]);
          case 8: return String.fromCharCode(a, b, c, d, bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7]);
        }
      }
    }
    return decodeUtf8OrWtf8(bytes.subarray(start, end));
  }
  bytesForString(id: number, fieldName: string) {
    const field = this.fieldOffset(id, fieldId(fieldName));
    if (!field || field.type !== "string")
      { throw new TypeError("Field is not a native AST string"); }
    const start = field.view.getUint32(field.offset, true),
      end = field.view.getUint32(field.offset + 4, true);
    return field.optional && start === NONE && end === 0
      ? null
      : this.stringBytes(start, end);
  }
  textBytes(id: number) {
    const start = this.byteStart(id), end = this.byteEnd(id);
    if (end > this._sourceLength) { throw new RangeError("Node source span out of bounds"); }
    return this.stringBytes(start, end);
  }
  text(id: number): string {
    const start = this.byteStart(id), end = this.byteEnd(id);
    if (end > this._sourceLength) { throw new RangeError("Node source span out of bounds"); }
    return this.string(start, end);
  }
  #keys(id: number): string[] {
    return ["type", "start", "end", ...layouts[this.kind(id)].fields.map(([field]) => fieldNames[field])];
  }
  /** Explicitly create editable native-shaped objects; this is the copying boundary. */
  materialize<T extends Node>(root: T): T;
  materialize<T extends Node>(root: AstNode<T>): T;
  materialize(): Node;
  materialize(root: unknown): unknown;
  materialize(root: unknown = this.root): unknown {
    const copies = new Map<object | number, object>(),
      pending: Array<[object, Record<string, unknown> | unknown[]]> = [];
    const copy = (value: unknown): unknown => {
      if (value === null || typeof value !== "object") { return value; }
      const key = value instanceof NodeView && value.ast === this ? value.nodeId : value;
      if (copies.has(key)) { return copies.get(key); }
      if (value instanceof RegExp) {
        const result = new RegExp(value.source, value.flags);
        result.lastIndex = value.lastIndex;
        copies.set(key, result);
        return result;
      }
      const result: unknown[] | Record<string, unknown> = isList(value) ? new Array<unknown>(value.length) : {};
      copies.set(key, result);
      pending.push([value, result]);
      return result;
    };
    const result = copy(root);
    while (pending.length) {
      const [value, output] = pending.pop()!;
      if (value instanceof NodeView) {
        if (value.ast !== this)
          { throw new TypeError("Node belongs to another AST buffer"); }
        for (const key of this.#keys(value.nodeId))
          { (output as Record<string, unknown>)[key] = copy((value as unknown as Record<string, unknown>)[key]); }
      } else if (isList(value)) {
        for (let i = 0; i < value.length; i++) { (output as unknown[])[i] = copy(listAt(value, i)); }
      } else
        { for (const key of Object.keys(value)) { (output as Record<string, unknown>)[key] = copy((value as Record<string, unknown>)[key]); } }
    }
    return result;
  }
  get comments() {
    return Array.from({ length: this._commentCount }, (_, index) => {
    const offset = this._comments + index * 8,
      view = this._view,
      ast = this;
    const start = view.getUint32(offset, true),
      end = view.getUint32(offset + 4, true);
    if (end > this._sourceLength || start > end || end - start < 2)
      { throw new RangeError("Invalid AST comment span"); }
    const bytes = this._bytes, sourceStart = this._strings + start;
    const block = bytes[sourceStart + 1] === 42;
    const prefix = bytes[sourceStart] === 60 ? 4 : bytes[sourceStart] === 45 ? 3 : 2;
    if (end - start < prefix + (block ? 2 : 0)) {
      throw new RangeError("Invalid AST comment span");
    }
    return Object.freeze({
      type: block ? "Block" : "Line",
      get start() {
        return ast.utf16Offset(start);
      },
      get end() {
        return ast.utf16Offset(end);
      },
      get value() {
        return ast.string(start + prefix, end - (block ? 2 : 0));
      },
    });
    });
  }
  get diagnostics() {
    let cursor = this._diagnostics;
    const limit = cursor + this._diagnosticLength;
    const byte = () => {
      if (cursor >= limit) { throw new RangeError("Truncated diagnostic"); }
      return this._view.getUint8(cursor++);
    };
    const word = () => {
      if (cursor + 4 > limit) { throw new RangeError("Truncated diagnostic"); }
      const result = this._view.getUint32(cursor, true);
      cursor += 4;
      return result;
    };
    const string = () => {
      const length = word();
      if (cursor + length > limit)
        { throw new RangeError("Truncated diagnostic"); }
      const value = decoder.decode(
        this._bytes.subarray(cursor, cursor + length),
      );
      cursor += length;
      return value;
    };
    const results = [];
    for (let index = 0; index < this._diagnosticCount; index++) {
      const severity = ["error", "warning", "hint", "info"][byte()];
      if (!severity) { throw new RangeError("Invalid diagnostic severity"); }
      const start = word(),
        end = word(),
        message = string(),
        hasHelp = byte();
      if (hasHelp > 1) { throw new RangeError("Invalid diagnostic help flag"); }
      const help = hasHelp ? string() : undefined,
        count = word(),
        labels = [];
      if (count * 12 > limit - cursor)
        { throw new RangeError("Invalid diagnostic label count"); }
      const ast = this;
      for (let label = 0; label < count; label++) {
        const start = word(),
          end = word(),
          message = string();
        labels.push(
          Object.freeze({
            get start() {
              return ast.utf16Offset(start);
            },
            get end() {
              return ast.utf16Offset(end);
            },
            message,
          }),
        );
      }
      results.push(
        Object.freeze({
          severity,
          get start() {
            return ast.utf16Offset(start);
          },
          get end() {
            return ast.utf16Offset(end);
          },
          message,
          help,
          labels: Object.freeze(labels),
        }),
      );
    }
    if (cursor !== limit)
      { throw new RangeError("Invalid diagnostic section length"); }
    return Object.freeze(results);
  }
}

/**
 * Open an AST buffer. Pass the exact `source` used to create the buffer to
 * enable source slices for strings that reference the original source.
 */
export function readAst(buffer: BufferInput, source?: string): AstView {
  return new AstView(buffer, source);
}

function inputBytes(input: BufferInput): Uint8Array {
  return ArrayBuffer.isView(input)
    ? new Uint8Array(input.buffer, input.byteOffset, input.byteLength)
    : new Uint8Array(input);
}

function decodeWtf8(bytes: Uint8Array): string {
  if (!bytes.includes(0xed)) { return decoder.decode(bytes); }
  let result = "",
    chars: number[] = [];
  for (let i = 0; i < bytes.length;) {
    const first = bytes[i++];
    let value, remaining, minimum;
    if (first < 0x80) {
      value = first;
      remaining = 0;
      minimum = 0;
    } else if (first >= 0xc2 && first <= 0xdf) {
      value = first & 31;
      remaining = 1;
      minimum = 0x80;
    } else if (first >= 0xe0 && first <= 0xef) {
      value = first & 15;
      remaining = 2;
      minimum = 0x800;
    } else if (first >= 0xf0 && first <= 0xf4) {
      value = first & 7;
      remaining = 3;
      minimum = 0x10000;
    } else { throw new RangeError("Invalid WTF-8 string"); }
    if (i + remaining > bytes.length)
      { throw new RangeError("Truncated WTF-8 string"); }
    while (remaining--) {
      const byte = bytes[i++];
      if ((byte & 0xc0) !== 0x80)
        { throw new RangeError("Invalid WTF-8 continuation"); }
      value = (value << 6) | (byte & 63);
    }
    if (value < minimum || value > 0x10ffff)
      { throw new RangeError("Invalid WTF-8 code point"); }
    if (value <= 0xffff) { chars.push(value); }
    else {
      value -= 0x10000;
      chars.push(0xd800 | (value >> 10), 0xdc00 | (value & 1023));
    }
    if (chars.length >= 2048) {
      result += String.fromCharCode(...chars);
      chars = [];
    }
  }
  return result + String.fromCharCode(...chars);
}

// Valid UTF-8 decodes natively; WTF-8 lone surrogates fail the fatal decode and
// route the range to the exact WTF-8 decoder below.
function decodeUtf8OrWtf8(bytes: Uint8Array): string {
  try {
    return decoder.decode(bytes);
  } catch {
    return decodeWtf8(bytes);
  }
}
