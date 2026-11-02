import { layouts, enums, NodeKind, SCHEMA_ID } from "./schema.js";
import {
  normalizeId,
  projectedField,
  projectedKeys,
  projectedList,
  parameters,
  mirrorKey,
} from "./projection.js";
import { NodeView, createNodeView } from "./node.js";
import { writeField, finishEdits, createNode, commitList } from "./writer.js";
import { ensureArena, ensureEditable, writableField, syncAliases, refreshParents } from "./storage.js";
import { ensureTopology, ensureNavigation } from "./topology.js";

export { NodeKind } from "./schema.js";
export { NodeView } from "./node.js";
export const FORMAT_VERSION = 2;
const HEADER_SIZE = 72;
const SPAN_SIZE = 8;
const PAYLOAD_SIZE = 16;
const NONE = 0xffffffff;
const LAYOUT_TEMPLATE = Symbol("AST view layout");
const decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
// Immutable parser metadata depends only on the small header flag set. Sharing
// it avoids a per-parse allocation/freeze and keeps its V8 layouts live.
const parserMeta = Array.from({ length: 20 }, (_, flags) => Object.freeze({
  lang: enums.Lang[flags >> 2],
  sourceType: enums.SourceType[flags & 3],
}));
const typedArrayPrototype = Object.getPrototypeOf(Uint8Array.prototype);
const typedArrayName = Object.getOwnPropertyDescriptor(typedArrayPrototype, Symbol.toStringTag).get;
const typedArrayAccess = {
  buffer: Object.getOwnPropertyDescriptor(typedArrayPrototype, "buffer").get,
  offset: Object.getOwnPropertyDescriptor(typedArrayPrototype, "byteOffset").get,
  length: Object.getOwnPropertyDescriptor(typedArrayPrototype, "byteLength").get,
};
const dataViewAccess = {
  buffer: Object.getOwnPropertyDescriptor(DataView.prototype, "buffer").get,
  offset: Object.getOwnPropertyDescriptor(DataView.prototype, "byteOffset").get,
  length: Object.getOwnPropertyDescriptor(DataView.prototype, "byteLength").get,
};

function inputBytes(input) {
  if (!ArrayBuffer.isView(input))
    return input instanceof ArrayBuffer ? new Uint8Array(input) : null;
  // Read intrinsic slots without specializing on a temporary Buffer subclass
  // map. The intrinsic brand check also handles views from another realm.
  const access = typedArrayName.call(input) === undefined ? dataViewAccess : typedArrayAccess;
  return new Uint8Array(access.buffer.call(input), access.offset.call(input), access.length.call(input));
}

function decodeWtf8(bytes) {
  if (!bytes.includes(0xed)) return decoder.decode(bytes);
  let result = "",
    chars = [];
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
    } else throw new RangeError("Invalid WTF-8 string");
    if (i + remaining > bytes.length)
      throw new RangeError("Truncated WTF-8 string");
    while (remaining--) {
      const byte = bytes[i++];
      if ((byte & 0xc0) !== 0x80)
        throw new RangeError("Invalid WTF-8 continuation");
      value = (value << 6) | (byte & 63);
    }
    if (value < minimum || value > 0x10ffff)
      throw new RangeError("Invalid WTF-8 code point");
    if (value <= 0xffff) chars.push(value);
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

export class AstView {
  // A buffer-free template keeps the completed layout reachable across GC.
  // Initialize fields before parsing so the constructor only updates slots.
  static #shape = new this(LAYOUT_TEMPLATE);
  #nodeCount = 0;
  #rootId = 0;
  #meta = null;
  _bytes = null;
  _view = null;
  _nodes = 0;
  _kinds = 0;
  _payloads = 0;
  _parents = null;
  _firstChildren = null;
  _nextSiblings = null;
  _aliases = null;
  _extraNodes = null;
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
  _isTs = false;
  _edits = null;
  _watchedLists = null;
  _listVersion = 0;
  _watchedObjects = null;
  _arrayNodes = null;
  _positionCheckpoints = null;
  _positionByte = 0;
  _positionUtf16 = 0;
  _utf8CursorByte = 0;
  _utf8CursorUnits = 0;
  _arena = null;
  _editor = null;
  _nextNodeId = 0;
  _currentParents = null;
  _aliasesNormalized = false;
  _encodingDepth = 0;
  _kindsChanged = false;
  _needsGraphValidation = false;
  constructor(input) {
    if (input === LAYOUT_TEMPLATE) return;
    const bytes = inputBytes(input);
    if (!bytes || bytes.byteLength < HEADER_SIZE)
      throw new RangeError("Invalid AST buffer");
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    if (
      view.getUint32(0, true) !== 0x32415753 ||
      view.getUint32(4, true) !== FORMAT_VERSION ||
      view.getUint32(8, true) !== SCHEMA_ID
    )
      throw new RangeError("AST format or schema version mismatch");
    const flags = view.getUint32(12, true),
      nodeCount = view.getUint32(20, true),
      rootId = view.getUint32(24, true),
      extraCount = view.getUint32(32, true);
    const nodes = view.getUint32(28, true),
      extra = view.getUint32(36, true),
      strings = view.getUint32(40, true),
      sourceLength = view.getUint32(44, true),
      stringLength = view.getUint32(48, true);
    const kinds = nodes + nodeCount * SPAN_SIZE;
    const payloads = Math.ceil((kinds + nodeCount) / 8) * 8;
    const comments = view.getUint32(52, true),
      commentCount = view.getUint32(56, true),
      diagnostics = view.getUint32(60, true),
      diagnosticCount = view.getUint32(64, true),
      diagnosticBytes = view.getUint32(68, true);
    if (
      view.getUint32(16, true) !== bytes.byteLength ||
      !nodeCount ||
      rootId >= nodeCount ||
      flags > 31 ||
      ((flags >> 2) & 7) > 4 ||
      nodes !== HEADER_SIZE ||
      extra !== payloads + nodeCount * PAYLOAD_SIZE ||
      strings !== extra + extraCount * 8 ||
      sourceLength > stringLength ||
      comments !== Math.ceil((strings + stringLength) / 4) * 4 ||
      diagnostics !== comments + commentCount * 8 ||
      diagnostics + diagnosticBytes !== bytes.byteLength ||
      diagnosticCount * 18 > diagnosticBytes
    )
      throw new RangeError("Invalid AST section layout");
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
    this._isTs = (flags >> 2) === 1 || (flags >> 2) === 3 || (flags >> 2) === 4;
    this._watchedLists = new Map();
    this._watchedObjects = new Map();
    this._arrayNodes = new WeakMap();
    this._positionCheckpoints = [0, 0];
    this.#nodeCount = nodeCount;
    this._nextNodeId = nodeCount;
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
  get totalNodeCount() { return Math.max(this._nextNodeId, this._arena?.nodeCount ?? this.nodeCount); }
  get extraDataCount() { return this._arena?.extraCount ?? this._extraCount; }
  get stringLength() { return this._arena?.stringLength ?? this._stringLength; }
  row(id) { return this._arena === null ? -1 : this._arena.row(id); }
  ensureArena() { return ensureArena(this); }
  ensureEditable(id) { return ensureEditable(this, id); }
  writableField(id, key) { return writableField(this, id, key); }
  refreshParents(id, previous) { return refreshParents(this, id, previous); }
  commitEdit(id, previous) { return refreshParents(this, id, previous); }
  _onTopology() { if (this._arena !== null) syncAliases(this); }
  createNode(input, kind) { return createNode(this, input, kind); }
  commitList(id, key, list) { return commitList(this, id, key, list); }
  mirrorKey(id, key) { return mirrorKey(id, key); }
  get editBuffer() { return this._arena?.bytes ?? null; }
  spliceList(node, key, start, deleteCount, ...values) {
    const id = node instanceof NodeView ? node.nodeId : node;
    const list = this.getField(id, key);
    if (!Array.isArray(list)) throw new TypeError("Expected an AST list");
    const removed = list.splice(start, deleteCount, ...values);
    this.commitList(id, key, list);
    return removed;
  }
  get meta() {
    return this.#meta;
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
    return this.string(0, this._sourceLength);
  }
  get sourceUtf16Length() {
    return this.utf16Offset(this._sourceLength);
  }
  get root() {
    return this.node(this.rootId);
  }
  get program() {
    if (this.kind(this.rootId) !== NodeKind.Program)
      throw new TypeError("AST root is not a Program");
    return this.node(this.rootId);
  }
  get materializedNodeCount() {
    return this._createdNodeCount;
  }
  getField(id, key, kind) {
    const edits = this._edits?.get(id);
    if (edits?.has(key)) return edits.get(key);
    return projectedField(this, id, key, this.row(id) === -1 ? kind : this.kind(id));
  }
  setField(id, key, value) {
    return writeField(this, id, key, value);
  }
  cloneNode(node) {
    if (!(node instanceof NodeView)) return { ...node };
    const ast = node.ast;
    const keys = new Set([
      ...projectedKeys(ast, node.nodeId),
      ...(ast._edits?.get(node.nodeId)?.keys() ?? []),
    ]);
    return Object.fromEntries([...keys].map((key) => [key, node[key]]));
  }
  collectEdits(direct) {
    const dirty = new Set();
    const mark = (id) => {
      for (let steps = 0; id !== NONE; steps++) {
        if (steps >= this.nodeCount)
          throw new RangeError("Cyclic AST parent chain");
        if (dirty.has(id)) break;
        dirty.add(id);
        id = this.parentId(id);
      }
    };
    for (const id of this._edits?.keys() ?? []) {
      direct?.add(id);
      mark(id);
    }
    for (const {
      owner,
      value,
      original,
      native,
    } of this._watchedLists.values()) {
      let changed = value.length !== original.length;
      for (let i = 0; !changed && i < value.length; i++)
        changed = !NodeView.isSame(value[i], original[i]);
      if (changed) {
        direct?.add(owner);
        mark(owner);
        if (native !== undefined) {
          direct?.add(native);
          mark(native);
        }
      }
    }
    for (const {
      owner,
      key,
      value,
      original,
    } of this._watchedObjects.values()) {
      const entries =
        value instanceof RegExp
          ? [value.source, value.flags]
          : Object.entries(value);
      const changed =
        value instanceof RegExp
          ? entries.some((value, i) => value !== original[i])
          : entries.length !== original.length ||
            entries.some(
              ([key, value], i) =>
                key !== original[i][0] || value !== original[i][1],
            );
      if (changed) {
        direct?.add(owner);
        mark(owner);
        if (value instanceof RegExp && key === "value")
          this.setField(owner, "regex", {
            pattern: value.source,
            flags: value.flags,
          });
      }
    }
    // A function's projected params array also includes its separate thisParam.
    // Revisit the native parameter wrapper whenever its owner changes.
    for (const id of dirty) {
      if (layouts[this.kind(id)].fields.params?.[2] === "node") {
        const params = this.ref(id, "params");
        if (params !== NONE) mark(params);
      }
    }
    return dirty;
  }
  encode(meta) {
    return finishEdits(this, meta, true);
  }
  borrowBuffers(meta) { return finishEdits(this, meta, false); }
  rawId(id) {
    if (!Number.isInteger(id) || id < 0) throw new RangeError("Invalid AST node ID");
    if (id < this.nodeCount) return id;
    if (this._arena?.has(id) && !this._extraNodes?.[id - this.nodeCount]) return id;
    if (this._parents === null) ensureTopology(this);
    const entry = this._extraNodes[id - this.nodeCount];
    if (!entry) throw new RangeError("Invalid AST node ID");
    return entry[0];
  }
  offset(id) { return this._payloads + this.rawId(id) * PAYLOAD_SIZE; }
  kind(id) {
    if (this._arena?.has(id)) return this._arena.kind(id);
    const kind = this._view.getUint8(this._kinds + this.rawId(id));
    if (kind >= layouts.length) throw new RangeError("Unknown AST node kind");
    return kind;
  }
  parentId(id) {
    if (id === this.rootId) return NONE;
    if (this._currentParents?.has(id)) return this._currentParents.get(id);
    return this._snapshotParentId(id);
  }
  _snapshotParentId(id) {
    if (id === this.#rootId) return NONE;
    if (this._parents === null) ensureTopology(this);
    this.rawId(id);
    const parent = id < this.nodeCount ? this._parents[id] : this._extraNodes[id - this.nodeCount]?.[1] ?? NONE;
    return parent === NONE ? NONE : parent & 0x7fffffff;
  }
  parentNode(id) {
    let parent = this.parentId(id);
    for (let steps = 0; parent !== NONE; steps++) {
      if (steps >= this.nodeCount)
        throw new RangeError("Cyclic AST parent chain");
      const kind = this.kind(parent);
      if (
        (kind === NodeKind.VariableDeclarator ||
          kind === NodeKind.CatchClause ||
          kind === NodeKind.FormalParameter) &&
        this.ref(parent, "typeAnnotation") === id
      ) {
        return this.node(
          this.ref(
            parent,
            kind === NodeKind.VariableDeclarator
              ? "id"
              : kind === NodeKind.CatchClause
                ? "param"
                : "pattern",
          ),
        );
      }
      if (
        this.kind(parent) !== NodeKind.FormalParameters &&
        normalizeId(this, parent) !== id
      )
        return this.node(parent);
      parent = this.parentId(parent);
    }
    return null;
  }
  firstChildId(id) {
    if (this._firstChildren === null) ensureNavigation(this);
    this.rawId(id);
    return id < this.nodeCount ? this._firstChildren[id] : this._extraNodes[id - this.nodeCount][2] >>> 0;
  }
  nextSiblingId(id) {
    if (this._firstChildren === null) ensureNavigation(this);
    this.rawId(id);
    return id < this.nodeCount ? this._nextSiblings[id] : this._extraNodes[id - this.nodeCount][3] >>> 0;
  }
  byteStart(id) { return this._arena?.has(id) ? this._arena.spanStart(id) : this._view.getUint32(this._nodes + this.rawId(id) * SPAN_SIZE, true); }
  byteEnd(id) { return this._arena?.has(id) ? this._arena.spanEnd(id) : this._view.getUint32(this._nodes + this.rawId(id) * SPAN_SIZE + 4, true); }
  start(id) {
    return this.utf16Offset(this.byteStart(id));
  }
  end(id) {
    return this.utf16Offset(this.byteEnd(id));
  }
  /** Incrementally index UTF-8 source at character boundaries, without decoding a source string. */
  utf16Offset(byte) {
    if (!Number.isInteger(byte) || byte < 0)
      throw new RangeError("Invalid source byte offset");
    // Preserve synthetic spans outside the source, as the existing AST API does.
    if (byte > this._sourceLength) return byte;
    const checkpoints = this._positionCheckpoints;
    let cursor, units;
    if (byte >= this._positionByte) {
      cursor = this._positionByte;
      units = this._positionUtf16;
    } else {
      let low = 0,
        high = checkpoints.length / 2;
      while (low + 1 < high) {
        const mid = (low + high) >>> 1;
        if (checkpoints[mid * 2] <= byte) low = mid;
        else high = mid;
      }
      cursor = checkpoints[low * 2];
      units = checkpoints[low * 2 + 1];
    }
    let result = units;
    while (cursor < byte) {
      const first = this._bytes[this._strings + cursor];
      const width =
        first < 0x80
          ? 1
          : first >= 0xc2 && first < 0xe0
            ? 2
            : first >= 0xe0 && first < 0xf0
              ? 3
              : first >= 0xf0 && first <= 0xf4
                ? 4
                : 0;
      if (!width || cursor + width > this._sourceLength)
        throw new RangeError("Invalid UTF-8 source");
      for (let i = 1; i < width; i++)
        if ((this._bytes[this._strings + cursor + i] & 0xc0) !== 0x80)
          throw new RangeError("Invalid UTF-8 source");
      result =
        byte < cursor + width
          ? units + (width === 4 && byte === cursor + 3 ? 1 : 0)
          : units + (width === 4 ? 2 : 1);
      cursor += width;
      units += width === 4 ? 2 : 1;
      if (
        cursor > this._positionByte &&
        cursor - checkpoints[checkpoints.length - 2] >= 256
      )
        checkpoints.push(cursor, units);
    }
    if (cursor > this._positionByte) {
      this._positionByte = cursor;
      this._positionUtf16 = units;
    }
    return result;
  }
  /** Convert modified ESTree positions back to the original source's UTF-8 offsets. */
  utf8Offset(position) {
    if (!Number.isInteger(position) || position < 0)
      throw new RangeError("Invalid UTF-16 source offset");
    if (!position) return 0;
    const sourceUnits = this.sourceUtf16Length;
    if (position > sourceUnits)
      throw new RangeError("AST span is outside the original source");
    if (sourceUnits === this._sourceLength) return position;
    let cursor, units;
    // Adjacent span endpoints usually share the same source prefix. Reuse a
    // nearby character boundary; large jumps still use the bounded index.
    if (position >= this._utf8CursorUnits && position - this._utf8CursorUnits <= 64) {
      cursor = this._utf8CursorByte;
      units = this._utf8CursorUnits;
    } else {
      const checkpoints = this._positionCheckpoints;
      let low = 0, high = checkpoints.length / 2;
      while (low + 1 < high) {
        const mid = (low + high) >>> 1;
        if (checkpoints[mid * 2 + 1] <= position) low = mid;
        else high = mid;
      }
      cursor = checkpoints[low * 2];
      units = checkpoints[low * 2 + 1];
    }
    while (units < position) {
      const first = this._bytes[this._strings + cursor];
      const width = first < 0x80 ? 1 : first < 0xe0 ? 2 : first < 0xf0 ? 3 : 4;
      if (width === 4 && position === units + 1) {
        // A split surrogate position is not a UTF-8 character boundary.
        this._utf8CursorByte = cursor;
        this._utf8CursorUnits = units;
        return cursor + 3;
      }
      cursor += width;
      units += width === 4 ? 2 : 1;
    }
    this._utf8CursorByte = cursor;
    this._utf8CursorUnits = units;
    return cursor;
  }
  node(id) {
    if (id === NONE) return null;
    let raw = this.rawId(id);
    const row = this.row(id);
    let kind = row === -1
      ? this._view.getUint8(this._kinds + raw)
      : this._arena.view.getUint8(row + 8);
    if (kind >= layouts.length) throw new RangeError("Unknown AST node kind");
    if (kind === NodeKind.FormalParameter || kind === NodeKind.TsIndexSignatureParameter) {
      id = normalizeId(this, id);
      raw = this.rawId(id);
      kind = this.kind(id);
    }
    if (kind === NodeKind.FormalParameters)
      return parameters(this, this.parentId(id), id);
    this._createdNodeCount++;
    return createNodeView(this, id, kind, this._payloads + raw * PAYLOAD_SIZE);
  }
  extraOffset(payload, pointer, slot) {
    const index = this._view.getUint32(payload + pointer, true) + slot;
    if (index >= this._extraCount) throw new RangeError("AST extra-data reference out of bounds");
    return this._extra + index * 8;
  }
  stringAt(offset, optional) {
    const start = this._view.getUint32(offset, true), end = this._view.getUint32(offset + 4, true);
    return optional && start === NONE && end === 0 ? null : this.string(start, end);
  }
  enumAt(offset, name, optional) {
    const value = this._view.getUint8(offset);
    if (optional && value === 255) return null;
    const values = enums[name];
    if (value >= values.length) throw new RangeError("Unknown AST enum value");
    return values[value];
  }
  listAt(id, offset, field, view = this._view) {
    const start = view.getUint32(offset, true), end = view.getUint32(offset + 4, true);
    if (start > end || end > this.extraDataCount) throw new RangeError("AST list out of bounds");
    return this.list([[start, end]], NONE, NONE, id, field);
  }
  fieldOffset(id, key, kind = this.kind(id)) {
    const row = this.row(id);
    if (row !== -1) kind = this._arena.kind(id);
    const layout = layouts[kind],
      field = layout.fields[key];
    if (!field) return null;
    let view = row === -1 ? this._view : this._arena.view;
    let offset = row === -1 ? this.offset(id) : row + 16;
    if (field[0]) {
      const index =
        view.getUint32(offset + layout.extra, true) + field[1];
      if (index >= this.extraDataCount)
        throw new RangeError("AST extra-data reference out of bounds");
      view = index < this._extraCount ? this._view : this._arena.view;
      offset = index < this._extraCount ? this._extra + index * 8 : this._arena.extraAddress(index);
    } else offset += field[1];
    return { view, offset, type: field[2], optional: field[3] };
  }
  childAt(id, offset, optional, view = this._view) {
    const child = view.getUint32(offset, true);
    if (child === NONE) {
      if (!optional) throw new RangeError("Missing required AST child");
      return NONE;
    }
    if (view !== this._view) {
      if (child >= this.totalNodeCount) throw new RangeError("AST child out of bounds");
      return child;
    }
    const count = this.nodeCount;
    if (child >= count) throw new RangeError("AST child out of bounds");
    if (this._parents === null) ensureTopology(this);
    if (this._aliases.size === 0) return child;
    const owner = id < count ? this._parents[id] : this._extraNodes[id - count]?.[1] ?? 0;
    if ((owner & 0x80000000) === 0) return child;
    return this._aliases.get(id)?.get(offset) ?? child;
  }
  ref(id, key) {
    const field = this.fieldOffset(id, key);
    if (!field) return NONE;
    if (field.type !== "node") throw new TypeError("Expected a node reference");
    return this.childAt(id, field.offset, field.optional, field.view);
  }
  range(id, key) {
    const field = this.fieldOffset(id, key);
    if (!field || field.type !== "list")
      throw new TypeError("Expected an AST list");
    const start = field.view.getUint32(field.offset, true),
      end = field.view.getUint32(field.offset + 4, true);
    if (start > end || end > this.extraDataCount)
      throw new RangeError("AST list out of bounds");
    return [start, end];
  }
  readExtraWord(index, byteOffset = 0) {
    if (!Number.isInteger(index) || index < 0 || index >= this.extraDataCount)
      throw new RangeError("AST extra-data reference out of bounds");
    return index < this._extraCount
      ? this._view.getUint32(this._extra + index * 8 + byteOffset, true)
      : this._arena.readExtraWord(index, byteOffset);
  }
  extraChild(owner, index, optional = true) {
    if (index < this._extraCount)
      return this.childAt(owner, this._extra + index * 8, optional);
    return this.childAt(owner, this._arena.extraAddress(index), optional, this._arena.view);
  }
  list(parts, prefix = NONE, suffix = NONE, owner, field, nativeOwner) {
    const fieldKey = field === undefined ? undefined : mirrorKey(owner, field);
    const current = fieldKey === undefined ? undefined : this._watchedLists.get(fieldKey);
    if (current) return current.value;
    const length =
      parts.reduce((total, [start, end]) => total + end - start, 0) +
      (prefix !== NONE) +
      (suffix !== NONE);
    const parent = nativeOwner ?? owner;
    const list = new Array(length);
    let index = 0;
    if (prefix !== NONE) list[index++] = this.node(prefix);
    for (const [start, end] of parts) {
      for (let offset = start; offset < end; offset++)
        list[index++] = this.node(this.extraChild(parent, offset, true));
    }
    if (suffix !== NONE) list[index] = this.node(suffix);
    if (fieldKey !== undefined && this._encodingDepth === 0) {
      let native;
      if (field === "params") {
        native =
          this.kind(owner) === NodeKind.FormalParameters
            ? owner
            : layouts[this.kind(owner)].fields.params?.[2] === "node"
              ? this.ref(owner, "params")
              : NONE;
        if (native !== NONE) this._arrayNodes.set(list, native);
        else native = undefined;
      }
      // Ordinary mutable arrays are the field's current value. Retain their
      // edit state so push/splice remain visible through subsequent reads.
      this._watchedLists.set(fieldKey, {
        owner, key: field, value: list, original: list.slice(), native,
      });
      this._listVersion++;
    }
    return list;
  }

  value(id, key, kind) {
    const field = this.fieldOffset(id, key, kind);
    if (!field) return undefined;
    const { offset, type, optional, view } = field;
    if (type === "node") return this.node(this.childAt(id, offset, optional, view));
    if (type === "list") return this.listAt(id, offset, key, view);
    if (type === "string") {
      const start = view.getUint32(offset, true),
        end = view.getUint32(offset + 4, true);
      return optional && start === NONE && end === 0
        ? null
        : this.string(start, end);
    }
    if (type.startsWith("enum:")) {
      const value = view.getUint8(offset);
      if (optional && value === 255) return null;
      const values = enums[type.slice(5)];
      if (value >= values.length)
        throw new RangeError("Unknown AST enum value");
      return values[value];
    }
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
      case "span":
        return Object.freeze({
          start: view.getUint32(offset, true),
          end: view.getUint32(offset + 4, true),
        });
      case "ignored":
        return undefined;
      default:
        throw new RangeError(`Unsupported AST field ${type}`);
    }
  }
  stringBytes(start, end) {
    if (
      !Number.isInteger(start) ||
      !Number.isInteger(end) ||
      start < 0 ||
      start > end ||
      end > this.stringLength
    )
      throw new RangeError("AST string out of bounds");
    if (end <= this._stringLength) return this._bytes.subarray(this._strings + start, this._strings + end);
    if (start < this._stringLength) throw new RangeError("AST string crosses storage regions");
    const offset = this._arena.stringsOffset - this._stringLength;
    return this._arena.bytes.subarray(offset + start, offset + end);
  }
  string(start, end) {
    if (Number.isInteger(start) && Number.isInteger(end) && start >= 0 && end >= start && end <= this.stringLength) {
      const length = end - start;
      if (length === 0) return "";
      if (length <= 8 && (end <= this._stringLength || start >= this._stringLength)) {
        const base = end <= this._stringLength;
        const bytes = base ? this._bytes : this._arena.bytes;
        const offset = base ? this._strings + start : this._arena.stringsOffset + start - this._stringLength;
        let mask = 0;
        for (let i = 0; i < length; i++) mask |= bytes[offset + i];
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
    }
    if (start === end) {
      this.stringBytes(start, end);
      return "";
    }
    return decodeWtf8(this.stringBytes(start, end));
  }
  bytesForString(id, fieldName) {
    const field = this.fieldOffset(id, fieldName);
    if (!field || field.type !== "string")
      throw new TypeError("Field is not a native AST string");
    const start = field.view.getUint32(field.offset, true),
      end = field.view.getUint32(field.offset + 4, true);
    return field.optional && start === NONE && end === 0
      ? null
      : this.stringBytes(start, end);
  }
  textBytes(id) {
    const start = this.byteStart(id), end = this.byteEnd(id);
    if (end > this._sourceLength) throw new RangeError("Node source span out of bounds");
    return this.stringBytes(start, end);
  }
  text(id) { return this.string(this.byteStart(id), this.byteEnd(id)); }
  readListCursor(id, key, cursor) {
    if (this._edits?.get(id)?.has(key) || (this._watchedLists.size !== 0 && this._watchedLists.has(cursor.fieldKey))) return false;
    return projectedList(this, id, key, cursor);
  }
  objectValue(id, key, create) {
    // Parser metadata has no mutable AST fields and needs no retained view.
    if (id === -1 || key === "comment" || this._encodingDepth !== 0) return create();
    const fieldKey = mirrorKey(id, key);
    let state = this._watchedObjects.get(fieldKey);
    if (!state) {
      const value = create();
      if (value === null || typeof value !== "object") return value;
      state = { owner: id, key, value,
        original: value instanceof RegExp
          ? [value.source, value.flags] : Object.entries(value ?? {}),
      };
      this._watchedObjects.set(fieldKey, state);
    }
    return state.value;
  }
  /** Traverse native child links without constructing node or list views. */
  traverse(visitor, root = this.rootId) {
    this.offset(root);
    if (this._firstChildren === null) ensureNavigation(this);
    const limit = this.nodeCount + this._extraNodes.length;
    let id = root,
      steps = 0;
    while (id !== NONE) {
      if (++steps > limit * 3)
        throw new RangeError("Cyclic AST traversal links");
      const action = visitor(id, this.kind(id));
      if (action === "stop") return;
      const child = action === "skip" ? NONE : this.firstChildId(id);
      if (child !== NONE) {
        id = child;
        continue;
      }
      while (id !== root && this.nextSiblingId(id) === NONE) {
        if (++steps > limit * 3)
          throw new RangeError("Cyclic AST traversal links");
        id = this._snapshotParentId(id);
        if (id === NONE) throw new RangeError("Broken AST parent chain");
      }
      if (id === root) return;
      id = this.nextSiblingId(id);
    }
  }
  /** Explicitly create editable ESTree objects; this is the copying boundary. */
  materialize(root = this.root) {
    const copies = new Map(),
      pending = [];
    const copy = (value) => {
      if (value === null || typeof value !== "object") return value;
      const key = value instanceof NodeView && value.ast === this ? value.nodeId : value;
      if (copies.has(key)) return copies.get(key);
      if (value instanceof RegExp) {
        const result = new RegExp(value.source, value.flags);
        result.lastIndex = value.lastIndex;
        copies.set(key, result);
        return result;
      }
      const result = Array.isArray(value) ? new Array(value.length) : {};
      copies.set(key, result);
      pending.push([value, result]);
      return result;
    };
    const result = copy(root);
    while (pending.length) {
      const [value, output] = pending.pop();
      if (value instanceof NodeView) {
        if (value.ast !== this)
          throw new TypeError("Node belongs to another AST buffer");
        for (const key of new Set([
          ...projectedKeys(this, value.nodeId),
          ...(this._edits?.get(value.nodeId)?.keys() ?? []),
        ]))
          output[key] = copy(value[key]);
      } else if (Array.isArray(value)) {
        for (let i = 0; i < value.length; i++) output[i] = copy(value[i]);
      } else
        for (const key of Object.keys(value)) output[key] = copy(value[key]);
    }
    return result;
  }
  get comments() {
    return this.objectValue(-1, "comments", () =>
      Array.from({ length: this._commentCount }, (_, index) =>
        this.objectValue(index, "comment", () => {
          const offset = this._comments + index * 8,
            view = this._view,
            ast = this;
          const start = view.getUint32(offset, true),
            end = view.getUint32(offset + 4, true);
          if (end > this._sourceLength || start > end || end - start < 2)
            throw new RangeError("Invalid AST comment span");
          const block = this._bytes[this._strings + start + 1] === 42;
          return Object.freeze({
            type: block ? "Block" : "Line",
            get start() {
              return ast.utf16Offset(start);
            },
            get end() {
              return ast.utf16Offset(end);
            },
            get value() {
              return ast.string(start + 2, end - (block ? 2 : 0));
            },
          });
        }),
      ),
    );
  }
  get diagnostics() {
    return this.objectValue(-1, "diagnostics", () => {
      let cursor = this._diagnostics;
      const byte = () => this._view.getUint8(cursor++);
      const word = () => {
        const result = this._view.getUint32(cursor, true);
        cursor += 4;
        return result;
      };
      const string = () => {
        const length = word();
        if (cursor + length > this._bytes.length)
          throw new RangeError("Truncated diagnostic");
        const value = decoder.decode(
          this._bytes.subarray(cursor, cursor + length),
        );
        cursor += length;
        return value;
      };
      const results = [];
      for (let index = 0; index < this._diagnosticCount; index++) {
        const severity = ["error", "warning", "hint", "info"][byte()];
        if (!severity) throw new RangeError("Invalid diagnostic severity");
        const start = word(),
          end = word(),
          message = string(),
          hasHelp = byte();
        if (hasHelp > 1) throw new RangeError("Invalid diagnostic help flag");
        const help = hasHelp ? string() : undefined,
          count = word(),
          labels = [];
        if (count * 12 > this._bytes.length - cursor)
          throw new RangeError("Invalid diagnostic label count");
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
      if (cursor !== this._bytes.length)
        throw new RangeError("Invalid diagnostic section length");
      return Object.freeze(results);
    });
  }
}

export function readAst(buffer) {
  return new AstView(buffer);
}
