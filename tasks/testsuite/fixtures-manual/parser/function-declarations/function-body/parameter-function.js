/*---
description: JavaScript body functions can share a simple parameter name
---*/
function outer(f) {
  function f() {}
}
