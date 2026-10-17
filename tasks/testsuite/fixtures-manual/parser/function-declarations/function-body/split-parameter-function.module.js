/*---
description: JavaScript body functions may shadow parameters in a separate scope
flags: [module]
---*/
function outer(f = 1) {
  function f() {}
}
