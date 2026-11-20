// Pure evaluation helpers used by the FormatJS plugin.
// Nothing here depends on whether a node is a plain object or a NodeView.
export const UNKNOWN = Symbol('not statically evaluable');

export const wrappers = new Set([
  'ParenthesizedExpression',
  'TSAsExpression',
  'TSTypeAssertion',
  'TSNonNullExpression',
  'TSSatisfiesExpression',
  'TSInstantiationExpression',
]);

export const functions = new Set([
  'FunctionDeclaration',
  'FunctionExpression',
  'ArrowFunctionExpression',
  'TSDeclareFunction',
  'TSEmptyBodyFunctionExpression',
]);

export function unwrap(node) {
  while (node && wrappers.has(node.type)) { node = node.expression; }
  return node;
}
