import type { AstView } from "#protocol/index";
import type { AstMeta, EnumValue, FieldDescriptor, NodeDescriptor } from "#protocol/types";
import { AstList, ListEdit, isList, listAt } from "#protocol/list";
import type { ListInput } from "#protocol/list";
import { NodeView } from "#protocol/node";
import { enums, nativeField, NodeKind } from "#protocol/schema";
import { writeNode, setSpan, fieldLocation, ensureArena } from "#protocol/storage";
import { Field, languages, sourceTypes } from "#protocol/protocol";
import { NONE } from "#protocol/layout";

const EMPTY = new Uint8Array();

/** Write a native field using the same location descriptor as reads. */
export function writeNativeField(ast: AstView, id: number, field: number, value: unknown): void {
  const kind = ast.kind(id);
  writeNativeFieldAt(ast, id, kind, field, nativeField(kind, field), value);
}

/** Write a native field with an already-resolved layout descriptor. */
export function writeNativeFieldAt(ast: AstView, id: number, kind: number, field: number,
  descriptor: FieldDescriptor | undefined, value: unknown, bump = true): void {
  if (!directWrite(ast, id, kind, field, value, descriptor, bump)) {
    throw new TypeError("Field has no native storage");
  }
}

/** Write a validated node ID at a native node field without bumping versions. */
export function writeNodeIdAt(ast: AstView, id: number, kind: number,
  descriptor: FieldDescriptor, childId: number): void {
  const { view, offset } = fieldLocation(ast, id, descriptor, kind);
  view.setUint32(offset, childId, true);
}

/** Write a complete list range for a newly allocated node without bumping versions. */
export function writeReferencesAt(ast: AstView, owner: number, kind: number,
  descriptor: FieldDescriptor, ids: readonly number[]): void {
  const start = ids.length === 0 ? 0 : ensureArena(ast).appendExtra(ids.length);
  for (let index = 0; index < ids.length; index++) { writeReference(ast, start + index, ids[index]); }
  const { view, offset } = fieldLocation(ast, owner, descriptor, kind);
  view.setUint32(offset, start, true);
  view.setUint32(offset + 4, start + ids.length, true);
}

/** Change a node's native kind without changing its identity or span. */
export function writeType(ast: AstView, id: number, type: string): void {
  const kind = ast.kind(id);
  const target = NodeKind[type as keyof typeof NodeKind];
  if (typeof target !== "number") { throw new TypeError("Unknown AST node type"); }
  if (target === kind) { return; }
  writeNode(ast, id, target, ast.byteStart(id), ast.byteEnd(id));
  ast._listVersion++;
  committed(ast);
}

export function writeSpan(ast: AstView, id: number, isStart: boolean, value: number): void {
  const byte = ast.utf8Offset(value);
  const start = isStart ? byte : ast.byteStart(id);
  const end = isStart ? ast.byteEnd(id) : byte;
  if (end < start) { throw new RangeError("AST span ends before it starts"); }
  setSpan(ast, id, start, end);
  committed(ast);
}

/** Expose current storage. Setters and list operations write bytes; the layout is published here. */
export function finishEdits(ast: AstView, options?: Partial<AstMeta>): [Uint8Array, Uint8Array] {
  const flags = options === undefined ? ast._editor?.flags ?? ast._flags : finishEditsFlags(ast, options);
  ast._view.setUint32(ast.buffer.byteLength - 4, flags, true);
  editor(ast).flags = flags;
  const arena = ast._arena;
  return [ast.buffer, arena === null || !arena.rowCount && !arena.extraUsed && !arena.stringUsed ? EMPTY : arena.borrow(ast.rootId,flags)];
}

/** Encode a reference belonging to this buffer; foreign views are cloned into it. */
export function referenceId(ast: AstView, value: unknown): number {
  if (value == null) { return NONE; }
  if (value instanceof NodeView) {
    return value.ast === ast ? value.nodeId : ast.clone(value).nodeId;
  }
  if (isNodeDescriptor(value)) { return createNode(ast, value).nodeId; }
  throw new TypeError("Expected a node view or a node descriptor");
}

function isNodeDescriptor(value: unknown): value is NodeDescriptor {
  return typeof value === "object" && value !== null && !(value instanceof NodeView) &&
    typeof (value as NodeDescriptor).kind === "number";
}

/** Allocate a descriptor's node in this AST and assign its remaining fields. */
function createNode(ast: AstView, descriptor: NodeDescriptor): NodeView {
  const start = descriptor.start, end = descriptor.end;
  const byteStart = start === undefined ? 0 : ast.utf8Offset(start);
  const node = ast.new_node(
    descriptor.kind,
    byteStart,
    end === undefined ? byteStart : ast.utf8Offset(end),
  ) as unknown as NodeView;
  for (const key in descriptor) {
    if (!Object.hasOwn(descriptor, key)) { continue; }
    if (key === "kind" || key === "start" || key === "end" || key === "type") { continue; }
    node.set(key, descriptor[key]);
  }
  return node;
}

export function listIds(ast: AstView, values: ListInput): number[] {
  const ids = new Array<number>(values.length);
  const source = values instanceof AstList ? values : values instanceof ListEdit ? values.source : undefined;
  for (let index = 0; index < ids.length; index++) {
    const id = source?.ast === ast && (values instanceof AstList || values instanceof ListEdit) ? values.nodeId(index) : undefined;
    ids[index] = id === undefined ? referenceId(ast, listAt(values, index)) : id;
  }
  return ids;
}

export function storeReferences(ast: AstView, owner: number, field: number, ids: readonly number[]): void {
  const [currentStart, currentEnd] = ast._range(owner, field);
  if (ids.length === currentEnd - currentStart) {
    let unchanged = true;
    for (let index = 0; index < ids.length; index++) {
      if (ast.extraChild(owner, currentStart + index, true) !== ids[index]) {
        unchanged = false;
        break;
      }
    }
    if (unchanged) { return; }
  }
  const start = reserveList(ast, owner, field, currentStart, currentEnd - currentStart, ids.length);
  for (let index = 0; index < ids.length; index++) { writeReference(ast, start + index, ids[index]); }
  finishReferences(ast, owner, field, start, ids.length);
}

/** Splice only changed slots. Set is O(1), push amortized O(1), and splice shifts its tail. */
export function spliceReferences(ast: AstView, owner: number, field: number, begin: number, removed: number, ids: readonly number[]): void {
  const [oldStart, oldEnd] = ast._range(owner, field), oldLength = oldEnd - oldStart;
  if (begin < 0 || removed < 0 || begin + removed > oldLength) { throw new RangeError("AST list index out of bounds"); }
  const length = oldLength - removed + ids.length;
  const start = reserveList(ast, owner, field, oldStart, oldLength, length);
  if (start !== oldStart) {
    for (let index = 0; index < begin; index++) { writeReference(ast, start + index, ast.extraChild(owner, oldStart + index)); }
    for (let index = begin + removed; index < oldLength; index++)
      { writeReference(ast, start + index - removed + ids.length, ast.extraChild(owner, oldStart + index)); }
  } else if (removed !== ids.length) {
    // Both ranges are in one storage region. copyWithin handles overlapping tails.
    const bytes = start < ast._extraCount ? ast._bytes : ast._arena!.bytes!;
    const offset = start < ast._extraCount ? ast._extra + start * 4 : ast._arena!.extraAddress(start);
    bytes.copyWithin(offset + (begin + ids.length) * 4, offset + (begin + removed) * 4, offset + oldLength * 4);
  }
  for (let index = 0; index < ids.length; index++) { writeReference(ast, start + begin + index, ids[index]); }
  finishReferences(ast, owner, field, start, length);
}

function editor(ast: AstView) {
  return ast._editor ??= { flags: ast._flags };
}

function flagsFor(meta: AstMeta): number {
  const lang = languages.indexOf(meta.lang);
  const sourceType = sourceTypes.indexOf(meta.sourceType);
  if (lang < 0 || sourceType < 0) { throw new TypeError("Invalid AST metadata"); }
  return sourceType | (lang << 2);
}

function currentMeta(ast: AstView): AstMeta {
  const flags = ast._editor?.flags;
  return flags === undefined ? ast.meta : {
    lang: languages[(flags >> 2) & 7],
    sourceType: sourceTypes[flags & 3],
  };
}

function committed(ast: AstView, bump = true) {
  if (bump) { ast._editVersion++; }
}

function directWrite(ast: AstView, id: number, kind: number, field: number, value: unknown,
  descriptor: FieldDescriptor | undefined, bump = true) {
  if (descriptor === undefined) { return false; }
  const type = descriptor[2], optional = descriptor[3];
  let encoded: number | string | null;
  if (type === "node") {
    encoded = referenceId(ast, value);
  } else if (type === "list") {
    if (!isList(value)) { return false; }
    if (value instanceof ListEdit && value.source.ast === ast && value.source.owner === id && value.source.field === field)
      { spliceReferences(ast, id, field, value.start, value.removed, listIds(ast, value.inserted)); }
    else { storeReferences(ast, id, field, listIds(ast, value)); }
    return true;
  } else if (type === "string") {
    if (value == null && optional) { encoded = null; }
    else if (typeof value !== "string") { return false; }
    else { encoded = value; }
  } else if (type.startsWith("enum:")) {
    if (value == null && optional) { encoded = 255; }
    else {
      encoded = enums[type.slice(5)].indexOf(value as EnumValue);
      if (encoded < 0) { throw new TypeError("Invalid AST enum value"); }
    }
  } else if (type === "bool") { encoded = value ? 1 : 0; }
  else {
    switch (type) {
      case "u8": case "i8": case "u16": case "i16": case "u32": case "i32": case "f64":
        if (typeof value !== "number") { return false; }
        encoded = value;
        break;
      default: return false;
    }
  }

  if (type === "string" && encoded !== null && ast.stringLength + (encoded as string).length * 3 >= NONE)
    { throw new RangeError("AST string pool is too large"); }
  if (type === "string") {
    storeString(ast, id, kind, descriptor, encoded as string | null);
  } else {
    const scalar = encoded as number;
    const { view, offset } = fieldLocation(ast, id, descriptor, kind);
    if (type === "node" || type === "u32") { view.setUint32(offset, scalar, true); }
    else if (type === "bool" || type === "u8" || type.startsWith("enum:")) { view.setUint8(offset, scalar); }
    else if (type === "i8") { view.setInt8(offset, scalar); }
    else if (type === "u16") { view.setUint16(offset, scalar, true); }
    else if (type === "i16") { view.setInt16(offset, scalar, true); }
    else if (type === "i32") { view.setInt32(offset, scalar, true); }
    else { view.setFloat64(offset, scalar, true); }
  }
  if (bump && kind === NodeKind.NumericLiteral && field === Field.value) {
    // Match the Rust value setter: an existing lexeme no longer describes the value.
    storeString(ast, id, kind, nativeField(kind, Field.raw)!, null);
  }
  if (bump && type === "node") {
    ast._listVersion++;
  }
  committed(ast, bump);
  return true;
}

/** Write a required or optional string at a constant payload offset. */
export function writeStringAt(ast: AstView, id: number, offset: number, optional: boolean, value: unknown): void {
  let start = NONE, end = 0;
  if (value == null) {
    if (!optional) { throw new TypeError("Field cannot be assigned directly"); }
  } else if (typeof value !== "string") {
    throw new TypeError("Field cannot be assigned directly");
  } else if (value !== "") {
    if (ast.stringLength + value.length * 3 >= NONE) { throw new RangeError("AST string pool is too large"); }
    ({ start, end } = ensureArena(ast).internString(value));
  } else {
    start = 0;
  }
  const row = ast.row(id);
  const view = row === -1 ? ast._view : ast._arena!.view!;
  const at = (row === -1 ? ast._payloads + id * 16 : row + 16) + offset;
  view.setUint32(at, start, true);
  view.setUint32(at + 4, end, true);
  committed(ast);
}

/** Write a node reference at a constant payload offset. */
export function writeNodeAt(ast: AstView, id: number, offset: number, value: unknown): void {
  const encoded = referenceId(ast, value);
  const row = ast.row(id);
  const view = row === -1 ? ast._view : ast._arena!.view!;
  view.setUint32((row === -1 ? ast._payloads + id * 16 : row + 16) + offset, encoded, true);
  ast._listVersion++;
  committed(ast);
}

/** Write a boolean at a constant payload offset. */
export function writeBoolAt(ast: AstView, id: number, offset: number, value: unknown): void {
  const row = ast.row(id);
  const view = row === -1 ? ast._view : ast._arena!.view!;
  view.setUint8((row === -1 ? ast._payloads + id * 16 : row + 16) + offset, value ? 1 : 0);
  committed(ast);
}

/** Write a number at a constant payload offset. */
export function writeNumberAt(ast: AstView, id: number, offset: number, type: string, value: unknown): void {
  if (typeof value !== "number") { throw new TypeError("Field cannot be assigned directly"); }
  const row = ast.row(id);
  const view = row === -1 ? ast._view : ast._arena!.view!;
  const at = (row === -1 ? ast._payloads + id * 16 : row + 16) + offset;
  switch (type) {
    case "u8": view.setUint8(at, value); break;
    case "i8": view.setInt8(at, value); break;
    case "u16": view.setUint16(at, value, true); break;
    case "i16": view.setInt16(at, value, true); break;
    case "u32": view.setUint32(at, value, true); break;
    case "i32": view.setInt32(at, value, true); break;
    default: view.setFloat64(at, value, true); break;
  }
  committed(ast);
}

/** Write an enum value at a constant payload offset. */
export function writeEnumAt(ast: AstView, id: number, offset: number, optional: boolean, table: readonly EnumValue[], value: unknown): void {
  let encoded: number;
  if (value == null && optional) { encoded = 255; }
  else {
    encoded = table.indexOf(value as EnumValue);
    if (encoded < 0) { throw new TypeError("Invalid AST enum value"); }
  }
  const row = ast.row(id);
  const view = row === -1 ? ast._view : ast._arena!.view!;
  view.setUint8((row === -1 ? ast._payloads + id * 16 : row + 16) + offset, encoded);
  committed(ast);
}

function storeString(ast: AstView, id: number, kind: number, descriptor: FieldDescriptor, value: string | null): void {
  let start = NONE, end = 0;
  if (value !== null && value !== "") {
    ({ start, end } = ensureArena(ast).internString(value));
  } else if (value === "") { start = 0; }
  // Appending can relocate the arena. Keep the descriptor, then reacquire bytes.
  const { view, offset } = fieldLocation(ast, id, descriptor, kind);
  view.setUint32(offset, start, true); view.setUint32(offset + 4, end, true);
}

function finishEditsFlags(ast: AstView, options: Partial<AstMeta>): number {
  const current = currentMeta(ast), meta = { ...current, ...options }, flags = flagsFor(meta);
  if ((current.lang === "js" || current.lang === "jsx") !== (meta.lang === "js" || meta.lang === "jsx"))
    { throw new TypeError("Cannot change between JavaScript and TypeScript AST languages"); }
  return flags;
}

/** Only slack needs metadata: a full list's range already gives its capacity. */
function reserveList(ast: AstView, owner: number, field: number, start: number, length: number, required: number) {
  let capacities = ast._listCapacities?.get(owner);
  const previous = capacities?.get(field);
  let capacity = previous?.[0] === start ? previous[1] : length;
  if (required > capacity) {
    capacity = Math.max(required, capacity * 2, 8);
    start = ensureArena(ast).appendExtra(capacity);
  }
  if (required === capacity) {
    if (capacities?.delete(field) && capacities.size === 0) {
      ast._listCapacities!.delete(owner);
      if (ast._listCapacities!.size === 0) { ast._listCapacities = undefined; }
    }
  } else if (previous?.[0] !== start) {
    if (capacities === undefined) {
      const lists = ast._listCapacities ??= new Map();
      lists.set(owner, capacities = new Map());
    }
    capacities.set(field, [start, capacity]);
  }
  return start;
}

function finishReferences(ast: AstView, owner: number, field: number, start: number, length: number) {
  const kind = ast.kind(owner);
  const { view, offset } = fieldLocation(ast, owner, nativeField(kind, field)!, kind);
  view.setUint32(offset, start, true);
  view.setUint32(offset + 4, start + length, true);
  ast._editVersion++;
  ast._listVersion++;
}

function writeReference(ast: AstView, index: number, id: number) {
  if (index < ast._extraCount) { ast._view.setUint32(ast._extra + index * 4, id, true); }
  else { ast._arena!.view!.setUint32(ast._arena!.extraAddress(index), id, true); }
}
