import { NodeView } from "./node.js";
import { layouts, enums, NodeKind } from "./schema.js";
import { nativeWritableFields, nativeFieldKinds } from "./references.js";
import { encodeNodeJs } from "./encode-js.js";
import { encodeNodeTs } from "./encode-ts.js";
import { ensureTopology } from "./topology.js";
import { mirrorKey } from "./projection.js";
import { captureReferences } from "./storage.js";

const NONE = 0xffffffff;
const EMPTY = new Uint8Array();
// The idle transaction owns only empty numeric indexes, never input nodes.
let idleTransaction = null;
const literalKinds = new Set([
  NodeKind.StringLiteral, NodeKind.NumericLiteral, NodeKind.BigIntLiteral,
  NodeKind.BooleanLiteral, NodeKind.NullLiteral, NodeKind.RegExpLiteral,
]);
const patternKinds = new Set([
  NodeKind.BindingIdentifier, NodeKind.IdentifierReference, NodeKind.IdentifierName,
  NodeKind.ArrayPattern, NodeKind.ObjectPattern, NodeKind.AssignmentPattern,
]);
const kindNames = new Map(layouts.map((layout, kind) => [layout.type, kind]));
const typeKinds = {
  Identifier: NodeKind.IdentifierReference,
  Property: NodeKind.ObjectProperty,
  RestElement: NodeKind.BindingRestElement,
  FunctionDeclaration: NodeKind.Function,
  FunctionExpression: NodeKind.Function,
  ClassDeclaration: NodeKind.Class,
  ClassExpression: NodeKind.Class,
  TSDeclareFunction: NodeKind.TsFunction,
  TSEmptyBodyFunctionExpression: NodeKind.TsFunction,
  TSAbstractMethodDefinition: NodeKind.TsMethodDefinition,
  TSAbstractPropertyDefinition: NodeKind.PropertyDefinition,
};

function assertIdle(ast) {
  if (ast._encodingDepth > 0)
    throw new TypeError("Cannot mutate an AST while its edit is being encoded");
}

function editor(ast) {
  return ast._editor ??= { syncing: false, flags: ast._view.getUint32(12, true) };
}

function flagsFor(meta) {
  const lang = enums.Lang.indexOf(meta.lang);
  const sourceType = enums.SourceType.indexOf(meta.sourceType);
  if (lang < 0 || sourceType < 0) throw new TypeError("Invalid AST metadata");
  return sourceType | (lang << 2);
}

function currentMeta(ast) {
  const flags = ast._editor?.flags;
  return flags === undefined ? ast.meta : {
    lang: enums.Lang[(flags >> 2) & 7],
    sourceType: enums.SourceType[flags & 3],
  };
}

function literalKind(node) {
  if (node.regex != null || node.value instanceof RegExp) return NodeKind.RegExpLiteral;
  if (node.bigint != null || typeof node.value === "bigint") return NodeKind.BigIntLiteral;
  switch (typeof node.value) {
    case "string": return NodeKind.StringLiteral;
    case "number": return NodeKind.NumericLiteral;
    case "boolean": return NodeKind.BooleanLiteral;
    default: return NodeKind.NullLiteral;
  }
}

function inputKind(node, previous) {
  if (!node || typeof node !== "object") throw new TypeError("Expected an AST node");
  const type = node.type;
  if (type === "Literal") return literalKind(node);
  if (previous !== undefined) return previous;
  if (type === "TSModuleDeclaration" && node.global) return NodeKind.TsGlobalDeclaration;
  if ((type === "FunctionDeclaration" || type === "FunctionExpression") &&
      (node.declare || node.body == null)) return NodeKind.TsFunction;
  const kind = Object.hasOwn(typeKinds, type) ? typeKinds[type] : kindNames.get(type);
  if (kind === undefined) throw new TypeError(`Unsupported ESTree node type ${type}`);
  return kind;
}

function nativeOwner(ast, id, key, value) {
  if (ast.kind(id) === NodeKind.TsThisParameter && key === "name" && value !== "this") {
    const parent = ast.parentId(id);
    if (parent !== NONE && ast.ref(parent, "thisParam") === id) return parent;
  }
  if ((key !== "typeAnnotation" && key !== "optional" && key !== "decorators") ||
      !patternKinds.has(ast.kind(id))) return id;
  const parent = ast.parentId(id);
  if (parent === NONE) return id;
  const kind = ast.kind(parent);
  const field = kind === NodeKind.VariableDeclarator ? "id"
    : kind === NodeKind.CatchClause ? "param"
      : kind === NodeKind.FormalParameter ? "pattern"
        : kind === NodeKind.TsIndexSignatureParameter ? "name" : null;
  return field !== null && ast.ref(parent, field) === id ? parent : id;
}

function checkOptionalWrite(ast, id, value) {
  if (!value) return;
  const owner = nativeOwner(ast, id, "optional", value);
  const kind = ast.kind(owner);
  if ((ast.kind(id) === NodeKind.BindingIdentifier &&
       (kind === NodeKind.VariableDeclarator || kind === NodeKind.CatchClause)) ||
      (kind === NodeKind.FormalParameter && ast.ref(owner, "initializer") !== NONE))
    throw new TypeError("This AST binding cannot be optional");
}

function invalidateMirrors(ast, id, key) {
  const fieldKey = mirrorKey(id, key);
  ast._watchedLists.delete(fieldKey);
  ast._watchedObjects.delete(fieldKey);
  if (key === "type" || key === "value" || key === "regex" || key === "bigint") {
    for (const [field, state] of ast._watchedObjects)
      if (state.owner === id) ast._watchedObjects.delete(field);
  }
  ast._listVersion++;
}

function committed(ast, ids, previous = null) {
  const arena = ast._arena;
  ast._nextNodeId = Math.max(ast._nextNodeId, arena.nodeCount);
  // Fragment children are new rows; the last row is the overwritten owner.
  for (let i = 0; i < ids.length; i++)
    ast.refreshParents(ids[i], i === ids.length - 1 ? previous : null);
  // A setter publishes counts as well as bytes; encode() only seals a snapshot.
  arena.borrow(ast.rootId, editor(ast).flags);
}

function isAncestor(ast, ancestor, node) {
  for (let steps = 0; node !== NONE; steps++) {
    if (node === ancestor) return true;
    if (steps >= ast.totalNodeCount) throw new TypeError("Cannot encode a cyclic AST");
    node = ast.parentId(node);
  }
  return false;
}

function compatibleChild(ast, owner, kinds, value) {
  if (!(value instanceof NodeView) || value.ast !== ast ||
      !kinds?.includes(ast.kind(value.nodeId))) return false;
  if (isAncestor(ast, value.nodeId, owner)) throw new TypeError("Cannot encode a cyclic AST");
  // A moved/shared node can have incoming edges beyond the current parent
  // index. Have Rust check the resulting graph before using trusted readback.
  if (ast.parentId(value.nodeId) !== owner) ast._needsGraphValidation = true;
  return true;
}

function directWrite(ast, id, key, value) {
  const kind = ast.kind(id);
  const writable = nativeWritableFields[kind]?.[key];
  if (!writable || literalKinds.has(kind) &&
      (key === "value" || key === "raw")) return false;
  const nativeKey = typeof writable === "string" ? writable : key;
  const descriptor = layouts[kind].fields[nativeKey];
  if (!descriptor) return false;
  const type = descriptor[2], optional = descriptor[3];
  let encoded = value;
  if (type === "node") {
    if (key === "key" || key === "property") return false;
    if (value == null) {
      if (!optional) throw new TypeError("Missing required AST child");
      encoded = NONE;
    } else {
      if (!compatibleChild(ast, id, nativeFieldKinds[kind]?.[nativeKey], value)) return false;
      encoded = value.nodeId;
    }
  } else if (type === "list") {
    if (!Array.isArray(value)) return false;
    for (let i = 0; i < value.length; i++) {
      if (value[i] == null) {
        if (!optional) return false;
      } else if (!compatibleChild(ast, id, nativeFieldKinds[kind]?.[nativeKey], value[i])) return false;
    }
  } else if (type === "string") {
    if (value == null && optional) encoded = null;
    else if (typeof value !== "string") return false;
  } else if (type.startsWith("enum:")) {
    if (value == null && optional) encoded = 255;
    else {
      encoded = enums[type.slice(5)].indexOf(value);
      if (encoded < 0) throw new TypeError("Invalid AST enum value");
    }
  } else if (type === "bool") encoded = value ? 1 : 0;
  else if (!["u8", "i8", "u16", "i16", "u32", "i32", "f64"].includes(type)) return false;
  else if (typeof value !== "number") return false;

  const arena = ast.ensureArena();
  ast.ensureEditable(id);
  const previous = type === "node" || type === "list"
    ? captureReferences(ast, id, nativeKey) : null;
  if (type === "string") {
    const [start, end] = encoded === null ? [NONE, 0] : arena.appendString(encoded);
    const { view, offset } = ast.writableField(id, nativeKey);
    view.setUint32(offset, start, true);
    view.setUint32(offset + 4, end, true);
  } else if (type === "list") {
    const start = arena.appendExtra(value.length);
    for (let i = 0; i < value.length; i++)
      arena.writeExtraWord(start + i, 0, value[i]?.nodeId ?? NONE);
    const { view, offset } = ast.writableField(id, nativeKey);
    view.setUint32(offset, start, true);
    view.setUint32(offset + 4, start + value.length, true);
  } else {
    const { view, offset } = ast.writableField(id, nativeKey);
    if (type === "node" || type === "u32") view.setUint32(offset, encoded, true);
    else if (type === "bool" || type === "u8" || type.startsWith("enum:")) view.setUint8(offset, encoded);
    else if (type === "i8") view.setInt8(offset, encoded);
    else if (type === "u16") view.setUint16(offset, encoded, true);
    else if (type === "i16") view.setInt16(offset, encoded, true);
    else if (type === "i32") view.setInt32(offset, encoded, true);
    else view.setFloat64(offset, encoded, true);
  }
  committed(ast, [id], previous);
  return true;
}

class FragmentTransaction {
  constructor() {
    this.ast = null;
    this.owner = NONE;
    this.root = null;
    this.kind = NONE;
    this.nodeBase = 0;
    this.extraBase = 0;
    this.stringBase = 0;
    this.dirty = new Set();
    this.active = new Map();
  }

  initialize(ast, owner, root, kind) {
    this.ast = ast;
    this.owner = owner;
    this.root = root;
    this.kind = kind;
    this.nodeBase = ast._arena.nodeCount;
    this.extraBase = ast._arena.extraCount;
    this.stringBase = ast._arena.stringLength;
    if (owner !== NONE) this.dirty.add(owner);
  }

  clear() {
    this.ast = null;
    this.owner = NONE;
    this.root = null;
    this.kind = NONE;
    this.nodeBase = 0;
    this.extraBase = 0;
    this.stringBase = 0;
    this.dirty.clear();
    this.active.clear();
  }

  original(node, kind) {
    if (Array.isArray(node)) {
      const id = kind === NodeKind.FormalParameters ? this.ast._arrayNodes.get(node) : undefined;
      return id !== undefined && this.ast.kind(id) === kind ? id : undefined;
    }
    if (!(node instanceof NodeView) || node.ast !== this.ast) return undefined;
    const id = node.nodeId;
    if (this.ast.kind(id) === kind) return id;
    const parent = this.ast.parentId(id);
    if (parent !== NONE && this.ast.kind(parent) === kind &&
        ((kind === NodeKind.FormalParameter && this.ast.ref(parent, "initializer") === NONE) ||
          kind === NodeKind.TsIndexSignatureParameter)) return parent;
    return undefined;
  }

  reuse(node, kind) {
    // The old wrapper supplies its span, but its array contents must be encoded.
    if (Array.isArray(node)) return undefined;
    if (node === this.root && kind === this.kind) return undefined;
    const id = this.original(node, kind);
    if (id === undefined || id === this.owner) return undefined;
    if (this.owner !== NONE && isAncestor(this.ast, id, this.owner))
      throw new TypeError("Cannot encode a cyclic AST");
    if (this.ast.parentId(id) !== this.owner) this.ast._needsGraphValidation = true;
    return id;
  }

  copySource() { return undefined; }

  enter(node, kind) {
    if (!node || typeof node !== "object") throw new TypeError("Expected an AST node");
    const key = node instanceof NodeView && node.ast === this.ast ? node.nodeId : node;
    let kinds = this.active.get(key);
    if (!kinds) this.active.set(key, kinds = new Set());
    if (kinds.has(kind)) throw new TypeError("Cannot encode a cyclic AST");
    kinds.add(kind);
  }

  leave(node, kind) {
    const key = node instanceof NodeView && node.ast === this.ast ? node.nodeId : node;
    this.active.get(key).delete(kind);
  }

  span(node, kind, fallback) {
    if (node instanceof NodeView && node.ast !== this.ast) return [0, 0];
    let id = this.original(node, kind);
    if (node === this.root && this.owner !== NONE) id = this.owner;
    if (id !== undefined) return [this.ast.byteStart(id), this.ast.byteEnd(id)];
    if (node instanceof NodeView && node.ast === this.ast)
      return [this.ast.byteStart(node.nodeId), this.ast.byteEnd(node.nodeId)];
    let [start, end] = fallback();
    if (start == null && end == null) return [0, 0];
    start ??= 0;
    end ??= start;
    if (kind === NodeKind.TemplateElement && this.ast._isTs && (start || end)) {
      start++;
      end -= node.tail ? 1 : 2;
    }
    if (end < start) throw new RangeError("AST span ends before it starts");
    return [this.ast.utf8Offset(start), this.ast.utf8Offset(end)];
  }

  finish(data) {
    const ast = this.ast, arena = ast._arena;
    if (!data.nodeCount) {
      if (this.owner !== NONE) throw new TypeError("AST edit did not encode its owner");
      return data.root;
    }
    const last = this.nodeBase + data.nodeCount - 1;
    if (data.root !== last) throw new TypeError("AST fragment root must be its last node");
    const count = data.nodeCount - (this.owner !== NONE ? 1 : 0);
    const previous = this.owner === NONE ? null : captureReferences(ast, this.owner);
    arena.reserve(arena.rowCount + data.nodeCount,
      arena.extraUsed + data.extraCount, arena.stringUsed + data.strings.length);
    const extra = arena.appendExtra(data.extraCount);
    if (extra !== this.extraBase) throw new TypeError("AST extra-data allocation changed during encoding");
    if (data.extraCount) arena.bytes.set(data.extra, arena.extraAddress(extra));
    const [strings] = arena.appendStringBytes(data.strings);
    if (strings !== this.stringBase) throw new TypeError("AST string allocation changed during encoding");
    const payload = new DataView(data.payload.buffer, data.payload.byteOffset, data.payload.byteLength);
    const changed = [];
    for (let i = 0; i < count; i++) {
      const id = this.nodeBase + i;
      arena.ensureRow(id, data.kinds[i], data.spans[i * 2], data.spans[i * 2 + 1]);
      arena.copyPayload(id, payload, i * 16);
      changed.push(id);
    }
    if (this.owner !== NONE) {
      const i = data.nodeCount - 1;
      if (ast.kind(this.owner) !== data.kinds[i]) {
        ast._kindsChanged = true;
        ast._needsGraphValidation = true;
      }
      arena.ensureRow(this.owner, data.kinds[i], data.spans[i * 2], data.spans[i * 2 + 1]);
      arena.copyPayload(this.owner, payload, i * 16);
      changed.push(this.owner);
    }
    committed(ast, changed, previous);
    return this.owner === NONE ? data.root : this.owner;
  }
}

function encodeFragment(ast, input, kind, owner = NONE) {
  // Getter reads during packing must not introduce occurrence IDs after the
  // codec has captured its contiguous allocation ranges.
  if (ast._parents === null) ensureTopology(ast);
  ast.ensureArena();
  // Isolate original shared occurrences before the fragment can overwrite one.
  if (owner !== NONE) ast.ensureEditable(owner);
  const transaction = idleTransaction ?? new FragmentTransaction();
  // Taking it out first also isolates nested encodes belonging to another AST.
  idleTransaction = null;
  try {
    transaction.initialize(ast, owner, input, kind);
    const meta = currentMeta(ast);
    const encode = meta.lang === "js" || meta.lang === "jsx" ? encodeNodeJs : encodeNodeTs;
    return encode(input, meta, transaction, kind);
  } finally {
    transaction.clear();
    idleTransaction ??= transaction;
  }
}

function writeSpan(ast, id, key, value) {
  let position = value;
  if (ast._isTs && ast.kind(id) === NodeKind.TemplateElement)
    position += key === "start" ? 1 : -(ast.getField(id, "tail") ? 1 : 2);
  const byte = ast.utf8Offset(position);
  const start = key === "start" ? byte : ast.byteStart(id);
  const end = key === "end" ? byte : ast.byteEnd(id);
  if (end < start) throw new RangeError("AST span ends before it starts");
  ast.ensureEditable(id);
  ast._arena.setSpan(id, start, end);
  committed(ast, [id]);
}

function retainRenamedThisParameter(ast, id, owner) {
  const params = ast.ref(owner, "params");
  const [start, end] = ast.range(params, "items");
  if (start === end) throw new TypeError("Renamed this parameter has no native parameter");
  const wrapper = ast.extraChild(params, start, false);
  const binding = ast.ref(wrapper, "pattern");
  const previous = captureReferences(ast, id);
  const wrapperPrevious = captureReferences(ast, wrapper, "pattern");
  const arena = ast._arena;
  const kind = ast.kind(binding), begin = ast.byteStart(binding), finish = ast.byteEnd(binding);
  if (ast.kind(id) !== kind) {
    ast._kindsChanged = true;
    ast._needsGraphValidation = true;
  }
  arena.ensureRow(id, kind, begin, finish);
  arena.copyPayload(id, arena.view, arena.nodePayload(binding));
  const field = ast.writableField(wrapper, "pattern");
  field.view.setUint32(field.offset, id, true);
  ast.refreshParents(id, previous);
  committed(ast, [wrapper], wrapperPrevious);
}

/** Commit one public field to the binary arena before returning. */
export function writeField(ast, id, key, value) {
  assertIdle(ast);
  ast.kind(id);
  editor(ast);
  if (key === "optional") checkOptionalWrite(ast, id, value);
  if (key === "type" && ast.getField(id, "type") === value) return;
  if (key === "start" || key === "end") {
    writeSpan(ast, id, key, value);
  } else if (key === "sourceType" && ast.kind(id) === NodeKind.Program) {
    const flags = flagsFor({ ...currentMeta(ast), sourceType: value });
    ast.ensureArena().borrow(ast.rootId, flags);
    editor(ast).flags = flags;
  } else if (!directWrite(ast, id, key, value)) {
    const owner = nativeOwner(ast, id, key, value);
    const renamedThis = ast.kind(id) === NodeKind.TsThisParameter && owner !== id && key === "name";
    const fields = new Map([[key, value]]);
    if (key === "value" && literalKinds.has(ast.kind(id))) {
      fields.set("raw", typeof value === "string" ? JSON.stringify(value)
        : typeof value === "bigint" ? `${value}n`
          : typeof value === "number" && !Number.isFinite(value) ? "" : String(value));
      fields.set("bigint", typeof value === "bigint" ? String(value) : undefined);
      fields.set("regex", value instanceof RegExp
        ? { pattern: value.source, flags: value.flags } : undefined);
    }
    ast._edits = new Map([[id, fields]]);
    ast._encodingDepth++;
    try {
      const node = ast.node(owner);
      const kind = owner === id
        ? inputKind(node, key === "type" ? undefined : ast.kind(owner))
        : ast.kind(owner);
      encodeFragment(ast, node, kind, owner);
      if (renamedThis) retainRenamedThisParameter(ast, id, owner);
    } finally {
      ast._edits = null;
      ast._encodingDepth--;
    }
  }
  invalidateMirrors(ast, id, key);
}

/** Commit a list and keep its public array populated with encoded node views. */
export function commitList(ast, id, key, array) {
  if (!Array.isArray(array)) throw new TypeError("Expected an AST list");
  writeField(ast, id, key, array);
  const fresh = ast.getField(id, key);
  if (!Array.isArray(fresh)) throw new TypeError("Encoded AST field is not a list");
  const fieldKey = mirrorKey(id, key);
  const watched = ast._watchedLists.get(fieldKey) ?? { owner: id, key, value: null, original: null, native: undefined };
  const native = watched.native;
  array.length = fresh.length;
  for (let i = 0; i < fresh.length; i++) array[i] = fresh[i];
  watched.value = array;
  watched.original = fresh.slice();
  watched.native = native;
  ast._arrayNodes.delete(array);
  if (native !== undefined) ast._arrayNodes.set(array, native);
  ast._watchedLists.set(fieldKey, watched);
  return array;
}

function listChanged(value, original) {
  if (value.length !== original.length) return true;
  for (let i = 0; i < value.length; i++)
    if (!NodeView.isSame(value[i], original[i])) return true;
  return false;
}

function objectEntries(value) {
  return value instanceof RegExp ? [value.source, value.flags] : Object.entries(value);
}

function objectChanged(value, original, entries) {
  if (entries.length !== original.length) return true;
  return value instanceof RegExp
    ? entries.some((entry, i) => entry !== original[i])
    : entries.some(([key, entry], i) => key !== original[i][0] || entry !== original[i][1]);
}

/** Flush retained public mirrors, then return the already encoded buffer pair. */
export function finishEdits(ast, options = {}, snapshot = true) {
  assertIdle(ast);
  const state = editor(ast);
  if (state.syncing) throw new TypeError("Cannot finish an AST edit recursively");
  state.syncing = true;
  try {
    const current = currentMeta(ast);
    const meta = { ...current, ...options };
    const flags = flagsFor(meta);
    const currentIsJs = current.lang === "js" || current.lang === "jsx";
    const requestedIsJs = meta.lang === "js" || meta.lang === "jsx";
    if (currentIsJs !== requestedIsJs)
      throw new TypeError("Cannot change between JavaScript and TypeScript AST languages");
    for (const watched of [...ast._watchedLists.values()]) {
      if (!listChanged(watched.value, watched.original)) continue;
      commitList(ast, watched.owner, watched.key, watched.value);
    }
    for (const [fieldKey, watched] of [...ast._watchedObjects]) {
      const entries = objectEntries(watched.value);
      if (!objectChanged(watched.value, watched.original, entries)) continue;
      writeField(ast, watched.owner, watched.key, watched.value);
      watched.original = entries;
      ast._watchedObjects.set(fieldKey, watched);
    }
    if (!ast._arena && flags === flagsFor(ast.meta)) return [ast.buffer, EMPTY];
    const arena = ast.ensureArena();
    return [ast.buffer, snapshot ? arena.snapshot(ast.rootId, flags) : arena.borrow(ast.rootId, flags)];
  } finally {
    state.syncing = false;
  }
}

/** Import a node immediately; only numeric references remain after this call. */
export function createNode(ast, input, kind) {
  assertIdle(ast);
  editor(ast);
  ast._encodingDepth++;
  try {
    const native = typeof kind === "string" ? NodeKind[kind] ?? kindNames.get(kind) : kind;
    const id = encodeFragment(ast, input, inputKind(input, native));
    return ast.node(id);
  } finally {
    ast._encodingDepth--;
  }
}
