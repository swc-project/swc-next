import { referenceReaders } from './references.js';

const NONE = 0xffffffff;

// Emit [reference-slot offset, native child ID] pairs into reusable storage.
// The generated descriptors include hidden native parameter/list wrappers.
function references(ast, raw, output) {
  if (raw >= ast.nodeCount) throw new RangeError('AST child out of bounds');
  const reader = referenceReaders[ast._view.getUint8(ast._kinds + raw)];
  if (reader === undefined) throw new RangeError('Unknown AST node kind');
  return reader(ast, ast._payloads + raw * 16, output);
}

/** Derive occurrence ownership lazily; never copy native payloads or strings. */
export function ensureTopology(ast, navigation = false) {
  if (ast._parents !== null) return;
  const count = ast.nodeCount;
  const parents = new Uint32Array(count).fill(NONE);
  const first = navigation ? new Uint32Array(count).fill(NONE) : null;
  const next = navigation ? new Uint32Array(count).fill(NONE) : null;
  // 0 = unseen, 1 = discovered/completed, 2 = on the current DFS path.
  const seen = new Uint8Array(count);
  const aliases = new Map();
  const extra = []; // [native ID, parent occurrence, first child, next sibling]
  const pending = [ast.rootId];
  const children = [];
  seen[ast.rootId] = 1;
  while (pending.length) {
    const token = pending.pop();
    const id = token < 0 ? ~token : token;
    const raw = id < count ? id : extra[id - count][0];
    if (token < 0) { seen[raw] = 1; continue; }
    if (seen[raw] === 2) throw new RangeError('Cyclic AST references');
    const childCount = references(ast, raw, children);
    // Leaves cannot participate in a cycle. They need neither an active marker
    // nor a second stack entry just to clear it on exit.
    if (childCount === 0) continue;
    seen[raw] = 2;
    pending.push(~id);
    let previous = NONE;
    for (let i = 0; i < childCount; i += 2) {
      const slot = children[i], child = children[i + 1];
      if (child >= count) throw new RangeError('AST child out of bounds');
      let occurrence = child;
      if (seen[child]) {
        occurrence = ast._nextNodeId++;
        if (occurrence >= 0x7fffffff) throw new RangeError('Too many AST occurrences');
        // Derived IDs and owners fit in signed 31 bits. Keep private sentinel
        // and flag values signed so these tuples retain integer elements.
        extra[occurrence - count] = [child, id, -1, -1];
        let edges = aliases.get(id);
        if (!edges) {
          aliases.set(id, edges = new Map());
          // Native IDs and derived occurrence IDs fit in 31 bits. The top bit
          // on this ownership entry marks the rare parents with aliased edges.
          if (id < count) parents[id] |= 0x80000000;
          else extra[id - count][1] |= 0x80000000;
        }
        edges.set(slot, occurrence);
      } else {
        seen[child] = 1;
        parents[child] = id;
      }
      if (navigation) {
        if (previous === NONE) {
          if (id < count) first[id] = occurrence;
          else extra[id - count][2] = occurrence;
        } else if (previous < count) next[previous] = occurrence;
        else extra[previous - count][3] = occurrence;
      }
      previous = occurrence;
      children[i + 1] = occurrence;
    }
    for (let i = childCount - 1; i >= 1; i -= 2) pending.push(children[i]);
  }
  ast._parents = parents;
  ast._firstChildren = first;
  ast._nextSiblings = next;
  ast._aliases = aliases;
  ast._extraNodes = extra;
  ast._onTopology();
}

/** Native navigation describes the immutable snapshot, independently of edits. */
export function ensureNavigation(ast) {
  if (ast._firstChildren !== null) return;
  if (ast._parents === null) return ensureTopology(ast, true);
  const count = ast.nodeCount;
  const extra = ast._extraNodes;
  // Include derived slots while building so a failed read cannot publish a
  // partial navigation chain through the existing occurrence tuples.
  const first = new Uint32Array(count + extra.length).fill(NONE);
  const next = new Uint32Array(count + extra.length).fill(NONE);
  const pending = [ast.rootId];
  const children = [];
  let remaining = count + extra.length;
  while (pending.length) {
    if (remaining-- === 0) throw new RangeError('Cyclic AST references');
    const id = pending.pop();
    const raw = id < count ? id : extra[id - count][0];
    const length = references(ast, raw, children);
    const aliases = ast._aliases.get(id);
    let previous = NONE;
    for (let i = 0; i < length; i += 2) {
      if (children[i + 1] >= count) throw new RangeError('AST child out of bounds');
      const child = aliases?.get(children[i]) ?? children[i + 1];
      if (previous === NONE) first[id] = child;
      else next[previous] = child;
      previous = child;
      pending.push(child);
    }
  }
  for (let i = 0; i < extra.length; i++) {
    if (!extra[i]) continue;
    extra[i][2] = first[count + i] | 0;
    extra[i][3] = next[count + i] | 0;
  }
  ast._firstChildren = first;
  ast._nextSiblings = next;
}
