import { childVisitors, nodeWalkers, dispatchNode, nativeChildVisitors } from "./child-visitors.js";
import { visitorKeys } from "./visitor-keys.js";
import { NodeView, AstList, sameList } from "@swc-next/ast";
import { compile as compileHooks, assertNode, assertSync, SKIP, REMOVED, REPLACED } from "./hook-compiler.js";

export { visitorKeys };

// Internal dispatch entries share generated child accessors. Public visitorKeys
// remain frozen; the hot path does not iterate those frozen arrays.
const baseDispatch = Object.create(null);
for (const [type, children] of Object.entries(childVisitors)) {
  baseDispatch[type] = { children, enter: null, leave: null };
}

function compile(hooks) {
  const { dispatch, types: hookedTypes, globalEnter, globalLeave } = compileHooks(hooks, baseDispatch);
  const walkers = Object.create(nodeWalkers);
  const types = globalEnter !== null || globalLeave !== null ? Object.keys(nodeWalkers) : hookedTypes;
  for (const type of types) {
    const entry = dispatch[type];
    const enterHook = entry.enter ?? globalEnter;
    const leaveHook = entry.leave ?? globalLeave;
    if (enterHook === null && leaveHook === null) { continue; }
    walkers[type] = enterHook === null ? walkLeave : walkEnter;
  }
  return {
    dispatch,
    walkers,
    globalEnter,
    globalLeave,
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
  reset(parent, key, ast = parent.ast) {
    this.ast = ast;
    this.parent = parent;
    this.key = key;
    this.array = null;
    // The range reader overwrites part zero. Unused owners and the previous
    // total are never observed before finish(), so only reset optional parts.
    this.start1 = this.end1 = 0;
    this.prefix = this.suffix = -1;
    this.version = this.ast._listVersion;
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
    if (this.array !== null || this.version === this.ast._listVersion) { return; }
    this.version = this.ast._listVersion;
    this.start1 = this.end1 = 0;
    this.prefix = this.suffix = -1;
    const owner = typeof this.parent === "number" ? this.parent : this.parent.nodeId;
    if (!this.ast.readListCursor(owner, this.key, this)) {
      throw new Error("The current node's container was changed outside the walk context");
    }
    this.finish();
  }
  get length() { return this.array === null ? this.total : this.array.length; }
  nodeId(index) {
    if (this.array !== null) { return this.array.get(index)?.nodeId ?? 0xffffffff; }
    if (this.prefix !== -1) {
      if (index === 0) { return this.prefix >>> 0; }
      index--;
    }
    const first = this.end0 - this.start0;
    if (index < first) { return this.ast.extraChild(this.owner0, this.start0 + index, true); }
    index -= first;
    if (index < this.end1 - this.start1) { return this.ast.extraChild(this.owner1, this.start1 + index, true); }
    return this.suffix === -1 ? 0xffffffff : this.suffix >>> 0;
  }
  at(index) {
    if (this.array !== null) { return this.array.get(index); }
    const ast = this.ast;
    if (this.prefix !== -1) {
      if (index === 0) { return ast._nodeAt(this.prefix >>> 0); }
      index--;
    }
    const first = this.end0 - this.start0;
    if (index < first) { return ast._nodeAt(ast.extraChild(this.owner0, this.start0 + index, true)); }
    index -= first;
    if (index < this.end1 - this.start1) { return ast._nodeAt(ast.extraChild(this.owner1, this.start1 + index, true)); }
    return this.suffix === -1 ? null : ast._nodeAt(this.suffix >>> 0);
  }
  materialize() {
    this.sync();
    if (this.array === null) {
      if (typeof this.parent === "number") { this.parent = this.ast._nodeAt(this.parent); }
      this.array = this.parent[this.key];
    }
    return this.array;
  }
  clear() { this.ast = this.parent = this.key = this.array = null; }
}

// One context per walk, with prototype methods instead of per-hook closures.
// A retained reference aliases later hooks; callers must not store it.
class WalkContext {
  #ast = null;
  #nativeReaders = null;
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

  constructor(state, compiled, ast) {
    this.initialize(state, compiled, ast);
  }

  initialize(state, compiled, ast) {
    this.#ast = ast;
    this.#nativeReaders = ast !== null && compiled.globalEnter === null && compiled.globalLeave === null
      ? nativeChildVisitors[Number(ast._isTs)] : null;
    this.#state = state;
    this.#dispatch = compiled.dispatch;
    this.#walkers = compiled.walkers;
    this.#globalEnter = compiled.globalEnter;
    this.#globalLeave = compiled.globalLeave;
    this.stopped = false;
  }

  #check() {
    const frame = this.#frame;
    if (frame === undefined) {
      throw new Error("Walk context is only valid during its hook");
    }
    return frame;
  }

  #editable() {
    const frame = this.#check();
    if (frame.flags & REMOVED) {
      throw new Error("The current node has been removed");
    }
    if (!frame.parent) {
      throw new Error("Cannot structurally edit the walk root");
    }
    if (frame.list instanceof ListCursor) { frame.list = frame.list.materialize(); }
    if (frame.list && !sameList(frame.parent[frame.key], frame.list)) {
      throw new Error(
        "The current node's container was changed outside the walk context",
      );
    }
    const current = frame.list
      ? frame.list instanceof AstList ? frame.list.get(frame.index) : frame.list[frame.index]
      : frame.parent[frame.key];
    if (!NodeView.isSame(current, frame.node)) {
      throw new Error(
        "The current node's container was changed outside the walk context",
      );
    }
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
    return this.#ancestors.slice(0, this.#depth).map(parent =>
      typeof parent === "number" ? this.#ast._nodeAt(parent) : parent);
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
  replace(node) {
    const frame = this.#editable();
    assertNode(node);
    if (NodeView.isSame(node, frame.node)) { return; }
    // Only the replacement root inherits missing positions.
    if (node.start == null) { node.start = frame.node.start; }
    if (node.end == null) { node.end = frame.node.end; }
    if (frame.list) {
      if (frame.list instanceof AstList) { frame.list.set(frame.index, node); }
      else { frame.list[frame.index] = node; }
      frame.node = frame.list instanceof AstList ? frame.list.get(frame.index) : frame.list[frame.index];
    } else {
      frame.parent[frame.key] = node;
      frame.node = frame.parent[frame.key];
    }
    frame.flags |= REPLACED;
  }
  remove() {
    const frame = this.#editable();
    if (!frame.list) { throw new Error("Removal requires an array element"); }
    frame.list.splice(frame.index, 1);
    frame.index--;
    frame.flags |= REMOVED;
  }
  #insert(nodes, after) {
    const frame = this.#editable();
    if (!frame.list) { throw new Error("Insertion requires an array element"); }
    const index = frame.index + (after ? 1 + frame.after : 0);
    let count;
    if (Array.isArray(nodes) || nodes instanceof AstList) {
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
      assertSync(handlers(frame.node, this));
    } else {
      for (let i = 0; i < handlers.length; i++) {
        const handler = handlers[i];
        assertSync(handler(frame.node, this));
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
    assertSync(this.#globalEnter(frame.node, this));
    this.#frame = undefined;
  }

  #invokeGlobalLeave(frame) {
    this.#frame = frame;
    this.#phase = "leave";
    assertSync(this.#globalLeave(frame.node, this));
    this.#frame = undefined;
  }

  reset() {
    this.#ast = this.#nativeReaders = null;
    this.#frame = undefined;
    this.#phase = undefined;
    this.#state = undefined;
    this.#ancestors.length = 0;
    for (const cursor of this.#listCursors) { cursor?.clear(); }
    if (this.#listCursors.length > 128) { this.#listCursors.length = 128; }
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
  visitScalarId(id, parent, key) {
    this.#ancestors[this.#depth++] = parent;
    this.visitId(id, parent, key, null, 0);
    this.#depth--;
    return this.stopped;
  }

  visitListField(parent, key, dispatch = null) {
    if (typeof parent !== "number" && !(parent instanceof NodeView)) {
      const value = parent[key];
      return value == null ? false : dispatch === null
        ? this.visitList(value, parent, key) : this.visitListKnown(value, parent, key, dispatch);
    }
    let cursor = this.#listCursors[this.#depth];
    if (!cursor) { this.#listCursors[this.#depth] = cursor = new ListCursor(); }
    const ast = typeof parent === "number" ? this.#ast : parent.ast;
    const owner = typeof parent === "number" ? parent : parent.nodeId;
    cursor.reset(parent, key, ast);
    if (!ast.readListCursor(owner, key, cursor)) {
      cursor.clear();
      if (typeof parent === "number") { parent = ast._nodeAt(parent); }
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
        if (this.#nativeReaders !== null) {
          const id = cursor.nodeId(i);
          if (id !== 0xffffffff) { i = this.visitId(id, parent, key, cursor, i); }
          if (this.stopped) { return true; }
          cursor.sync();
          continue;
        }
        const child = cursor.at(i);
        if (child != null) {
          if (dispatch === null) { i = this.visit(child, parent, key, cursor, i); }
          else {
            if (typeof child !== "object") { assertNode(child); }
            i = dispatch(child, this, parent, key, cursor, i, this.#walkers);
          }
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

  visitListKnown(value, parent, key, dispatch) {
    this.#ancestors[this.#depth++] = parent;
    for (let i = 0; i < value.length; i++) {
      const child = value instanceof AstList ? value.get(i) : value[i];
      if (child != null) {
        if (typeof child !== "object") { assertNode(child); }
        i = dispatch(child, this, parent, key, value, i, this.#walkers);
      }
      if (this.stopped) { return true; }
    }
    this.#depth--;
    return this.stopped;
  }

  visitList(value, parent, key) {
    this.#ancestors[this.#depth++] = parent;
    for (let i = 0; i < value.length; i++) {
      const child = value instanceof AstList ? value.get(i) : value[i];
      if (child != null) { i = this.visit(child, parent, key, value, i); }
      if (this.stopped) { return true; }
    }
    this.#depth--;
    return this.stopped;
  }

  visit(node, parent, key, list, index) {
    if (this.#nativeReaders !== null && node instanceof NodeView) {
      return this.visitId(node.nodeId, parent, key, list, index);
    }
    return this.visitNode(node, parent, key, list, index);
  }

  visitId(id, parent, key, list, index) {
    const ast = this.#ast, kind = ast._kindAt(id);
    const reader = this.#nativeReaders[kind];
    if (reader !== null) {
      const type = ast._typeAt(id, kind), entry = this.#dispatch[type];
      if (entry !== undefined && entry.enter === null && entry.leave === null) {
        reader(ast, id, this);
        if (!this.stopped && ast._typeAt(id) !== type) {
          const node = ast._nodeAt(id);
          return this.leaveChanged(node, typeof parent === "number" ? ast._nodeAt(parent) : parent, key, list, index);
        }
        return index;
      }
    }
    return this.visitNode(ast._nodeAt(id), typeof parent === "number" ? ast._nodeAt(parent) : parent, key, list, index);
  }

  visitNode(node, parent, key, list, index) {
    if (typeof node !== "object") { assertNode(node); }
    // A global enter hook uses the same entry path for every supported type.
    // Resolve its handlers once inside visitHooks instead of dispatching twice.
    if (this.#globalEnter !== null) {
      return this.visitHooks(node, parent, key, list, index);
    }
    return dispatchNode(node, this, parent, key, list, index, this.#walkers);
  }

  leaveChanged(node, parent, key, list, index) {
    const entry = this.#dispatch[node.type];
    if (entry === undefined) { assertNode(node); }
    const leave = entry.leave ?? this.#globalLeave;
    if (leave === null) { return index; }
    this.#prepare(node, parent, key, list, index, 0);
    if (leave === this.#globalLeave) {
      this.#invokeGlobalLeave(this.#storage);
    } else {
      this.invoke(leave, this.#storage, "leave");
    }
    return this.#storage.index;
  }

  visitLeaveHooks(node, parent, key, list, index) {
    const type = node.type;
    const handlers = this.#dispatch[type];
    if (handlers.children !== null) {
      handlers.children(node, this);
      if (this.stopped) { return index; }
    }
    if (node.type !== type) {
      return this.leaveChanged(node, parent, key, list, index);
    }
    const leave = handlers.leave ?? this.#globalLeave;
    this.#prepare(node, parent, key, list, index, 0);
    if (leave !== null && leave === this.#globalLeave) {
      this.#invokeGlobalLeave(this.#storage);
    } else {
      this.invoke(leave, this.#storage, "leave");
    }
    return this.#storage.index;
  }

  visitHooks(node, parent, key, list, index) {
    let type = node.type;
    let handlers = this.#dispatch[type];
    if (handlers === undefined) { assertNode(node); }
    const enter = handlers.enter ?? this.#globalEnter;
    const frame = this.#storage;
    let after = 0;
    let skip = false;
    if (enter !== null) {
      this.#prepare(node, parent, key, list, index, 0);
      if (enter === this.#globalEnter) {
        this.#invokeGlobalEnter(frame);
      } else {
        this.invoke(enter, frame, "enter");
      }
      index = frame.index;
      if (this.stopped || frame.flags & REMOVED) { return index; }
      node = frame.node;
      after = frame.after;
      skip = frame.flags & SKIP;
      const enteredType = node.type;
      if (enteredType !== type) {
        type = enteredType;
        handlers = this.#dispatch[type];
        if (handlers === undefined) { assertNode(node); }
      }
    }
    if (!skip && handlers.children !== null) {
      handlers.children(node, this);
      if (this.stopped) { return index; }
    }
    const finalType = node.type;
    if (finalType !== type) {
      handlers = this.#dispatch[finalType];
      if (handlers === undefined) { assertNode(node); }
    }
    const leave = handlers.leave ?? this.#globalLeave;
    if (leave !== null) {
      this.#prepare(node, parent, key, list, index, after);
      if (leave === this.#globalLeave) {
        this.#invokeGlobalLeave(frame);
      } else {
        this.invoke(leave, frame, "leave");
      }
      index = frame.index;
    }
    return index;
  }
}

// Reuse cleared traversal state across calls. Nested walks acquire a different
// context; the pool retains no nodes, buffers, handlers or caller state.
const contexts = [];

function runWalk(root, compiled, state) {
  const ast = root instanceof NodeView ? root.ast : null;
  let ctx = contexts.pop();
  if (ctx === undefined) { ctx = new WalkContext(state, compiled, ast); }
  else { ctx.initialize(state, compiled, ast); }
  try {
    ctx.visit(root, null, null, null, 0);
  } finally {
    ctx.reset();
    if (contexts.length < 4) { contexts.push(ctx); }
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
