/*---
description: Re-export names pair surrogate escapes across PS line continuations.
esid: sec-module-semantics-static-semantics-early-errors
flags: [module]
---*/

export { "\uD800\ \uDC00" as "\uD800\ \uDC00" } from "mod";
