/*---
description: Default-exported async functions retain duplicate-binding checks after a newline
flags: [module]
negative: { phase: parse, type: SyntaxError }
---*/
let f;
export default
async function f() {}
