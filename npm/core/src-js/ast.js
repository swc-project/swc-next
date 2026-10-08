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

/** Walk a supported ESTree/TS-ESTree subtree in place and return its root. */
export function walk(root, hooks, state) {
  assertNode(root);
  const dispatch = compile(hooks);
  let stopped = false;

  function invoke(handler, frame, phase) {
    let active = true;
    const check = () => {
      if (!active)
        throw new Error("Walk context is only valid during its hook");
    };
    const editable = () => {
      check();
      if (frame.removed) throw new Error("The current node has been removed");
      if (!frame.parent)
        throw new Error("Cannot structurally edit the walk root");
      if (frame.list && frame.parent.node[frame.key] !== frame.list) {
        throw new Error(
          "The current node's container was changed outside the walk context",
        );
      }
      const current = frame.list
        ? frame.list[frame.cursor.index]
        : frame.parent.node[frame.key];
      if (current !== frame.node)
        throw new Error(
          "The current node's container was changed outside the walk context",
        );
    };
    const insert = (nodes, after) => {
      editable();
      if (!frame.list) throw new Error("Insertion requires an array element");
      nodes = Array.isArray(nodes) ? nodes : [nodes];
      nodes.forEach(assertNode);
      const index = frame.cursor.index + (after ? 1 + frame.after : 0);
      frame.list.splice(index, 0, ...nodes);
      if (after) frame.after += nodes.length;
      else frame.cursor.index += nodes.length;
    };
    const ctx = Object.freeze({
      get node() {
        check();
        return frame.node;
      },
      get parent() {
        check();
        return frame.parent?.node ?? null;
      },
      get key() {
        check();
        return frame.key;
      },
      get index() {
        check();
        return frame.removed ? null : (frame.cursor?.index ?? null);
      },
      get state() {
        check();
        return state;
      },
      ancestors() {
        check();
        const result = [];
        for (let p = frame.parent; p; p = p.parent) result.push(p.node);
        return result.reverse();
      },
      skip() {
        check();
        if (phase !== "enter")
          throw new Error("skip() is only valid in enter hooks");
        frame.skip = true;
      },
      stop() {
        check();
        stopped = true;
      },
      replace(node) {
        editable();
        assertNode(node);
        if (node === frame.node) return;
        // Only the replacement root inherits missing positions. No source-map
        // accuracy is implied for freshly constructed descendants.
        if (node.start == null) node.start = frame.node.start;
        if (node.end == null) node.end = frame.node.end;
        if (frame.list) frame.list[frame.cursor.index] = node;
        else frame.parent.node[frame.key] = node;
        frame.node = node;
        frame.replaced = true;
      },
      remove() {
        editable();
        // Do not silently null required scalar fields or repair parent grammar.
        if (!frame.list) throw new Error("Removal requires an array element");
        frame.list.splice(frame.cursor.index, 1);
        frame.cursor.index--;
        frame.removed = true;
      },
      insertBefore(nodes) {
        insert(nodes, false);
      },
      insertAfter(nodes) {
        insert(nodes, true);
      },
    });
    try {
      assertSync(handler(frame.node, ctx));
    } finally {
      active = false;
    }
  }

  function visit(node, parent, key, list, cursor) {
    assertNode(node);
    const frame = {
      node,
      parent,
      key,
      list,
      cursor,
      after: 0,
      skip: false,
      removed: false,
      replaced: false,
    };
    if (dispatch.enter) invoke(dispatch.enter, frame, "enter");
    if (!stopped && !frame.removed && !frame.replaced && !frame.skip) {
      for (const handler of dispatch.table.get(frame.node.type)?.enter ?? []) {
        invoke(handler, frame, "enter");
        if (stopped || frame.removed || frame.replaced || frame.skip) break;
      }
    }
    if (stopped || frame.removed) return;
    if (!frame.skip) {
      for (const childKey of visitorKeys[frame.node.type]) {
        const value = frame.node[childKey];
        if (Array.isArray(value)) {
          const position = { index: 0 };
          for (; position.index < value.length; position.index++) {
            const child = value[position.index];
            if (child != null) visit(child, frame, childKey, value, position);
            if (stopped) return;
          }
        } else if (value != null) {
          visit(value, frame, childKey, null, null);
          if (stopped) return;
        }
      }
    }
    frame.replaced = false;
    for (const handler of dispatch.table.get(frame.node.type)?.leave ?? []) {
      invoke(handler, frame, "leave");
      if (stopped || frame.removed || frame.replaced) return;
    }
    if (dispatch.leave) invoke(dispatch.leave, frame, "leave");
  }

  visit(root, null, null, null, null);
  return root;
}
