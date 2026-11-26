// Pure evaluation helpers used by the FormatJS plugin.
// Helpers for native SWC Next node views.
export const UNKNOWN = Symbol('not statically evaluable');

export const wrappers = new Set([
  'ParenthesizedExpression',
  'TsAsExpression',
  'TsTypeAssertion',
  'TsNonNullExpression',
  'TsSatisfiesExpression',
  'TsInstantiationExpression',
]);

export const functions = new Set([
  'Function',
  'ArrowFunctionExpression',
  'TsFunction',
]);

export function unwrap(node) {
  while (node && wrappers.has(node.type)) { node = node.expression; }
  return node;
}
