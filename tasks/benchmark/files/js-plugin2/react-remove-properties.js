// Ported and modified from swc-project/plugins (Apache-2.0).
// AST views: AstList replacement for JSX attributes.
import { createWalker } from '@swc-next/walker2';
import { reactRemovePropertiesPatterns } from './common/react-remove-properties.js';

export function reactRemoveProperties(options = true) {
  return createReactRemovePropertiesPlugin(reactRemovePropertiesPatterns(options));
}

/** Internal: build the compiled plugin from validated patterns. */
export function createReactRemovePropertiesPlugin(patterns) {
  if (patterns === null) { return function reactRemovePropertiesPlugin() {}; }
  const walker = createWalker({
    JsxOpeningElement(ctx) {
      const node = ctx.node;
      const attributes = node.attributes.filter(attribute => attribute.type !== 'JsxAttribute'
        || attribute.name.type !== 'JsxIdentifier'
        || !patterns.some(pattern => pattern.test(attribute.name.name)));
      node.attributes = attributes;
    },
  });
  return function reactRemovePropertiesPlugin(program) { walker.walk(program); };
}

export default reactRemoveProperties;
