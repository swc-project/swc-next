// Static evaluation ported and modified from swc-project/plugins (Apache-2.0).
// AST views: numeric node IDs key the scopes; no NodeView is created
// just to look a binding up.
import { createWalker } from '@swc-next/walker/ast';
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
  enter(nodeId, ctx) {
    const state = ctx.state;
    const type = ctx.type;
    let scope = state.scope;
    const declaresId = type === 'FunctionDeclaration' || type === 'ClassDeclaration'
      || type === 'TSDeclareFunction' || type === 'TSEnumDeclaration';
    const isFunction = functions.has(type);
    const opens = type === 'Program' || isFunction || type === 'BlockStatement' || type === 'CatchClause'
      || type === 'ForStatement' || type === 'ForInStatement' || type === 'ForOfStatement'
      || type === 'SwitchStatement' || type === 'ClassDeclaration' || type === 'ClassExpression'
      || type === 'StaticBlock';
    // Field reads share one view per node; type-only nodes never create one.
    const node = declaresId || isFunction || type === 'ClassExpression' || type === 'CatchClause'
      || type === 'ImportDeclaration' || type === 'VariableDeclarator' ? ctx.node : null;
    if (declaresId) { declarePattern(node.id, scope); }
    if (opens) {
      scope = { parent: scope, bindings: new Map(), function: isFunction || type === 'Program' || type === 'StaticBlock' };
      state.opened.add(nodeId);
    }
    // binding() only looks up identifiers, so only they need a scope entry.
    if (type === 'Identifier' || type === 'JSXIdentifier') { state.evaluator.scopes.set(nodeId, scope); }
    if (isFunction) {
      declarePattern(node.id, scope);
      for (const parameter of node.params) { declarePattern(parameter, scope); }
    }
    if (type === 'ClassExpression') { declarePattern(node.id, scope); }
    if (type === 'CatchClause') { declarePattern(node.param, scope); }
    if (type === 'ImportDeclaration') {
      for (const specifier of node.specifiers) {
        declarePattern(specifier.local, scope);
        if (node.source.value === 'react-intl' && node.importKind !== 'type' && specifier.importKind !== 'type') {
          scope.bindings.get(specifier.local.name).imported = specifier.type === 'ImportNamespaceSpecifier'
            ? '*' : specifier.imported?.name ?? specifier.imported?.value;
        }
      }
    }
    if (type === 'VariableDeclarator') {
      let target = scope;
      const kind = ctx.parent.kind;
      if (kind === 'var') { while (!target.function) { target = target.parent; } }
      const pattern = node.id;
      declarePattern(pattern, target);
      if (pattern.type === 'Identifier') { state.declarations.push({ binding: target.bindings.get(pattern.name), init: kind === 'var' ? null : node.init }); }
    }
    state.scope = scope;
  },
  // Leave uses only the numeric identity, so an untouched node is never
  // materialized.
  leave(nodeId, ctx) {
    const state = ctx.state;
    if (state.opened.has(nodeId)) { state.scope = state.scope.parent; }
  },
});

export class Evaluator extends EvaluatorBase {
  // Keep the evaluator layout alive without retaining a program or its buffer.
  static #shape = new this(emptyEvaluator);

  constructor(program) {
    super();
    if (program === emptyEvaluator) { return; }
    const state = { evaluator: this, scope: undefined, opened: new Set(), declarations: [] };
    scopeWalker.walk(program.ast, program.nodeId, state);
    for (const { binding, init } of state.declarations) { if (init) { this.values.set(binding, init); } }
  }

  binding(node) {
    const name = node.name;
    let scope = this.scopes.get(node.nodeId);
    while (scope) {
      if (scope.bindings.has(name)) { return scope.bindings.get(name); }
      scope = scope.parent;
    }
  }
}
