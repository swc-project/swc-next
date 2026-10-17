/*---
description: TypeScript function bodies reject var and function merges
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
function outer() {
  function f() {}
  var f;
}
