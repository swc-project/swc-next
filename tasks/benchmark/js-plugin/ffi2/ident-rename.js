import { createWalker } from "@swc-next/core-ffi2";

function identifier(ctx) { ctx.node.name = "a"; }

const walker = createWalker({
  BindingIdentifier: identifier,
  IdentifierName: identifier,
  IdentifierReference: identifier,
  LabelIdentifier: identifier,
});

/** Rename every identifier to "a"; TS `this` parameters have no name storage. */
export function identRename() {
  return function identRenamePlugin(program) {
    walker.walk(program);
  };
}

export default identRename;
