import type { AstView } from "./index.ts";
import type { ListCursor } from "./types.ts";
import { NODE_KIND_COUNT, FIELD_COUNT, CURSOR_FIELD_COUNT, fieldIds, fieldRules, fieldRuleIds, ReadRule, keyTables, keyIds, staticTypes, cursorFieldIds, cursorRules, cursorRuleIds, cursorRecipes, kindFlags } from "./protocol.ts";
export { staticTypes } from "./protocol.ts";
// ESTree projection over the generated native field layout. Reading a field
// never recursively materializes a subtree; ownership comes from buffer links.
import { layouts, enums, NodeKind } from "./schema.ts";
import { NONE } from "./layout.ts";

class TemplateValue {
  constructor(readonly ast: AstView, readonly id: number) {}
  get raw(): string { return this.ast.value(this.id, "raw") as string; }
  set raw(value: string) { this.ast.writeNativeField(this.id, "raw", value.includes("\r") ? value.replace(/\r\n?/g, "\n") : value); }
  get cooked(): string | null { return this.ast.value(this.id, "isCookedUndefined") ? null : this.ast.value(this.id, "cooked") as string; }
  set cooked(value: string | null) {
    this.ast.writeNativeField(this.id, "cooked", value ?? "");
    this.ast.writeNativeField(this.id, "isCookedUndefined", value === null);
  }
  toJSON() { return { raw: this.raw, cooked: this.cooked }; }
}

class RegexFields {
  constructor(readonly ast: AstView, readonly id: number) {}
  get pattern(): string { return this.ast.value(this.id, "pattern") as string; }
  set pattern(value: string) { this.ast.writeNativeField(this.id, "pattern", value); }
  get flags(): string { return [...this.ast.value(this.id, "flags") as string].sort().join(""); }
  set flags(value: string) { this.ast.writeNativeField(this.id, "flags", value); }
  toJSON() { return { pattern: this.pattern, flags: this.flags }; }
}
interface SpanLocation {
  id: number;
  field: "start" | "end";
  adjustment: number;
  additional?: number;
}

export function isProjectedValue(value: unknown): value is TemplateValue | RegexFields {
  return value instanceof TemplateValue || value instanceof RegexFields;
}
export function projectedKeys(ast: AstView, id: number): readonly string[] {
  const kind = ast.kind(id),
    keys = keyTables[keyIds[Number(ast._isTs) * NODE_KIND_COUNT + kind]];
  if (kind === NodeKind.FormalParameter) {
    // The wrapper is public; its pattern child only supplies one shape key.
    const initializer = ast.ref(id, "initializer") !== NONE,
      patternKind = ast.kind(ast.ref(id, "pattern")),
      patternKey =
        patternKind === NodeKind.BindingIdentifier ? "name" :
        patternKind === NodeKind.ArrayPattern ? "elements" :
        patternKind === NodeKind.ObjectPattern ? "properties" : null;
    return keys.filter((key) =>
      key === "type" || key === "start" || key === "end" ||
      (initializer ? key === "left" || key === "right" : key === patternKey) ||
      (ast._isTs && (key === "decorators" || key === "optional" || key === "typeAnnotation"))
    );
  }
  return kind === NodeKind.TsModuleDeclaration
    ? keys.filter((key) =>
        (key !== "body" || ast.ref(id, "body") !== NONE) &&
        (key !== "attributes" || ast.ref(id, "attributes") !== NONE))
    : keys;
}

export function nodeType(ast: AstView, id: number, name: string): string {
  switch (name) {
    case "FormalParameter": {
      const pattern = ast.ref(id, "pattern");
      return ast.ref(id, "initializer") !== NONE
        ? "AssignmentPattern"
        : layouts[ast.kind(pattern)].type;
    }
    case "Function":
      return ast.value(id, "functionType") as string;
    case "TsFunction":
      return ast.value(id, "isMethod") ? "TSEmptyBodyFunctionExpression" : "TSDeclareFunction";
    case "Class":
      return ast.value(id, "classType") as string;
    case "MethodDefinition":
    case "TsMethodDefinition":
      return ast._isTs && ast.value(id, "abstract")
        ? "TSAbstractMethodDefinition"
        : "MethodDefinition";
    case "PropertyDefinition": {
      const accessor = ast.value(id, "accessor");
      return ast._isTs && ast.value(id, "abstract")
        ? accessor
          ? "TSAbstractAccessorProperty"
          : "TSAbstractPropertyDefinition"
        : accessor
          ? "AccessorProperty"
          : "PropertyDefinition";
    }
    default:
      return layouts[ast.kind(id)].type;
  }
}

/** Resolve the native span and UTF-16 adjustment for both reads and writes. */
export function spanLocation(ast: AstView, id: number, field: "start" | "end", kind: number): SpanLocation {
  const name = layouts[kind].name;
  const ts = ast._isTs;
  if (field === "start") {
    if (name === "Program" && ts) { return programStart(ast, id); }
    if (name === "FormalParameter") { return { id: ast.ref(id, "pattern"), field, adjustment: 0 }; }
    if (name === "TemplateElement" && ts) { return { id, field, adjustment: -1 }; }
    return { id, field, adjustment: 0 };
  }
  if (name === "TemplateElement" && ts)
    { return { id, field, adjustment: ast.value(id, "tail") ? 1 : 2 }; }
  if (ts && ownsAnnotation(name)) {
    const annotation = ast.ref(id, "typeAnnotation");
    if (annotation !== NONE) { return { id: annotation, field, adjustment: 0 }; }
  }
  if (field === "end" && ts &&
      (name === "BindingIdentifier" || name === "ArrayPattern" || name === "ObjectPattern") &&
      ast.value(id, "optional")) {
    return { id, field, adjustment: 1 };
  }
  return { id, field, adjustment: 0 };
}
/** Native storage is the default; generated maps contain projection exceptions. */
export function fieldRule(isTs: boolean, kind: number, field: string): number {
  const column = fieldIds[field];
  return typeof column === "number"
    ? fieldRules[fieldRuleIds[Number(isTs) * NODE_KIND_COUNT + kind] * FIELD_COUNT + column]
    : ReadRule.Native;
}

/** Kinds whose public decorators list has no native storage. */
export function hasEmptyDecorators(isTs: boolean, kind: number): boolean {
  return (kindFlags[kind] & 1) !== 0 || kind === NodeKind.TsIndexSignatureParameter ||
    (kind === NodeKind.FormalParameter && !isTs);
}

export function projectedField(ast: AstView, id: number, field: string, kind = ast.kind(id)): unknown {
  if (field === "type") { return staticTypes[kind] ?? nodeType(ast, id, layouts[kind].name); }
  if (field === "start" || field === "end") {
    const location = spanLocation(ast, id, field, kind);
    const value = location.field === "start" ? ast.start(location.id) : ast.end(location.id);
    return (location.additional === undefined ? value : Math.min(value, ast.start(location.additional))) + location.adjustment;
  }
  const rule = fieldRule(ast._isTs, kind, field);
  switch (rule) {
    case ReadRule.Native: return ast.value(id, field, kind);
    case ReadRule.False: return false;
    case ReadRule.True: return true;
    case ReadRule.Null: return null;
    case ReadRule.List: return ast.list(id, field);
    case ReadRule.Params: return ast.list(id, field);
    case ReadRule.Pattern: return ast.ref(id, "initializer") === NONE ? undefined : ast.value(id, "pattern");
    case ReadRule.Initializer: return ast.ref(id, "initializer") === NONE ? undefined : ast.value(id, "initializer");
    case ReadRule.SourceType: return ast._editor ? enums.SourceType[ast._editor.flags & 3] : ast.meta.sourceType;
    case ReadRule.Directive: return ast.value(id, "value");
    case ReadRule.NumericRaw: return ast.value(id, "raw") ?? String(ast.value(id, "value"));
    case ReadRule.BigIntRaw: return `${ast.value(id, "raw")}n`;
    case ReadRule.BigInt: return readBigInt(ast, id);
    case ReadRule.BigIntText: return readBigInt(ast, id).toString();
    case ReadRule.BooleanRaw: return ast.value(id, "value") ? "true" : "false";
    case ReadRule.NullRaw: return "null";
    case ReadRule.RegexRaw: return `/${ast.value(id, "pattern")}/${ast.value(id, "flags")}`;
    case ReadRule.RegexFields: return new RegexFields(ast, id);
    case ReadRule.RegexValue:
      try { return new RegExp(ast.value(id, "pattern") as string, ast.value(id, "flags") as string); }
      catch { return null; }
    case ReadRule.TemplateValue: return new TemplateValue(ast, id);
    case ReadRule.Text: return ast.value(id, "value");
    case ReadRule.Init: return "init";
    case ReadRule.Optional: return ast.value(id, "optional") ?? false;
    case ReadRule.ParameterOptional:
      return ast.ref(id, "initializer") === NONE ? ast.value(id, "optional") : false;
    case ReadRule.ParameterAnnotation: {
      if (ast.ref(id, "initializer") !== NONE) { return null; }
      const pattern = ast.ref(id, "pattern");
      return ast.value(pattern, "typeAnnotation", ast.kind(pattern));
    }
    case ReadRule.DelegateName: {
      if (kind === NodeKind.TsIndexSignatureParameter)
        { return ast.value(ast.ref(id, "name"), "name"); }
      if (ast.ref(id, "initializer") !== NONE) { return undefined; }
      const pattern = ast.ref(id, "pattern");
      return ast.kind(pattern) === NodeKind.BindingIdentifier
        ? ast.value(pattern, "name") : undefined;
    }
    case ReadRule.DelegateList: {
      if (ast.ref(id, "initializer") !== NONE) { return undefined; }
      const pattern = ast.ref(id, "pattern"), patternKind = ast.kind(pattern);
      return (field === "elements" && patternKind === NodeKind.ArrayPattern) ||
        (field === "properties" && patternKind === NodeKind.ObjectPattern)
        ? ast.list(pattern, field) : undefined;
    }
    case ReadRule.EmptyList: return ast.list(id, field);
    case ReadRule.ModuleBody: return ast.ref(id, "body") === NONE ? undefined : ast.value(id, "body");
    case ReadRule.Global: return "global";
    case ReadRule.This: return "this";
    case ReadRule.Value: return "value";
    default: return undefined;
  }
}

export function projectedList(ast: AstView, id: number, key: string, cursor: ListCursor): boolean {
  const row = ast.row(id);
  if (row === -1) { ast.validateId(id); }
  const view = row === -1 ? ast._view : ast._arena!.view!;
  const kind = row === -1 ? view.getUint8(ast._kinds + id) : view.getUint8(row + 8);
  if (kind === NodeKind.FormalParameters && key === "params") {
    const [start, end] = ast.range(id, "items");
    cursor.suffix = ast.ref(id, "rest") | 0;
    cursor.setRange(0, id, start, end);
    return true;
  }
  if (key === "decorators" && hasEmptyDecorators(ast._isTs, kind)) {
    cursor.start0 = cursor.end0 = 0;
    return true;
  }
  const fieldId = cursorFieldIds[key];
  if (typeof fieldId !== "number") { return false; }
  const rule = cursorRules[cursorRuleIds[Number(ast._isTs) * NODE_KIND_COUNT + kind] * CURSOR_FIELD_COUNT + fieldId];
  const payload = row === -1 ? ast._payloads + id * 16 : row + 16;
  if (rule === 0) { return false; }
  const recipe = cursorRecipes[rule];
  if (recipe[0] === -2) { cursor.start0 = cursor.end0 = 0; return true; }
  if (recipe[4] & 4) {
    const params = ast.ref(id, "params", true);
    if (params === NONE) { cursor.start0 = cursor.end0 = 0; return true; }
    const paramsRow = ast.row(params);
    cursor.prefix = ast._isTs ? ast.ref(id, "thisParam") | 0 : -1;
    cursor.suffix = ast.ref(params, "rest") | 0;
    return cursorRange(
      ast,
      params,
      ast._payloadAt(params, paramsRow),
      recipe[0], recipe[1],
      cursor,
      0,
      paramsRow === -1 ? ast._view : ast._arena!.view!,
    );
  }
  if (recipe[4] & 2) { cursor.suffix = ast.ref(id, "rest") | 0; }
  cursorRange(ast, id, payload, recipe[0], recipe[1], cursor, 0, view);
  if (recipe[2] !== -2)
    { cursorRange(ast, id, payload, recipe[2], recipe[3], cursor, 1, view); }
  return true;
}
function programStart(ast: AstView, id: number): SpanLocation {
  for (const key of ["directives", "body"]) {
    const [start, end] = ast.range(id, key);
    if (start === end) { continue; }
    const first = ast.extraChild(id, start, false);
    const declaration = ast.ref(first, "declaration");
    const owner = declaration === NONE ? first : declaration;
    const field = ast.fieldOffset(owner, "decorators");
    if (field) {
      const [start, end] = ast.range(owner, "decorators");
      if (start !== end)
        { return { id: first, field: "start", adjustment: 0, additional: ast.extraChild(owner, start, false) }; }
    }
    return { id: first, field: "start", adjustment: 0 };
  }
  return { id, field: "end", adjustment: 0 };
}

function ownsAnnotation(name: string): boolean {
  return name === "BindingIdentifier" || name === "ArrayPattern" || name === "ObjectPattern" || name === "AssignmentPattern" || name === "BindingRestElement";
}

function readBigInt(ast: AstView, id: number) {
  return BigInt((ast.value(id, "raw") as string).replaceAll("_", ""));
}
function cursorRange(ast: AstView, id: number, payload: number, pointer: number, slot: number, cursor: ListCursor, part: number, payloadView: DataView): boolean {
  let offset = payload + slot, view = payloadView;
  if (pointer >= 0) {
    const extraStart = payloadView.getUint32(payload + pointer, true);
    const index = extraStart + slot;
    const limit = extraStart < ast._extraCount ? ast._extraCount : ast.extraDataCount;
    if (index + 2 > limit) { throw new RangeError("AST extra-data reference out of bounds"); }
    view = index < ast._extraCount ? ast._view : ast._arena!.view!;
    offset = index < ast._extraCount ? ast._extra + index * 4 : ast._arena!.extraAddress(index);
  }
  const start = view.getUint32(offset, true),
    end = view.getUint32(offset + 4, true);
  if (start > end || end > ast.extraDataCount)
    { throw new RangeError("AST list out of bounds"); }
  cursor.setRange(part, id, start, end);
  return true;
}

