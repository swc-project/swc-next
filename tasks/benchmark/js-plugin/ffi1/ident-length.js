import { createWalker } from "@swc-next/walker";

function count(node, context) {
  if (node.name !== "this" || context.key !== "params") {
    context.state.length += node.name.length;
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
