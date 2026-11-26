import { NodeKind, NodeView } from "../protocol/index.ts";

const baseEntries = Object.values(NodeKind).map(() => ({ enter: null, leave: null }));

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
export function compile(hooks) {
  if (!hooks || typeof hooks !== "object") { throw new TypeError("Expected a hooks object"); }
  const entries = Object.entries(hooks);
  const table = new Map();
  let enter, leave;
  const add = (kind, name, value) => {
    const pair = typeof value === "function" ? { enter: value } : value;
    if (!pair || typeof pair !== "object") { throw new TypeError(`Invalid hook: ${name}`); }
    for (const key of Object.keys(pair)) {
      if ((key !== "enter" && key !== "leave") ||
          (pair[key] !== undefined && typeof pair[key] !== "function")) {
        throw new TypeError(`Invalid hook: ${name}.${key}`);
      }
    }
    const entry = table.get(kind) ?? { enter: [], leave: [] };
    if (pair.enter) { entry.enter.push(pair.enter); }
    if (pair.leave) { entry.leave.unshift(pair.leave); }
    table.set(kind, entry);
  };
  for (const [name, value] of entries) {
    if (value === undefined) { continue; }
    if (name === "enter" || name === "leave") {
      if (typeof value !== "function") { throw new TypeError(`Invalid hook: ${name}`); }
      if (name === "enter") { enter = value; }
      else { leave = value; }
    } else {
      const kind = NodeKind[name];
      if (typeof kind !== "number") { throw new TypeError(`Unknown AST hook: ${name}`); }
      add(kind, name, value);
    }
  }
  for (const pair of table.values()) {
    if (enter) { pair.enter.unshift(enter); }
    if (leave) { pair.leave.push(leave); }
  }
  const compact = handlers => handlers.length === 0
    ? null : handlers.length === 1 ? handlers[0] : handlers;
  const compiled = table.size === 0 ? baseEntries : baseEntries.slice();
  for (const [kind, pair] of table) {
    compiled[kind] = { enter: compact(pair.enter), leave: compact(pair.leave) };
  }
  return { entries: compiled, globalEnter: enter ?? null, globalLeave: leave ?? null };
}
