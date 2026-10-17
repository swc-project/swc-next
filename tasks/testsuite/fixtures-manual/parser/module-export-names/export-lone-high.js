/*---
description: Export names reject a lone high surrogate.
esid: sec-module-semantics-static-semantics-early-errors
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/

const x = 1;
export { x as "\uD800" };
