import { createWalker } from "@swc-next/walker/ast";

function count(_nodeId, context) {
  context.state.count++;
}

const walker = createWalker({
  FunctionDeclaration: count,
  FunctionExpression: count,
  ArrowFunctionExpression: count,
});

/** Count implementations, including method values; type-only signatures are excluded. */
export function countFunctions() {
  return function countFunctionsPlugin(program, context) {
    const state = { count: 0 };
    walker.walk(program.ast, program.nodeId, state);
    context.metadata.functionCount = state.count;
  };
}
