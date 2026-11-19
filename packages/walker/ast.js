// Numeric AST-only walker.
//
// Traversal frames, ancestor stacks and list cursors hold numbers only. A
// NodeView is created only when a hook touches `ctx.node`, `ctx.parent` or
// `ctx.ancestors()`. Scalar child reads use precomputed direct descriptors for
// native fields; projected exceptions fall back to the numeric projection
// bridge in ast-projection.js.
//
// Public entry points:
//   walk(ast, rootId, hooks, state?) -> rootId
//   createWalker(hooks).walk(ast, rootId, state?) -> rootId
//
// `ast` is the AstView returned by `parseSync(...)` and `rootId` is a
// numeric node ID (`ast.program.nodeId`). Hooks receive `(nodeId, ctx)`.
import { visitorKeys } from "./visitor-keys.js";
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
  staticTypes,
  dynamicTypes,
  NodeKind,
  layouts,
} from "@swc-next/ast/internal";
import { readChildId, readList, isProjectedChildRule } from "./ast-projection.js";
import { compile, isNodeType, assertNode, assertSync, SKIP, REMOVED, REPLACED } from "./hook-compiler.js";

export { visitorKeys, NONE };

function isListField(isTs, kind, key) {
  if (kind === NodeKind.FormalParameter && (key === "elements" || key === "properties")) {
    // DelegateList: the visible wrapper owns the key, the hidden pattern owns
    // the range. `readList` decides whether the list exists for this wrapper.
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
function directField(isTs, kind, key) {
  if (fieldRule(isTs, kind, key) !== ReadRule.Native) { return null; }
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

// A plan keeps the frozen visitor keys for kind changes plus the fields that
// can be children under the planned kind. Fields whose read rule is statically
// NONE are left out of `hot`; a live kind change re-resolves the remaining
// keys through #traverseRemaining.
function buildPlan(isTs, kind, type) {
  const keys = visitorKeys[type];
  if (keys === undefined) { return null; }
  const hot = [];
  for (let index = 0; index < keys.length; index++) {
    const key = keys[index];
    if (isListField(isTs, kind, key)) {
      hot.push({ index, key, op: PLAN_LIST, direct: null });
      continue;
    }
    const direct = directField(isTs, kind, key);
    if (direct !== null) {
      hot.push({ index, key, op: PLAN_DIRECT, direct });
      continue;
    }
    if (isProjectedChildRule(fieldRule(isTs, kind, key))) {
      hot.push({ index, key, op: PLAN_BRIDGE, direct: null });
    }
  }
  return { keys, hot };
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

const STATIC_PLANS = [buildStaticPlans(false), buildStaticPlans(true)];
const DYNAMIC_PLANS = [buildDynamicPlans(false), buildDynamicPlans(true)];

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
  reset(ast, parentId, key) {
    this.ast = ast;
    this.parentId = parentId;
    this.key = key;
    this.start1 = this.end1 = 0;
    this.prefix = this.suffix = -1;
    this.version = ast._listVersion;
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
    if (!readList(this.ast, this.parentId, this.key, this)) {
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
  #isTs = false;
  #storage = {
    id: 0,
    parentId: NONE,
    key: null,
    list: null,
    index: 0,
    after: 0,
    flags: 0,
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
    this.#isTs = ast._isTs;
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
    return this.#ast._typeAt(frame.id);
  }
  get kind() {
    const frame = this.#check();
    return this.#ast._kindAt(frame.id);
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
    const frame = this.#check();
    return this.#ast.node(frame.id);
  }
  get parent() {
    const frame = this.#check();
    return frame.parentId === NONE ? null : this.#ast.node(frame.parentId);
  }
  ancestors() {
    this.#check();
    const result = new Array(this.#depth);
    for (let index = 0; index < this.#depth; index++) {
      result[index] = this.#ast.node(this.#ancestors[index]);
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
    if (sameNode(node, this.#ast, frame.id)) { return; }
    const ast = this.#ast;
    // Only the replacement root inherits missing positions.
    if (node.start == null) { node.start = ast.start(frame.id); }
    if (node.end == null) { node.end = ast.end(frame.id); }
    if (frame.list !== null) {
      frame.list.set(frame.index, node);
      frame.id = frame.list.nodeId(frame.index);
    } else {
      const parent = ast.node(frame.parentId);
      parent[frame.key] = node;
      frame.id = parent[frame.key].nodeId;
    }
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
      assertSync(handlers(frame.id, this));
    } else {
      for (let index = 0; index < handlers.length; index++) {
        const handler = handlers[index];
        assertSync(handler(frame.id, this));
        if (
          this.stopped ||
          frame.flags & (REMOVED | REPLACED) ||
          (phase === "enter" && frame.flags & SKIP)
        ) {
          break;
        }
      }
    }
    this.#frame = undefined;
  }

  // Keep global enter and leave at separate call sites for V8 feedback.
  #invokeGlobalEnter(frame) {
    this.#frame = frame;
    this.#phase = "enter";
    assertSync(this.#globalEnter(frame.id, this));
    this.#frame = undefined;
  }

  #invokeGlobalLeave(frame) {
    this.#frame = frame;
    this.#phase = "leave";
    assertSync(this.#globalLeave(frame.id, this));
    this.#frame = undefined;
  }

  #prepare(id, parentId, key, list, index, after) {
    const frame = this.#storage;
    frame.id = id;
    frame.parentId = parentId;
    frame.key = key;
    frame.list = list;
    frame.index = index;
    frame.after = after;
    frame.flags = 0;
  }

  #traversePlan(id, kind, plan) {
    if (plan === undefined || plan === null) {
      throw new TypeError(`Unsupported AST node type: ${this.#ast._typeAt(id, kind)}`);
    }
    const hot = plan.hot;
    // No field can expose a child, so no descendant edit can reach this node.
    if (hot.length === 0) { return; }
    const ast = this.#ast;
    let version = ast._editVersion;
    for (let index = 0; index < hot.length; index++) {
      const field = hot[index];
      if (field.op === PLAN_LIST) {
        if (this.#visitList(id, field.key)) { return; }
      } else {
        const childId = field.op === PLAN_DIRECT
          ? readNativeChild(ast, id, field.direct)
          : readChildId(ast, id, field.key, kind);
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
        const direct = directField(this.#isTs, kind, key);
        const childId = direct !== null
          ? readNativeChild(ast, id, direct)
          : fieldRule(this.#isTs, kind, key) !== ReadRule.Native
            ? readChildId(ast, id, key, kind)
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

  #visitList(id, key) {
    const ast = this.#ast;
    let cursor = this.#listCursors[this.#depth];
    if (cursor === undefined) {
      cursor = this.#listCursors[this.#depth] = new NumericListCursor();
    }
    cursor.reset(ast, id, key);
    if (!readList(ast, id, key, cursor)) {
      cursor.clear();
      return false;
    }
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

  #leaveChanged(id, type, parentId, key, list, index) {
    const entry = this.#dispatch[type];
    if (entry === undefined) {
      throw new TypeError(`Unsupported AST node type: ${type}`);
    }
    const leave = entry.leave ?? this.#globalLeave;
    if (leave === null) { return index; }
    this.#prepare(id, parentId, key, list, index, 0);
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
    const dynamic = staticTypes[kind] == null;
    let type;
    let plan;
    let entry;
    if (dynamic) {
      type = ast._typeAt(id, kind);
      const plans = this.#dynamicPlans[kind];
      plan = plans === undefined || plans === null ? undefined : plans.get(type);
      entry = this.#dispatch[type];
    } else {
      type = staticTypes[kind];
      plan = this.#plans[kind];
      entry = this.#entries[kind];
    }
    if (entry === undefined) {
      throw new TypeError(`Unsupported AST node type: ${type}`);
    }
    const enter = entry.enter ?? this.#globalEnter;
    const leave = entry.leave ?? this.#globalLeave;
    if (enter === null && leave === null) {
      const version = ast._editVersion;
      this.#traversePlan(id, kind, plan);
      // A descendant may have changed this node's kind in place. Dispatch the
      // leave hook by the live type for every kind, static or dynamic.
      if (!this.stopped && ast._editVersion !== version) {
        const finalType = ast._typeAt(id);
        if (finalType !== type) {
          return this.#leaveChanged(id, finalType, parentId, key, list, index);
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
      this.#prepare(currentId, parentId, key, list, index, 0);
      if (enter === this.#globalEnter) {
        this.#invokeGlobalEnter(frame);
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
          const plans = this.#dynamicPlans[currentKind];
          currentPlan = plans === undefined || plans === null ? undefined : plans.get(currentType);
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
      const version = ast._editVersion;
      this.#traversePlan(currentId, currentKind, currentPlan);
      if (this.stopped) { return index; }
      // Match the ordinary walker: a leave hook dispatches by the live type.
      if (ast._editVersion !== version) {
        const finalType = ast._typeAt(currentId);
        if (finalType !== currentType) {
          currentType = finalType;
          currentEntry = this.#dispatch[finalType];
          if (currentEntry === undefined) {
            throw new TypeError(`Unsupported AST node type: ${finalType}`);
          }
        }
      }
    }
    const leave = currentEntry.leave ?? this.#globalLeave;
    if (leave !== null) {
      this.#prepare(currentId, parentId, key, list, index, after);
      if (leave === this.#globalLeave) {
        this.#invokeGlobalLeave(frame);
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

function runWalk(ast, rootId, compiled, state) {
  if (ast === null || typeof ast !== "object" ||
    typeof ast._kindAt !== "function" || typeof ast.node !== "function" ||
    typeof ast.validateId !== "function") {
    throw new TypeError("Expected an AST view");
  }
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
  return rootId;
}

/**
 * Traverse an AST view by numeric node ID without materializing NodeViews.
 * @param {import("@swc-next/ast").AstView} ast
 * @param {number} rootId
 * @param {object} hooks
 * @param {any} [state]
 * @returns {number} rootId
 */
export function walk(ast, rootId, hooks, state) {
  return runWalk(ast, rootId, compile(hooks), state);
}

/**
 * Compile and snapshot numeric hooks for repeated synchronous AST traversal.
 * @param {object} hooks
 */
export function createWalker(hooks) {
  const compiled = compile(hooks);
  return Object.freeze({
    walk(ast, rootId, state) {
      return runWalk(ast, rootId, compiled, state);
    },
  });
}
