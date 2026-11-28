import { childVisitors, nodeWalkers, dispatchNode } from "./child-visitors.js";
import type {
  ChildVisitor,
  ChildWalker,
  DispatchNode,
  NodeList,
  WalkerTable,
} from "./traversal-types.js";
import { visitorKeys, type NodeType } from "./visitor-keys.js";

export { visitorKeys };
export type { NodeType };

/** Structural ESTree node. Supply a node union to WalkHooks for field-level typing. */
export interface AstNode {
  type: NodeType;
  start?: number;
  end?: number;
  [field: string]: any;
}

/**
 * Current traversal position and editing methods. Reused within each walk;
 * do not retain it beyond the receiving synchronous hook or detach its methods.
 * Use context methods for structural edits; assign scalar properties directly.
 */
export interface WalkContext<State = unknown> {
  /** Current node, updated by `replace`; the hook's node argument is unchanged. */
  readonly node: AstNode;
  /** Parent node, or `null` at the root. */
  readonly parent: AstNode | null;
  /** Child field on the parent, or `null` at the root. */
  readonly key: string | null;
  /** Array index, or `null` at the root, in scalar fields, or after removal. */
  readonly index: number | null;
  /** State passed to this walk without cloning; `undefined` when omitted. */
  readonly state: State;
  /** A new snapshot ordered from root to parent, excluding the current node. */
  ancestors(): AstNode[];
  /** Enter only: skip children and remaining enter hooks, but still run leave hooks. */
  skip(): void;
  /** End the entire walk after this hook, without pending leave hooks. */
  stop(): void;
  /**
   * Replace a non-root node and end remaining hooks in the current phase.
   * During enter, visit the replacement's children and leave hooks, without
   * re-entering it. During leave, do not traverse it. Missing `start` / `end`
   * inherit the old values. Passing the current node is a no-op.
   */
  replace(node: AstNode): void;
  /**
   * Remove the current array element, skipping its remaining hooks and children.
   * Root and scalar-field removal, and further structural edits, throw.
   */
  remove(): void;
  /** Insert array siblings in order, without visiting the inserted nodes. */
  insertBefore(nodes: AstNode | readonly AstNode[]): void;
  /** Insert array siblings in order; they are visited later unless the walk stops. */
  insertAfter(nodes: AstNode | readonly AstNode[]): void;
}

/** A synchronous callback that must return `undefined`; edit through the context. */
export type WalkHandler<Node extends AstNode = AstNode, State = unknown> = (
  node: Node,
  context: WalkContext<State>,
) => void;

/** A function runs on enter; an object can register enter and/or leave callbacks. */
export type WalkHook<Node extends AstNode = AstNode, State = unknown> =
  | WalkHandler<Node, State>
  | { enter?: WalkHandler<Node, State>; leave?: WalkHandler<Node, State> };

type NodeOfType<
  Node extends AstNode,
  Type extends NodeType,
> = Node extends unknown
  ? Type extends Node["type"]
    ? Node & { type: Type }
    : never
  : never;

/**
 * Node-type hooks, `Function` / `Class` aliases, and global `enter` / `leave` hooks.
 * Supply a discriminated `Node` union for field-level typing.
 */
export type WalkHooks<State = unknown, Node extends AstNode = AstNode> = {
  [Type in Node["type"]]?: WalkHook<NodeOfType<Node, Type>, State>;
} & {
  enter?: WalkHandler<Node, State>;
  leave?: WalkHandler<Node, State>;
} & {
  [Alias in keyof typeof aliases]?: WalkHook<
    NodeOfType<Node, (typeof aliases)[Alias][number]>,
    State
  >;
};

declare const mergedHooks: unique symbol;

/** A merged snapshot for {@link walk}, {@link createWalker}, or another {@link merge}. */
export interface MergedWalkHooks<State = unknown> {
  readonly [mergedHooks]: (state: State) => void;
}

/** A snapshot of hook functions, reusable across independent or nested walks. */
export interface CompiledWalker<State = unknown> {
  /**
   * Like {@link walk}, using captured hooks and an independent context per call.
   * @param root - An acyclic supported node or subtree.
   * @param state - Per-call value exposed as `ctx.state`.
   * @returns The original root, mutated in place.
   */
  walk<Root extends AstNode>(root: Root, state?: State): Root;
}

type Handlers = WalkHandler | readonly WalkHandler[];
type DispatchEntry = { children: ChildVisitor | null; enter: Handlers | null; leave: Handlers | null };
type CompiledHooks = {
  dispatch: Record<string, DispatchEntry>;
  walkers: WalkerTable;
  globalEnter: Handlers | null;
  globalLeave: Handlers | null;
};
type Frame = {
  node: AstNode | null;
  parent: AstNode | null;
  key: string | null;
  list: NodeList | null;
  index: number;
  after: number;
  flags: number;
};

// Internal dispatch entries share generated child accessors. Public visitorKeys
// remain frozen; the hot path does not iterate those frozen arrays.
const baseDispatch: Record<string, DispatchEntry> = Object.create(null);
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
} as const satisfies Record<string, readonly NodeType[]>;
const SKIP = 1;
const REMOVED = 2;
const REPLACED = 4;
const owns = (object: object, key: PropertyKey) => Object.prototype.hasOwnProperty.call(object, key);
// Every supported type has an entry; the table has no inherited properties.
const isNodeType = (type: string) => baseDispatch[type] !== undefined;

function assertNode(value: unknown): asserts value is AstNode {
  const node = value as AstNode | null | undefined;
  if (!node || typeof node !== "object" || !isNodeType(node.type)) {
    throw new TypeError(`Unsupported AST node type: ${node?.type}`);
  }
}

function assertSync(result: unknown) {
  if (result != null && typeof (result as PromiseLike<unknown>).then === "function") {
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

function validHandlers(value: unknown): value is Handlers {
  if (typeof value === "function") return true;
  if (!Array.isArray(value)) return false;
  for (let index = 0; index < value.length; index++) {
    if (!owns(value, index) || typeof value[index] !== "function") return false;
  }
  return true;
}

function append(handlers: WalkHandler[], value: Handlers | undefined, prepend = false) {
  if (value === undefined) return;
  if (typeof value !== "function" && Array.isArray(value)) {
    if (prepend) handlers.unshift(...value);
    else handlers.push(...value);
  } else if (prepend) handlers.unshift(value as WalkHandler);
  else handlers.push(value as WalkHandler);
}

function compact(handlers: readonly WalkHandler[]): Handlers | null {
  return handlers.length === 0 ? null : handlers.length === 1 ? handlers[0] : handlers;
}

function compile(hooks: unknown): CompiledHooks {
  if (!hooks || typeof hooks !== "object")
    throw new TypeError("Expected a hooks object");
  const entries: [string, unknown][] = Object.entries(hooks);
  const table = new Map<string, { enter: WalkHandler[]; leave: WalkHandler[] }>();
  let enter: Handlers | undefined, leave: Handlers | undefined;
  const add = (name: string, value: unknown) => {
    if (typeof value === "function") {
      const entry = table.get(name);
      if (entry === undefined) table.set(name, { enter: [value as WalkHandler], leave: [] });
      else entry.enter.push(value as WalkHandler);
      return;
    }
    if (!value || typeof value !== "object")
      throw new TypeError(`Invalid hook: ${name}`);
    const pair = value as Record<string, Handlers | undefined>;
    for (const key of Object.keys(pair)) {
      if (
        (key !== "enter" && key !== "leave") ||
        (pair[key] !== undefined && !validHandlers(pair[key]))
      ) {
        throw new TypeError(`Invalid hook: ${name}.${key}`);
      }
    }
    const entry = table.get(name) ?? { enter: [], leave: [] };
    append(entry.enter, pair.enter);
    append(entry.leave, pair.leave, true);
    table.set(name, entry);
  };
  for (const [name, value] of entries) {
    if (value === undefined) continue;
    if (name === "enter" || name === "leave") {
      if (!validHandlers(value))
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
      for (const type of aliases[name as keyof typeof aliases]) add(type, value);
    }
  }
  for (const [name, value] of entries) {
    if (value !== undefined && isNodeType(name)) add(name, value);
  }
  for (const pair of table.values()) {
    append(pair.enter, enter, true);
    append(pair.leave, leave);
  }
  const globalEnter = enter !== undefined && typeof enter !== "function"
    ? compact(enter.slice())
    : enter ?? null;
  const globalLeave = leave !== undefined && typeof leave !== "function"
    ? compact(leave.slice())
    : leave ?? null;
  const dispatch: CompiledHooks["dispatch"] =
    table.size === 0 ? baseDispatch : Object.create(baseDispatch);
  for (const [type, pair] of table) {
    dispatch[type] = {
      children: childVisitors[type as NodeType],
      enter: compact(pair.enter),
      leave: compact(pair.leave),
    };
  }
  const walkers: WalkerTable = Object.create(nodeWalkers);
  const types = globalEnter || globalLeave ? Object.keys(nodeWalkers) : [...table.keys()];
  for (const type of types) {
    const entry = dispatch[type];
    const enterHook = entry.enter ?? globalEnter;
    const leaveHook = entry.leave ?? globalLeave;
    if (enterHook === null && leaveHook === null) continue;
    walkers[type] = enterHook === null ? walkLeave : walkEnter;
  }
  return {
    dispatch,
    walkers,
    globalEnter,
    globalLeave,
  };
}

/**
 * Merge visitors into a frozen snapshot.
 * Pass the result to {@link walk} or {@link createWalker} for one traversal.
 * At each node, callbacks run in visitor array order, preserving each visitor's
 * global/alias/concrete hook order. Traversal controls are shared as documented
 * on WalkContext. Later changes to the input visitors do not affect the result.
 */
export function merge<State = unknown, Node extends AstNode = AstNode>(
  visitors: readonly (WalkHooks<State, Node> | MergedWalkHooks<State>)[],
): MergedWalkHooks<State> {
  if (!Array.isArray(visitors)) throw new TypeError("Expected an array of visitors");
  const compiled = visitors.map(compile);
  const hooks: Record<string, { enter: readonly WalkHandler[]; leave: readonly WalkHandler[] }> =
    Object.create(null);
  const append = (handlers: WalkHandler[], hook: Handlers | null) => {
    if (Array.isArray(hook)) handlers.push(...hook);
    else if (hook !== null) handlers.push(hook as WalkHandler);
  };
  const hasGlobalHooks = compiled.some(
    (visitor) => visitor.globalEnter !== null || visitor.globalLeave !== null,
  );
  let types: string[];
  if (hasGlobalHooks) {
    types = Object.keys(baseDispatch);
  } else {
    const sparseTypes = new Set<string>();
    for (const { dispatch } of compiled) {
      if (dispatch !== baseDispatch) {
        for (const type of Object.keys(dispatch)) sparseTypes.add(type);
      }
    }
    types = [...sparseTypes].sort();
  }
  for (const type of types) {
    const enter: WalkHandler[] = [];
    const leave: WalkHandler[] = [];
    for (const visitor of compiled) {
      const entry = visitor.dispatch[type];
      append(enter, entry.enter ?? visitor.globalEnter);
      append(leave, entry.leave ?? visitor.globalLeave);
    }
    if (enter.length === 0 && leave.length === 0) continue;
    hooks[type] = Object.freeze({
      enter: Object.freeze(enter),
      leave: Object.freeze(leave),
    });
  }
  return Object.freeze(hooks) as unknown as MergedWalkHooks<State>;
}

function walkEnter(node: AstNode, walker: ChildWalker, parent: AstNode | null, key: string | null, list: NodeList | null, index: number) {
  return walker.visitHooks(node, parent, key, list, index);
}
function walkLeave(node: AstNode, walker: ChildWalker, parent: AstNode | null, key: string | null, list: NodeList | null, index: number) {
  return walker.visitLeaveHooks(node, parent, key, list, index);
}

// One context per walk, with prototype methods instead of per-hook closures.
// A retained reference aliases later hooks; callers must not store it.
class WalkerContext implements WalkContext, ChildWalker {
  #frame: Frame | undefined;
  #phase: "enter" | "leave" | undefined;
  #state: unknown;
  #ancestors: AstNode[] = [];
  // Reuse depth slots instead of shrinking/growing the array for every node.
  // Only the live prefix is exposed; reset releases retained references.
  #depth = 0;
  #dispatch: CompiledHooks["dispatch"] | undefined;
  #walkers: WalkerTable | undefined;
  #globalEnter: CompiledHooks["globalEnter"] | undefined;
  #globalLeave: CompiledHooks["globalLeave"] | undefined;
  #storage: Frame = {
    node: null,
    parent: null,
    key: null,
    list: null,
    index: 0,
    after: 0,
    flags: 0,
  };
  stopped = false;

  constructor(state: unknown, compiled: CompiledHooks) {
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
    if (frame.list && frame.parent[frame.key!] !== frame.list) {
      throw new Error(
        "The current node's container was changed outside the walk context",
      );
    }
    const current = frame.list
      ? frame.list[frame.index]
      : frame.parent[frame.key!];
    if (current !== frame.node)
      throw new Error(
        "The current node's container was changed outside the walk context",
      );
    return frame as Frame & { node: AstNode; parent: AstNode; key: string };
  }

  get node() {
    return this.#check().node!;
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
    this.#frame!.flags |= SKIP;
  }
  stop() {
    this.#check();
    this.stopped = true;
  }
  replace(node: AstNode) {
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
  #insert(nodes: AstNode | readonly AstNode[], after: boolean) {
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
  insertBefore(nodes: AstNode | readonly AstNode[]) {
    this.#insert(nodes, false);
  }
  insertAfter(nodes: AstNode | readonly AstNode[]) {
    this.#insert(nodes, true);
  }

  invoke(handlers: Handlers, frame: Frame, phase: "enter" | "leave") {
    this.#frame = frame;
    this.#phase = phase;
    if (typeof handlers === "function") {
      assertSync(handlers(frame.node!, this));
    } else {
      for (let i = 0; i < handlers.length; i++) {
        const handler = handlers[i];
        assertSync(handler(frame.node!, this));
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

  #prepare(node: AstNode, parent: AstNode | null, key: string | null, list: NodeList | null, index: number, after: number) {
    const frame = this.#storage;
    frame.node = node;
    frame.parent = parent;
    frame.key = key;
    frame.list = list;
    frame.index = index;
    frame.after = after;
    frame.flags = 0;
  }

  visitScalar(value: AstNode, parent: AstNode, key: string) {
    this.#ancestors[this.#depth++] = parent;
    this.visit(value, parent, key, null, 0);
    this.#depth--;
    return this.stopped;
  }

  visitListKnown(value: NodeList, parent: AstNode, key: string, dispatch: DispatchNode) {
    this.#ancestors[this.#depth++] = parent;
    for (let i = 0; i < value.length; i++) {
      const child = value[i];
      if (child != null) {
        if (typeof child !== "object") assertNode(child);
        i = dispatch(child, this, parent, key, value, i, this.#walkers!);
      }
      if (this.stopped) return true;
    }
    this.#depth--;
    return this.stopped;
  }

  visitList(value: NodeList, parent: AstNode, key: string) {
    this.#ancestors[this.#depth++] = parent;
    for (let i = 0; i < value.length; i++) {
      const child = value[i];
      if (child != null) i = this.visit(child, parent, key, value, i);
      if (this.stopped) return true;
    }
    this.#depth--;
    return this.stopped;
  }

  visit(node: AstNode, parent: AstNode | null, key: string | null, list: NodeList | null, index: number): number {
    if (typeof node !== "object") assertNode(node);
    return dispatchNode(node, this, parent, key, list, index, this.#walkers!);
  }

  leaveChanged(node: AstNode, parent: AstNode | null, key: string | null, list: NodeList | null, index: number): number {
    const entry = this.#dispatch![node.type];
    if (entry === undefined) assertNode(node);
    const leave = entry.leave ?? (this.#globalLeave as Handlers | null);
    if (leave === null) return index;
    this.#prepare(node, parent, key, list, index, 0);
    this.invoke(leave, this.#storage, "leave");
    return this.#storage.index;
  }

  visitLeaveHooks(node: AstNode, parent: AstNode | null, key: string | null, list: NodeList | null, index: number): number {
    const handlers = this.#dispatch![node.type];
    const type = node.type;
    if (handlers.children !== null) {
      handlers.children(node, this);
      if (this.stopped) return index;
    }
    if (node.type !== type)
      return this.leaveChanged(node, parent, key, list, index);
    this.#prepare(node, parent, key, list, index, 0);
    this.invoke((handlers.leave ?? this.#globalLeave)!, this.#storage, "leave");
    return this.#storage.index;
  }

  visitHooks(node: AstNode, parent: AstNode | null, key: string | null, list: NodeList | null, index: number): number {
    let handlers = this.#dispatch![node.type];
    const enter = handlers.enter ?? (this.#globalEnter as Handlers | null);
    const frame = this.#storage;
    let type = node.type;
    let after = 0;
    let skip: boolean | number = false;
    if (enter !== null) {
      this.#prepare(node, parent, key, list, index, 0);
      this.invoke(enter, frame, "enter");
      index = frame.index;
      if (this.stopped || frame.flags & REMOVED) return index;
      node = frame.node!;
      after = frame.after;
      skip = frame.flags & SKIP;
      if (node.type !== type) {
        type = node.type;
        handlers = this.#dispatch![type];
        if (handlers === undefined) assertNode(node);
      }
    }
    if (!skip && handlers.children !== null) {
      handlers.children(node, this);
      if (this.stopped) return index;
    }
    if (node.type !== type) {
      handlers = this.#dispatch![node.type];
      if (handlers === undefined) assertNode(node);
    }
    const leave = handlers.leave ?? (this.#globalLeave as Handlers | null);
    if (leave !== null) {
      this.#prepare(node, parent, key, list, index, after);
      this.invoke(leave, frame, "leave");
      index = frame.index;
    }
    return index;
  }
}

function runWalk<Root extends AstNode>(root: Root, compiled: CompiledHooks, state: unknown): Root {
  const ctx = new WalkerContext(state, compiled);
  try {
    ctx.visit(root, null, null, null, 0);
  } finally {
    ctx.reset();
  }
  return root;
}

/**
 * Traverse and edit an existing ESTree / TS-ESTree tree synchronously in place.
 * Hooks must return `undefined`. Use {@link WalkContext} for structural edits;
 * the root cannot be replaced or removed. Child traversal follows {@link visitorKeys}.
 * Use {@link createWalker} to reuse compiled hooks across trees.
 *
 * @param root - An acyclic supported node or subtree.
 * @param hooks - Node-type, alias, and/or global callbacks, or a merged snapshot.
 * @param state - Optional value exposed as `ctx.state`.
 * @returns The original root, mutated in place.
 * @throws {TypeError} For invalid hooks, unsupported nodes, or non-undefined returns.
 * @throws {Error} For invalid context use or structural edits.
 *
 * @example
 * ```ts
 * import { parseSync, walk } from "@swc-next/core";
 *
 * const { program } = parseSync("debugger; run();");
 * walk(program, {
 *   DebuggerStatement(node, ctx) { ctx.remove(); },
 * });
 * ```
 */
export function walk<Root extends AstNode, State = unknown, Node extends AstNode = AstNode>(
  root: Root,
  hooks: WalkHooks<State, Node> | MergedWalkHooks<State>,
  state?: State,
): Root {
  assertNode(root);
  return runWalk(root, compile(hooks), state);
}

/**
 * Validate and snapshot hooks once for repeated synchronous tree traversal.
 * Later hook changes do not affect the snapshot; callback closures remain shared.
 * Use {@link merge} to combine multiple visitors before creating a walker.
 *
 * @param hooks - Callbacks with the same rules as {@link walk}.
 * @returns A frozen walker with a `walk(root, state?)` method.
 * @throws {TypeError} For invalid hook names or callback definitions.
 *
 * @example
 * ```ts
 * import { createWalker, parseSync } from "@swc-next/core";
 *
 * const collectNames = createWalker<string[]>({
 *   Identifier(node, ctx) { ctx.state.push(node.name); },
 * });
 * for (const source of ["first();", "second();"]) {
 *   const { program } = parseSync(source);
 *   const names: string[] = [];
 *   collectNames.walk(program, names);
 * }
 * ```
 */
export function createWalker<State = unknown, Node extends AstNode = AstNode>(
  hooks: WalkHooks<State, Node> | MergedWalkHooks<State>,
): CompiledWalker<State> {
  const compiled = compile(hooks);
  return Object.freeze({
    walk<Root extends AstNode>(root: Root, state?: State): Root {
      assertNode(root);
      return runWalk(root, compiled, state);
    },
  });
}
