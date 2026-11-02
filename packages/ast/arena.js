import { SCHEMA_ID } from "./schema.js";

const HEADER_SIZE = 96;
const ROW_SIZE = 32;
const EXTRA_SIZE = 8;
const U32_MAX = 0xffffffff;
const PAGE_SHIFT = 10;
const PAGE_SIZE = 1 << PAGE_SHIFT;
const PAGE_MASK = PAGE_SIZE - 1;

function u32(value, name) {
  if (!Number.isInteger(value) || value < 0 || value > U32_MAX)
    throw new RangeError(`${name} exceeds the AST buffer limit`);
  return value;
}

function kindValue(value) {
  if (!Number.isInteger(value) || value < 0 || value > 0xff)
    throw new RangeError("Invalid AST node kind");
  return value;
}

function spanValues(start, end) {
  u32(start, "Span start");
  u32(end, "Span end");
  if (end < start) throw new RangeError("AST span ends before it starts");
}

function wordOffset(value, size) {
  if (!Number.isInteger(value) || value < 0 || value + 4 > size || value % 4)
    throw new RangeError("Invalid AST word offset");
  return value;
}

function capacity(current, required, minimum) {
  if (required <= current) return current;
  return Math.max(required, Math.min(U32_MAX, Math.max(minimum, current * 2)));
}

function align8(value) {
  return Math.ceil(value / 8) * 8;
}

function wtf8Length(value) {
  let length = 0;
  for (let i = 0; i < value.length; i++) {
    const code = value.charCodeAt(i);
    if (code < 0x80) length++;
    else if (code < 0x800) length += 2;
    else if (
      code >= 0xd800 && code <= 0xdbff &&
      value.charCodeAt(i + 1) >= 0xdc00 && value.charCodeAt(i + 1) <= 0xdfff
    ) {
      length += 4;
      i++;
    } else length += 3;
  }
  return length;
}

function writeWtf8(bytes, offset, value) {
  for (let i = 0; i < value.length; i++) {
    let code = value.charCodeAt(i);
    if (code < 0x80) bytes[offset++] = code;
    else if (code < 0x800) {
      bytes[offset++] = 0xc0 | (code >> 6);
      bytes[offset++] = 0x80 | (code & 63);
    } else {
      const next = value.charCodeAt(i + 1);
      if (code >= 0xd800 && code <= 0xdbff && next >= 0xdc00 && next <= 0xdfff) {
        code = 0x10000 + ((code - 0xd800) << 10) + next - 0xdc00;
        bytes[offset++] = 0xf0 | (code >> 18);
        bytes[offset++] = 0x80 | ((code >> 12) & 63);
        bytes[offset++] = 0x80 | ((code >> 6) & 63);
        bytes[offset++] = 0x80 | (code & 63);
        i++;
      } else {
        // Lone surrogates retain their code units in WTF-8.
        bytes[offset++] = 0xe0 | (code >> 12);
        bytes[offset++] = 0x80 | ((code >> 6) & 63);
        bytes[offset++] = 0x80 | (code & 63);
      }
    }
  }
}

/**
 * Mutable SWE3 rows, extra data, and strings in one buffer. Counts are committed
 * to the header when publishing. Direct view/bytes writes must first call
 * ensureWritable(), and addresses must be reacquired after a possible growth.
 */
export class DeltaArena {
  // Keep the empty layout alive when all input ASTs in a batch are collected.
  static #shape = new this({ nodeCount: 0, extraCount: 0, stringLength: 0,
    byteLength: 0, rootId: 0, flags: 0 });
  constructor(base) {
    this.baseNodeCount = u32(base.nodeCount, "Base node count");
    this.baseExtraCount = u32(base.extraCount, "Base extra-data count");
    this.baseStringLength = u32(base.stringLength, "Base string length");
    this.baseByteLength = u32(base.byteLength, "Base byte length");
    this.schemaId = u32(base.schemaId ?? SCHEMA_ID, "Schema ID");
    if (this.schemaId !== SCHEMA_ID)
      throw new RangeError("AST schema version mismatch");
    this.rootId = u32(base.rootId, "Root node ID");
    this.flags = u32(base.flags, "AST flags");
    this.bytes = null;
    this.view = null;
    this.rowsOffset = HEADER_SIZE;
    this.extraOffset = HEADER_SIZE;
    this.stringsOffset = HEADER_SIZE;
    this.rowCount = 0;
    this.newNodeCount = 0;
    this.extraUsed = 0;
    this.stringUsed = 0;
    this.rowCapacity = 0;
    this.extraCapacity = 0;
    this.stringsCapacity = 0;
    // Zero is absent; all other entries store a physical row index plus one.
    // Pages contain only numbers, and do not retain node or field values.
    this._pages = [];
    this._sealed = false;
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

  has(id) {
    return this.row(id) !== -1;
  }

  row(id) {
    if (!Number.isInteger(id) || id < 0 || id >= this.nodeCount) return -1;
    const page = this._pages[id >>> PAGE_SHIFT];
    const index = page === undefined ? 0 : page[id & PAGE_MASK];
    return index === 0 ? -1 : this.rowsOffset + (index - 1) * ROW_SIZE;
  }

  _requiredRow(id) {
    const offset = this.row(id);
    if (offset === -1) throw new RangeError("AST node has no delta row");
    return offset;
  }

  ensureRow(id, kind, start = 0, end = start) {
    u32(id, "Node ID");
    kindValue(kind);
    spanValues(start, end);
    const total = this.nodeCount;
    if (id > total || (id === total && total === U32_MAX))
      throw new RangeError("New AST node IDs must be contiguous");
    let offset = this.row(id);
    if (offset === -1) {
      this.reserve(this.rowCount + 1);
      const pageId = id >>> PAGE_SHIFT;
      let page = this._pages[pageId];
      if (page === undefined) this._pages[pageId] = page = new Uint32Array(PAGE_SIZE);
      offset = this.rowsOffset + this.rowCount * ROW_SIZE;
      page[id & PAGE_MASK] = ++this.rowCount;
      if (id === total) this.newNodeCount++;
      this.view.setUint32(offset + 12, id, true);
    } else this.ensureWritable();
    this.view.setUint32(offset, start, true);
    this.view.setUint32(offset + 4, end, true);
    this.view.setUint8(offset + 8, kind);
    this.view.setUint8(offset + 9, 1);
    return offset;
  }

  nodePayload(id) {
    const offset = this.row(id);
    return offset === -1 ? -1 : offset + 16;
  }

  kind(id) {
    const offset = this._requiredRow(id);
    return this.view.getUint8(offset + 8);
  }

  spanStart(id) {
    const offset = this._requiredRow(id);
    return this.view.getUint32(offset, true);
  }

  spanEnd(id) {
    const offset = this._requiredRow(id);
    return this.view.getUint32(offset + 4, true);
  }

  setKind(id, kind) {
    kindValue(kind);
    const offset = this._requiredRow(id);
    this.ensureWritable().setUint8(offset + 8, kind);
  }

  setSpan(id, start, end) {
    spanValues(start, end);
    const offset = this._requiredRow(id);
    const view = this.ensureWritable();
    view.setUint32(offset, start, true);
    view.setUint32(offset + 4, end, true);
  }

  setPayloadWord(id, byteOffset, value) {
    wordOffset(byteOffset, 16);
    u32(value, "Payload word");
    const offset = this._requiredRow(id) + 16 + byteOffset;
    this.ensureWritable().setUint32(offset, value, true);
  }

  copyPayload(id, source, sourceOffset) {
    const offset = this._requiredRow(id) + 16;
    // Read before making the arena writable, including when source aliases it.
    const a = source.getUint32(sourceOffset, true);
    const b = source.getUint32(sourceOffset + 4, true);
    const c = source.getUint32(sourceOffset + 8, true);
    const d = source.getUint32(sourceOffset + 12, true);
    const view = this.ensureWritable();
    view.setUint32(offset, a, true);
    view.setUint32(offset + 4, b, true);
    view.setUint32(offset + 8, c, true);
    view.setUint32(offset + 12, d, true);
  }

  appendExtra(count) {
    u32(count, "Extra-data count");
    const start = this.extraCount;
    u32(start + count, "Extra-data count");
    if (count !== 0) {
      this.reserve(this.rowCount, this.extraUsed + count);
      this.extraUsed += count;
    }
    return start;
  }

  extraAddress(index) {
    if (!Number.isInteger(index) || index < this.baseExtraCount || index >= this.extraCount)
      throw new RangeError("Extra-data index is outside the delta arena");
    return this.extraOffset + (index - this.baseExtraCount) * EXTRA_SIZE;
  }

  readExtraWord(index, byteOffset = 0) {
    wordOffset(byteOffset, EXTRA_SIZE);
    const offset = this.extraAddress(index) + byteOffset;
    return this.view.getUint32(offset, true);
  }

  writeExtraWord(index, byteOffset, value) {
    wordOffset(byteOffset, EXTRA_SIZE);
    u32(value, "Extra-data word");
    const offset = this.extraAddress(index) + byteOffset;
    this.ensureWritable().setUint32(offset, value, true);
  }

  appendString(value) {
    if (typeof value !== "string") throw new TypeError("Expected an AST string");
    const length = wtf8Length(value);
    const start = this.stringLength;
    const end = u32(start + length, "String pool");
    if (length !== 0) {
      this.reserve(this.rowCount, this.extraUsed, this.stringUsed + length);
      writeWtf8(this.bytes, this.stringsOffset + this.stringUsed, value);
      this.stringUsed += length;
    }
    return [start, end];
  }

  appendStringBytes(bytes) {
    if (!(bytes instanceof Uint8Array)) throw new TypeError("Expected encoded AST string bytes");
    const length = bytes.byteLength;
    const start = this.stringLength;
    const end = u32(start + length, "String pool");
    if (length !== 0) {
      this.reserve(this.rowCount, this.extraUsed, this.stringUsed + length);
      this.bytes.set(bytes, this.stringsOffset + this.stringUsed);
      this.stringUsed += length;
    }
    return [start, end];
  }

  /** Reserve absolute section counts, excluding the immutable base sections. */
  reserve(rows = this.rowCount, extra = this.extraUsed, strings = this.stringUsed) {
    u32(rows, "Row capacity");
    u32(extra, "Extra-data capacity");
    u32(strings, "String capacity");
    let rowCapacity = capacity(this.rowCapacity, rows, 4);
    let extraCapacity = capacity(this.extraCapacity, extra, 8);
    let stringsCapacity = align8(capacity(this.stringsCapacity, strings, 64));
    if (HEADER_SIZE + rowCapacity * ROW_SIZE + extraCapacity * EXTRA_SIZE + stringsCapacity > U32_MAX) {
      // Near the wire limit, use the required capacity instead of doubling.
      rowCapacity = Math.max(this.rowCapacity, rows);
      extraCapacity = Math.max(this.extraCapacity, extra);
      stringsCapacity = align8(Math.max(this.stringsCapacity, strings));
    }
    if (
      rowCapacity !== this.rowCapacity ||
      extraCapacity !== this.extraCapacity ||
      stringsCapacity !== this.stringsCapacity
    ) this._resize(rowCapacity, extraCapacity, stringsCapacity);
    else this.ensureWritable();
    return this.view;
  }

  ensureWritable() {
    if (this.bytes === null || this._sealed)
      this._resize(this.rowCapacity, this.extraCapacity, this.stringsCapacity);
    return this.view;
  }

  _resize(rowCapacity, extraCapacity, stringsCapacity) {
    const extraOffset = HEADER_SIZE + rowCapacity * ROW_SIZE;
    const stringsOffset = extraOffset + extraCapacity * EXTRA_SIZE;
    const length = u32(stringsOffset + stringsCapacity, "Delta arena capacity");
    const bytes = new Uint8Array(length);
    if (this.bytes !== null) {
      bytes.set(this.bytes.subarray(this.rowsOffset, this.rowsOffset + this.rowCount * ROW_SIZE), HEADER_SIZE);
      bytes.set(this.bytes.subarray(this.extraOffset, this.extraOffset + this.extraUsed * EXTRA_SIZE), extraOffset);
      bytes.set(this.bytes.subarray(this.stringsOffset, this.stringsOffset + this.stringUsed), stringsOffset);
    }
    this.bytes = bytes;
    this.view = new DataView(bytes.buffer);
    this.extraOffset = extraOffset;
    this.stringsOffset = stringsOffset;
    this.rowCapacity = rowCapacity;
    this.extraCapacity = extraCapacity;
    this.stringsCapacity = stringsCapacity;
    this._sealed = false;
    this._writeHeader();
  }

  _writeHeader() {
    const view = this.view;
    view.setUint32(0, 0x33455753, true);
    view.setUint32(4, 3, true);
    view.setUint32(8, this.schemaId, true);
    view.setUint32(12, this.bytes.byteLength, true);
    view.setUint32(16, this.baseNodeCount, true);
    view.setUint32(20, this.baseExtraCount, true);
    view.setUint32(24, this.baseStringLength, true);
    view.setUint32(28, this.baseByteLength, true);
    view.setUint32(32, this.newNodeCount, true);
    view.setUint32(36, this.rowCount, true);
    view.setUint32(40, this.extraUsed, true);
    view.setUint32(44, this.stringUsed, true);
    view.setUint32(48, this.rootId, true);
    view.setUint32(52, this.flags, true);
    view.setUint32(56, this.rowsOffset, true);
    view.setUint32(60, this.rowCapacity, true);
    view.setUint32(64, this.extraOffset, true);
    view.setUint32(68, this.extraCapacity, true);
    view.setUint32(72, this.stringsOffset, true);
    view.setUint32(76, this.stringsCapacity, true);
    view.setUint32(80, 0, true);
    view.setUint32(84, 0, true);
    view.setUint32(88, 0, true);
    view.setUint32(92, 0, true);
  }

  _publish(rootId, flags) {
    u32(rootId, "Root node ID");
    u32(flags, "AST flags");
    if (rootId >= this.nodeCount) throw new RangeError("Invalid AST root node ID");
    if (
      rootId !== this.rootId || flags !== this.flags ||
      (this._sealed && (
        rootId !== this.view.getUint32(48, true) ||
        flags !== this.view.getUint32(52, true)
      ))
    ) {
      this.ensureWritable();
      this.rootId = rootId;
      this.flags = flags;
    } else if (this.bytes === null) this.ensureWritable();
    if (!this._sealed) this._writeHeader();
    return this.bytes;
  }

  /** Public snapshots keep their contents when the arena is edited again. */
  snapshot(rootId = this.rootId, flags = this.flags) {
    const bytes = this._publish(rootId, flags);
    this._sealed = true;
    return bytes;
  }

  /** The caller must finish reading synchronously before any subsequent edit. */
  borrow(rootId = this.rootId, flags = this.flags) {
    return this._publish(rootId, flags);
  }
}
