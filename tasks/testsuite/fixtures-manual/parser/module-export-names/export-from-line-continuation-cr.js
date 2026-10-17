/*---
description: Re-export names pair surrogate escapes across CR line continuations.
esid: sec-module-semantics-static-semantics-early-errors
flags: [module]
---*/

export { "\uD800\\uDC00" as "\uD800\\uDC00" } from "mod";
