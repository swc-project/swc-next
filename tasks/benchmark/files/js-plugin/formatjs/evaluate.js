// Static evaluation for main's materialized AST. Scope maps are keyed by node
// objects; no numeric node IDs or AST views are involved.
import { createWalker } from '@swc-next/walker';
import { UNKNOWN, unwrap, functions } from '../common/formatjs/evaluate-common.js';
import { EvaluatorBase } from '../common/formatjs/evaluator-base.js';

export { UNKNOWN, unwrap };

const emptyEvaluator = Symbol('empty evaluator');

function declarePattern(pattern, scope) {
  if (!pattern) { return; }
  switch (pattern.type) {
    case 'Identifier':
      if (!scope.bindings.has(pattern.name)) { scope.bindings.set(pattern.name, { imported: undefined }); }
      break;
    case 'AssignmentPattern': declarePattern(pattern.left, scope); break;
    case 'RestElement': declarePattern(pattern.argument, scope); break;
    case 'ArrayPattern': for (const element of pattern.elements) { declarePattern(element, scope); } break;
    case 'ObjectPattern':
      for (const property of pattern.properties) { declarePattern(property.type === 'RestElement' ? property.argument : property.value, scope); }
      break;
    case 'TSParameterProperty': declarePattern(pattern.parameter, scope); break;
  }
}

const scopeWalker = createWalker({
  enter(node, ctx) {
    const state = ctx.state;
    const type = node.type;
    let scope = state.scope;
    const declaresId = type === 'FunctionDeclaration' || type === 'ClassDeclaration'
      || type === 'TSDeclareFunction' || type === 'TSEnumDeclaration';
    const isFunction = functions.has(type);
    const opens = type === 'Program' || isFunction || type === 'BlockStatement' || type === 'CatchClause'
      || type === 'ForStatement' || type === 'ForInStatement' || type === 'ForOfStatement'
      || type === 'SwitchStatement' || type === 'ClassDeclaration' || type === 'ClassExpression'
      || type === 'StaticBlock';
    const target = declaresId || isFunction || type === 'ClassExpression' || type === 'CatchClause'
      || type === 'ImportDeclaration' || type === 'VariableDeclarator' ? node : null;
    if (declaresId) { declarePattern(target.id, scope); }
    if (opens) {
      scope = { parent: scope, bindings: new Map(), function: isFunction || type === 'Program' || type === 'StaticBlock' };
      state.opened.add(node);
    }
    if (type === 'Identifier' || type === 'JSXIdentifier') { state.evaluator.scopes.set(node, scope); }
    if (isFunction) {
      declarePattern(target.id, scope);
      for (const parameter of target.params) { declarePattern(parameter, scope); }
    }
    if (type === 'ClassExpression') { declarePattern(target.id, scope); }
    if (type === 'CatchClause') { declarePattern(target.param, scope); }
    if (type === 'ImportDeclaration') {
      for (const specifier of target.specifiers) {
        declarePattern(specifier.local, scope);
        if (target.source.value === 'react-intl' && target.importKind !== 'type' && specifier.importKind !== 'type') {
          scope.bindings.get(specifier.local.name).imported = specifier.type === 'ImportNamespaceSpecifier'
            ? '*' : specifier.imported?.name ?? specifier.imported?.value;
        }
      }
    }
    if (type === 'VariableDeclarator') {
      let targetScope = scope;
      const kind = ctx.parent.kind;
      if (kind === 'var') { while (!targetScope.function) { targetScope = targetScope.parent; } }
      const pattern = target.id;
      declarePattern(pattern, targetScope);
      if (pattern.type === 'Identifier') { state.declarations.push({ binding: targetScope.bindings.get(pattern.name), init: kind === 'var' ? null : target.init }); }
    }
    state.scope = scope;
  },
  leave(node, ctx) {
    const state = ctx.state;
    if (state.opened.has(node)) { state.scope = state.scope.parent; }
  },
});

export class Evaluator extends EvaluatorBase {
  static #shape = new this(emptyEvaluator);

  constructor(program) {
    super();
    if (program === emptyEvaluator) { return; }
    const state = { evaluator: this, scope: undefined, opened: new Set(), declarations: [] };
    scopeWalker.walk(program, state);
    for (const { binding, init } of state.declarations) { if (init) { this.values.set(binding, init); } }
  }

  binding(node) {
    const name = node.name;
    let scope = this.scopes.get(node);
    while (scope) {
      if (scope.bindings.has(name)) { return scope.bindings.get(name); }
      scope = scope.parent;
    }
  }
}
