/*---
description: Export names reject a lone low surrogate.
esid: sec-module-semantics-static-semantics-early-errors
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/

const x = 1;
export { x as "\uDC00" };
