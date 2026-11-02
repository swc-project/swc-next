import { DeltaArena } from "./arena.js";
import { layouts, SCHEMA_ID } from "./schema.js";
import { nativeExtraCounts } from "./references.js";
import { ensureTopology } from "./topology.js";

const NONE = 0xffffffff;

function baseRow(ast, id, raw) {
  const view = ast._view;
  const kind = view.getUint8(ast._kinds + raw);
  ast._arena.ensureRow(id, kind,
    view.getUint32(ast._nodes + raw * 8, true),
    view.getUint32(ast._nodes + raw * 8 + 4, true));
  ast._arena.copyPayload(id, view, ast._payloads + raw * 16);
}

export function ensureArena(ast) {
  if (ast._arena === null) {
    ast._arena = new DeltaArena({
      nodeCount: ast.nodeCount, extraCount: ast._extraCount,
      stringLength: ast._stringLength, byteLength: ast.buffer.byteLength,
      rootId: ast.rootId, flags: ast._view.getUint32(12, true), schemaId: SCHEMA_ID,
    });
    if (ast._parents !== null) syncAliases(ast);
  }
  return ast._arena;
}

function copyExtra(ast, start, count) {
  if (start + count > ast.extraDataCount) throw new RangeError("AST extra-data reference out of bounds");
  const destination = ast._arena.appendExtra(count);
  for (let i = 0; i < count; i++) {
    const lo = ast.readExtraWord(start + i, 0);
    const hi = ast.readExtraWord(start + i, 4);
    ast._arena.writeExtraWord(destination + i, 0, lo);
    ast._arena.writeExtraWord(destination + i, 4, hi);
  }
  return destination;
}

function writableDescriptor(ast, id, descriptor) {
  const arena = ast._arena;
  arena.ensureWritable();
  const kind = arena.kind(id), layout = layouts[kind];
  if (!descriptor[0]) return { view: arena.view, offset: arena.nodePayload(id) + descriptor[1] };
  let start = arena.view.getUint32(arena.nodePayload(id) + layout.extra, true);
  if (start < ast._extraCount) {
    start = copyExtra(ast, start, nativeExtraCounts[kind]);
    arena.setPayloadWord(id, layout.extra, start);
  }
  return { view: arena.view, offset: arena.extraAddress(start + descriptor[1]) };
}

/** Make implicit shared occurrences explicit before any original row is changed. */
export function syncAliases(ast) {
  if (ast._arena === null || ast._parents === null || ast._aliasesNormalized) return;
  const arena = ast._arena;
  // The same logical-ID allocator serves newly created nodes and occurrences.
  while (arena.nodeCount < ast._nextNodeId) {
    const id = arena.nodeCount;
    const entry = ast._extraNodes[id - ast.nodeCount];
    if (!entry) throw new RangeError("Missing AST occurrence");
    baseRow(ast, id, entry[0]);
  }
  for (const [id, aliases] of ast._aliases) {
    const raw = id < ast.nodeCount ? id : ast._extraNodes[id - ast.nodeCount][0];
    if (!arena.has(id)) baseRow(ast, id, raw);
    const kind = ast._view.getUint8(ast._kinds + raw);
    const layout = layouts[kind], payload = ast._payloads + raw * 16;
    for (const field of layout.children) {
      const source = field[0]
        ? ast._extra + (ast._view.getUint32(payload + layout.extra, true) + field[1]) * 8
        : payload + field[1];
      if (field[2] === "node") {
        const child = aliases.get(source);
        if (child === undefined) continue;
        const destination = writableDescriptor(ast, id, field);
        destination.view.setUint32(destination.offset, child, true);
      } else {
        const start = ast._view.getUint32(source, true), end = ast._view.getUint32(source + 4, true);
        let changed = false;
        for (let i = start; i < end && !changed; i++) changed = aliases.has(ast._extra + i * 8);
        if (!changed) continue;
        const next = copyExtra(ast, start, end - start);
        for (let i = start; i < end; i++) {
          const child = aliases.get(ast._extra + i * 8);
          if (child !== undefined) arena.writeExtraWord(next + i - start, 0, child);
        }
        const destination = writableDescriptor(ast, id, field);
        destination.view.setUint32(destination.offset, next, true);
        destination.view.setUint32(destination.offset + 4, next + end - start, true);
      }
    }
  }
  ast._aliasesNormalized = true;
}

export function ensureEditable(ast, id) {
  // Base writes must first isolate any other structural use of the same row.
  if (ast._parents === null && id < ast.nodeCount) ensureTopology(ast);
  const arena = ensureArena(ast);
  if (ast._parents !== null) syncAliases(ast);
  if (!arena.has(id)) baseRow(ast, id, ast.rawId(id));
  arena.ensureWritable();
  return arena;
}

export function writableField(ast, id, key) {
  ensureEditable(ast, id);
  const kind = ast.kind(id), field = layouts[kind].fields[key];
  if (!field) throw new TypeError("Field has no native storage");
  const location = writableDescriptor(ast, id, field);
  return { view: location.view, offset: location.offset, type: field[2], optional: field[3] };
}

/** Capture old child IDs and append-only list ranges before replacing bytes. */
export function captureReferences(ast, id, key) {
  const layout = layouts[ast.kind(id)];
  const fields = key === undefined ? layout.children : [layout.fields[key]];
  if (fields.length === 0) return null;
  const row = ast.row(id);
  const payloadView = row === -1 ? ast._view : ast._arena.view;
  const payload = row === -1 ? ast.offset(id) : row + 16;
  const references = [];
  for (const field of fields) {
    let view = payloadView, offset = payload + field[1];
    if (field[0]) {
      const index = payloadView.getUint32(payload + layout.extra, true) + field[1];
      view = index < ast._extraCount ? ast._view : ast._arena.view;
      offset = index < ast._extraCount ? ast._extra + index * 8 : ast._arena.extraAddress(index);
    }
    if (field[2] === "node") {
      const child = ast.childAt(id, offset, true, view);
      if (child !== NONE) references.push(child, -1);
    }
    else {
      const start = view.getUint32(offset, true), end = view.getUint32(offset + 4, true);
      if (start > end || end > ast.extraDataCount) throw new RangeError("AST list out of bounds");
      references.push(start, end);
    }
  }
  return references;
}

function clearParent(ast, parents, owner, child) {
  if (child !== NONE && ast.parentId(child) === owner) {
    // Deleting the entry would incorrectly restore the immutable base parent.
    parents.set(child, NONE);
  }
}

/** Parent metadata is numeric indexing, not an AST value store. */
export function refreshParents(ast, id, previous = null) {
  const parents = ast._currentParents ??= new Map();
  if (previous !== null) {
    for (let i = 0; i < previous.length; i += 2) {
      const start = previous[i], end = previous[i + 1];
      if (end === -1) clearParent(ast, parents, id, start);
      else {
        for (let index = start; index < end; index++)
          clearParent(ast, parents, id, ast.extraChild(id, index));
      }
    }
  }
  const kind = ast.kind(id), layout = layouts[kind];
  const row = ast.row(id);
  const payloadView = row === -1 ? ast._view : ast._arena.view;
  const payload = row === -1 ? ast.offset(id) : row + 16;
  for (const field of layout.children) {
    let view = payloadView, offset = payload + field[1];
    if (field[0]) {
      const index = payloadView.getUint32(payload + layout.extra, true) + field[1];
      view = index < ast._extraCount ? ast._view : ast._arena.view;
      offset = index < ast._extraCount ? ast._extra + index * 8 : ast._arena.extraAddress(index);
    }
    if (field[2] === "node") {
      const child = ast.childAt(id, offset, true, view);
      if (child !== NONE) parents.set(child, id);
    } else {
      const start = view.getUint32(offset, true), end = view.getUint32(offset + 4, true);
      if (start > end || end > ast.extraDataCount) throw new RangeError("AST list out of bounds");
      for (let i = start; i < end; i++) {
        const child = ast.extraChild(id, i);
        if (child !== NONE) parents.set(child, id);
      }
    }
  }
}
