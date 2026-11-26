// Plain ESTree node construction for main's materialized AST.

function span(original) {
  return { start: original.start, end: original.end };
}

export function literal(value, original) {
  const node = { type: 'Literal', value, ...span(original) };
  node.raw = value === null ? 'null' : typeof value === 'string' ? JSON.stringify(value) : String(value);
  return node;
}

export function identifier(name, original) {
  return { type: 'Identifier', name, ...span(original) };
}

export function property(name, value, original, stringKey = false) {
  const key = stringKey ? literal(name, original) : identifier(name, original);
  return { type: 'Property', key, value, kind: 'init', computed: false, method: false, shorthand: false, ...span(original) };
}

export function expressionContainer(value, original) {
  return { type: 'JSXExpressionContainer', expression: value, ...span(original) };
}

export function jsxAttribute(name, value, original) {
  const key = { type: 'JSXIdentifier', name, ...span(original) };
  return { type: 'JSXAttribute', name: key, value: jsxValue(value), ...span(original) };
}

export function jsxValue(value) {
  if (value.type === 'Literal' && typeof value.value === 'string') {
    if (value.raw.slice(1, -1) !== value.value) { value.raw = ''; }
  }
  return value;
}

export function jsonToAst(value, original) {
  if (Array.isArray(value)) {
    return { type: 'ArrayExpression', elements: value.map(item => jsonToAst(item, original)), ...span(original) };
  }
  if (value !== null && typeof value === 'object') {
    return {
      type: 'ObjectExpression',
      properties: Object.entries(value).map(([key, item]) => property(key, jsonToAst(item, original), original, true)),
      ...span(original),
    };
  }
  return literal(value, original);
}
