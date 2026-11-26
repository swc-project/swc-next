// Buffer-backed AST walker.
//
// The walker is specialized for native AST views. Traversal frames,
// ancestor stacks and list cursors hold numbers only; a NodeView is created
// only when a hook first touches `ctx.node`, `ctx.parent` or `ctx.ancestors()`.
// Child traversal uses offsets generated from the native AST layout.
//
// Public entry points:
//   walk(root, hooks, state?) -> root
//   createWalker(hooks).walk(root, state?) -> root
//
// `root` is a NodeView from an AstView (`ast.program` or `ast.node(id)`).
// Hooks receive `(ctx)`; the current node is materialized lazily by `ctx.node`,
// and `ctx` also exposes numeric `nodeId`, `parentId`, `kind` and `ancestorIds()`.
import { NodeView } from "#protocol/index";
import { visitorKeys } from "./visitor-keys.js";
import { childVisitors } from "./child-visitors.js";
import { NONE, fieldId, fieldNames, layouts, nativeField } from "#protocol/internal";
import { compile, assertNode, assertSync, SKIP, REMOVED, REPLACED } from "./hook-compiler.js";

export { visitorKeys, NONE };

function childDescriptor(kind, field) {
  const desc = nativeField(kind, field);
  if (desc === undefined || (desc[2] !== "node" && desc[2] !== "list")) { return null; }
  return [desc[0] ? layouts[kind].extra : -1, desc[1], desc[3], desc[2] === "list"];
}

const plans = layouts.map(({ name }, kind) => {
  const fields = visitorKeys[name].map(fieldId);
  const children = fields.map(field => childDescriptor(kind, field));
  return { fields, children };
});

function sameNode(node, ast, id) {
  return node !== null && typeof node === "object" && node.ast === ast && node.nodeId === id;
}

function sameListLike(left, right) {
  return left === right || (left != null && right != null &&
    left.ast === right.ast && left.owner === right.owner && left.field === right.field);
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

// A range cursor never creates an AstList unless the hook requests an edit.
class NumericListCursor {
  numeric = true;
  ast = null;
  owner = 0;
  field = 0;
  start = 0;
  end = 0;
  version = -1;
  kind = -1;
  descriptor = null;

  reset(ast, owner, field, kind, descriptor) {
    this.ast = ast;
    this.owner = owner;
    this.field = field;
    this.kind = kind;
    this.descriptor = descriptor;
    this.#readRange();
  }

  #readRange() {
    const { ast, owner, descriptor } = this;
    const row = ast.row(owner);
    const payload = ast._payloadAt(owner, row);
    let view = row === -1 ? ast._view : ast._arena.view;
    let offset = payload + descriptor[1];
    if (descriptor[0] !== -1) {
      const start = view.getUint32(payload + descriptor[0], true);
      const index = start + descriptor[1];
      const limit = start < ast._extraCount ? ast._extraCount : ast.extraDataCount;
      if (index + 2 > limit) { throw new RangeError("AST extra-data reference out of bounds"); }
      view = index < ast._extraCount ? ast._view : ast._arena.view;
      offset = index < ast._extraCount ? ast._extra + index * 4 : ast._arena.extraAddress(index);
    }
    const start = view.getUint32(offset, true), end = view.getUint32(offset + 4, true);
    if (start > end || end > ast.extraDataCount) { throw new RangeError("AST list out of bounds"); }
    this.start = start;
    this.end = end;
    this.version = ast._listVersion;
  }

  sync() {
    const ast = this.ast;
    if (this.version === ast._listVersion) { return; }
    const kind = ast._kindAt(this.owner);
    if (kind !== this.kind) {
      const descriptor = childDescriptor(kind, this.field);
      if (!descriptor?.[3]) { throw new Error("The current node's container was changed outside the walk context"); }
      this.kind = kind;
      this.descriptor = descriptor;
    }
    this.#readRange();
  }
  get length() { return this.end - this.start; }
  nodeId(index) { return this.ast.extraChild(this.owner, this.start + index, true); }
  clear() { this.ast = null; this.descriptor = null; }
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
  #entries;
  #globalEnter;
  #globalLeave;
  #node = null;
  #storage = {
    id: 0,
    parentId: NONE,
    field: 0,
    list: null,
    index: 0,
    after: 0,
    flags: 0,
    kind: 0,
    version: -1,
  };
  stopped = false;

  initialize(ast, compiled, state) {
    this.#ast = ast;
    this.#state = state;
    this.#entries = compiled.entries;
    this.#globalEnter = compiled.globalEnter;
    this.#globalLeave = compiled.globalLeave;
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
  get type() { return layouts[this.kind].name; }
  get kind() {
    const frame = this.#check();
    const ast = this.#ast;
    return frame.version === ast._editVersion ? frame.kind : ast._kindAt(frame.id);
  }
  get key() {
    return fieldNames[this.#check().field] || null;
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
      const fresh = ast.value(frame.parentId, frame.field);
      if (list.numeric === true) {
        if (!isListLike(fresh) || !sameListLike(list, fresh)) {
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
      const desc = childDescriptor(ast._kindAt(frame.parentId), frame.field);
      currentId = desc !== null && !desc[3] ? readNativeChild(ast, frame.parentId, desc) : NONE;
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
      ast.writeNativeField(frame.parentId, frame.field, node);
      frame.id = ast._ref(frame.parentId, frame.field);
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

  #prepare(id, parentId, field, list, index, after, kind) {
    const frame = this.#storage;
    frame.id = id;
    this.#node = null;
    frame.parentId = parentId;
    frame.field = field;
    frame.list = list;
    frame.index = index;
    frame.after = after;
    frame.flags = 0;
    frame.kind = kind;
    frame.version = this.#ast._editVersion;
  }

  #traversePlan(id, kind, plan) {
    const ast = this.#ast;
    const reader = childVisitors[kind];
    if (reader !== null) { reader(ast, id, this, plan, kind); return; }
    let version = ast._editVersion;
    const { fields, children } = plan;
    for (let index = 0; index < children.length; index++) {
      if (this.#visitField(id, fields[index], children[index])) { return; }
      if (ast._editVersion !== version) {
        version = ast._editVersion;
        const liveKind = ast._kindAt(id);
        if (liveKind !== kind) {
          this.#traverseRemaining(id, liveKind, fields, index + 1, version);
          return;
        }
      }
    }
  }

  // Preserve the original field order after a descendant changes the parent kind.
  #traverseRemaining(id, kind, fields, start, version) {
    const ast = this.#ast;
    for (let index = start; index < fields.length; index++) {
      const field = fields[index];
      const descriptor = childDescriptor(kind, field);
      if (descriptor !== null && this.#visitField(id, field, descriptor)) { return; }
      if (ast._editVersion !== version) {
        version = ast._editVersion;
        kind = ast._kindAt(id);
      }
    }
  }

  #visitField(id, field, descriptor) {
    if (descriptor[3]) { return this.#visitList(id, field, descriptor); }
    const child = readNativeChild(this.#ast, id, descriptor);
    return child !== NONE && this.visitScalarId(child, id, field);
  }

  #visitList(id, field, descriptor) {
    const ast = this.#ast;
    let cursor = this.#listCursors[this.#depth];
    if (cursor === undefined) {
      cursor = this.#listCursors[this.#depth] = new NumericListCursor();
    }
    cursor.reset(ast, id, field, ast._kindAt(id), descriptor);
    if (cursor.length === 0) { cursor.clear(); return false; }
    this.#ancestors[this.#depth++] = id;
    try {
      for (let index = 0; index < cursor.length; index++) {
        const child = cursor.nodeId(index);
        if (child !== NONE) { index = this.#visit(child, id, field, cursor, index); }
        if (this.stopped) { return true; }
        cursor.sync();
      }
      return false;
    } finally {
      this.#depth--;
      cursor.clear();
    }
  }

  // Entry points for the generated straight-line child readers.
  visitScalarId(child, parent, field) {
    this.#ancestors[this.#depth++] = parent;
    this.#visit(child, parent, field, null, 0);
    this.#depth--;
    return this.stopped;
  }

  visitListField(id, field, descriptor) {
    return this.#visitList(id, field, descriptor);
  }

  visitRemaining(id, kind, plan, index) {
    this.#traverseRemaining(id, kind, plan.fields, index, this.#ast._editVersion);
    return this.stopped;
  }

  #visit(id, parentId, field, list, index) {
    const ast = this.#ast;
    const kind = ast._kindAt(id);
    const plan = plans[kind];
    const entry = this.#entries[kind];
    if (entry.enter === null && entry.leave === null && this.#globalEnter === null && this.#globalLeave === null) {
      if (plan.children.length === 0) { return index; }
      const version = ast._editVersion;
      this.#traversePlan(id, kind, plan);
      if (!this.stopped && ast._editVersion !== version) {
        const liveKind = ast._kindAt(id);
        const leave = this.#entries[liveKind].leave;
        if (liveKind !== kind && leave !== null) {
          this.#prepare(id, parentId, field, list, index, 0, liveKind);
          this.invoke(leave, this.#storage, "leave");
          return this.#storage.index;
        }
      }
      return index;
    }
    return this.#visitHooked(id, kind, plan, entry, parentId, field, list, index);
  }

  #visitHooked(id, kind, plan, entry, parentId, field, list, index) {
    const ast = this.#ast;
    const frame = this.#storage;
    let after = 0;
    let skip = false;
    let currentId = id;
    let currentKind = kind;
    let currentPlan = plan;
    let currentEntry = entry;
    const enter = entry.enter ?? this.#globalEnter;
    if (enter !== null) {
      const version = ast._editVersion;
      this.#prepare(currentId, parentId, field, list, index, 0, currentKind);
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
        currentPlan = plans[currentKind];
        currentEntry = this.#entries[currentKind];
      }
      after = frame.after;
      skip = frame.flags & SKIP;
    }
    if (!skip && currentPlan.children.length !== 0) {
      const version = ast._editVersion;
      this.#traversePlan(currentId, currentKind, currentPlan);
      if (this.stopped) { return index; }
      if (ast._editVersion !== version) {
        currentKind = ast._kindAt(currentId);
        currentEntry = this.#entries[currentKind];
      }
    }
    const leave = currentEntry.leave ?? this.#globalLeave;
    if (leave !== null) {
      this.#prepare(currentId, parentId, field, list, index, after, currentKind);
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
    this.#visit(id, NONE, 0, null, 0);
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
    this.#storage.field = 0;
    this.#storage.list = null;
    this.#entries = this.#globalEnter = this.#globalLeave = undefined;
    this.stopped = false;
  }
}

// Reuse cleared traversal state across calls. Nested walks acquire a different
// context; the pool retains no AST, views, handlers or caller state.
const contexts = [];

function runWalk(root, compiled, state) {
  if (!(root instanceof NodeView)) {
    throw new TypeError("Expected a native AST NodeView");
  }
  const ast = root.ast;
  const rootId = root.nodeId;
  ast.validateId(rootId);
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
 * @param {import("#protocol/index").NodeView} root
 * @param {object} hooks
 * @param {any} [state]
 * @returns {import("#protocol/index").NodeView} root
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
