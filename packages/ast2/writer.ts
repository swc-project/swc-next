import type { AstView } from "./index.ts";
import type { AstMeta, EnumValue, FieldDescriptor, NodeDescriptor } from "./types.ts";
import { AstList, ListEdit, isList, listAt } from "./list.ts";
import type { ListInput } from "./list.ts";
import { NodeView } from "./node.ts";
import { layouts, enums, NodeKind } from "./schema.ts";
import { writeNode, setSpan, fieldLocation, ensureArena } from "./storage.ts";
import { ReadRule, kindFlags, typeKinds } from "./protocol.ts";
import { fieldRule, spanLocation, scanOptionalMarker } from "./projection.ts";
import { NONE } from "./layout.ts";

const EMPTY = new Uint8Array();

/** Write a native field using the same location descriptor as reads. */
export function writeNativeField(ast: AstView, id: number, key: string, value: unknown): void {
  const kind = ast.kind(id);
  writeNativeFieldAt(ast, id, kind, key, layouts[kind].fields[key], value);
}

/** Write a native field with an already-resolved layout descriptor. */
export function writeNativeFieldAt(ast: AstView, id: number, kind: number, key: string,
  descriptor: FieldDescriptor, value: unknown, bump = true): void {
  rejectSourceRawWrite(ast, id, kind, key);
  if (!directWrite(ast, id, kind, key, value, descriptor, bump)) {
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

/** Parser-backed TypeScript template raw is projected from the source. */
function rejectSourceRawWrite(ast: AstView, id: number, kind: number, key: string): void {
  if (key === "raw" && kind === NodeKind.TemplateElement && ast._isTs && id < ast.nodeCount) {
    throw new TypeError("Replace the template element to change raw");
  }
}

/** Commit one public field to the binary arena before returning. */
export function writeField(ast: AstView, id: number, key: string, value: unknown) {
  const kind = ast.kind(id);
  if (key === "type") {
    if (kind === NodeKind.FormalParameter || kind === NodeKind.TsIndexSignatureParameter) {
      if (ast.getField(id, "type", kind) === value) { return; }
      throw new TypeError("Derived AST node type cannot be reassigned");
    }
    if (ast.getField(id, "type", kind) === value) { return; }
    const target = typeof value === "string" ? typeKinds[value] : undefined;
    if (typeof target !== "number") { throw new TypeError("Unsupported AST node type; allocate its native kind with new_node"); }
    if (target === NodeKind.TemplateElement && ast._isTs && id < ast.nodeCount) {
      throw new TypeError("Allocate a new template element with new_node");
    }
    writeNode(ast, id, target, ast.byteStart(id), ast.byteEnd(id));
    ast._listVersion++;
    committed(ast);
  } else if (key === "start" || key === "end") {
    writeSpan(ast, id, key, value, kind);
  } else if (key === "sourceType" && kind === NodeKind.Program) {
    const flags = flagsFor({ ...currentMeta(ast), sourceType: value as AstMeta["sourceType"] });
    ast._view.setUint32(ast.buffer.byteLength - 4, flags, true);
    editor(ast).flags = flags;
    ast._editVersion++;
  } else if (key === "value" && (kindFlags[kind] & 4)) {
    writeLiteralValue(ast, id, value, kind);
  } else if (kind === NodeKind.FormalParameter && writeParameterField(ast, id, key, value)) {
    return;
  } else if (kind === NodeKind.TsIndexSignatureParameter && writeIndexSignatureParameterField(ast, id, key, value)) {
    return;
  } else if (key === "optional") {
    const rule = fieldRule(ast._isTs, kind, key);
    if (rule === ReadRule.False || rule === ReadRule.Missing) {
      // This projected field is a constant; there is no native slot to write.
      if (value) { throw new TypeError("Field has no native storage"); }
      return;
    }
    if (!directWrite(ast, id, kind, key, value)) { throw new TypeError("Field cannot be assigned directly"); }
  } else {
    const rule = fieldRule(ast._isTs, kind, key);
    if (rule === ReadRule.Missing) { throw new TypeError("Field is not part of the public projection"); }
    if (isList(value) && (rule === ReadRule.List || rule === ReadRule.Params || rule === ReadRule.DelegateList ||
        rule === ReadRule.EmptyList || (rule === ReadRule.Native && layouts[kind].fields[key]?.[2] === "list"))) {
      writeProjectedList(ast, id, key, value);
    } else {
      const nativeKey = rule === ReadRule.Pattern ? "pattern" : rule === ReadRule.Initializer ? "initializer" : rule === ReadRule.Directive || rule === ReadRule.Text ? "value" : key;
      rejectSourceRawWrite(ast, id, kind, nativeKey);
      if (!directWrite(ast, id, kind, nativeKey, value)) { throw new TypeError("Field cannot be assigned directly"); }
    }
  }
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

export function storeReferences(ast: AstView, owner: number, key: string, ids: readonly number[]): void {
  const [currentStart, currentEnd] = ast.range(owner, key);
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
  const start = reserveList(ast, owner, key, currentStart, currentEnd - currentStart, ids.length);
  for (let index = 0; index < ids.length; index++) { writeReference(ast, start + index, ids[index]); }
  finishReferences(ast, owner, key, start, ids.length);
}

/** Splice only changed slots. Set is O(1), push amortized O(1), and splice shifts its tail. */
export function spliceReferences(ast: AstView, owner: number, key: string, begin: number, removed: number, ids: readonly number[]): void {
  const [oldStart, oldEnd] = ast.range(owner, key), oldLength = oldEnd - oldStart;
  if (begin < 0 || removed < 0 || begin + removed > oldLength) { throw new RangeError("AST list index out of bounds"); }
  const length = oldLength - removed + ids.length;
  const start = reserveList(ast, owner, key, oldStart, oldLength, length);
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
  finishReferences(ast, owner, key, start, length);
}

/** Projected lists are written to their native ranges, preserving existing IDs. */
export function writeProjectedList(ast: AstView, owner: number, key: string, input: ListInput): void {
  const kind = ast.kind(owner);
  if (key === "decorators" && !layouts[kind].fields[key]) {
    if (input.length === 0) { return; }
    throw new TypeError("This pattern has no decorator owner");
  }
  if (input instanceof ListEdit && spliceProjected(ast, owner, key, input)) { return; }
  const ids = listIds(ast, input);
  if (key === "params" && (kind === NodeKind.FormalParameters || (kindFlags[kind] & 2) !== 0)) {
    let params = kind === NodeKind.FormalParameters ? owner : ast.ref(owner, "params", true);
    if (params === NONE) {
      const allocated = ast.new_node(NodeKind.FormalParameters);
      allocated.set("kind", kind === NodeKind.ArrowFunctionExpression ? "arrow" : kind === NodeKind.Function || kind === NodeKind.TsFunction ? "formal" : "signature");
      params = allocated.nodeId;
      ast.writeNativeField(owner, "params", allocated);
    }
    const values = ids.map(id => id === NONE ? NONE : parameter(ast, id));
    const prefix = values.length && values[0] !== NONE && ast._isTs && ast.kind(values[0]) === NodeKind.TsThisParameter ? values.shift()! : NONE;
    const rest = values.length && values.at(-1) !== NONE && ast.kind(values.at(-1)!) === NodeKind.FormalParameterRest ? values.pop()! : NONE;
    if (kind !== NodeKind.FormalParameters && layouts[kind].fields.thisParam) { ast.writeNativeField(owner, "thisParam", prefix === NONE ? null : ast.nativeNode(prefix)); }
    ast.writeNativeField(params, "rest", rest === NONE ? null : ast.nativeNode(rest));
    storeReferences(ast, params, "items", values);
    return;
  }
  if (key === "body" && (kind === NodeKind.Program || kind === NodeKind.FunctionBody || kind === NodeKind.TsModuleBlock)) {
    const directives: number[] = [], body: number[] = [];
    for (const id of ids) {
      if (id !== NONE && ast.kind(id) === NodeKind.Directive && body.length === 0) { directives.push(id); }
      else if (id !== NONE && ast.kind(id) === NodeKind.Directive) {
        const statement = ast.new_node(NodeKind.ExpressionStatement);
        statement.set("expression", ast.nativeNode(ast.ref(id, "expression")));
        body.push(statement.nodeId);
      } else { body.push(id); }
    }
    storeReferences(ast, owner, "directives", directives);
    storeReferences(ast, owner, "body", body);
    return;
  }
  if ((kind === NodeKind.ArrayPattern && key === "elements") || (kind === NodeKind.ObjectPattern && key === "properties")) {
    const items = [...ids];
    const rest = items.length && items.at(-1) !== NONE && ast.kind(items.at(-1)!) === NodeKind.BindingRestElement ? items.pop()! : NONE;
    ast.writeNativeField(owner, "rest", rest === NONE ? null : ast.nativeNode(rest));
    storeReferences(ast, owner, key, items);
    return;
  }
  storeReferences(ast, owner, key, ids);
}

function editor(ast: AstView) {
  return ast._editor ??= { flags: ast._flags };
}

function flagsFor(meta: AstMeta): number {
  const lang = enums.Lang.indexOf(meta.lang);
  const sourceType = enums.SourceType.indexOf(meta.sourceType);
  if (lang < 0 || sourceType < 0) { throw new TypeError("Invalid AST metadata"); }
  return sourceType | (lang << 2);
}

function currentMeta(ast: AstView): AstMeta {
  const flags = ast._editor?.flags;
  return flags === undefined ? ast.meta : {
    lang: enums.Lang[(flags >> 2) & 7],
    sourceType: enums.SourceType[flags & 3],
  };
}

function committed(ast: AstView, bump = true) {
  if (bump) { ast._editVersion++; }
}

function directWrite(ast: AstView, id: number, kind: number, key: string, value: unknown,
  descriptor = layouts[kind].fields[key], bump = true) {
  if (!Array.isArray(descriptor)) { return false; }
  const type = descriptor[2], optional = descriptor[3];
  let encoded: number | string | null;
  if (type === "node") {
    encoded = referenceId(ast, value);
  } else if (type === "list") {
    if (!isList(value)) { return false; }
    if (value instanceof ListEdit && value.source.native && value.source.ast === ast && value.source.owner === id && value.source.key === key)
      { spliceReferences(ast, id, key, value.start, value.removed, listIds(ast, value.inserted)); }
    else { storeReferences(ast, id, key, listIds(ast, value)); }
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
    storeString(ast, id, kind, key, encoded as string | null, descriptor);
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

function storeString(ast: AstView, id: number, kind: number, key: string, value: string | null,
  descriptor: FieldDescriptor = layouts[kind].fields[key]): void {
  let start = NONE, end = 0;
  if (value !== null && value !== "") {
    ({ start, end } = ensureArena(ast).internString(value));
  } else if (value === "") { start = 0; }
  // Appending can relocate the arena. Keep the descriptor, then reacquire bytes.
  const { view, offset } = fieldLocation(ast, id, descriptor, kind);
  view.setUint32(offset, start, true); view.setUint32(offset + 4, end, true);
}

function writeLiteralValue(ast: AstView, id: number, value: unknown, oldKind: number): void {
  let kind: number, first: string | null = null, second: string | null = null;
  if (value == null) { kind = NodeKind.NullLiteral; }
  else if (typeof value === "string") { kind = NodeKind.StringLiteral; first = value; second = JSON.stringify(value); }
  else if (typeof value === "number") { kind = NodeKind.NumericLiteral; }
  else if (typeof value === "boolean") { kind = NodeKind.BooleanLiteral; }
  else if (typeof value === "bigint") {
    if (value < 0n) {
      const argument = ast.new_node(NodeKind.BigIntLiteral, ast.byteStart(id), ast.byteEnd(id));
      argument.set("raw", String(-value));
      writeNode(ast, id, NodeKind.UnaryExpression, ast.byteStart(id), ast.byteEnd(id));
      ast._listVersion++;
      writeNativeField(ast, id, "operator", "-");
      writeNativeField(ast, id, "argument", argument);
      // A negative BigInt key is a unary expression and needs brackets.
      for (let owner = 0; owner < ast.totalNodeCount; owner++) {
        const fields = layouts[ast._kindAt(owner)].fields;
        if (fields.computed !== undefined && fields.key?.[2] === "node" && ast.ref(owner, "key", true) === id) {
          writeNativeField(ast, owner, "computed", true);
        }
      }
      return;
    }
    kind = NodeKind.BigIntLiteral; first = String(value);
  } else if (value instanceof RegExp) {
    kind = NodeKind.RegExpLiteral; first = value.source; second = value.flags;
  } else { throw new TypeError("Invalid literal value"); }
  const strings = (first?.length ?? 0) * 3 + (second?.length ?? 0) * 3;
  if (strings) {
    const arena = ensureArena(ast);
    arena.reserve(arena.rowCount, arena.extraUsed, arena.stringUsed + strings);
  }
  if (kind !== oldKind) {
    writeNode(ast, id, kind, ast.byteStart(id), ast.byteEnd(id));
    ast._listVersion++;
  }
  if (kind === NodeKind.NumericLiteral) {
    const { view, offset } = fieldLocation(ast, id, layouts[kind].fields.value, kind);
    view.setFloat64(offset, value as number, true);
    storeString(ast, id, kind, "raw", null);
  } else if (kind === NodeKind.BooleanLiteral) {
    const { view, offset } = fieldLocation(ast, id, layouts[kind].fields.value, kind);
    view.setUint8(offset, value ? 1 : 0);
  } else if (kind === NodeKind.StringLiteral) { storeString(ast, id, kind, "value", first); storeString(ast, id, kind, "raw", second); }
  else if (kind === NodeKind.BigIntLiteral) { storeString(ast, id, kind, "raw", first); }
  else if (kind === NodeKind.RegExpLiteral) { storeString(ast, id, kind, "pattern", first); storeString(ast, id, kind, "flags", second); }
  committed(ast);
}

function writeSpan(ast: AstView, id: number, key: "start" | "end", value: unknown, kind: number) {
  const name = layouts[kind].name;
  if (key === "end" &&
      (name === "BindingIdentifier" || name === "ArrayPattern" || name === "ObjectPattern") &&
      ast.ref(id, "typeAnnotation") === NONE && ast.value(id, "optional")) {
    const start = ast.byteStart(id), end = ast.utf8Offset(value as number);
    if (end < start) { throw new RangeError("AST span ends before it starts"); }
    if (scanOptionalMarker(ast, end) !== -1) { throw new RangeError("Optional pattern end must include the ? marker"); }
    setSpan(ast, id, start, end);
    committed(ast);
    return;
  }
  const missing = kind === NodeKind.TemplateElement && ast._isTs &&
    ast.byteStart(id) === 0 && ast.byteEnd(id) === 0;
  const location = spanLocation(ast, id, key, kind);
  const byte = ast.utf8Offset((value as number) - location.adjustment);
  let start = location.field === "start" ? byte : ast.byteStart(location.id);
  let end = location.field === "end" ? byte : ast.byteEnd(location.id);
  if (missing) {
    // First write on a cloned template element: materialize the endpoint that
    // previously projected to 0 so the new span round-trips.
    if (key === "start") { end = byte; } else { start = 1; }
  }
  if (end < start) { throw new RangeError("AST span ends before it starts"); }
  if (location.additional !== undefined && byte > ast.byteEnd(location.additional))
    { throw new RangeError("AST span ends before it starts"); }
  setSpan(ast, location.id, start, end);
  if (location.additional !== undefined) { setSpan(ast, location.additional, byte, ast.byteEnd(location.additional)); }
  committed(ast);
}

function writeParameterField(ast: AstView, id: number, key: string, value: unknown): boolean {
  const initializer = ast.ref(id, "initializer") !== NONE;
  const pattern = ast.ref(id, "pattern");
  switch (key) {
    case "left":
      if (!initializer) { throw new TypeError("A parameter without an initializer has no left node"); }
      writeNativeField(ast, id, "pattern", value);
      return true;
    case "right":
      if (!initializer) { throw new TypeError("A parameter without an initializer has no right node"); }
      writeNativeField(ast, id, "initializer", value);
      return true;
    case "name":
      if (initializer || ast.kind(pattern) !== NodeKind.BindingIdentifier)
        { throw new TypeError("This parameter has no name field"); }
      writeNativeField(ast, pattern, "name", value);
      return true;
    case "elements":
      if (initializer || ast.kind(pattern) !== NodeKind.ArrayPattern)
        { throw new TypeError("This parameter has no elements field"); }
      if (!isList(value)) { throw new TypeError("Expected an AST list"); }
      writeProjectedList(ast, pattern, "elements", value);
      return true;
    case "properties":
      if (initializer || ast.kind(pattern) !== NodeKind.ObjectPattern)
        { throw new TypeError("This parameter has no properties field"); }
      if (!isList(value)) { throw new TypeError("Expected an AST list"); }
      writeProjectedList(ast, pattern, "properties", value);
      return true;
    case "typeAnnotation":
      if (initializer) { throw new TypeError("A parameter with an initializer has no type annotation"); }
      writeNativeField(ast, pattern, "typeAnnotation", value);
      return true;
    case "optional":
      if (initializer) {
        if (value) { throw new TypeError("AssignmentPattern cannot be optional"); }
        return true;
      }
      writeNativeField(ast, pattern, "optional", value);
      return true;
    default:
      return false;
  }
}

function writeIndexSignatureParameterField(ast: AstView, id: number, key: string, value: unknown): boolean {
  switch (key) {
    case "name":
      writeNativeField(ast, ast.ref(id, "name"), "name", value);
      return true;
    case "typeAnnotation":
      writeNativeField(ast, id, "typeAnnotation", value);
      return true;
    case "optional":
      if (value) { throw new TypeError("Field has no native storage"); }
      return true;
    default:
      return false;
  }
}

function finishEditsFlags(ast: AstView, options: Partial<AstMeta>): number {
  const current = currentMeta(ast), meta = { ...current, ...options }, flags = flagsFor(meta);
  if ((current.lang === "js" || current.lang === "jsx") !== (meta.lang === "js" || meta.lang === "jsx"))
    { throw new TypeError("Cannot change between JavaScript and TypeScript AST languages"); }
  return flags;
}

/** Only slack needs metadata: a full list's range already gives its capacity. */
function reserveList(ast: AstView, owner: number, key: string, start: number, length: number, required: number) {
  let capacities = ast._listCapacities?.get(owner);
  const previous = capacities?.get(key);
  let capacity = previous?.[0] === start ? previous[1] : length;
  if (required > capacity) {
    capacity = Math.max(required, capacity * 2, 8);
    start = ensureArena(ast).appendExtra(capacity);
  }
  if (required === capacity) {
    if (capacities?.delete(key) && capacities.size === 0) {
      ast._listCapacities!.delete(owner);
      if (ast._listCapacities!.size === 0) { ast._listCapacities = undefined; }
    }
  } else if (previous?.[0] !== start) {
    if (capacities === undefined) {
      const lists = ast._listCapacities ??= new Map();
      lists.set(owner, capacities = new Map());
    }
    capacities.set(key, [start, capacity]);
  }
  return start;
}

function finishReferences(ast: AstView, owner: number, key: string, start: number, length: number) {
  const kind = ast.kind(owner);
  const field = fieldLocation(ast, owner, layouts[kind].fields[key], kind);
  field.view.setUint32(field.offset, start, true);
  field.view.setUint32(field.offset + 4, start + length, true);
  ast._editVersion++;
  ast._listVersion++;
}

function writeReference(ast: AstView, index: number, id: number) {
  if (index < ast._extraCount) { ast._view.setUint32(ast._extra + index * 4, id, true); }
  else { ast._arena!.view!.setUint32(ast._arena!.extraAddress(index), id, true); }
}

function spliceProjected(ast: AstView, owner: number, key: string, input: ListEdit): boolean {
  if (input.source.ast !== ast || input.source.owner !== owner || input.source.key !== key) { return false; }
  const kind = ast.kind(owner);
  const ids = listIds(ast, input.inserted);
  if (key === "params" && (kind === NodeKind.FormalParameters || (kindFlags[kind] & 2) !== 0)) {
    const params = kind === NodeKind.FormalParameters ? owner : ast.ref(owner, "params", true);
    if (params === NONE) { return false; }
    const prefix = Number(kind !== NodeKind.FormalParameters && ast._isTs && ast.ref(owner, "thisParam") !== NONE);
    const suffix = Number(ast.ref(params, "rest") !== NONE);
    if (input.start < prefix || input.start + input.removed > input.source.length - suffix) { return false; }
    if (ids.some(id => {
      if (id === NONE) { return true; }
      const kind = ast.kind(id);
      return kind === NodeKind.TsThisParameter || kind === NodeKind.FormalParameterRest || kind === NodeKind.BindingRestElement;
    })) { return false; }
    spliceReferences(ast, params, "items", input.start - prefix, input.removed, ids.map(id => parameter(ast, id)));
    return true;
  }
  const descriptor = layouts[kind].fields[key];
  if (descriptor?.[2] !== "list") { return false; }
  let offset = 0, suffix = 0;
  if (key === "body" && (kind === NodeKind.Program || kind === NodeKind.FunctionBody || kind === NodeKind.TsModuleBlock)) {
    const [start, end] = ast.range(owner, "directives");
    offset = end - start;
    if (ids.some(id => id !== NONE && ast.kind(id) === NodeKind.Directive)) { return false; }
  } else if (kind === NodeKind.ArrayPattern || kind === NodeKind.ObjectPattern) {
    suffix = Number(ast.ref(owner, "rest") !== NONE);
    if (ids.some(id => id !== NONE && ast.kind(id) === NodeKind.BindingRestElement)) { return false; }
  }
  if (input.start < offset || input.start + input.removed > input.source.length - suffix) { return false; }
  spliceReferences(ast, owner, key, input.start - offset, input.removed, ids);
  return true;
}

function parameter(ast: AstView, id: number): number {
  const kind = ast.kind(id);
  if (kind === NodeKind.FormalParameter || kind === NodeKind.FormalParameterRest || kind === NodeKind.TsThisParameter) { return id; }
  const value = ast.nativeNode(id);
  if (kind === NodeKind.BindingRestElement) {
    const rest = ast.new_node(NodeKind.FormalParameterRest);
    rest.set("argument", value.get("argument"));
    rest.set("typeAnnotation", value.get("typeAnnotation"));
    return rest.nodeId;
  }
  const item = ast.new_node(NodeKind.FormalParameter);
  if (kind === NodeKind.AssignmentPattern) { item.set("pattern", value.get("left")); item.set("initializer", value.get("right")); }
  else { item.set("pattern", value); }
  return item.nodeId;
}
