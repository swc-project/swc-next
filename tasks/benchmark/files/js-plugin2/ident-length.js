import { createWalker } from "@swc-next/walker2";

function identifier(ctx) { ctx.state.length += ctx.node.name.length; }

const walker = createWalker({
  BindingIdentifier: identifier,
  IdentifierName: identifier,
  IdentifierReference: identifier,
  LabelIdentifier: identifier,
});

/** Sum identifier name lengths; TS `this` parameters have no name storage. */
export function identLength() {
  return function identLengthPlugin(program, context) {
    const state = { length: 0 };
    walker.walk(program, state);
    context.metadata.identifierLength = state.length;
  };
}

export default identLength;
