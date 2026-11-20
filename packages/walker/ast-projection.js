// Numeric child access over the AST projection.
//
// Every function here returns numbers only. Nothing calls `ast.node`,
// `ast.value`, `ast.getField`, `ast.list` or `ast.nativeNode`, so reading a
// child never creates a NodeView, an ordinary AST node or an AstList.
//
// `readChildId` mirrors the generated NodeView getters for node-valued fields.
// `readList` mirrors `projectedList` and adds the FormalParameter DelegateList
// case, where the wrapper is the visible parent but the list range belongs to
// its hidden ArrayPattern/ObjectPattern child.
import {
  NONE, fieldRule, ReadRule, NodeKind,
} from "@swc-next/ast/internal";

// Only these projected rules can expose a child. Every other non-native rule
// (Missing, Null, scalars, ...) makes readChildId return NONE for this kind.
export function isProjectedChildRule(rule) {
  return rule === ReadRule.Pattern || rule === ReadRule.Initializer ||
    rule === ReadRule.ParameterAnnotation || rule === ReadRule.ModuleBody;
}

// Numeric counterpart of `ast.ref` with the kind already resolved. List and
// scalar fields return NONE instead of throwing, so callers may probe a key
// before deciding between readChildId and readList.
function nativeChild(ast, id, key, kind) {
  const field = ast.fieldOffset(id, key, kind);
  if (field === null || field.type !== "node") {
    return NONE;
  }
  return ast.childAt(id, field.offset, field.optional, field.view);
}

/**
 * Resolve the visible child node for `key`.
 * @returns {number} the native child ID, or NONE when the projected field is
 * not a child node (scalar, list, absent, or hidden by a wrapper).
 */
export function readChildId(ast, id, key, kind = ast._kindAt(id)) {
  switch (fieldRule(ast._isTs, kind, key)) {
    case ReadRule.Native:
      return nativeChild(ast, id, key, kind);
    case ReadRule.Pattern:
      return nativeChild(ast, id, "initializer", kind) === NONE
        ? NONE
        : nativeChild(ast, id, "pattern", kind);
    case ReadRule.Initializer:
      return nativeChild(ast, id, "initializer", kind);
    case ReadRule.ParameterAnnotation: {
      if (nativeChild(ast, id, "initializer", kind) !== NONE) {
        return NONE;
      }
      const pattern = nativeChild(ast, id, "pattern", kind);
      return pattern === NONE
        ? NONE
        : nativeChild(ast, pattern, "typeAnnotation", ast._kindAt(pattern));
    }
    case ReadRule.ModuleBody:
      return nativeChild(ast, id, "body", kind);
    default:
      return NONE;
  }
}

/**
 * Fill `cursor` with the numeric range(s) of the visible list for `key`.
 * The cursor only needs `prefix`, `suffix`, `start0`, `end0` and
 * `setRange(part, owner, start, end)`; owners are node IDs.
 * `kind` may be passed when the caller already resolved it.
 * @returns {boolean} true when the projection exposes a list (possibly empty).
 */
export function readList(ast, id, key, cursor, kind = ast._kindAt(id)) {
  if (kind === NodeKind.FormalParameter && (key === "elements" || key === "properties")) {
    // DelegateList: a wrapper with an initializer exposes AssignmentPattern,
    // whose list keys are absent; otherwise the shape key comes from the
    // hidden pattern and the range is owned by that pattern.
    if (nativeChild(ast, id, "initializer", kind) !== NONE) {
      return false;
    }
    const pattern = nativeChild(ast, id, "pattern", kind);
    const patternKind = ast._kindAt(pattern);
    if ((key === "elements" && patternKind === NodeKind.ArrayPattern) ||
      (key === "properties" && patternKind === NodeKind.ObjectPattern)) {
      return ast.readListCursor(pattern, key, cursor);
    }
    return false;
  }
  return ast.readListCursor(id, key, cursor);
}
