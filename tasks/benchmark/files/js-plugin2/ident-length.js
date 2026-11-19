import * as core from "@swc-next/core/ffi2";
import { createWalker } from "@swc-next/walker2";

const NodeKind = core.NodeKind;

function count(context) {
  if (NodeKind === undefined ? context.type !== "TsThisParameter" : context.kind !== NodeKind.TsThisParameter) {
    context.state.length += context.node.name.length;
  }
}

const walker = createWalker({ Identifier: count });

/** Sum identifier name lengths; TS `this` parameters have no name storage. */
export function identLength() {
  return function identLengthPlugin(program, context) {
    const state = { length: 0 };
    walker.walk(program, state);
    context.metadata.identifierLength = state.length;
  };
}

export default identLength;
