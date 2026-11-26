// FormatJS transform ported and modified from swc-project/plugins (Apache-2.0).
// AST views: numeric walker callbacks, NodeView field setters and
// AstList replacement.
import { createWalker } from '@swc-next/walker2';
import { Evaluator, UNKNOWN } from './evaluate.js';
import { unwrap } from '../common/formatjs/evaluate-common.js';
import { interpolate } from '../common/formatjs/interpolate.js';
import {
  FormatJSError, MessageParser, descriptorKeys, formatJSNames, formatJSSettings,
  pragmaMetadata, sourceLocator, validateOptions,
} from '../common/formatjs/options.js';
import { jsonToAst, literal, property, expressionContainer, jsxAttribute, jsxValue } from './ast.js';

export { FormatJSError };

export function formatjs(options = {}) {
  validateOptions(options);
  return createFormatjsPlugin(formatJSSettings(options));
}

/** Internal: build the compiled plugin from validated and normalized settings. */
export function createFormatjsPlugin(settings) {
  const { componentNames, functionNames } = formatJSNames(settings);

  function fail(node, state, cause) {
    throw new FormatJSError(cause ? 'Message failed to parse.' : 'Messages must be statically evaluate-able for extraction.', node, state.filename, cause);
  }

  function entries(node, jsx, state) {
    const { evaluator } = state;
    const result = [];
    for (const item of (jsx ? node.attributes : node.properties)) {
      let key, value;
      if (jsx) {
        if (item.type !== 'JsxAttribute' || !item.value) { continue; }
        key = item.name.type === 'JsxNamespacedName' ? item.name.name.name : item.name.name;
        if (!descriptorKeys.has(key)) { continue; }
        value = item.value.type === 'JsxExpressionContainer' ? item.value.expression : item.value;
      } else {
        if (item.type !== 'ObjectProperty' || item.kind !== 'Init' || item.method) { continue; }
        key = evaluator.key(item);
        if (key === UNKNOWN) { fail(item, state); }
        if (!descriptorKeys.has(key)) { continue; }
        value = item.value;
      }
      result.push({ item, key, value });
    }
    return result;
  }

  function processDescriptor(node, state, jsx = false) {
    const { evaluator, filename, locate, metadata } = state;
    if (!node || (!jsx && node.type !== 'ObjectExpression')) { return; }
    try {
      const fields = entries(node, jsx, state);
      const defaultField = fields.findLast(field => field.key === 'defaultMessage');
      // Descriptors provided through spreads, ID-only references and precompiled ASTs need no extraction.
      if (defaultField && unwrap(defaultField.value)?.type === 'ArrayExpression') { return; }
      const descriptor = {};
      let hashDescription;
      for (const { value, key } of fields) {
        const object = evaluator.object(value);
        if (object && key === 'description') {
          descriptor.description = evaluator.description(object, false);
          hashDescription = evaluator.description(object, true);
          continue;
        }
        const evaluated = evaluator.value(value);
        if (typeof evaluated !== 'string') { fail(value, state); }
        descriptor[key] = evaluated;
        if (key === 'description') { hashDescription = evaluated; }
      }
      if (!defaultField) { return; }
      const message = settings.preserveWhitespace ? descriptor.defaultMessage
        : descriptor.defaultMessage.replace(/\p{White_Space}+/gu, ' ').trim();
      let ast;
      try { ast = MessageParser.parse(message, { captureLocation: false, shouldParseSkeletons: true, requiresOtherClause: true }); }
      catch (error) { fail(defaultField.value, state, error); }
      let id = descriptor.id;
      if (id === undefined && message) {
        const content = hashDescription && typeof hashDescription === 'object' ? `${message}#${JSON.stringify(hashDescription)}`
          : hashDescription ? `${message}#${hashDescription}` : message;
        id = interpolate(settings.idInterpolationPattern, content, filename);
      }
      const extracted = { id: id ?? '', defaultMessage: message };
      if (descriptor.description !== undefined) { extracted.description = descriptor.description; }
      if (locate) { extracted.loc = locate(node); }

      // Finish evaluation before editing, so throws:false leaves failed descriptors intact.
      const byItem = new Map(fields.map(field => [field.item.nodeId, field]));
      const output = [];
      let hasId = false;
      for (const item of (jsx ? node.attributes : node.properties)) {
        const field = byItem.get(item.nodeId);
        if (!field) { output.push(item); continue; }
        if (field.key === 'description' || (field.key === 'defaultMessage' && settings.removeDefaultMessage)) { continue; }
        let value;
        if (field.key === 'id') {
          hasId = true;
          value = literal(id, field.value);
        } else if (field.key === 'defaultMessage') {
          if (jsx && !settings.ast && unwrap(field.value).type !== 'BinaryExpression') {
            output.push(item);
            continue;
          }
          value = settings.ast ? jsonToAst(ast, field.value) : literal(message, field.value);
        }
        if (jsx) {
          item.value = settings.ast && field.key === 'defaultMessage'
            ? expressionContainer(value, item.value) : jsxValue(value);
        } else {
          item.value = value;
          item.shorthand = false;
        }
        output.push(item);
      }
      if (!hasId && id !== undefined) {
        output.unshift(jsx ? jsxAttribute('id', literal(id, node), node) : property('id', literal(id, node), node));
      }
      if (jsx) { node.attributes = output; }
      else { node.properties = output; }
      metadata.messages.push(extracted);
    } catch (error) {
      if (settings.throws || !(error instanceof FormatJSError)) { throw error; }
    }
  }

  function callName(callee, evaluator) {
    if (callee.type === 'IdentifierReference') { return evaluator.binding(callee)?.imported ?? callee.name; }
    if (callee.type === 'MemberExpression' && !callee.computed && !callee.optional) {
      const name = callee.property.name;
      if (functionNames.has(name)) { return name; }
      if (callee.object.type === 'IdentifierReference' && evaluator.binding(callee.object)?.imported === '*') { return name; }
    }
  }

  const walker = createWalker({
    VariableDeclarator: { leave(ctx) {
      const node = ctx.node;
      if (node.id.type === 'BindingIdentifier' && node.init) { ctx.state.evaluator.store(node.id, node.init); }
    } },
    AssignmentExpression: { leave(ctx) {
      const node = ctx.node;
      const { evaluator } = ctx.state;
      if (node.left.type === 'IdentifierReference' && node.operator === 'Assign') { evaluator.store(node.left, node.right); }
      else { evaluator.invalidate(node.left); }
    } },
    UpdateExpression: { leave(ctx) { ctx.state.evaluator.invalidate(ctx.node.argument); } },
    CallExpression: { leave(ctx) {
      const node = ctx.node;
      const state = ctx.state;
      const { evaluator, filename } = state;
      if (node.optional) { return; }
      const name = callName(node.callee, evaluator);
      if (name === 'defineMessage' || name === 'defineMessages') {
        const first = node.arguments.get(0);
        if (first?.type !== 'ObjectExpression') {
          if (settings.throws) { throw new FormatJSError(`${name} must be called with an object expression.`, node, filename); }
          return;
        }
        if (name === 'defineMessage') { processDescriptor(first, state); }
        else {
          for (const field of first.properties) {
            if (field.type === 'ObjectProperty' && field.kind === 'Init' && !field.method && !field.shorthand) { processDescriptor(field.value, state); }
          }
        }
      } else if (functionNames.has(name)) { processDescriptor(node.arguments.get(0), state); }
    } },
    JsxOpeningElement: { leave(ctx) {
      const node = ctx.node;
      const state = ctx.state;
      const { evaluator } = state;
      let name;
      if (node.name.type === 'JsxIdentifier') {
        // Jsx identifiers use the same lexical scope lookup as regular identifiers.
        name = evaluator.binding(node.name)?.imported ?? node.name.name;
      } else if (node.name.type === 'JsxMemberExpression') { name = node.name.property.name; }
      if (componentNames.has(name)) { processDescriptor(node, state, true); }
    } },
  });

  return function formatjsPlugin(program, context = {}) {
    const filename = context.filename ?? 'unknown.js';
    if (settings.extractSourceLocation && typeof context.source !== 'string') {
      throw new TypeError('extractSourceLocation requires context.source (provided by transformSync)');
    }
    const metadata = { messages: [], meta: pragmaMetadata(context.comments ?? [], settings.pragma) };
    context.metadata ??= {};
    context.metadata.formatjs = metadata;
    const { sourceType } = program.ast.meta;
    if (sourceType === 'script' || sourceType === 'commonjs') { return; }
    const locate = settings.extractSourceLocation ? sourceLocator(context.source, filename) : undefined;
    const evaluator = new Evaluator(program);

    walker.walk(program, { evaluator, filename, locate, metadata });
  };
}

export default formatjs;
