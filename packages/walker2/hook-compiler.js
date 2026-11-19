import { NodeView } from "@swc-next/ast2";
import { visitorKeys } from "./visitor-keys.js";

const aliases = {
  Function: [
    "FunctionDeclaration", "FunctionExpression", "ArrowFunctionExpression",
    "TSDeclareFunction", "TSEmptyBodyFunctionExpression",
  ],
  Class: ["ClassDeclaration", "ClassExpression"],
};
const baseDispatch = Object.create(null);
for (const type of Object.keys(visitorKeys)) {
  baseDispatch[type] = { enter: null, leave: null };
}
const owns = (object, key) => Object.prototype.hasOwnProperty.call(object, key);

export const isNodeType = (type) => baseDispatch[type] !== undefined;

export const SKIP = 1;
export const REMOVED = 2;
export const REPLACED = 4;

/** Accept structural edit inputs: views or `{ kind, ...fields }` descriptors. */
export function assertNode(node) {
  if (node instanceof NodeView) { return; }
  if (node !== null && typeof node === "object" && typeof node.kind === "number") { return; }
  throw new TypeError("Expected a node view or a node descriptor");
}

export function assertSync(result) {
  if (result != null && typeof result.then === "function") {
    // Observe an async callback's rejection before rejecting its return value.
    Promise.resolve(result).catch(() => {});
    throw new TypeError("AST hooks must be synchronous");
  }
  if (result !== undefined) {
    throw new TypeError("AST hooks must return undefined; use the context to edit nodes");
  }
}

/** Compile callback order once; each walker supplies its own traversal metadata. */
export function compile(hooks, base = baseDispatch) {
  if (!hooks || typeof hooks !== "object") { throw new TypeError("Expected a hooks object"); }
  const entries = Object.entries(hooks);
  const table = new Map();
  let enter, leave;
  const add = (name, value) => {
    const pair = typeof value === "function" ? { enter: value } : value;
    if (!pair || typeof pair !== "object") { throw new TypeError(`Invalid hook: ${name}`); }
    for (const key of Object.keys(pair)) {
      if ((key !== "enter" && key !== "leave") ||
          (pair[key] !== undefined && typeof pair[key] !== "function")) {
        throw new TypeError(`Invalid hook: ${name}.${key}`);
      }
    }
    const entry = table.get(name) ?? { enter: [], leave: [] };
    if (pair.enter) { entry.enter.push(pair.enter); }
    if (pair.leave) { entry.leave.unshift(pair.leave); }
    table.set(name, entry);
  };
  for (const [name, value] of entries) {
    if (value === undefined) { continue; }
    if (name === "enter" || name === "leave") {
      if (typeof value !== "function") { throw new TypeError(`Invalid hook: ${name}`); }
      if (name === "enter") { enter = value; }
      else { leave = value; }
    } else if (!owns(aliases, name) && !isNodeType(name)) {
      throw new TypeError(`Unknown AST hook: ${name}`);
    }
  }
  // Alias hooks surround concrete hooks, independent of property order.
  for (const [name, value] of entries) {
    if (value !== undefined && owns(aliases, name)) {
      for (const type of aliases[name]) { add(type, value); }
    }
  }
  for (const [name, value] of entries) {
    if (value !== undefined && isNodeType(name)) { add(name, value); }
  }
  for (const pair of table.values()) {
    if (enter) { pair.enter.unshift(enter); }
    if (leave) { pair.leave.push(leave); }
  }
  const compact = handlers => handlers.length === 0
    ? null : handlers.length === 1 ? handlers[0] : handlers;
  const dispatch = table.size === 0 ? base : Object.create(base);
  for (const [type, pair] of table) {
    dispatch[type] = { ...base[type], enter: compact(pair.enter), leave: compact(pair.leave) };
  }
  return { dispatch, types: [...table.keys()], globalEnter: enter ?? null, globalLeave: leave ?? null };
}
