/*---
description: TypeScript function bodies reject var and function merges
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
function outer() {
  var f;
  function f() {}
}
