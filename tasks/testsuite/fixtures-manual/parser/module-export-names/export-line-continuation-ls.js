/*---
description: Export names pair surrogate escapes across LS line continuations.
esid: sec-module-semantics-static-semantics-early-errors
flags: [module]
---*/

const x = 1;
export { x as "\uD800\ \uDC00" };
