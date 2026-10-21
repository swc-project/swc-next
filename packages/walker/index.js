import { childVisitors, nodeWalkers, dispatchNode } from "./child-visitors.js";
import { visitorKeys } from "./visitor-keys.js";

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

// One context per walk, with prototype methods instead of per-hook closures.
// A retained reference aliases later hooks; callers must not store it.
class WalkContext {
  #frame;
  #phase;
  #state;
  #ancestors = [];
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
    this.#state = state;
    this.#dispatch = compiled.dispatch;
    this.#walkers = compiled.walkers;
    this.#globalEnter = compiled.globalEnter;
    this.#globalLeave = compiled.globalLeave;
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
    if (frame.list && frame.parent[frame.key] !== frame.list) {
      throw new Error(
        "The current node's container was changed outside the walk context",
      );
    }
    const current = frame.list
      ? frame.list[frame.index]
      : frame.parent[frame.key];
    if (current !== frame.node)
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
    if (node === frame.node) return;
    // Only the replacement root inherits missing positions.
    if (node.start == null) node.start = frame.node.start;
    if (node.end == null) node.end = frame.node.end;
    if (frame.list) frame.list[frame.index] = node;
    else frame.parent[frame.key] = node;
    frame.node = node;
    frame.flags |= REPLACED;
  }
  remove() {
    const frame = this.#editable();
    if (!frame.list) throw new Error("Removal requires an array element");
    frame.list.splice(frame.index, 1);
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
    this.#ancestors.length = 0;
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
    const handlers = this.#dispatch[node.type];
    const type = node.type;
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
    let handlers = this.#dispatch[node.type];
    const enter = handlers.enter ?? this.#globalEnter;
    const frame = this.#storage;
    let type = node.type;
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
      if (node.type !== type) {
        type = node.type;
        handlers = this.#dispatch[type];
        if (handlers === undefined) assertNode(node);
      }
    }
    if (!skip && handlers.children !== null) {
      handlers.children(node, this);
      if (this.stopped) return index;
    }
    if (node.type !== type) {
      handlers = this.#dispatch[node.type];
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

function runWalk(root, compiled, state) {
  const ctx = new WalkContext(state, compiled);
  try {
    ctx.visit(root, null, null, null, 0);
  } finally {
    ctx.reset();
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
