// Ported and modified from swc-project/plugins (Apache-2.0).
// Upstream adapted this transform from Codex-/swc-plugin-import-meta-env by Alex Miller.
// AST views: numeric walker callbacks and a NodeView field setter.
import { createWalker } from '@swc-next/walker2';
import { identifier } from './formatjs/ast.js';

const walker = createWalker({
  MemberExpression(ctx) {
    const node = ctx.node;
    const object = node.object;
    if (!node.computed && !node.optional && node.property.type === 'Identifier' && node.property.name === 'env'
      && object.type === 'MetaProperty' && object.meta.name === 'import' && object.property.name === 'meta') {
      node.object = identifier('process', object);
    }
  },
});

export function importMetaEnv() {
  return function importMetaEnvPlugin(program) { walker.walk(program); };
}

export default importMetaEnv;
