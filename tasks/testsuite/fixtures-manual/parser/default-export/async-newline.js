/*---
description: Default-exported async functions retain module bindings after a newline
flags: [module]
---*/
export default
async function f() {}
export { f };
