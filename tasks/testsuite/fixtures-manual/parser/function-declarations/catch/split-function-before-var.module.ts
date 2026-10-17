/*---
description: Vars conflict with preceding functions in a separate catch body scope
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
try {} catch ([error = 1]) {
  function f() {}
  var f;
}
