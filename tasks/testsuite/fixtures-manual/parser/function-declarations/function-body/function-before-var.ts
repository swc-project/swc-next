/*---
description: TypeScript function bodies reject var and function merges
negative:
  phase: early
  type: SyntaxError
---*/
function outer() {
  function f() {}
  var f;
}
