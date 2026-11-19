import * as core from "@swc-next/core/ffi2";
import { createWalker } from "@swc-next/walker2";

const NodeKind = core.NodeKind;

const walker = createWalker({
  Identifier(context) {
    if (NodeKind === undefined ? context.type !== "TsThisParameter" : context.kind !== NodeKind.TsThisParameter) {
      context.node.name = "a";
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
