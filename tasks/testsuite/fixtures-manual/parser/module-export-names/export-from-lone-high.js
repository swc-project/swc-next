/*---
description: Re-export names reject a lone high surrogate.
esid: sec-module-semantics-static-semantics-early-errors
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/

export { "\uD800" as "\uD800" } from "mod";
