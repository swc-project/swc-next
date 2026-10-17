/*---
description: Re-export names reject a lone low surrogate.
esid: sec-module-semantics-static-semantics-early-errors
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/

export { "\uDC00" as "\uDC00" } from "mod";
