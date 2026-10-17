/*---
description: JavaScript body functions can share a simple parameter name
flags: [module]
---*/
function outer(f) {
  function f() {}
}
