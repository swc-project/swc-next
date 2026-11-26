import { createWalker } from "@swc-next/walker";

const walker = createWalker({
  Identifier(node, context) {
    if (node.name !== "this" || context.key !== "params") {
      node.name = "a";
    }
  },
});

/** Rename every identifier to "a"; TS `this` parameters have no name storage. */
export function identRename() {
  return function identRenamePlugin(program) {
    walker.walk(program);
  };
}

export default identRename;
