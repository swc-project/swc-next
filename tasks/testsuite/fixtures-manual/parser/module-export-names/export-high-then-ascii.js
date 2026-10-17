/*---
description: Export names reject a high surrogate followed by a continuation and a non-surrogate.
esid: sec-module-semantics-static-semantics-early-errors
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/

const x = 1;
export { x as "\uD800\
\u0041" };
