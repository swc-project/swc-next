import { visitorKeys } from "./visitor-keys.js";

export { visitorKeys };

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
const owns = (object, key) => Object.prototype.hasOwnProperty.call(object, key);

function assertNode(node) {
  if (!node || typeof node !== "object" || !owns(visitorKeys, node.type)) {
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
  for (const [name, value] of Object.entries(hooks)) {
    if (value === undefined) continue;
    if (name === "enter" || name === "leave") {
      if (typeof value !== "function")
        throw new TypeError(`Invalid hook: ${name}`);
      if (name === "enter") enter = value;
      else leave = value;
    } else if (!owns(aliases, name) && !owns(visitorKeys, name)) {
      throw new TypeError(`Unknown AST hook: ${name}`);
    }
  }
  // Aliases surround concrete hooks, independent of object property order.
  for (const [name, value] of Object.entries(hooks)) {
    if (value !== undefined && owns(aliases, name)) {
      for (const type of aliases[name]) add(type, value);
    }
  }
  for (const [name, value] of Object.entries(hooks)) {
    if (value !== undefined && owns(visitorKeys, name)) add(name, value);
  }
  return { table, enter, leave };
}

// One context per walk, with prototype methods instead of per-hook closures.
// A retained reference aliases later hooks; callers must not store it.
class WalkContext {
  #frame;
  #phase;
  #state;
  #active = false;
  stopped = false;

  constructor(state) {
    this.#state = state;
  }

  #check() {
    if (!this.#active)
      throw new Error("Walk context is only valid during its hook");
  }

  #editable() {
    this.#check();
    const frame = this.#frame;
    if (frame.removed) throw new Error("The current node has been removed");
    if (!frame.parent)
      throw new Error("Cannot structurally edit the walk root");
    if (frame.list && frame.parent.node[frame.key] !== frame.list) {
      throw new Error(
        "The current node's container was changed outside the walk context",
      );
    }
    const current = frame.list
      ? frame.list[frame.index]
      : frame.parent.node[frame.key];
    if (current !== frame.node)
      throw new Error(
        "The current node's container was changed outside the walk context",
      );
    return frame;
  }

  get node() {
    this.#check();
    return this.#frame.node;
  }
  get parent() {
    this.#check();
    return this.#frame.parent?.node ?? null;
  }
  get key() {
    this.#check();
    return this.#frame.key;
  }
  get index() {
    this.#check();
    const frame = this.#frame;
    return frame.removed || !frame.list ? null : frame.index;
  }
  get state() {
    this.#check();
    return this.#state;
  }
  ancestors() {
    this.#check();
    const result = [];
    for (let p = this.#frame.parent; p; p = p.parent) result.push(p.node);
    return result.reverse();
  }
  skip() {
    this.#check();
    if (this.#phase !== "enter")
      throw new Error("skip() is only valid in enter hooks");
    this.#frame.skip = true;
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
    else frame.parent.node[frame.key] = node;
    frame.node = node;
    frame.replaced = true;
  }
  remove() {
    const frame = this.#editable();
    if (!frame.list) throw new Error("Removal requires an array element");
    frame.list.splice(frame.index, 1);
    frame.index--;
    frame.removed = true;
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

  invoke(handler, frame, phase) {
    this.#frame = frame;
    this.#phase = phase;
    this.#active = true;
    try {
      assertSync(handler(frame.node, this));
    } finally {
      this.#active = false;
      // A saved context must not retain the AST after the callback.
      this.#frame = undefined;
    }
  }
}

/** Walk a supported ESTree/TS-ESTree subtree in place and return its root. */
export function walk(root, hooks, state) {
  assertNode(root);
  const dispatch = compile(hooks);
  const ctx = new WalkContext(state);
  // Siblings reuse the same depth slot. Active ancestors occupy earlier slots,
  // so edits and ancestor snapshots stay valid without per-node allocations.
  const frames = [];

  function visit(node, parent, key, list, index, depth) {
    assertNode(node);
    let frame = frames[depth];
    if (!frame) {
      frame = frames[depth] = {
        node: null,
        parent: null,
        key: null,
        list: null,
        index: 0,
        after: 0,
        skip: false,
        removed: false,
        replaced: false,
      };
    }
    frame.node = node;
    frame.parent = parent;
    frame.key = key;
    frame.list = list;
    frame.index = index;
    frame.after = 0;
    frame.skip = false;
    frame.removed = false;
    frame.replaced = false;

    if (dispatch.enter) ctx.invoke(dispatch.enter, frame, "enter");
    if (!ctx.stopped && !frame.removed && !frame.replaced && !frame.skip) {
      const handlers = dispatch.table.get(frame.node.type);
      if (handlers) {
        for (const handler of handlers.enter) {
          ctx.invoke(handler, frame, "enter");
          if (ctx.stopped || frame.removed || frame.replaced || frame.skip) break;
        }
      }
    }
    if (ctx.stopped || frame.removed) return frame.index;
    if (!frame.skip) {
      for (const childKey of visitorKeys[frame.node.type]) {
        const value = frame.node[childKey];
        if (Array.isArray(value)) {
          for (let i = 0; i < value.length; i++) {
            const child = value[i];
            if (child != null)
              i = visit(child, frame, childKey, value, i, depth + 1);
            if (ctx.stopped) return frame.index;
          }
        } else if (value != null) {
          visit(value, frame, childKey, null, 0, depth + 1);
          if (ctx.stopped) return frame.index;
        }
      }
    }
    frame.replaced = false;
    const handlers = dispatch.table.get(frame.node.type);
    if (handlers) {
      for (const handler of handlers.leave) {
        ctx.invoke(handler, frame, "leave");
        if (ctx.stopped || frame.removed || frame.replaced) return frame.index;
      }
    }
    if (dispatch.leave) ctx.invoke(dispatch.leave, frame, "leave");
    return frame.index;
  }

  visit(root, null, null, null, 0, 0);
  return root;
}
