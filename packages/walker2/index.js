// Buffer-backed AST walker.
//
// The walker is specialized for `@swc-next/ast2` views. Traversal frames,
// ancestor stacks and list cursors hold numbers only; a NodeView is created
// only when a hook first touches `ctx.node`, `ctx.parent` or `ctx.ancestors()`,
// and is released when the hook phase ends. Scalar child reads use precomputed direct descriptors for
// native fields; projected exceptions fall back to the numeric projection
// bridge exported by `@swc-next/ast2/internal`.
//
// Public entry points:
//   walk(root, hooks, state?) -> root
//   createWalker(hooks).walk(root, state?) -> root
//
// `root` is a NodeView from an AstView (`ast.program` or `ast.node(id)`).
// Hooks receive `(ctx)`; the current node is materialized lazily by `ctx.node`,
// and `ctx` also exposes numeric `nodeId`, `parentId`, `kind` and `ancestorIds()`.
import { NodeView } from "@swc-next/ast2";
import { visitorKeys } from "./visitor-keys.js";
import { childVisitors } from "./child-visitors.js";
import {
  NONE,
  NODE_KIND_COUNT,
  CURSOR_FIELD_COUNT,
  ReadRule,
  cursorFieldIds,
  cursorRules,
  cursorRuleIds,
  fieldRule,
  hasEmptyDecorators,
  listDescriptor,
  fillList,
  isEmptyListDescriptor,
  staticTypes,
  dynamicTypes,
  NodeKind,
  layouts,
  readChildId,
  readChildIdWithRule,
  isProjectedChildRule,
} from "@swc-next/ast2/internal";
import { compile, isNodeType, assertNode, assertSync, SKIP, REMOVED, REPLACED } from "./hook-compiler.js";

export { visitorKeys, NONE };

function isListField(isTs, kind, key) {
  if (kind === NodeKind.FormalParameter && (key === "elements" || key === "properties")) {
    // DelegateList: the visible wrapper owns the key, the hidden pattern owns
    // the range. `listDescriptor`/`fillList` decide whether the list exists.
    return true;
  }
  if (key === "decorators" && hasEmptyDecorators(isTs, kind)) {
    return true;
  }
  const fieldId = cursorFieldIds[key];
  if (typeof fieldId !== "number") { return false; }
  return cursorRules[cursorRuleIds[(isTs ? NODE_KIND_COUNT : 0) + kind] * CURSOR_FIELD_COUNT + fieldId] !== 0;
}

// A direct descriptor is [extraPointer, slot, optional] with -1 as "no extra
// pointer". Reading it reproduces fieldLocation + childAt without allocating
// the metadata objects those helpers return.
function directField(isTs, kind, key, rule = fieldRule(isTs, kind, key)) {
  if (rule !== ReadRule.Native) { return null; }
  const layout = layouts[kind];
  const field = layout === undefined ? undefined : layout.fields[key];
  if (!Array.isArray(field) || field[2] !== "node") { return null; }
  if (field[0]) {
    if (layout.extra === null || layout.extra === undefined) { return null; }
    return [layout.extra, field[1], field[3] === true];
  }
  return [-1, field[1], field[3] === true];
}

const PLAN_LIST = 0;
const PLAN_DIRECT = 1;
const PLAN_BRIDGE = 2;

// FormalParameter projects to the pattern's public type. Read the two native
// slots directly so plan selection does not go through the string projection.
const FORMAL_PATTERN = layouts[NodeKind.FormalParameter].fields.pattern;
const FORMAL_INITIALIZER = layouts[NodeKind.FormalParameter].fields.initializer;
const FUNCTION_TYPE = layouts[NodeKind.Function].fields.functionType;

// A plan keeps the frozen visitor keys for kind changes plus the fields that
// can be children under the planned kind. Fields whose read rule is statically
// NONE are left out of `hot`; a live kind change re-resolves the remaining
// keys through #traverseRemaining.
function buildPlan(isTs, kind, type) {
  const keys = visitorKeys[type];
  if (keys === undefined) { return null; }
  const hot = [];
  const lists = Object.create(null);
  for (let index = 0; index < keys.length; index++) {
    const key = keys[index];
    const desc = listDescriptor(isTs, kind, key);
    if (desc !== null) {
      if (isEmptyListDescriptor(desc)) { continue; }
      hot.push({ index, key, op: PLAN_LIST, desc });
      lists[key] = desc;
      continue;
    }
    const rule = fieldRule(isTs, kind, key);
    const direct = directField(isTs, kind, key, rule);
    if (direct !== null) {
      hot.push({ index, key, op: PLAN_DIRECT, direct });
      continue;
    }
    if (isProjectedChildRule(rule)) {
      hot.push({ index, key, op: PLAN_BRIDGE, rule, direct: null });
    }
  }
  return { keys, hot, lists };
}

function buildStaticPlans(isTs) {
  const plans = new Array(NODE_KIND_COUNT);
  for (let kind = 0; kind < NODE_KIND_COUNT; kind++) {
    const type = staticTypes[kind];
    plans[kind] = type == null ? null : buildPlan(isTs, kind, type);
  }
  return plans;
}

function buildDynamicPlans(isTs) {
  const plans = new Array(NODE_KIND_COUNT);
  for (const [kind, types] of dynamicTypes) {
    const table = new Map();
    for (const type of types) {
      const plan = buildPlan(isTs, kind, type);
      if (plan === null) { throw new TypeError(`Unsupported AST node type: ${type}`); }
      table.set(type, plan);
    }
    plans[kind] = table;
  }
  for (let kind = 0; kind < NODE_KIND_COUNT; kind++) {
    if (staticTypes[kind] == null && plans[kind] === undefined) {
      throw new TypeError(`Missing dynamic AST node types for kind ${kind}`);
    }
  }
  return plans;
}

function sameVisitorKeys(types) {
  const first = visitorKeys[types[0]];
  if (first === undefined) { return false; }
  for (let typeIndex = 1; typeIndex < types.length; typeIndex++) {
    const keys = visitorKeys[types[typeIndex]];
    if (keys === undefined || keys.length !== first.length) { return false; }
    for (let keyIndex = 0; keyIndex < keys.length; keyIndex++) {
      if (keys[keyIndex] !== first[keyIndex]) { return false; }
    }
  }
  return true;
}

// Dynamic kinds whose public types share the same visitor keys can traverse
// with one kind plan. Their type string is only needed when a hook is
// registered for one of those types.
function buildKindPlans(isTs) {
  const plans = new Array(NODE_KIND_COUNT);
  for (const [kind, types] of dynamicTypes) {
    if (!sameVisitorKeys(types)) { continue; }
    const plan = buildPlan(isTs, kind, types[0]);
    if (plan !== null) { plans[kind] = plan; }
  }
  return plans;
}

const STATIC_PLANS = [buildStaticPlans(false), buildStaticPlans(true)];
const DYNAMIC_PLANS = [buildDynamicPlans(false), buildDynamicPlans(true)];
const KIND_PLANS = [buildKindPlans(false), buildKindPlans(true)];
const UNHOOKED_ENTRY = { enter: null, leave: null };

function buildDynamicHooked(compiled) {
  const hooked = new Array(NODE_KIND_COUNT).fill(false);
  if (compiled.globalEnter !== null || compiled.globalLeave !== null) {
    hooked.fill(true);
    return hooked;
  }
  for (const [kind, types] of dynamicTypes) {
    for (const type of types) {
      const entry = compiled.dispatch[type];
      if (entry !== undefined && (entry.enter !== null || entry.leave !== null)) {
        hooked[kind] = true;
        break;
      }
    }
  }
  return hooked;
}

function buildEntries(dispatch) {
  const entries = new Array(NODE_KIND_COUNT);
  for (let kind = 0; kind < NODE_KIND_COUNT; kind++) {
    const type = staticTypes[kind];
    entries[kind] = type == null ? undefined : dispatch[type];
  }
  return entries;
}

function sameNode(node, ast, id) {
  return node !== null && typeof node === "object" &&
    node.ast === ast && node.nodeId === id;
}

function sameListLike(left, right) {
  return left === right || (left !== null && right !== null &&
    left.ast === right.ast && left.owner === right.owner &&
    left.key === right.key && left.native === right.native);
}

function cursorMatchesList(cursor, list, ast, parentId) {
  return list !== null && list !== undefined &&
    typeof list.set === "function" &&
    list.ast === ast && list.key === cursor.key &&
    (list.owner === parentId || list.owner === cursor.owner0 || list.owner === cursor.owner1);
}

function isListLike(value) {
  return value !== null && typeof value === "object" &&
    typeof value.get === "function" && typeof value.length === "number";
}

// Numeric counterpart of fieldLocation + childAt for a native node field.
function readNativeChild(ast, id, descriptor) {
  const row = ast.row(id);
  const payload = ast._payloadAt(id, row);
  let view = row === -1 ? ast._view : ast._arena.view;
  let offset;
  const pointer = descriptor[0];
  if (pointer !== -1) {
    const start = view.getUint32(payload + pointer, true);
    const index = start + descriptor[1];
    const limit = start < ast._extraCount ? ast._extraCount : ast.extraDataCount;
    if (index + 1 > limit) {
      throw new RangeError("AST extra-data reference out of bounds");
    }
    view = index < ast._extraCount ? ast._view : ast._arena.view;
    offset = index < ast._extraCount ? ast._extra + index * 4 : ast._arena.extraAddress(index);
  } else {
    offset = payload + descriptor[1];
  }
  const child = view.getUint32(offset, true);
  if (child === NONE) {
    if (!descriptor[2]) {
      throw new RangeError("Missing required AST child");
    }
    return NONE;
  }
  if (child >= ast.totalNodeCount) {
    throw new RangeError("AST child out of bounds");
  }
  return child;
}

// Numeric counterpart of ListCursor. It owns no NodeView, AstList or array;
// an edit materializes the real public list through the visible parent.
class NumericListCursor {
  numeric = true;
  ast = null;
  parentId = 0;
  key = null;
  prefix = -1;
  suffix = -1;
  owner0 = 0;
  start0 = 0;
  end0 = 0;
  owner1 = 0;
  start1 = 0;
  end1 = 0;
  total = 0;
  version = 0;
  kind = -1;
  descriptor = null;
  reset(ast, parentId, key) {
    this.ast = ast;
    this.parentId = parentId;
    this.key = key;
    this.start1 = this.end1 = 0;
    this.prefix = this.suffix = -1;
    this.version = ast._listVersion;
    this.kind = -1;
    this.descriptor = null;
  }
  setRange(part, owner, start, end) {
    if (part === 0) {
      this.owner0 = owner;
      this.start0 = start;
      this.end0 = end;
    } else {
      this.owner1 = owner;
      this.start1 = start;
      this.end1 = end;
    }
  }
  finish() {
    if (this.start0 === this.end0) {
      this.start0 = this.start1;
      this.end0 = this.end1;
      this.owner0 = this.owner1;
      this.start1 = this.end1 = 0;
    }
    this.total = this.end0 - this.start0 + this.end1 - this.start1 +
      (this.prefix !== -1) + (this.suffix !== -1);
  }
  sync() {
    if (this.version === this.ast._listVersion) { return; }
    this.version = this.ast._listVersion;
    this.start1 = this.end1 = 0;
    this.prefix = this.suffix = -1;
    const kind = this.ast._kindAt(this.parentId);
    if (kind !== this.kind || this.descriptor === null) {
      this.kind = kind;
      this.descriptor = listDescriptor(this.ast._isTs, kind, this.key);
    }
    if (this.descriptor === null || !fillList(this.ast, this.parentId, this, this.descriptor)) {
      throw new Error("The current node's container was changed outside the walk context");
    }
    this.finish();
  }
  get length() { return this.total; }
  nodeId(index) {
    if (this.prefix !== -1) {
      if (index === 0) { return this.prefix >>> 0; }
      index--;
    }
    const first = this.end0 - this.start0;
    if (index < first) {
      return this.ast.extraChild(this.owner0, this.start0 + index, true);
    }
    index -= first;
    if (index < this.end1 - this.start1) {
      return this.ast.extraChild(this.owner1, this.start1 + index, true);
    }
    return this.suffix === -1 ? NONE : this.suffix >>> 0;
  }
  clear() {
    this.ast = null;
    this.parentId = 0;
    this.key = null;
    this.kind = -1;
    this.descriptor = null;
  }
}

// One context per walk. Prototype methods keep per-hook closures out of the
// traversal; private fields keep the storage shape stable across walks.
class WalkContext {
  #ast = null;
  #state;
  #frame;
  #phase;
  #ancestors = [];
  #listCursors = [];
  #depth = 0;
  #dispatch;
  #entries;
  #globalEnter;
  #globalLeave;
  #plans;
  #dynamicPlans;
  #kindPlans;
  #dynamicHooked;
  #nativeReaders;
  #isTs = false;
  #node = null;
  #storage = {
    id: 0,
    parentId: NONE,
    key: null,
    list: null,
    index: 0,
    after: 0,
    flags: 0,
    kind: 0,
    type: null,
    version: -1,
  };
  stopped = false;

  initialize(ast, compiled, state) {
    const lang = ast._isTs ? 1 : 0;
    this.#ast = ast;
    this.#state = state;
    this.#dispatch = compiled.dispatch;
    this.#entries = compiled.entries ?? (compiled.entries = buildEntries(compiled.dispatch));
    this.#globalEnter = compiled.globalEnter;
    this.#globalLeave = compiled.globalLeave;
    this.#plans = STATIC_PLANS[lang];
    this.#dynamicPlans = DYNAMIC_PLANS[lang];
    this.#kindPlans = KIND_PLANS[lang];
    this.#dynamicHooked = compiled.dynamicHooked ?? (compiled.dynamicHooked = buildDynamicHooked(compiled));
    this.#nativeReaders = childVisitors[lang];
    this.#isTs = ast._isTs;
    this.#node = null;
    this.stopped = false;
  }

  #check() {
    const frame = this.#frame;
    if (frame === undefined) {
      throw new Error("Walk context is only valid during its hook");
    }
    return frame;
  }

  get nodeId() {
    return this.#check().id;
  }
  get parentId() {
    return this.#check().parentId;
  }
  get type() {
    const frame = this.#check();
    const ast = this.#ast;
    return frame.version === ast._editVersion ? frame.type : ast._typeAt(frame.id);
  }
  get kind() {
    const frame = this.#check();
    const ast = this.#ast;
    return frame.version === ast._editVersion ? frame.kind : ast._kindAt(frame.id);
  }
  get key() {
    return this.#check().key;
  }
  get index() {
    const frame = this.#check();
    return frame.flags & REMOVED || frame.list === null ? null : frame.index;
  }
  get state() {
    this.#check();
    return this.#state;
  }
  get ast() {
    this.#check();
    return this.#ast;
  }
  get node() {
    const node = this.#node;
    if (node !== null) { return node; }
    const frame = this.#check();
    return (this.#node = this.#ast._nodeAt(frame.id));
  }
  get parent() {
    const frame = this.#check();
    return frame.parentId === NONE ? null : this.#ast._nodeAt(frame.parentId);
  }
  ancestors() {
    this.#check();
    const result = new Array(this.#depth);
    for (let index = 0; index < this.#depth; index++) {
      result[index] = this.#ast._nodeAt(this.#ancestors[index]);
    }
    return result;
  }
  ancestorIds() {
    this.#check();
    return this.#ancestors.slice(0, this.#depth);
  }
  skip() {
    this.#check();
    if (this.#phase !== "enter") {
      throw new Error("skip() is only valid in enter hooks");
    }
    this.#frame.flags |= SKIP;
  }
  stop() {
    this.#check();
    this.stopped = true;
  }

  #editable() {
    const frame = this.#check();
    if (frame.flags & REMOVED) {
      throw new Error("The current node has been removed");
    }
    if (frame.parentId === NONE) {
      throw new Error("Cannot structurally edit the walk root");
    }
    const ast = this.#ast;
    let list = frame.list;
    if (list !== null) {
      const parent = ast.node(frame.parentId);
      const fresh = parent[frame.key];
      if (list.numeric === true) {
        if (!cursorMatchesList(list, fresh, ast, frame.parentId)) {
          throw new Error(
            "The current node's container was changed outside the walk context",
          );
        }
        frame.list = list = fresh;
      } else if (!sameListLike(fresh, list)) {
        throw new Error(
          "The current node's container was changed outside the walk context",
        );
      }
    }
    let currentId;
    if (list !== null) {
      if (frame.index < 0 || frame.index >= list.length) {
        throw new Error(
          "The current node's container was changed outside the walk context",
        );
      }
      currentId = list.nodeId(frame.index);
    } else {
      currentId = readChildId(ast, frame.parentId, frame.key, ast._kindAt(frame.parentId));
    }
    if (currentId !== frame.id) {
      throw new Error(
        "The current node's container was changed outside the walk context",
      );
    }
    return frame;
  }

  replace(node) {
    const frame = this.#editable();
    assertNode(node);
    const ast = this.#ast;
    if (node instanceof NodeView) {
      if (sameNode(node, ast, frame.id)) { return; }
      // Only the replacement root inherits missing positions.
      if (node.start == null) { node.start = ast.start(frame.id); }
      if (node.end == null) { node.end = ast.end(frame.id); }
    } else if (node.start == null || node.end == null) {
      node = { ...node, start: node.start ?? ast.start(frame.id), end: node.end ?? ast.end(frame.id) };
    }
    if (frame.list !== null) {
      frame.list.set(frame.index, node);
      frame.id = frame.list.nodeId(frame.index);
    } else {
      const parent = ast.node(frame.parentId);
      parent[frame.key] = node;
      frame.id = parent[frame.key].nodeId;
    }
    this.#node = null;
    frame.flags |= REPLACED;
  }

  remove() {
    const frame = this.#editable();
    if (frame.list === null) {
      throw new Error("Removal requires an array element");
    }
    frame.list.splice(frame.index, 1);
    frame.index--;
    frame.flags |= REMOVED;
  }

  #insert(nodes, after) {
    const frame = this.#editable();
    if (frame.list === null) {
      throw new Error("Insertion requires an array element");
    }
    const index = frame.index + (after ? 1 + frame.after : 0);
    let count;
    if (Array.isArray(nodes) || isListLike(nodes)) {
      nodes.forEach(assertNode);
      frame.list.splice(index, 0, ...nodes);
      count = nodes.length;
    } else {
      assertNode(nodes);
      frame.list.splice(index, 0, nodes);
      count = 1;
    }
    if (after) { frame.after += count; }
    else { frame.index += count; }
  }
  insertBefore(nodes) {
    this.#insert(nodes, false);
  }
  insertAfter(nodes) {
    this.#insert(nodes, true);
  }

  invoke(handlers, frame, phase) {
    this.#frame = frame;
    this.#phase = phase;
    if (typeof handlers === "function") {
      assertSync(handlers(this));
    } else {
      for (let index = 0; index < handlers.length; index++) {
        const handler = handlers[index];
        assertSync(handler(this));
        if (
          this.stopped ||
          frame.flags & (REMOVED | REPLACED) ||
          (phase === "enter" && frame.flags & SKIP)
        ) {
          break;
        }
      }
    }
    this.#node = null;
    this.#frame = undefined;
  }

  // Keep global enter and leave at separate call sites for V8 feedback.
  #invokeGlobalEnter(frame) {
    this.#frame = frame;
    this.#phase = "enter";
    assertSync(this.#globalEnter(this));
    this.#node = null;
    this.#frame = undefined;
  }

  #invokeGlobalLeave(frame) {
    this.#frame = frame;
    this.#phase = "leave";
    assertSync(this.#globalLeave(this));
    this.#node = null;
    this.#frame = undefined;
  }

  #prepare(id, parentId, key, list, index, after, kind, type) {
    const frame = this.#storage;
    frame.id = id;
    this.#node = null;
    frame.parentId = parentId;
    frame.key = key;
    frame.list = list;
    frame.index = index;
    frame.after = after;
    frame.flags = 0;
    frame.kind = kind;
    frame.type = type;
    frame.version = this.#ast._editVersion;
  }

  #traversePlan(id, kind, plan) {
    if (plan === undefined || plan === null) {
      throw new TypeError(`Unsupported AST node type: ${this.#ast._typeAt(id, kind)}`);
    }
    const hot = plan.hot;
    // No field can expose a child, so no descendant edit can reach this node.
    if (hot.length === 0) { return; }
    const ast = this.#ast;
    const reader = this.#nativeReaders[kind];
    if (reader !== null) {
      if (reader(ast, id, this, plan, kind)) { return; }
      return;
    }
    let version = ast._editVersion;
    for (let index = 0; index < hot.length; index++) {
      const field = hot[index];
      if (field.op === PLAN_LIST) {
        if (this.#visitList(id, field.key, field.desc)) { return; }
      } else {
        const childId = field.op === PLAN_DIRECT
          ? readNativeChild(ast, id, field.direct)
          : readChildIdWithRule(ast, id, field.key, kind, field.rule);
        if (childId !== NONE) {
          this.#ancestors[this.#depth++] = id;
          this.#visit(childId, id, field.key, null, 0);
          this.#depth--;
          if (this.stopped) { return; }
        }
      }
      if (ast._editVersion !== version) {
        version = ast._editVersion;
        const liveKind = ast._kindAt(id);
        if (liveKind !== kind) {
          this.#traverseRemaining(id, liveKind, plan.keys, field.index + 1, version);
          return;
        }
      }
    }
  }

  // Cold path: a descendant changed this node's kind. Keep the original
  // visitor key order but resolve every remaining key against the live layout,
  // so a stale direct offset is never read.
  #traverseRemaining(id, kind, keys, start, version) {
    const ast = this.#ast;
    for (let index = start; index < keys.length; index++) {
      const key = keys[index];
      if (isListField(this.#isTs, kind, key)) {
        if (this.#visitList(id, key)) { return; }
      } else {
        const rule = fieldRule(this.#isTs, kind, key);
        const direct = directField(this.#isTs, kind, key, rule);
        const childId = direct !== null
          ? readNativeChild(ast, id, direct)
          : rule !== ReadRule.Native
            ? readChildIdWithRule(ast, id, key, kind, rule)
            : NONE;
        if (childId !== NONE) {
          this.#ancestors[this.#depth++] = id;
          this.#visit(childId, id, key, null, 0);
          this.#depth--;
          if (this.stopped) { return; }
        }
      }
      if (ast._editVersion !== version) {
        version = ast._editVersion;
        kind = ast._kindAt(id);
      }
    }
  }

  #visitList(id, key, desc) {
    const ast = this.#ast;
    let cursor = this.#listCursors[this.#depth];
    if (cursor === undefined) {
      cursor = this.#listCursors[this.#depth] = new NumericListCursor();
    }
    cursor.reset(ast, id, key);
    const kind = ast._kindAt(id);
    if (desc === undefined) { desc = listDescriptor(this.#isTs, kind, key); }
    if (desc === null || !fillList(ast, id, cursor, desc)) {
      cursor.clear();
      return false;
    }
    cursor.kind = kind;
    cursor.descriptor = desc;
    cursor.finish();
    if (cursor.total === 0) {
      cursor.clear();
      return false;
    }
    this.#ancestors[this.#depth++] = id;
    try {
      for (let index = 0; index < cursor.length; index++) {
        const childId = cursor.nodeId(index);
        if (childId !== NONE) {
          index = this.#visit(childId, id, key, cursor, index);
        }
        if (this.stopped) { return true; }
        cursor.sync();
      }
      return false;
    } finally {
      this.#depth--;
      cursor.clear();
    }
  }

  // Entry points used by the hot straight-line readers in
  // child-visitors.js. They keep the numeric frame and ancestor rules.
  visitScalarId(childId, parentId, key) {
    this.#ancestors[this.#depth++] = parentId;
    this.#visit(childId, parentId, key, null, 0);
    this.#depth--;
    return this.stopped;
  }

  visitListField(id, key, plan) {
    return this.#visitList(id, key, plan === undefined ? undefined : plan.lists[key]);
  }

  visitRemaining(id, kind, plan, index) {
    this.#traverseRemaining(id, kind, plan.keys, index, this.#ast._editVersion);
    return this.stopped;
  }

  #leaveChanged(id, type, parentId, key, list, index) {
    const entry = this.#dispatch[type];
    if (entry === undefined) {
      throw new TypeError(`Unsupported AST node type: ${type}`);
    }
    const leave = entry.leave ?? this.#globalLeave;
    if (leave === null) { return index; }
    this.#prepare(id, parentId, key, list, index, 0, this.#ast._kindAt(id), type);
    if (leave === this.#globalLeave) {
      this.#invokeGlobalLeave(this.#storage);
    } else {
      this.invoke(leave, this.#storage, "leave");
    }
    return this.#storage.index;
  }

  #visit(id, parentId, key, list, index) {
    const ast = this.#ast;
    const kind = ast._kindAt(id);
    const staticType = staticTypes[kind];
    let type;
    let plan;
    let entry;
    if (staticType == null) {
      if (kind === NodeKind.FormalParameter) {
        const row = ast.row(id);
        const payload = ast._payloadAt(id, row);
        const view = row === -1 ? ast._view : ast._arena.view;
        const pattern = ast.childAt(id, payload + FORMAL_PATTERN[1], FORMAL_PATTERN[3], view);
        const initializer = ast.childAt(id, payload + FORMAL_INITIALIZER[1], FORMAL_INITIALIZER[3], view);
        const patternKind = ast._kindAt(pattern);
        type = initializer !== NONE ? "AssignmentPattern" : layouts[patternKind].type;
        const plans = this.#dynamicPlans[kind];
        plan = plans === undefined || plans === null ? undefined : plans.get(type);
        if (this.#dynamicHooked[kind]) {
          entry = this.#dispatch[type];
        } else {
          type = null;
          entry = UNHOOKED_ENTRY;
        }
      } else {
        const kindPlan = this.#kindPlans[kind];
        if (kindPlan !== undefined) {
          plan = kindPlan;
          if (this.#dynamicHooked[kind]) {
            if (kind === NodeKind.Function) {
              const row = ast.row(id);
              const payload = ast._payloadAt(id, row);
              const view = row === -1 ? ast._view : ast._arena.view;
              type = ast.enumAt(payload + FUNCTION_TYPE[1], "FunctionType", false, view);
            } else {
              type = ast._typeAt(id, kind);
            }
            entry = this.#dispatch[type];
          } else {
            type = null;
            entry = UNHOOKED_ENTRY;
          }
        } else {
          type = ast._typeAt(id, kind);
          const plans = this.#dynamicPlans[kind];
          plan = plans === undefined || plans === null ? undefined : plans.get(type);
          entry = this.#dispatch[type];
        }
      }
    } else {
      type = staticType;
      plan = this.#plans[kind];
      entry = this.#entries[kind];
    }
    if (entry === undefined) {
      throw new TypeError(`Unsupported AST node type: ${type}`);
    }
    const enter = entry.enter ?? this.#globalEnter;
    const leave = entry.leave ?? this.#globalLeave;
    if (enter === null && leave === null) {
      if (plan !== undefined && plan !== null && plan.hot.length === 0) { return index; }
      const version = ast._editVersion;
      this.#traversePlan(id, kind, plan);
      // A descendant may have changed this node's kind in place. Dispatch the
      // leave hook by the live type for every kind, static or dynamic.
      if (!this.stopped && ast._editVersion !== version) {
        if (type !== null) {
          const finalType = ast._typeAt(id);
          if (finalType !== type) {
            return this.#leaveChanged(id, finalType, parentId, key, list, index);
          }
        } else if (ast._kindAt(id) !== kind) {
          // The original kind had no hooks, but a descendant may have changed
          // it to a kind whose leave hook must still run.
          const finalType = ast._typeAt(id);
          const entry = this.#dispatch[finalType];
          const leave = entry === undefined ? this.#globalLeave : entry.leave ?? this.#globalLeave;
          if (leave !== null) {
            return this.#leaveChanged(id, finalType, parentId, key, list, index);
          }
        }
      }
      return index;
    }
    return this.#visitHooked(id, kind, type, plan, entry, parentId, key, list, index);
  }

  #visitHooked(id, kind, type, plan, entry, parentId, key, list, index) {
    const ast = this.#ast;
    const frame = this.#storage;
    let after = 0;
    let skip = false;
    let currentId = id;
    let currentKind = kind;
    let currentType = type;
    let currentPlan = plan;
    let currentEntry = entry;
    const enter = entry.enter ?? this.#globalEnter;
    if (enter !== null) {
      const version = ast._editVersion;
      this.#prepare(currentId, parentId, key, list, index, 0, currentKind, currentType);
      if (enter === this.#globalEnter) {
        this.#invokeGlobalEnter(frame);
      } else if (typeof enter === "function") {
        this.#frame = frame;
        this.#phase = "enter";
        assertSync(enter(this));
        this.#node = null;
        this.#frame = undefined;
      } else {
        this.invoke(enter, frame, "enter");
      }
      index = frame.index;
      if (this.stopped || frame.flags & REMOVED) { return index; }
      // A replacement returns a normalized ID; a direct in-place edit keeps
      // the ID but may change the kind. Re-resolve both before reading fields.
      if (frame.id !== currentId || ast._editVersion !== version) {
        currentId = frame.id;
        currentKind = ast._kindAt(currentId);
        if (staticTypes[currentKind] == null) {
          currentType = ast._typeAt(currentId, currentKind);
          const kindPlan = this.#kindPlans[currentKind];
          if (kindPlan !== undefined) {
            currentPlan = kindPlan;
          } else {
            const plans = this.#dynamicPlans[currentKind];
            currentPlan = plans === undefined || plans === null ? undefined : plans.get(currentType);
          }
          currentEntry = this.#dispatch[currentType];
        } else {
          currentType = staticTypes[currentKind];
          currentPlan = this.#plans[currentKind];
          currentEntry = this.#entries[currentKind];
        }
        if (currentEntry === undefined) {
          throw new TypeError(`Unsupported AST node type: ${currentType}`);
        }
      }
      after = frame.after;
      skip = frame.flags & SKIP;
    }
    if (!skip) {
      if (currentPlan !== undefined && currentPlan !== null && currentPlan.hot.length === 0) {
        // No child fields; no descendant edit can change this node here.
      } else {
        const version = ast._editVersion;
        this.#traversePlan(currentId, currentKind, currentPlan);
        if (this.stopped) { return index; }
        // Match the ordinary walker: a leave hook dispatches by the live type.
        if (ast._editVersion !== version) {
          const finalType = ast._typeAt(currentId);
          currentKind = ast._kindAt(currentId);
          if (finalType !== currentType) {
            currentType = finalType;
            currentEntry = this.#dispatch[finalType];
            if (currentEntry === undefined) {
              throw new TypeError(`Unsupported AST node type: ${finalType}`);
            }
          }
        }
      }
    }
    const leave = currentEntry.leave ?? this.#globalLeave;
    if (leave !== null) {
      this.#prepare(currentId, parentId, key, list, index, after, currentKind, currentType);
      if (leave === this.#globalLeave) {
        this.#invokeGlobalLeave(frame);
      } else if (typeof leave === "function") {
        this.#frame = frame;
        this.#phase = "leave";
        assertSync(leave(this));
        this.#node = null;
        this.#frame = undefined;
      } else {
        this.invoke(leave, frame, "leave");
      }
      index = frame.index;
    }
    return index;
  }

  visitRoot(id) {
    this.#visit(id, NONE, null, null, 0);
  }

  reset() {
    this.#ast = null;
    this.#node = null;
    this.#frame = undefined;
    this.#phase = undefined;
    this.#state = undefined;
    this.#ancestors.length = 0;
    for (const cursor of this.#listCursors) {
      if (cursor !== undefined) { cursor.clear(); }
    }
    if (this.#listCursors.length > 128) { this.#listCursors.length = 128; }
    this.#depth = 0;
    this.#storage.id = 0;
    this.#storage.parentId = NONE;
    this.#storage.key = null;
    this.#storage.list = null;
    this.#dispatch =
      this.#entries =
      this.#globalEnter =
      this.#globalLeave =
      this.#plans =
      this.#dynamicPlans =
        undefined;
    this.stopped = false;
  }
}

// Reuse cleared traversal state across calls. Nested walks acquire a different
// context; the pool retains no AST, views, handlers or caller state.
const contexts = [];

function runWalk(root, compiled, state) {
  if (!(root instanceof NodeView)) {
    throw new TypeError("Expected a NodeView from @swc-next/ast2");
  }
  const ast = root.ast;
  const rootId = root.nodeId;
  ast.validateId(rootId);
  const kind = ast._kindAt(rootId);
  const type = ast._typeAt(rootId, kind);
  if (!isNodeType(type)) {
    throw new TypeError(`Unsupported AST node type: ${type}`);
  }
  let ctx = contexts.pop();
  if (ctx === undefined) { ctx = new WalkContext(); }
  ctx.initialize(ast, compiled, state);
  try {
    ctx.visitRoot(rootId);
  } finally {
    ctx.reset();
    if (contexts.length < 4) { contexts.push(ctx); }
  }
  return root;
}

/**
 * Traverse a buffer-backed AST synchronously from a NodeView.
 * @param {import("@swc-next/ast2").NodeView} root
 * @param {object} hooks
 * @param {any} [state]
 * @returns {import("@swc-next/ast2").NodeView} root
 */
export function walk(root, hooks, state) {
  return runWalk(root, compile(hooks), state);
}

/**
 * Compile and snapshot hooks for repeated synchronous AST traversal.
 * @param {object} hooks
 */
export function createWalker(hooks) {
  const compiled = compile(hooks);
  return Object.freeze({
    walk(root, state) {
      return runWalk(root, compiled, state);
    },
  });
}
