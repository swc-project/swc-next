// Representation-independent static evaluation used by the FormatJS
// implementations. Subclasses provide `binding`, which resolves an identifier
// against the representation-specific scope map.
import { UNKNOWN, unwrap } from './evaluate-common.js';

export class EvaluatorBase {
  scopes = new Map();
  values = new Map();

  store(identifier, expression) {
    const binding = this.binding(identifier);
    if (!binding) { return; }
    const previous = unwrap(this.values.get(binding));
    const next = unwrap(expression);
    if (previous?.type === 'ObjectExpression' && next?.type !== 'ObjectExpression') {
      // React Compiler caches immutable object literals in $[number].
      if (next?.type === 'MemberExpression' && !next.optional && next.computed
        && unwrap(next.object)?.type === 'Identifier' && unwrap(next.object).name === '$'
        && unwrap(next.property)?.type === 'Literal' && typeof unwrap(next.property).value === 'number') { return; }
      this.values.delete(binding);
    } else if (expression) { this.values.set(binding, expression); }
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
    if (target.type === 'Identifier') { this.values.delete(this.binding(target)); }
    else if (target.type === 'ObjectPattern') { for (const p of target.properties) { this.invalidate(p.value ?? p.argument); } }
    else if (target.type === 'ArrayPattern') { for (const p of target.elements) { this.invalidate(p); } }
    else if (target.type === 'AssignmentPattern') { this.invalidate(target.left); }
    else if (target.type === 'RestElement') { this.invalidate(target.argument); }
  }

  object(expression, depth = 0) {
    if (depth > 16) { return; }
    const node = unwrap(expression);
    if (node?.type === 'ObjectExpression') { return node; }
    if (node?.type === 'Identifier') { return this.object(this.values.get(this.binding(node)), depth + 1); }
    if (node?.type === 'MemberExpression') { return this.object(this.member(node, depth + 1), depth + 1); }
  }

  key(node, depth = 0) {
    if (!node.computed && (node.key ?? node.property)?.type === 'Identifier') { return (node.key ?? node.property).name; }
    const value = this.value(node.key ?? node.property, depth + 1);
    return value === UNKNOWN ? UNKNOWN : String(value);
  }

  member(node, depth) {
    if (depth > 16 || node.optional) { return; }
    const object = this.object(node.object, depth + 1);
    const key = this.key(node, depth + 1);
    if (!object || key === UNKNOWN) { return; }
    for (let i = object.properties.length - 1; i >= 0; i--) {
      const property = object.properties[i];
      if (property.type !== 'Property') { return; }
      const propertyKey = this.key(property, depth + 1);
      if (propertyKey === UNKNOWN) { return; }
      if (propertyKey !== key) { continue; }
      if (property.kind !== 'init' || property.method) { return; }
      return property.value;
    }
  }

  value(expression, depth = 0) {
    if (depth > 16) { return UNKNOWN; }
    const node = unwrap(expression);
    if (!node) { return UNKNOWN; }
    switch (node.type) {
      case 'Literal': return node.regex ? UNKNOWN : node.bigint ? BigInt(node.bigint) : node.value;
      case 'Identifier': return this.value(this.values.get(this.binding(node)), depth + 1);
      case 'MemberExpression': return this.value(this.member(node, depth + 1), depth + 1);
      case 'TemplateLiteral': {
        let result = '';
        for (let i = 0; i < node.quasis.length; i++) {
          const cooked = node.quasis[i].value.cooked;
          if (cooked === null) { return UNKNOWN; }
          result += cooked;
          if (i < node.expressions.length) {
            const value = this.value(node.expressions[i], depth + 1);
            if (value === UNKNOWN) { return UNKNOWN; }
            result += String(value);
          }
        }
        return result;
      }
      case 'UnaryExpression': {
        const value = this.value(node.argument, depth + 1);
        if (value === UNKNOWN) { return UNKNOWN; }
        if (node.operator === '!') { return !value; }
        if (typeof value === 'string') { return UNKNOWN; }
        if (node.operator === '-') { return -value; }
        if (node.operator === '+' && typeof value !== 'bigint') { return +value; }
        return UNKNOWN;
      }
      case 'BinaryExpression': {
        if (node.operator !== '+') { return UNKNOWN; }
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
      if (property.type !== 'Property' || property.computed || property.shorthand || property.method || property.kind !== 'init') { continue; }
      const key = this.key(property);
      const value = forHash ? this.value(property.value) : property.value.type === 'Literal' ? property.value.value : UNKNOWN;
      if (key === UNKNOWN || value === UNKNOWN || (!forHash && typeof value !== 'string')) { continue; }
      if (typeof value === 'string' || typeof value === 'boolean' || typeof value === 'number') { entries.set(key, value); }
    }
    return Object.fromEntries(forHash ? [...entries].sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0) : entries);
  }
}
