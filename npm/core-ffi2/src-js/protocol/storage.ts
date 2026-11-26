import type { AstView } from "./index.ts";
import type { FieldDescriptor } from "./types.ts";
import { layouts, nativeExtraCounts, nativeField } from "./schema.ts";
import { initialFields } from "./protocol.ts";
import { NONE, NONE as U32_MAX, ROW_SIZE, EXTRA_SIZE, UTF8_STRINGS } from "./layout.ts";

const encoder = new TextEncoder();

export const isWellFormed: (value: string) => boolean = typeof "".isWellFormed === "function"
  ? value => value.isWellFormed()
  : value => {
      for (let index = 0; index < value.length; index++) {
        const code = value.charCodeAt(index);
        if (code >= 0xd800 && code <= 0xdbff) {
          const next = value.charCodeAt(++index);
          if (!(next >= 0xdc00 && next <= 0xdfff)) { return false; }
        } else if (code >= 0xdc00 && code <= 0xdfff) { return false; }
      }
      return true;
    };

/**
 * Appended nodes, extra data, and strings in one buffer. Array counts and AST
 * metadata are committed before publishing. Direct view/bytes writes must first call
 * ensureWritable(), and addresses must be reacquired after a possible growth.
 * Strings use UTF-8/WTF-8, so native reads preserve their handles when merging.
 */
export class DeltaArena {
  declare baseNodeCount: number;
  declare baseExtraCount: number;
  declare baseStringLength: number;
  declare baseByteLength: number;
  declare rootId: number;
  declare flags: number;
  declare rowsOffset: number;
  declare metadataOffset: number;
  declare extraOffset: number;
  declare stringsOffset: number;
  declare rowCount: number;
  declare newNodeCount: number;
  declare extraUsed: number;
  declare stringUsed: number;
  declare rowCapacity: number;
  declare extraCapacity: number;
  declare stringsCapacity: number;
  declare bytes?: Uint8Array;
  declare view?: DataView;
  declare stringIds?: Map<string, { start: number; end: number }>;
  // Keep the empty layout alive when all input ASTs in a batch are collected.
  static #shape = new this({ nodeCount: 0, extraCount: 0, stringLength: 0,
    byteLength: 0, rootId: 0, flags: 0 });
  constructor(base: {nodeCount:number;extraCount:number;stringLength:number;byteLength:number;rootId:number;flags:number}) {
    this.baseNodeCount = u32(base.nodeCount, "Base node count");
    this.baseExtraCount = u32(base.extraCount, "Base extra-data count");
    this.baseStringLength = u32(base.stringLength, "Base string length");
    this.baseByteLength = u32(base.byteLength, "Base byte length");
    this.rootId = u32(base.rootId, "Root node ID");
    this.flags = u32(base.flags, "AST flags");
    this.bytes = undefined;
    this.view = undefined;
    this.rowsOffset = 8;
    this.metadataOffset = 0;
    this.extraOffset = 8;
    this.stringsOffset = 8;
    this.rowCount = 0;
    this.newNodeCount = 0;
    this.extraUsed = 0;
    this.stringUsed = 0;
    this.rowCapacity = 0;
    this.extraCapacity = 0;
    this.stringsCapacity = 0;
  }

  get nodeCount() {
    return this.baseNodeCount + this.newNodeCount;
  }

  get extraCount() {
    return this.baseExtraCount + this.extraUsed;
  }

  get stringLength() {
    return this.baseStringLength + this.stringUsed;
  }

  has(id: number) {
    return this.row(id) !== -1;
  }

  row(id: number) {
    return Number.isInteger(id) && id >= this.baseNodeCount && id < this.nodeCount
      ? this.rowsOffset + (id - this.baseNodeCount) * ROW_SIZE : -1;
  }

  _requiredRow(id: number) {
    const offset = this.row(id);
    if (offset === -1) { throw new RangeError("AST node has no delta row"); }
    return offset;
  }

  ensureRow(id: number, kind: number, start = 0, end = start) {
    u32(id, "Node ID"); kindValue(kind); spanValues(start, end);
    if (id < this.baseNodeCount || id > this.nodeCount || id === U32_MAX)
      { throw new RangeError("New AST node IDs must be contiguous"); }
    if (id === this.nodeCount) {
      this.reserve(this.rowCount + 1);
      this.rowCount++; this.newNodeCount++;
    } else { this.ensureWritable(); }
    const offset = this.rowsOffset + (id - this.baseNodeCount) * ROW_SIZE;
    this.view!.setUint32(offset, start, true);
    this.view!.setUint32(offset + 4, end, true);
    this.view!.setUint8(offset + 8, kind);
    return offset;
  }

  nodePayload(id: number) {
    const offset = this.row(id);
    return offset === -1 ? -1 : offset + 16;
  }

  kind(id: number) {
    const offset = this._requiredRow(id);
    return this.view!.getUint8(offset + 8);
  }

  spanStart(id: number) {
    const offset = this._requiredRow(id);
    return this.view!.getUint32(offset, true);
  }

  spanEnd(id: number) {
    const offset = this._requiredRow(id);
    return this.view!.getUint32(offset + 4, true);
  }

  setSpan(id: number, start: number, end: number) {
    spanValues(start, end);
    const offset = this._requiredRow(id);
    const view = this.ensureWritable();
    view.setUint32(offset, start, true);
    view.setUint32(offset + 4, end, true);
  }

  /** Append one validated node row and its extra block with a single capacity check. */
  appendNode(kind: number, start: number, end: number, extraCount: number) {
    this.reserve(this.rowCount + 1, this.extraUsed + extraCount);
    const id = this.nodeCount;
    this.rowCount++; this.newNodeCount++;
    const offset = this.rowsOffset + (id - this.baseNodeCount) * ROW_SIZE;
    this.view!.setUint32(offset, start, true);
    this.view!.setUint32(offset + 4, end, true);
    this.view!.setUint8(offset + 8, kind);
    const extra = this.extraCount;
    this.extraUsed += extraCount;
    return { offset, extra };
  }

  appendExtra(count: number) {
    u32(count, "Extra-data count");
    const start = this.extraCount;
    u32(start + count, "Extra-data count");
    if (count !== 0) {
      this.reserve(this.rowCount, this.extraUsed + count);
      this.extraUsed += count;
    }
    return start;
  }

  extraAddress(index: number) {
    if (!Number.isInteger(index) || index < this.baseExtraCount || index >= this.extraCount)
      { throw new RangeError("Extra-data index is outside the delta arena"); }
    return this.extraOffset + (index - this.baseExtraCount) * EXTRA_SIZE;
  }

  /** Reuse the pool range of an identical appended string; append only on a miss. */
  internString(value: string): { start: number; end: number } {
    const ids = this.stringIds ??= new Map();
    const known = ids.get(value);
    if (known !== undefined) { return known; }
    const start = this.stringLength;
    this.appendString(value);
    const range = { start, end: this.stringLength };
    ids.set(value, range);
    return range;
  }

  /**
   * Append a string to the pool as UTF-8/WTF-8. The handle is `stringLength` before
   * and after the call, so committing a string allocates no pair.
   */
  appendString(value: string) {
    if (typeof value !== "string") { throw new TypeError("Expected an AST string"); }
    const length = value.length * 3;
    if (length === 0) { return; }
    const used = this.stringUsed;
    u32(this.baseStringLength + used + length, "String pool");
    this.reserve(this.rowCount, this.extraUsed, used + length);
    const offset = this.stringsOffset + used;
    const written = isWellFormed(value)
      ? encoder.encodeInto(value, this.bytes!.subarray(offset, offset + length)).written
      : encodeWtf8(value, this.bytes!, offset);
    this.stringUsed = used + written;
  }

  /** Reserve absolute section counts, excluding the immutable base sections. */
  reserve(rows = this.rowCount, extra = this.extraUsed, strings = this.stringUsed) {
    u32(rows, "Row capacity");
    u32(extra, "Extra-data capacity");
    u32(strings, "String capacity");
    let rowCapacity = capacity(this.rowCapacity, rows, 4);
    let extraCapacity = capacity(this.extraCapacity, extra, 8);
    let stringsCapacity = align8(capacity(this.stringsCapacity, strings, 64));
    if (8 + rowCapacity * ROW_SIZE + 8 + extraCapacity * EXTRA_SIZE + 8 + stringsCapacity + 28 > U32_MAX) {
      // Near the wire limit, use the required capacity instead of doubling.
      rowCapacity = Math.max(this.rowCapacity, rows);
      extraCapacity = Math.max(this.extraCapacity, extra);
      stringsCapacity = align8(Math.max(this.stringsCapacity, strings));
    }
    if (
      rowCapacity !== this.rowCapacity ||
      extraCapacity !== this.extraCapacity ||
      stringsCapacity !== this.stringsCapacity
    ) { this._resize(rowCapacity, extraCapacity, stringsCapacity); }
    else { this.ensureWritable(); }
    return this.view!;
  }

  ensureWritable() {
    if (this.bytes === undefined)
      { this._resize(this.rowCapacity, this.extraCapacity, this.stringsCapacity); }
    return this.view!;
  }

  _resize(rowCapacity: number, extraCapacity: number, stringsCapacity: number) {
    const extraOffset = 8 + rowCapacity * ROW_SIZE + 8;
    const stringsOffset = extraOffset + extraCapacity * EXTRA_SIZE + 8;
    const metadataOffset = stringsOffset + stringsCapacity;
    const length = u32(metadataOffset + 28, "Delta arena capacity");
    const bytes = new Uint8Array(length);
    if (this.bytes !== undefined) {
      bytes.set(this.bytes!.subarray(this.rowsOffset, this.rowsOffset + this.rowCount * ROW_SIZE), 8);
      bytes.set(this.bytes!.subarray(this.extraOffset, this.extraOffset + this.extraUsed * EXTRA_SIZE), extraOffset);
      bytes.set(this.bytes!.subarray(this.stringsOffset, this.stringsOffset + this.stringUsed), stringsOffset);
    }
    this.bytes = bytes;
    this.view = new DataView(bytes.buffer);
    this.extraOffset = extraOffset;
    this.stringsOffset = stringsOffset;
    this.metadataOffset = metadataOffset;
    this.rowCapacity = rowCapacity;
    this.extraCapacity = extraCapacity;
    this.stringsCapacity = stringsCapacity;
    this._writeLayout();
  }

  _writeLayout() {
    const view = this.view!;
    view.setUint32(0, this.rowCount, true);
    view.setUint32(4, this.rowCapacity | UTF8_STRINGS, true);
    view.setUint32(this.extraOffset - 8, this.extraUsed, true);
    view.setUint32(this.extraOffset - 4, this.extraCapacity, true);
    view.setUint32(this.stringsOffset - 8, this.stringUsed, true);
    view.setUint32(this.stringsOffset - 4, this.stringsCapacity, true);
    const metadata = this.metadataOffset;
    view.setUint32(metadata, this.rootId, true);
    view.setUint32(metadata + 4, this.flags, true);
    view.setUint32(metadata + 8, this.newNodeCount, true);
    view.setUint32(metadata + 12, this.baseNodeCount, true);
    view.setUint32(metadata + 16, this.baseExtraCount, true);
    view.setUint32(metadata + 20, this.baseStringLength, true);
    view.setUint32(metadata + 24, this.baseByteLength, true);
  }

  _publish(rootId: number, flags: number) {
    u32(rootId, "Root node ID");
    u32(flags, "AST flags");
    if (rootId >= this.nodeCount) { throw new RangeError("Invalid AST root node ID"); }
    this.ensureWritable();
    this.rootId = rootId; this.flags = flags;
    this._writeLayout();
    return this.bytes!;
  }

  /** The caller must finish reading synchronously before any subsequent edit. */
  borrow(rootId = this.rootId, flags = this.flags) {
    return this._publish(rootId, flags);
  }
}

export function ensureArena(ast: AstView) {
  if (ast._arena === null) {
    ast._arena = new DeltaArena({ nodeCount: ast.nodeCount, extraCount: ast._extraCount,
      stringLength: ast._stringLength, byteLength: ast.buffer.byteLength,
      rootId: ast.rootId, flags: ast._flags });
  }
  return ast._arena;
}

export function fieldLocation(ast: AstView, id: number, descriptor: FieldDescriptor, kind = ast.kind(id)) {
  const layout = layouts[kind], arena = ast._arena;
  const row = ast.row(id);
  let view = row === -1 ? ast._view : arena!.view!;
  const payload = ast._payloadAt(id, row);
  if (!descriptor[0]) { return { view, offset: payload + descriptor[1] }; }
  const start = view.getUint32(payload + layout.extra!, true);
  const index = start + descriptor[1];
  const words = descriptor[2] === "list" || descriptor[2] === "string" || descriptor[2] === "f64" ? 2 : 1;
  const limit = start < ast._extraCount ? ast._extraCount : ast.extraDataCount;
  if (index + words > limit) { throw new RangeError("AST extra-data reference out of bounds"); }
  view = index < ast._extraCount ? ast._view : arena!.view!;
  return { view, offset: index < ast._extraCount ? ast._extra + index * 4 : arena!.extraAddress(index) };
}

/** Allocate a native slot. Required child references must be filled before attachment. */
export function newNode(ast: AstView, kind: number, start = 0, end = start): number {
  const id = newNodeRaw(ast, kind, start, end);
  initializeFields(ast, id, kind);
  return id;
}

/** Allocate a slot without defaults; callers must write every field. */
export function newNodeRaw(ast: AstView, kind: number, start = 0, end = start, bump = true): number {
  if (!Number.isInteger(kind) || !layouts[kind]) { throw new RangeError("Invalid AST node kind"); }
  if (!Number.isInteger(start) || !Number.isInteger(end) || start < 0 || end < start || end > ast._sourceLength) { throw new RangeError("Invalid AST source span"); }
  const arena = ensureArena(ast), layout = layouts[kind], count = nativeExtraCounts[kind];
  const id = arena.nodeCount;
  const { offset, extra } = arena.appendNode(kind, start, end, count);
  const payload = offset + 16;
  arena.bytes!.fill(0, payload, payload + 16);
  if (layout.extra !== null) { arena.view!.setUint32(payload + layout.extra, extra, true); }
  if (bump) { ast._editVersion++; }
  return id;
}

/** Write a complete node slot in its current storage, allocating only new logical IDs. */
export function writeNode(ast: AstView, id: number, kind: number, start: number, end: number) {
  const capacities = ast._listCapacities;
  if (capacities?.delete(id) && capacities.size === 0) { ast._listCapacities = undefined; }
  const layout = layouts[kind], count = nativeExtraCounts[kind];
  if (id < ast.nodeCount) {
    const payload = ast._payloads + id * 16;
    ast._view.setUint32(ast._nodes + id * 8, start, true);
    ast._view.setUint32(ast._nodes + id * 8 + 4, end, true);
    ast._view.setUint8(ast._kinds + id, kind);
    ast._bytes.fill(0, payload, payload + 16);
    if (layout.extra !== null) {
      const extra = ensureArena(ast).appendExtra(count);
      ast._view.setUint32(payload + layout.extra, extra, true);
    }
  } else {
    const arena = ensureArena(ast);
    const row = arena.ensureRow(id, kind, start, end);
    arena.bytes!.fill(0, row + 16, row + 32);
    if (layout.extra !== null) {
      const extra = arena.appendExtra(count);
      arena.view!.setUint32(row + 16 + layout.extra, extra, true);
    }
  }
  initializeFields(ast, id, kind);
}

export function setSpan(ast: AstView, id: number, start: number, end: number) {
  if (id < ast.nodeCount) {
    ast._view.setUint32(ast._nodes + id * 8, start, true);
    ast._view.setUint32(ast._nodes + id * 8 + 4, end, true);
  } else { ast._arena!.setSpan(id, start, end); }
}

/** Preserve lone surrogates that TextEncoder would replace with U+FFFD. */
function encodeWtf8(value: string, bytes: Uint8Array, offset: number): number {
  const start = offset;
  for (let index = 0; index < value.length; index++) {
    let code = value.charCodeAt(index);
    const next = value.charCodeAt(index + 1);
    if (code >= 0xd800 && code <= 0xdbff && next >= 0xdc00 && next <= 0xdfff) {
      code = 0x10000 + ((code - 0xd800) << 10) + next - 0xdc00;
      index++;
    }
    if (code < 0x80) { bytes[offset++] = code; }
    else if (code < 0x800) {
      bytes[offset++] = 0xc0 | (code >> 6);
      bytes[offset++] = 0x80 | (code & 63);
    } else if (code < 0x10000) {
      bytes[offset++] = 0xe0 | (code >> 12);
      bytes[offset++] = 0x80 | ((code >> 6) & 63);
      bytes[offset++] = 0x80 | (code & 63);
    } else {
      bytes[offset++] = 0xf0 | (code >> 18);
      bytes[offset++] = 0x80 | ((code >> 12) & 63);
      bytes[offset++] = 0x80 | ((code >> 6) & 63);
      bytes[offset++] = 0x80 | (code & 63);
    }
  }
  return offset - start;
}

function u32(value: number, name: string) {
  if (!Number.isInteger(value) || value < 0 || value > U32_MAX)
    { throw new RangeError(`${name} exceeds the AST buffer limit`); }
  return value;
}

function kindValue(value: number) {
  if (!Number.isInteger(value) || value < 0 || value > 0xff)
    { throw new RangeError("Invalid AST node kind"); }
  return value;
}

function spanValues(start: number, end: number) {
  u32(start, "Span start");
  u32(end, "Span end");
  if (end < start) { throw new RangeError("AST span ends before it starts"); }
}

function capacity(current: number, required: number, minimum: number) {
  if (required <= current) { return current; }
  return Math.max(required, Math.min(U32_MAX, Math.max(minimum, current * 2)));
}

function align8(value: number) {
  return Math.ceil(value / 8) * 8;
}

function initializeFields(ast: AstView, id: number, kind: number) {
  for (const fieldId of initialFields[kind]) {
    const descriptor = nativeField(kind, fieldId)!;
    const field = fieldLocation(ast, id, descriptor, kind);
    if (descriptor[2] === "node" || descriptor[2] === "string" && descriptor[3]) { field.view.setUint32(field.offset, NONE, true); }
    else if (descriptor[2].startsWith("enum:") && descriptor[3]) { field.view.setUint8(field.offset, 255); }
  }
}
