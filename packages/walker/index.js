import { childVisitors, nodeWalkers, dispatchNode } from "./child-visitors.js";
import { visitorKeys } from "./visitor-keys.js";
import { NodeView } from "@swc-next/ast";

export { visitorKeys };

// Internal dispatch entries share generated child accessors. Public visitorKeys
// remain frozen; the hot path does not iterate those frozen arrays.
const baseDispatch = Object.create(null);
for (const [type, children] of Object.entries(childVisitors)) {
  baseDispatch[type] = { children, enter: null, leave: null };
}

const aliases = {
  Function: [
    "FunctionDeclaration",
    "FunctionExpression",
    "ArrowFunctionExpression",
    "TSDeclareFunction",
    "TSEmptyBodyFunctionExpression",
  ],
  Class: ["ClassDeclaration", "ClassExpression"],
};
const SKIP = 1;
const REMOVED = 2;
const REPLACED = 4;
const owns = (object, key) => Object.prototype.hasOwnProperty.call(object, key);
// Every supported type has an entry; the table has no inherited properties.
const isNodeType = (type) => baseDispatch[type] !== undefined;

function assertNode(node) {
  if (!node || typeof node !== "object" || !isNodeType(node.type)) {
    throw new TypeError(`Unsupported AST node type: ${node?.type}`);
  }
}

function assertSync(result) {
  if (result != null && typeof result.then === "function") {
    // An async callback may already have started. Observe rejection, but never
    // continue a synchronous traversal with its pending mutation context.
    Promise.resolve(result).catch(() => {});
    throw new TypeError("AST hooks must be synchronous");
  }
  if (result !== undefined)
    throw new TypeError(
      "AST hooks must return undefined; use the context to edit nodes",
    );
}

function compile(hooks) {
  if (!hooks || typeof hooks !== "object")
    throw new TypeError("Expected a hooks object");
  const entries = Object.entries(hooks);
  const table = new Map();
  let enter, leave;
  const add = (name, value) => {
    const pair = typeof value === "function" ? { enter: value } : value;
    if (!pair || typeof pair !== "object")
      throw new TypeError(`Invalid hook: ${name}`);
    for (const key of Object.keys(pair)) {
      if (
        (key !== "enter" && key !== "leave") ||
        (pair[key] !== undefined && typeof pair[key] !== "function")
      ) {
        throw new TypeError(`Invalid hook: ${name}.${key}`);
      }
    }
    const entry = table.get(name) ?? { enter: [], leave: [] };
    if (pair.enter) entry.enter.push(pair.enter);
    if (pair.leave) entry.leave.unshift(pair.leave);
    table.set(name, entry);
  };
  for (const [name, value] of entries) {
    if (value === undefined) continue;
    if (name === "enter" || name === "leave") {
      if (typeof value !== "function")
        throw new TypeError(`Invalid hook: ${name}`);
      if (name === "enter") enter = value;
      else leave = value;
    } else if (!owns(aliases, name) && !isNodeType(name)) {
      throw new TypeError(`Unknown AST hook: ${name}`);
    }
  }
  // Aliases surround concrete hooks, independent of object property order.
  for (const [name, value] of entries) {
    if (value !== undefined && owns(aliases, name)) {
      for (const type of aliases[name]) add(type, value);
    }
  }
  for (const [name, value] of entries) {
    if (value !== undefined && isNodeType(name)) add(name, value);
  }
  for (const pair of table.values()) {
    if (enter) pair.enter.unshift(enter);
    if (leave) pair.leave.push(leave);
  }
  const compact = (handlers) =>
    handlers.length === 0
      ? null
      : handlers.length === 1
        ? handlers[0]
        : handlers;
  const dispatch =
    table.size === 0 ? baseDispatch : Object.create(baseDispatch);
  for (const [type, pair] of table) {
    dispatch[type] = {
      children: childVisitors[type],
      enter: compact(pair.enter),
      leave: compact(pair.leave),
    };
  }
  const walkers = Object.create(nodeWalkers);
  const types = enter || leave ? Object.keys(nodeWalkers) : [...table.keys()];
  for (const type of types) {
    const entry = dispatch[type];
    const enterHook = entry.enter ?? enter ?? null;
    const leaveHook = entry.leave ?? leave ?? null;
    if (enterHook === null && leaveHook === null) continue;
    walkers[type] = enterHook === null ? walkLeave : walkEnter;
  }
  return {
    dispatch,
    walkers,
    globalEnter: enter ?? null,
    globalLeave: leave ?? null,
  };
}

function walkEnter(node, walker, parent, key, list, index) {
  return walker.visitHooks(node, parent, key, list, index);
}
function walkLeave(node, walker, parent, key, list, index) {
  return walker.visitLeaveHooks(node, parent, key, list, index);
}

// Private traversal state, never returned as an AST list. One cursor is reused
// per recursion depth; public list access and edits switch it to the real array.
class ListCursor {
  ast = null;
  parent = null;
  key = null;
  _fieldKey = -1;
  array = null;
  start0 = 0;
  end0 = 0;
  owner0 = 0;
  start1 = 0;
  end1 = 0;
  owner1 = 0;
  prefix = -1;
  suffix = -1;
  total = 0;
  version = 0;
  reset(parent, key) {
    this.ast = parent.ast;
    this.parent = parent;
    this.key = key;
    this._fieldKey = -1;
    this.array = null;
    // The range reader overwrites part zero. Unused owners and the previous
    // total are never observed before finish(), so only reset optional parts.
    this.start1 = this.end1 = 0;
    this.prefix = this.suffix = -1;
    this.version = this.ast._listVersion;
  }
  get fieldKey() {
    if (this._fieldKey === -1) this._fieldKey = this.ast.mirrorKey(this.parent.nodeId, this.key);
    return this._fieldKey;
  }
  setRange(part, owner, start, end) {
    if (part === 0) { this.owner0 = owner; this.start0 = start; this.end0 = end; }
    else { this.owner1 = owner; this.start1 = start; this.end1 = end; }
  }
  finish() {
    if (this.start0 === this.end0) {
      this.start0 = this.start1; this.end0 = this.end1; this.owner0 = this.owner1;
      this.start1 = this.end1 = 0;
    }
    this.total = this.end0 - this.start0 + this.end1 - this.start1
      + (this.prefix !== -1) + (this.suffix !== -1);
  }
  sync() {
    if (this.array !== null || this.version === this.ast._listVersion) return;
    this.version = this.ast._listVersion;
    const state = this.ast._watchedLists.get(this.fieldKey);
    if (state) this.array = state.value;
  }
  get length() { return this.array === null ? this.total : this.array.length; }
  at(index) {
    if (this.array !== null) return this.array[index];
    const ast = this.ast;
    if (this.prefix !== -1) {
      if (index === 0) return ast.node(this.prefix);
      index--;
    }
    const first = this.end0 - this.start0;
    if (index < first) return ast.node(ast.extraChild(this.owner0, this.start0 + index, true));
    index -= first;
    if (index < this.end1 - this.start1) return ast.node(ast.extraChild(this.owner1, this.start1 + index, true));
    return this.suffix === -1 ? null : ast.node(this.suffix);
  }
  materialize() {
    this.sync();
    if (this.array === null) {
      if (this.ast._edits?.get(this.parent.nodeId)?.has(this.key)) {
        throw new Error("The current node's container was changed outside the walk context");
      }
      this.array = this.parent[this.key];
    }
    return this.array;
  }
  clear() { this.ast = this.parent = this.key = this.array = null; this._fieldKey = -1; }
}

// One context per walk, with prototype methods instead of per-hook closures.
// A retained reference aliases later hooks; callers must not store it.
class WalkContext {
  #frame;
  #phase;
  #state;
  #ancestors = [];
  #listCursors = [];
  // Reuse depth slots instead of shrinking/growing the array for every node.
  // Only the live prefix is exposed; reset releases retained references.
  #depth = 0;
  #dispatch;
  #walkers;
  #globalEnter;
  #globalLeave;
  #storage = {
    node: null,
    parent: null,
    key: null,
    list: null,
    index: 0,
    after: 0,
    flags: 0,
  };
  stopped = false;

  constructor(state, compiled) {
    this.initialize(state, compiled);
  }

  initialize(state, compiled) {
    this.#state = state;
    this.#dispatch = compiled.dispatch;
    this.#walkers = compiled.walkers;
    this.#globalEnter = compiled.globalEnter;
    this.#globalLeave = compiled.globalLeave;
    this.stopped = false;
  }

  #check() {
    const frame = this.#frame;
    if (frame === undefined)
      throw new Error("Walk context is only valid during its hook");
    return frame;
  }

  #editable() {
    const frame = this.#check();
    if (frame.flags & REMOVED)
      throw new Error("The current node has been removed");
    if (!frame.parent)
      throw new Error("Cannot structurally edit the walk root");
    if (frame.list instanceof ListCursor) frame.list = frame.list.materialize();
    if (frame.list && frame.parent[frame.key] !== frame.list) {
      throw new Error(
        "The current node's container was changed outside the walk context",
      );
    }
    const current = frame.list
      ? frame.list[frame.index]
      : frame.parent[frame.key];
    if (!NodeView.isSame(current, frame.node))
      throw new Error(
        "The current node's container was changed outside the walk context",
      );
    return frame;
  }

  get node() {
    return this.#check().node;
  }
  get parent() {
    return this.#check().parent;
  }
  get key() {
    return this.#check().key;
  }
  get index() {
    const frame = this.#check();
    return frame.flags & REMOVED || !frame.list ? null : frame.index;
  }
  get state() {
    this.#check();
    return this.#state;
  }
  ancestors() {
    this.#check();
    return this.#ancestors.slice(0, this.#depth);
  }
  skip() {
    this.#check();
    if (this.#phase !== "enter")
      throw new Error("skip() is only valid in enter hooks");
    this.#frame.flags |= SKIP;
  }
  stop() {
    this.#check();
    this.stopped = true;
  }
  replace(node) {
    const frame = this.#editable();
    assertNode(node);
    if (NodeView.isSame(node, frame.node)) return;
    // Only the replacement root inherits missing positions.
    if (node.start == null) node.start = frame.node.start;
    if (node.end == null) node.end = frame.node.end;
    if (frame.list) {
      frame.list[frame.index] = node;
      if (frame.parent instanceof NodeView)
        frame.parent.ast.commitList(frame.parent.nodeId, frame.key, frame.list);
      frame.node = frame.list[frame.index];
    } else {
      frame.parent[frame.key] = node;
      frame.node = frame.parent[frame.key];
    }
    frame.flags |= REPLACED;
  }
  remove() {
    const frame = this.#editable();
    if (!frame.list) throw new Error("Removal requires an array element");
    frame.list.splice(frame.index, 1);
    if (frame.parent instanceof NodeView)
      frame.parent.ast.commitList(frame.parent.nodeId, frame.key, frame.list);
    frame.index--;
    frame.flags |= REMOVED;
  }
  #insert(nodes, after) {
    const frame = this.#editable();
    if (!frame.list) throw new Error("Insertion requires an array element");
    const index = frame.index + (after ? 1 + frame.after : 0);
    let count;
    if (Array.isArray(nodes)) {
      nodes.forEach(assertNode);
      frame.list.splice(index, 0, ...nodes);
      count = nodes.length;
    } else {
      assertNode(nodes);
      frame.list.splice(index, 0, nodes);
      count = 1;
    }
    if (after) frame.after += count;
    else frame.index += count;
    if (frame.parent instanceof NodeView)
      frame.parent.ast.commitList(frame.parent.nodeId, frame.key, frame.list);
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
      assertSync(handlers(frame.node, this));
    } else {
      for (let i = 0; i < handlers.length; i++) {
        const handler = handlers[i];
        assertSync(handler(frame.node, this));
        if (
          this.stopped ||
          frame.flags & (REMOVED | REPLACED) ||
          (phase === "enter" && frame.flags & SKIP)
        )
          break;
      }
    }
    this.#frame = undefined;
  }

  reset() {
    this.#frame = undefined;
    this.#phase = undefined;
    this.#state = undefined;
    this.#ancestors.length = 0;
    for (const cursor of this.#listCursors) cursor?.clear();
    if (this.#listCursors.length > 128) this.#listCursors.length = 128;
    this.#depth = 0;
    this.#storage.node = this.#storage.parent = this.#storage.list = null;
    this.#dispatch =
      this.#walkers =
      this.#globalEnter =
      this.#globalLeave =
        undefined;
  }

  #prepare(node, parent, key, list, index, after) {
    const frame = this.#storage;
    frame.node = node;
    frame.parent = parent;
    frame.key = key;
    frame.list = list;
    frame.index = index;
    frame.after = after;
    frame.flags = 0;
  }

  visitScalar(value, parent, key) {
    this.#ancestors[this.#depth++] = parent;
    this.visit(value, parent, key, null, 0);
    this.#depth--;
    return this.stopped;
  }

  visitListField(parent, key, dispatch = null) {
    if (!(parent instanceof NodeView)) {
      const value = parent[key];
      return value == null ? false : dispatch === null
        ? this.visitList(value, parent, key) : this.visitListKnown(value, parent, key, dispatch);
    }
    let cursor = this.#listCursors[this.#depth];
    if (!cursor) this.#listCursors[this.#depth] = cursor = new ListCursor();
    cursor.reset(parent, key);
    if (!parent.ast.readListCursor(parent.nodeId, key, cursor)) {
      cursor.clear();
      const value = parent[key];
      return value == null ? false : dispatch === null
        ? this.visitList(value, parent, key) : this.visitListKnown(value, parent, key, dispatch);
    }
    cursor.finish();
    if (cursor.total === 0) {
      cursor.clear();
      return false;
    }
    this.#ancestors[this.#depth++] = parent;
    try {
      for (let i = 0; i < cursor.length; i++) {
        const child = cursor.at(i);
        if (child != null) {
          if (dispatch === null) i = this.visit(child, parent, key, cursor, i);
          else {
            if (typeof child !== "object") assertNode(child);
            i = dispatch(child, this, parent, key, cursor, i, this.#walkers);
          }
        }
        if (this.stopped) return true;
        cursor.sync();
      }
      return false;
    } finally {
      this.#depth--;
      cursor.clear();
    }
  }

  visitListKnown(value, parent, key, dispatch) {
    this.#ancestors[this.#depth++] = parent;
    for (let i = 0; i < value.length; i++) {
      const child = value[i];
      if (child != null) {
        if (typeof child !== "object") assertNode(child);
        i = dispatch(child, this, parent, key, value, i, this.#walkers);
      }
      if (this.stopped) return true;
    }
    this.#depth--;
    return this.stopped;
  }

  visitList(value, parent, key) {
    this.#ancestors[this.#depth++] = parent;
    for (let i = 0; i < value.length; i++) {
      const child = value[i];
      if (child != null) i = this.visit(child, parent, key, value, i);
      if (this.stopped) return true;
    }
    this.#depth--;
    return this.stopped;
  }

  visit(node, parent, key, list, index) {
    if (typeof node !== "object") assertNode(node);
    // A global enter hook uses the same entry path for every supported type.
    // Resolve its handlers once inside visitHooks instead of dispatching twice.
    if (this.#globalEnter !== null)
      return this.visitHooks(node, parent, key, list, index);
    return dispatchNode(node, this, parent, key, list, index, this.#walkers);
  }

  leaveChanged(node, parent, key, list, index) {
    const entry = this.#dispatch[node.type];
    if (entry === undefined) assertNode(node);
    const leave = entry.leave ?? this.#globalLeave;
    if (leave === null) return index;
    this.#prepare(node, parent, key, list, index, 0);
    this.invoke(leave, this.#storage, "leave");
    return this.#storage.index;
  }

  visitLeaveHooks(node, parent, key, list, index) {
    const type = node.type;
    const handlers = this.#dispatch[type];
    if (handlers.children !== null) {
      handlers.children(node, this);
      if (this.stopped) return index;
    }
    if (node.type !== type)
      return this.leaveChanged(node, parent, key, list, index);
    this.#prepare(node, parent, key, list, index, 0);
    this.invoke(handlers.leave ?? this.#globalLeave, this.#storage, "leave");
    return this.#storage.index;
  }

  visitHooks(node, parent, key, list, index) {
    let type = node.type;
    let handlers = this.#dispatch[type];
    if (handlers === undefined) assertNode(node);
    const enter = handlers.enter ?? this.#globalEnter;
    const frame = this.#storage;
    let after = 0;
    let skip = false;
    if (enter !== null) {
      this.#prepare(node, parent, key, list, index, 0);
      this.invoke(enter, frame, "enter");
      index = frame.index;
      if (this.stopped || frame.flags & REMOVED) return index;
      node = frame.node;
      after = frame.after;
      skip = frame.flags & SKIP;
      const enteredType = node.type;
      if (enteredType !== type) {
        type = enteredType;
        handlers = this.#dispatch[type];
        if (handlers === undefined) assertNode(node);
      }
    }
    if (!skip && handlers.children !== null) {
      handlers.children(node, this);
      if (this.stopped) return index;
    }
    const finalType = node.type;
    if (finalType !== type) {
      handlers = this.#dispatch[finalType];
      if (handlers === undefined) assertNode(node);
    }
    const leave = handlers.leave ?? this.#globalLeave;
    if (leave !== null) {
      this.#prepare(node, parent, key, list, index, after);
      this.invoke(leave, frame, "leave");
      index = frame.index;
    }
    return index;
  }
}

// Reuse cleared traversal state across calls. Nested walks acquire a different
// context; the pool retains no nodes, buffers, handlers or caller state.
const contexts = [];

function runWalk(root, compiled, state) {
  let ctx = contexts.pop();
  if (ctx === undefined) ctx = new WalkContext(state, compiled);
  else ctx.initialize(state, compiled);
  try {
    ctx.visit(root, null, null, null, 0);
  } finally {
    ctx.reset();
    if (contexts.length < 4) contexts.push(ctx);
  }
  return root;
}

/**
 * Traverse and edit an existing ESTree / TS-ESTree tree synchronously in place.
 * @type {typeof import("./index.js").walk}
 */
export function walk(root, hooks, state) {
  assertNode(root);
  return runWalk(root, compile(hooks), state);
}

/**
 * Validate and snapshot hooks once for repeated synchronous tree traversal.
 * @type {typeof import("./index.js").createWalker}
 */
export function createWalker(hooks) {
  const compiled = compile(hooks);
  return Object.freeze({
    walk(root, state) {
      assertNode(root);
      return runWalk(root, compiled, state);
    },
  });
}
