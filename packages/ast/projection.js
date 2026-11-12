// ESTree projection over the generated native field layout. Reading a field
// never recursively materializes a subtree; ownership comes from buffer links.
import { layouts, NodeKind } from "./schema.js";

const NONE = 0xffffffff;
const sourceTypes = ["module", "script", "commonjs", "unambiguous"];
const literals = new Set([
  "StringLiteral",
  "NumericLiteral",
  "BigIntLiteral",
  "BooleanLiteral",
  "NullLiteral",
  "RegExpLiteral",
]);
const patterns = new Set([
  "BindingIdentifier",
  "ArrayPattern",
  "ObjectPattern",
  "IdentifierName",
]);
const functionTypes = new Set([
  "Function",
  "TsFunction",
  "ArrowFunctionExpression",
  "TsFunctionType",
  "TsConstructorType",
  "TsMethodSignature",
  "TsCallSignatureDeclaration",
  "TsConstructSignatureDeclaration",
]);
const specialKeys = {
  Program: ["sourceType hashbang body", ""],
  FunctionBody: ["body", ""],
  TsModuleBlock: ["body", ""],
  FormalParameter: ["left right", "decorators optional typeAnnotation"],
  FormalParameterRest: ["argument", "decorators optional typeAnnotation value"],
  VariableDeclarator: ["id init", "definite"],
  CatchClause: ["param body", ""],
  Function: [
    "id generator async params body expression",
    "typeParameters returnType declare",
  ],
  TsFunction: [
    "id generator async params body expression typeParameters returnType declare",
    "",
  ],
  ArrowFunctionExpression: [
    "id generator async params body expression",
    "typeParameters returnType",
  ],
  Directive: ["expression directive", ""],
  StringLiteral: ["value raw", ""],
  NumericLiteral: ["value raw", ""],
  BigIntLiteral: ["value raw bigint", ""],
  BooleanLiteral: ["value raw", ""],
  NullLiteral: ["value raw", ""],
  RegExpLiteral: ["value raw regex", ""],
  TemplateElement: ["value tail", ""],
  Class: [
    "decorators id superClass body",
    "typeParameters superTypeArguments implements abstract declare",
  ],
  MethodDefinition: [
    "decorators key value kind computed static",
    "override optional accessibility",
  ],
  TsMethodDefinition: [
    "decorators key value kind computed static",
    "override optional accessibility",
  ],
  PropertyDefinition: [
    "decorators key value computed static",
    "typeAnnotation declare override optional definite readonly accessibility",
  ],
  UnaryExpression: ["operator prefix argument", ""],
  BindingProperty: ["kind key value method shorthand computed", "optional"],
  ArrayPattern: ["elements", "decorators optional typeAnnotation"],
  ObjectPattern: ["properties", "decorators optional typeAnnotation"],
  JsxText: ["value raw", ""],
  TsFunctionType: ["typeParameters params returnType", ""],
  TsConstructorType: ["abstract typeParameters params returnType", ""],
  TsMethodSignature: [
    "key computed optional kind typeParameters params returnType accessibility readonly static",
    "",
  ],
  TsCallSignatureDeclaration: ["typeParameters params returnType", ""],
  TsConstructSignatureDeclaration: ["typeParameters params returnType", ""],
  TsMappedType: [
    "key constraint nameType typeAnnotation optional readonly",
    "",
  ],
  TsModuleDeclaration: ["id kind declare global body", ""],
  TsGlobalDeclaration: ["id body kind declare global", ""],
  TsThisParameter: ["decorators name optional typeAnnotation", ""],
};

const keyTables = [false, true].map((isTs) =>
  layouts.map((layout) => {
    const special = specialKeys[layout.name];
    const fields = special
      ? `${special[0]} ${isTs ? special[1] : ""}`.trim().split(/\s+/)
      : [
          ...layout.keys.filter((key) => isTs || !layout.ts.includes(key)),
          ...(isTs ? Object.keys(layout.extras) : []),
        ];
    return Object.freeze(["type", "start", "end", ...new Set(fields)]);
  }),
);
const isTs = (ast) => ast._isTs;

// Public arrays and literal objects have numeric owner/field keys. The field
// ordinals depend only on the schema; walking never allocates an index string.
const mirrorFields = Object.create(null);
let mirrorFieldCount = 0;
for (const keys of keyTables[1]) {
  for (const key of keys) {
    if (mirrorFields[key] === undefined) mirrorFields[key] = mirrorFieldCount++;
  }
}
const mirrorStride = 2 ** Math.ceil(Math.log2(mirrorFieldCount));
export function mirrorKey(id, field) {
  return id * mirrorStride + mirrorFields[field];
}

export function normalizeId(ast, id) {
  for (let steps = 0; steps <= ast.nodeCount; steps++) {
    const kind = ast.kind(id);
    if (
      kind === NodeKind.FormalParameter &&
      ast.ref(id, "initializer") === NONE
    )
      id = ast.ref(id, "pattern");
    else if (kind === NodeKind.TsIndexSignatureParameter)
      id = ast.ref(id, "name");
    else return id;
  }
  throw new RangeError("Cyclic AST projection");
}

export function parameters(ast, owner, params = ast.ref(owner, "params")) {
  const prefix =
    owner !== NONE && isTs(ast) ? ast.ref(owner, "thisParam") : NONE;
  return ast.list(
    [ast.range(params, "items")],
    prefix,
    ast.ref(params, "rest"),
    owner === NONE ? params : owner,
    "params",
    params,
  );
}

export function projectedKeys(ast, id) {
  const kind = ast.kind(id),
    keys = keyTables[Number(isTs(ast))][kind];
  return kind === NodeKind.TsModuleDeclaration && ast.ref(id, "body") === NONE
    ? keys.filter((key) => key !== "body")
    : keys;
}

const dynamicTypes = new Set([
  "Function",
  "TsFunction",
  "Class",
  "MethodDefinition",
  "TsMethodDefinition",
  "PropertyDefinition",
]);
const staticTypeOverrides = {
  FunctionBody: "BlockStatement",
  FormalParameter: "AssignmentPattern",
  FormalParameterRest: "RestElement",
  Directive: "ExpressionStatement",
  BindingProperty: "Property",
  TsGlobalDeclaration: "TSModuleDeclaration",
  TsThisParameter: "Identifier",
};
export const staticTypes = Object.freeze(
  layouts.map((layout) =>
    dynamicTypes.has(layout.name)
      ? null
      : literals.has(layout.name)
        ? "Literal"
        : (staticTypeOverrides[layout.name] ?? layout.type),
  ),
);

export function nodeType(ast, id, name) {
  if (literals.has(name)) return "Literal";
  switch (name) {
    case "FunctionBody":
      return "BlockStatement";
    case "FormalParameter":
      return "AssignmentPattern";
    case "FormalParameterRest":
      return "RestElement";
    case "Directive":
      return "ExpressionStatement";
    case "BindingProperty":
      return "Property";
    case "Function":
      return ast.value(id, "functionType");
    case "TsFunction": {
      const parent = ast.parentId(id);
      return parent !== NONE && ast.kind(parent) === NodeKind.TsMethodDefinition
        ? "TSEmptyBodyFunctionExpression"
        : "TSDeclareFunction";
    }
    case "Class":
      return ast.value(id, "classType");
    case "MethodDefinition":
    case "TsMethodDefinition":
      return isTs(ast) && ast.value(id, "abstract")
        ? "TSAbstractMethodDefinition"
        : "MethodDefinition";
    case "PropertyDefinition": {
      const accessor = ast.value(id, "accessor");
      return isTs(ast) && ast.value(id, "abstract")
        ? accessor
          ? "TSAbstractAccessorProperty"
          : "TSAbstractPropertyDefinition"
        : accessor
          ? "AccessorProperty"
          : "PropertyDefinition";
    }
    case "TsGlobalDeclaration":
      return "TSModuleDeclaration";
    case "TsThisParameter":
      return "Identifier";
    default:
      return layouts[ast.kind(id)].type;
  }
}

function programStart(ast, id) {
  for (const key of ["directives", "body"]) {
    const [start, end] = ast.range(id, key);
    if (start === end) continue;
    const first = ast.extraChild(id, start, false);
    const declaration = ast.ref(first, "declaration");
    const owner = declaration === NONE ? first : declaration;
    const field = ast.fieldOffset(owner, "decorators");
    if (field) {
      const [start, end] = ast.range(owner, "decorators");
      if (start !== end)
        return Math.min(
          ast.start(first),
          ast.start(ast.extraChild(owner, start, false)),
        );
    }
    return ast.start(first);
  }
  return ast.end(id);
}

function patternOwner(ast, id, name) {
  if (!patterns.has(name)) return null;
  const parent = ast.parentId(id);
  if (parent === NONE) return null;
  const owner = layouts[ast.kind(parent)].name;
  const field =
    owner === "VariableDeclarator"
      ? "id"
      : owner === "CatchClause"
        ? "param"
        : owner === "FormalParameter"
          ? "pattern"
          : owner === "TsIndexSignatureParameter"
            ? "name"
            : null;
  return field && ast.ref(parent, field) === id
    ? { id: parent, name: owner }
    : null;
}

function span(ast, id, name, field) {
  const ts = isTs(ast);
  if (field === "start") {
    if (name === "Program" && ts) return programStart(ast, id);
    if (name === "FormalParameter") return ast.start(ast.ref(id, "pattern"));
    if (name === "TemplateElement" && ts) return ast.start(id) - 1;
    const owner = ts && patternOwner(ast, id, name);
    if (owner && owner.name === "TsIndexSignatureParameter")
      return ast.start(owner.id);
    return ast.start(id);
  }
  if (name === "TemplateElement" && ts)
    return ast.end(id) + (ast.value(id, "tail") ? 1 : 2);
  const owner = patternOwner(ast, id, name);
  if (owner?.name === "FormalParameter") {
    if (ast.ref(owner.id, "initializer") === NONE) return ast.end(owner.id);
    if (ts) {
      const annotation = ast.ref(owner.id, "typeAnnotation");
      return annotation !== NONE
        ? ast.end(annotation)
        : ast.end(id) + (ast.value(owner.id, "optional") ? 1 : 0);
    }
  } else if (ts && owner) {
    if (owner.name === "TsIndexSignatureParameter") return ast.end(owner.id);
    const annotation = ast.ref(owner.id, "typeAnnotation");
    if (annotation !== NONE) return ast.end(annotation);
  }
  return ast.end(id);
}

function readNative(ast, id, field, kind) {
  return ast.value(id, field, kind);
}
function readEmptyList(ast, id, field) {
  return ast.list([], NONE, NONE, id, field);
}
const readFalse = () => false;
const readTrue = () => true;
const readNull = () => null;

function readBody(ast, id) {
  return ast.list(
    [ast.range(id, "directives"), ast.range(id, "body")],
    NONE,
    NONE,
    id,
    "body",
  );
}
function readRestList(ast, id, field) {
  return ast.list(
    [ast.range(id, field)],
    NONE,
    ast.ref(id, "rest"),
    id,
    field,
  );
}
function readParameters(ast, id) {
  return parameters(ast, id);
}
function readBigInt(ast, id) {
  return BigInt(ast.value(id, "raw").replaceAll("_", ""));
}
function readPatternField(ast, id, field, kind) {
  const layout = layouts[kind];
  const owner = patternOwner(ast, id, layout.name);
  if (owner) {
    if (field === "typeAnnotation") return ast.value(owner.id, field);
    if (
      owner.name === "FormalParameter" &&
      (field === "optional" || ast.ref(owner.id, "initializer") === NONE)
    ) {
      return ast.value(owner.id, field);
    }
  }
  const extra = layout.extras[field];
  return Array.isArray(extra)
    ? readEmptyList(ast, id, field)
    : Object.hasOwn(layout.extras, field)
      ? extra
      : ast.value(id, field, kind);
}

// Only fields whose ESTree representation differs from native storage need a
// custom reader. These are normal functions, shared by all node views.
const fieldProjections = {
  Program: { sourceType: (ast) => ast._editor ? sourceTypes[ast._editor.flags & 3] : ast.meta.sourceType, body: readBody },
  FunctionBody: { body: readBody },
  TsModuleBlock: { body: readBody },
  ArrayPattern: { elements: readRestList },
  ObjectPattern: { properties: readRestList },
  FormalParameter: {
    left: (ast, id) => ast.value(id, "pattern"),
    right: (ast, id) => ast.value(id, "initializer"),
    optional: readFalse,
    typeAnnotation: readNull,
  },
  FormalParameterRest: { optional: readFalse, value: readNull },
  Function: { expression: readFalse, declare: readFalse },
  TsFunction: { expression: readFalse, body: readNull },
  ArrowFunctionExpression: { id: readNull, generator: readFalse },
  Directive: { directive: (ast, id) => ast.value(id, "value") },
  NumericLiteral: {
    raw: (ast, id) => ast.value(id, "raw") ?? String(ast.value(id, "value")),
  },
  BigIntLiteral: {
    raw: (ast, id) => `${ast.value(id, "raw")}n`,
    value: readBigInt,
    bigint: (ast, id) => readBigInt(ast, id).toString(),
  },
  BooleanLiteral: {
    raw: (ast, id) => (ast.value(id, "value") ? "true" : "false"),
  },
  NullLiteral: { raw: () => "null", value: readNull },
  RegExpLiteral: {
    raw: (ast, id) => `/${ast.value(id, "pattern")}/${ast.value(id, "flags")}`,
    regex(ast, id) {
      return ast.objectValue(id, "regex", () => ({
        pattern: ast.value(id, "pattern"),
        flags: [...ast.value(id, "flags")].sort().join(""),
      }));
    },
    value(ast, id) {
      return ast.objectValue(id, "value", () => {
        try {
          return new RegExp(ast.value(id, "pattern"), ast.value(id, "flags"));
        } catch {
          return null;
        }
      });
    },
  },
  TemplateElement: {
    value(ast, id) {
      return ast.objectValue(id, "value", () => ({
        raw: ast.value(id, "raw"),
        cooked: ast.value(id, "isCookedUndefined")
          ? null
          : ast.value(id, "cooked"),
      }));
    },
  },
  UnaryExpression: { prefix: readTrue },
  BindingProperty: {
    kind: () => "init",
    method: readFalse,
    optional: readFalse,
  },
  JsxText: { value: (ast, id) => ast.text(id), raw: (ast, id) => ast.text(id) },
  TsMethodSignature: {
    accessibility: readNull,
    readonly: readFalse,
    static: readFalse,
  },
  TsMappedType: { optional: (ast, id) => ast.value(id, "optional") ?? false },
  TsModuleDeclaration: {
    global: readFalse,
    body: (ast, id) =>
      ast.ref(id, "body") === NONE ? undefined : ast.value(id, "body"),
  },
  TsGlobalDeclaration: { kind: () => "global", global: readTrue },
  TsThisParameter: {
    name: () => "this",
    decorators: readEmptyList,
    optional: readFalse,
  },
};

// Resolve language visibility, defaults and projection precedence once. Null
// prototypes also make unknown names (including Object.prototype names) absent.
const fieldReaders = [false, true].map((ts, language) =>
  layouts.map((layout, kind) => {
    const readers = Object.create(null);
    const special = fieldProjections[layout.name];
    for (const field of keyTables[language][kind]) {
      let reader = readNative;
      if (ts && Object.hasOwn(layout.extras, field)) {
        const extra = layout.extras[field];
        reader = Array.isArray(extra) ? readEmptyList : () => extra;
      }
      if (special && Object.hasOwn(special, field)) reader = special[field];
      if (field === "params" && functionTypes.has(layout.name))
        reader = readParameters;
      if (
        ts &&
        patterns.has(layout.name) &&
        (field === "typeAnnotation" ||
          field === "optional" ||
          field === "decorators")
      ) {
        reader = readPatternField;
      }
      readers[field] = reader;
    }
    return readers;
  }),
);

export function projectedField(ast, id, field, kind = ast.kind(id)) {
  if (field === "type")
    return staticTypes[kind] ?? nodeType(ast, id, layouts[kind].name);
  if (field === "start" || field === "end")
    return span(ast, id, layouts[kind].name, field);
  const reader = fieldReaders[Number(ast._isTs)][kind][field];
  return reader === undefined ? undefined : reader(ast, id, field, kind);
}

function listRange(layout, key) {
  const field = layout.fields[key];
  return field?.[2] === "list"
    ? { pointer: field[0] ? layout.extra : -1, offset: field[1] }
    : null;
}

// Fixed-shape recipes contain only layout metadata, never AST values. The hot
// path reads up to two ranges and an optional rest node without rediscovering
// native fields, checking language key sets, or allocating descriptor objects.
const cursorLayouts = [false, true].map((ts, language) =>
  layouts.map((layout, kind) => {
    const recipes = Object.create(null);
    const name = layout.name;
    for (const key of keyTables[language][kind]) {
      const recipe = {
        first: null,
        second: null,
        rest: false,
        parameters: false,
      };
      if (key === "params" && functionTypes.has(name)) {
        recipe.parameters = true;
        recipe.first = listRange(layouts[NodeKind.FormalParameters], "items");
      } else if (
        key === "body" &&
        (name === "Program" ||
          name === "FunctionBody" ||
          name === "TsModuleBlock")
      ) {
        recipe.first = listRange(layout, "directives");
        recipe.second = listRange(layout, "body");
      } else if (
        (name === "ArrayPattern" && key === "elements") ||
        (name === "ObjectPattern" && key === "properties")
      ) {
        recipe.first = listRange(layout, key);
        recipe.rest = true;
      } else if (ts && key === "decorators" && patterns.has(name)) {
        continue; // Its owner can supply the public array instead.
      } else if (
        (name === "TsThisParameter" && key === "decorators") ||
        (ts &&
          Object.hasOwn(layout.extras, key) &&
          Array.isArray(layout.extras[key]))
      ) {
        // A null first range denotes the canonical empty list.
      } else {
        recipe.first = listRange(layout, key);
        if (recipe.first === null) continue;
      }
      recipes[key] = recipe;
    }
    return recipes;
  }),
);

function cursorRange(ast, id, payload, range, cursor, part, payloadView) {
  let offset = payload + range.offset, view = payloadView;
  if (range.pointer >= 0) {
    const index = payloadView.getUint32(payload + range.pointer, true) + range.offset;
    if (index >= ast.extraDataCount) throw new RangeError("AST extra-data reference out of bounds");
    view = index < ast._extraCount ? ast._view : ast._arena.view;
    offset = index < ast._extraCount ? ast._extra + index * 8 : ast._arena.extraAddress(index);
  }
  const start = view.getUint32(offset, true),
    end = view.getUint32(offset + 4, true);
  if (start > end || end > ast.extraDataCount)
    throw new RangeError("AST list out of bounds");
  cursor.setRange(part, id, start, end);
  return true;
}

export function projectedList(ast, id, key, cursor) {
  const row = ast.row(id);
  const raw = row === -1 ? ast.rawId(id) : 0;
  const view = row === -1 ? ast._view : ast._arena.view;
  const kind = row === -1 ? view.getUint8(ast._kinds + raw) : view.getUint8(row + 8);
  const recipes = cursorLayouts[Number(ast._isTs)][kind];
  if (recipes === undefined) throw new RangeError("Unknown AST node kind");
  const recipe = recipes[key];
  if (recipe === undefined) return false;
  if (recipe.first === null) {
    cursor.start0 = cursor.end0 = 0;
    return true;
  }
  if (recipe.parameters) {
    const params = ast.ref(id, "params");
    const paramsRow = ast.row(params);
    cursor.prefix = ast._isTs ? ast.ref(id, "thisParam") | 0 : -1;
    cursor.suffix = ast.ref(params, "rest") | 0;
    return cursorRange(
      ast,
      params,
      paramsRow === -1 ? ast.offset(params) : paramsRow + 16,
      recipe.first,
      cursor,
      0,
      paramsRow === -1 ? ast._view : ast._arena.view,
    );
  }
  if (recipe.rest) cursor.suffix = ast.ref(id, "rest") | 0;
  const payload = row === -1 ? ast._payloads + raw * 16 : row + 16;
  cursorRange(ast, id, payload, recipe.first, cursor, 0, view);
  if (recipe.second !== null)
    cursorRange(ast, id, payload, recipe.second, cursor, 1, view);
  return true;
}
