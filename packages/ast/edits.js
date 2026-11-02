import { NodeView } from "./node.js";
import { NodeKind, SCHEMA_ID } from "./schema.js";
import { encodeEditsJs } from "./encode-js.js";
import { encodeEditsTs } from "./encode-ts.js";

const NONE = 0xffffffff;
const HEADER_SIZE = 64;
const SPAN_SIZE = 8;
const PAYLOAD_SIZE = 16;

// Retain one empty context across encodes, including batches that release every AST.
// Taking it out of the slot before initialization keeps nested encodes independent.
let idleTransaction = null;

function index(value, name) {
  if (!Number.isSafeInteger(value) || value < 0 || value >= NONE)
    throw new RangeError(`${name} exceeds the AST buffer limit`);
  return value;
}

/** Encode a new root and changed paths, retaining references into the original buffer. */
export function encodeEdits(ast, options = {}) {
  const direct = new Set();
  const dirty = ast.collectEdits(direct);
  const meta = { ...ast.meta, ...options };
  if (options.sourceType === undefined)
    meta.sourceType = ast.program.sourceType;
  if (
    !dirty.size &&
    meta.lang === ast.meta.lang &&
    meta.sourceType === ast.meta.sourceType
  )
    return [ast.buffer, new Uint8Array()];
  const transaction = idleTransaction ?? new Transaction();
  idleTransaction = null;
  try {
    transaction.initialize(ast, dirty, direct, meta.lang);
    const encode =
      meta.lang === "js" || meta.lang === "jsx" ? encodeEditsJs : encodeEditsTs;
    return [ast.buffer, encode(ast.program, meta, transaction)];
  } finally {
    transaction.clear();
    idleTransaction ??= transaction;
  }
}

class Transaction {
  constructor() {
    this.ast = null;
    this.dirty = null;
    this.direct = null;
    this.copyNative = false;
    this.nodeBase = 0;
    this.extraBase = 0;
    this.stringBase = 0;
    this.active = new Map();
  }
  initialize(ast, dirty, direct, lang) {
    this.ast = ast;
    this.dirty = dirty;
    this.direct = direct;
    this.copyNative = lang === ast.meta.lang;
    this.nodeBase = ast.nodeCount;
    this.extraBase = ast._extraCount;
    this.stringBase = ast._stringLength;
  }
  clear() {
    // Drop all input references even when encoding failed before leave() ran.
    this.ast = null;
    this.dirty = null;
    this.direct = null;
    this.copyNative = false;
    this.nodeBase = 0;
    this.extraBase = 0;
    this.stringBase = 0;
    this.active.clear();
  }
  original(node, kind) {
    if (Array.isArray(node)) {
      const id = this.ast._arrayNodes.get(node);
      return id !== undefined && this.ast.kind(id) === kind ? id : undefined;
    }
    if (!(node instanceof NodeView) || node.ast !== this.ast) return undefined;
    const id = node.nodeId;
    if (this.ast.kind(id) === kind) return id;
    const parent = this.ast.parentId(id);
    if (
      parent !== NONE &&
      this.ast.kind(parent) === kind &&
      ((kind === NodeKind.FormalParameter &&
        this.ast.ref(parent, "initializer") === NONE) ||
        kind === NodeKind.TsIndexSignatureParameter)
    )
      return parent;
    return undefined;
  }
  reuse(node, kind) {
    const id = this.original(node, kind);
    return id !== undefined && !this.dirty.has(id) ? this.ast.rawId(id) : undefined;
  }
  copySource(node, kind) {
    // Do not use original(): its parameter projections can change native kinds.
    if (!this.copyNative || !(node instanceof NodeView) || node.ast !== this.ast)
      return undefined;
    const id = node.nodeId;
    return !this.direct.has(id) && this.ast.kind(id) === kind ? id : undefined;
  }
  enter(node, kind) {
    if (!node || typeof node !== "object")
      throw new TypeError("Expected an AST node");
    const key = node instanceof NodeView && node.ast === this.ast ? node.nodeId : node;
    let kinds = this.active.get(key);
    if (!kinds) {
      kinds = new Set();
      this.active.set(key, kinds);
    }
    if (kinds.has(kind)) throw new TypeError("Cannot encode a cyclic AST");
    kinds.add(kind);
  }
  leave(node, kind) {
    const key = node instanceof NodeView && node.ast === this.ast ? node.nodeId : node;
    this.active.get(key).delete(kind);
  }
  span(node, kind, fallback) {
    const id = this.original(node, kind);
    const view = node instanceof NodeView && node.ast === this.ast;
    const edits = view ? this.ast._edits?.get(node.nodeId) : undefined;
    if (id !== undefined && !edits?.has("start") && !edits?.has("end"))
      return [this.ast.byteStart(id), this.ast.byteEnd(id)];
    let [start, end] = fallback();
    if (start == null && end == null) return [0, 0];
    start ??= 0;
    end ??= start;
    if (
      kind === NodeKind.TemplateElement &&
      ["ts", "tsx", "dts"].includes(this.ast.meta.lang) &&
      (start || end)
    ) {
      start++;
      end -= node.tail ? 1 : 2;
    }
    if (end < start) throw new RangeError("AST span ends before it starts");
    return [this.ast.utf8Offset(start), this.ast.utf8Offset(end)];
  }
  finish(data) {
    index(this.nodeBase + data.nodeCount, "Node count");
    index(this.extraBase + data.extraCount, "Extra-data count");
    index(this.stringBase + data.strings.length, "String pool");
    const nodes = HEADER_SIZE;
    const kinds = index(nodes + data.nodeCount * SPAN_SIZE, "Kind section");
    const payloads = index(Math.ceil((kinds + data.nodeCount) / 8) * 8, "Payload section");
    const extra = index(payloads + data.nodeCount * PAYLOAD_SIZE, "Extra-data section");
    const strings = index(extra + data.extra.length, "Extra-data section");
    const length = index(strings + data.strings.length, "Extension size");
    const bytes = new Uint8Array(length);
    const view = new DataView(bytes.buffer);
    const header = [
      0x32455753,
      2,
      SCHEMA_ID,
      length,
      this.nodeBase,
      this.extraBase,
      this.stringBase,
      this.ast.buffer.length,
      data.nodeCount,
      data.extraCount,
      data.strings.length,
      data.root,
      data.flags,
      nodes,
      extra,
      strings,
    ];
    header.forEach((value, i) => view.setUint32(i * 4, value, true));
    for (let i = 0; i < data.nodeCount; i++) {
      view.setUint32(nodes + i * SPAN_SIZE, data.spans[i * 2], true);
      view.setUint32(nodes + i * SPAN_SIZE + 4, data.spans[i * 2 + 1], true);
    }
    bytes.set(data.kinds.subarray(0, data.nodeCount), kinds);
    bytes.set(data.payload.subarray(0, data.nodeCount * PAYLOAD_SIZE), payloads);
    bytes.set(data.extra, extra);
    bytes.set(data.strings, strings);
    return bytes;
  }
}
