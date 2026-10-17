/*---
description: Re-export names reject a high surrogate followed by a continuation and a non-surrogate.
esid: sec-module-semantics-static-semantics-early-errors
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/

export { "\uD800\
\u0041" as "\uD800\
\u0041" } from "mod";
