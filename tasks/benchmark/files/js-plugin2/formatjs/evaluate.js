// Static evaluation ported and modified from swc-project/plugins (Apache-2.0).
// AST views: numeric node IDs key the scopes; no NodeView is created
// just to look a binding up.
import { createWalker, NONE } from '@swc-next/walker2';
import { UNKNOWN, unwrap, functions } from '../common/formatjs/evaluate-common.js';
import { EvaluatorBase } from '../common/formatjs/evaluator-base.js';

export { UNKNOWN, unwrap };

const emptyEvaluator = Symbol('empty evaluator');

function declarePattern(pattern, scope) {
  if (!pattern) { return; }
  const type = pattern.type;
  // Identifier patterns dominate declarations; keep them out of the switch.
  if (type === 'Identifier') {
    if (!scope.bindings.has(pattern.name)) { scope.bindings.set(pattern.name, { imported: undefined }); }
    return;
  }
  switch (type) {
    case 'AssignmentPattern': declarePattern(pattern.left, scope); break;
    case 'RestElement': declarePattern(pattern.argument, scope); break;
    case 'ArrayPattern': for (const element of pattern.elements) { declarePattern(element, scope); } break;
    case 'ObjectPattern':
      for (const property of pattern.properties) {
        declarePattern(property.type === 'RestElement' ? property.argument : property.value, scope);
      }
      break;
    case 'TSParameterProperty': declarePattern(pattern.parameter, scope); break;
  }
}

const scopeWalker = createWalker({
  enter(ctx) {
    const state = ctx.state;
    const type = ctx.type;
    // Identifiers dominate the walk; they only need a scope entry.
    if (type === 'Identifier' || type === 'JSXIdentifier') {
      const scope = state.scope;
      if (scope !== undefined) { state.evaluator.scopeIds[ctx.nodeId] = scope.index + 1; }
      return;
    }
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
      scope = { parent: scope, bindings: new Map(), function: isFunction || type === 'Program' || type === 'StaticBlock', index: state.evaluator.scopeTable.length };
      state.evaluator.scopeTable.push(scope);
      state.opened[ctx.nodeId] = 1;
    }
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
      if (pattern.type === 'Identifier') {
        const init = kind === 'var' ? NONE : ctx.ast.ref(ctx.nodeId, 'init');
        state.declarations.push({ binding: target.bindings.get(pattern.name), init: init === NONE ? null : init });
      }
    }
    state.scope = scope;
  },
  // Leave uses only the numeric identity, so an untouched node is never
  // materialized.
  leave(ctx) {
    const state = ctx.state;
    if (state.opened[ctx.nodeId] !== 0) { state.scope = state.scope.parent; }
  },
});

export class Evaluator extends EvaluatorBase {
  // Keep the evaluator layout alive without retaining a program or its buffer.
  static #shape = new this(emptyEvaluator);
  // Dense node IDs make typed arrays cheaper than Set/Map on the scope path.
  scopeIds = null;
  scopeTable = [];
  opened = null;

  constructor(program) {
    super();
    if (program === emptyEvaluator) { return; }
    this.ast = program.ast;
    const nodeCount = program.ast.totalNodeCount;
    this.scopeIds = new Uint32Array(nodeCount);
    this.opened = new Uint8Array(nodeCount);
    const state = { evaluator: this, scope: undefined, opened: this.opened, declarations: [] };
    scopeWalker.walk(program, state);
    for (const { binding, init } of state.declarations) { if (init !== null) { this.values.set(binding, init); } }
  }

  binding(node) {
    const name = node.name;
    const index = this.scopeIds[node.nodeId];
    if (index === 0) { return; }
    let scope = this.scopeTable[index - 1];
    while (scope) {
      if (scope.bindings.has(name)) { return scope.bindings.get(name); }
      scope = scope.parent;
    }
  }
}
