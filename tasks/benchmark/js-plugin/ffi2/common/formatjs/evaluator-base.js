// Static evaluation over numeric node IDs. Subclasses provide `binding` and
// set `ast`; stored bindings materialize a view only when evaluation reads them.
import { UNKNOWN, unwrap } from './evaluate-common.js';

export class EvaluatorBase {
  scopes = new Map();
  values = new Map();
  ast = null;

  // Stored bindings are node IDs; `undefined` means the binding is unknown.
  resolve(stored) { return stored === undefined ? undefined : this.ast.node(stored); }

  store(identifier, expression) {
    const binding = this.binding(identifier);
    if (!binding) { return; }
    const previous = unwrap(this.resolve(this.values.get(binding)));
    const next = unwrap(expression);
    if (previous?.type === 'ObjectExpression' && next?.type !== 'ObjectExpression') {
      // React Compiler caches immutable object literals in $[number].
      if (next?.type === 'MemberExpression' && !next.optional && next.computed
        && unwrap(next.object)?.type === 'IdentifierReference' && unwrap(next.object).name === '$'
        && unwrap(next.property)?.type === 'NumericLiteral') { return; }
      this.values.delete(binding);
    } else if (expression) { this.values.set(binding, expression.nodeId); }
    else { this.values.delete(binding); }
  }

  invalidate(target) {
    target = unwrap(target);
    if (!target) { return; }
    if (target.type === 'MemberExpression') {
      let base = target.object;
      while (unwrap(base)?.type === 'MemberExpression') { base = unwrap(base).object; }
      target = unwrap(base);
    }
    if (target.type === 'IdentifierReference' || target.type === 'BindingIdentifier') { this.values.delete(this.binding(target)); }
    else if (target.type === 'ObjectPattern') {
      for (const p of target.properties) { this.invalidate(p.value); }
      this.invalidate(target.rest);
    }
    else if (target.type === 'ArrayPattern') {
      for (const p of target.elements) { this.invalidate(p); }
      this.invalidate(target.rest);
    }
    else if (target.type === 'AssignmentPattern') { this.invalidate(target.left); }
    else if (target.type === 'BindingRestElement') { this.invalidate(target.argument); }
  }

  object(expression, depth = 0) {
    if (depth > 16) { return; }
    const node = unwrap(expression);
    if (node?.type === 'ObjectExpression') { return node; }
    if (node?.type === 'IdentifierReference') { return this.object(this.resolve(this.values.get(this.binding(node))), depth + 1); }
    if (node?.type === 'MemberExpression') { return this.object(this.member(node, depth + 1), depth + 1); }
  }

  key(node, depth = 0) {
    if (!node.computed && (node.key ?? node.property)?.type === 'IdentifierName') { return (node.key ?? node.property).name; }
    const value = this.value(node.key ?? node.property, depth + 1);
    return value === UNKNOWN ? UNKNOWN : String(value);
  }

  member(node, depth) {
    if (depth > 16 || node.optional) { return; }
    const object = this.object(node.object, depth + 1);
    const key = this.key(node, depth + 1);
    if (!object || key === UNKNOWN) { return; }
    for (let i = object.properties.length - 1; i >= 0; i--) {
      const property = object.properties.get(i);
      if (property.type !== 'ObjectProperty') { return; }
      const propertyKey = this.key(property, depth + 1);
      if (propertyKey === UNKNOWN) { return; }
      if (propertyKey !== key) { continue; }
      if (property.kind !== 'Init' || property.method) { return; }
      return property.value;
    }
  }

  value(expression, depth = 0) {
    if (depth > 16) { return UNKNOWN; }
    const node = unwrap(expression);
    if (!node) { return UNKNOWN; }
    switch (node.type) {
      case 'StringLiteral':
      case 'NumericLiteral':
      case 'BooleanLiteral': return node.value;
      case 'NullLiteral': return null;
      case 'BigIntLiteral': return BigInt(node.raw.replaceAll('_', ''));
      case 'IdentifierReference': return this.value(this.resolve(this.values.get(this.binding(node))), depth + 1);
      case 'MemberExpression': return this.value(this.member(node, depth + 1), depth + 1);
      case 'TemplateLiteral': {
        let result = '';
        for (let i = 0; i < node.quasis.length; i++) {
          const quasi = node.quasis.get(i);
          if (quasi.isCookedUndefined) { return UNKNOWN; }
          result += quasi.cooked;
          if (i < node.expressions.length) {
            const value = this.value(node.expressions.get(i), depth + 1);
            if (value === UNKNOWN) { return UNKNOWN; }
            result += String(value);
          }
        }
        return result;
      }
      case 'UnaryExpression': {
        const value = this.value(node.argument, depth + 1);
        if (value === UNKNOWN) { return UNKNOWN; }
        if (node.operator === 'LogicalNot') { return !value; }
        if (typeof value === 'string') { return UNKNOWN; }
        if (node.operator === 'Negate') { return -value; }
        if (node.operator === 'Positive' && typeof value !== 'bigint') { return +value; }
        return UNKNOWN;
      }
      case 'BinaryExpression': {
        if (node.operator !== 'Add') { return UNKNOWN; }
        const left = this.value(node.left, depth + 1);
        const right = this.value(node.right, depth + 1);
        if (left === UNKNOWN || right === UNKNOWN) { return UNKNOWN; }
        if (typeof left === 'string' || typeof right === 'string') { return String(left) + String(right); }
        if (typeof left === 'bigint' || typeof right === 'bigint') { return UNKNOWN; }
        return Number(left) + Number(right);
      }
      case 'ConditionalExpression': {
        const test = this.value(node.test, depth + 1);
        return test === UNKNOWN ? UNKNOWN : this.value(test ? node.consequent : node.alternate, depth + 1);
      }
      default: return UNKNOWN;
    }
  }

  description(object, forHash) {
    const entries = new Map();
    for (const property of object.properties) {
      if (property.type !== 'ObjectProperty' || property.computed || property.shorthand || property.method || property.kind !== 'Init') { continue; }
      const key = this.key(property);
      const value = forHash ? this.value(property.value) : property.value.type === 'StringLiteral' ? property.value.value : UNKNOWN;
      if (key === UNKNOWN || value === UNKNOWN || (!forHash && typeof value !== 'string')) { continue; }
      if (typeof value === 'string' || typeof value === 'boolean' || typeof value === 'number') { entries.set(key, value); }
    }
    return Object.fromEntries(forHash ? [...entries].sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0) : entries);
  }
}
