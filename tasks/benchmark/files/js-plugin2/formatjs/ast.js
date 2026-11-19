// NodeView node construction for the AST views.
// Every `original` argument is a NodeView from the same AstView, so there is
// no representation branch here.
import { NodeKind } from '@swc-next/core/ffi2';

function allocate(kind, original) {
  const ast = original.ast;
  return ast.new_node(kind, ast.utf8Offset(original.start), ast.utf8Offset(original.end));
}

export function literal(value, original) {
  const kind = value === null ? NodeKind.NullLiteral : typeof value === 'string' ? NodeKind.StringLiteral
    : typeof value === 'number' ? NodeKind.NumericLiteral : typeof value === 'boolean' ? NodeKind.BooleanLiteral : null;
  if (kind === null) { throw new TypeError('Unsupported message literal'); }
  const node = allocate(kind, original);
  if (kind !== NodeKind.NullLiteral) { node.value = value; }
  return node;
}

export function identifier(name, original) {
  const node = allocate(NodeKind.IdentifierReference, original);
  node.name = name;
  return node;
}

export function property(name, value, original, stringKey = false) {
  const key = stringKey ? literal(name, original) : identifier(name, original);
  const node = allocate(NodeKind.ObjectProperty, original);
  node.set('key', key);
  node.set('value', value);
  return node;
}

export function expressionContainer(value, original) {
  const node = allocate(NodeKind.JsxExpressionContainer, original);
  node.set('expression', value);
  return node;
}

export function jsxAttribute(name, value, original) {
  const key = allocate(NodeKind.JsxIdentifier, original);
  key.name = name;
  const node = allocate(NodeKind.JsxAttribute, original);
  node.set('name', key);
  node.set('value', jsxValue(value));
  return node;
}

export function jsxValue(value) {
  if (value.type === 'Literal' && typeof value.value === 'string') {
    if (value.raw.slice(1, -1) !== value.value) { value.set('raw', ''); }
  }
  return value;
}

export function jsonToAst(value, original) {
  if (Array.isArray(value)) {
    const elements = value.map(item => jsonToAst(item, original));
    const node = allocate(NodeKind.ArrayExpression, original);
    node.elements.push(...elements);
    return node;
  }
  if (value !== null && typeof value === 'object') {
    const properties = Object.entries(value).map(([key, item]) => property(key, jsonToAst(item, original), original, true));
    const node = allocate(NodeKind.ObjectExpression, original);
    node.properties.push(...properties);
    return node;
  }
  return literal(value, original);
}
