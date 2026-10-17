/*---
description: Functions conflict with preceding vars in a separate catch body scope
negative:
  phase: early
  type: SyntaxError
---*/
try {} catch ([error = 1]) {
  var f;
  function f() {}
}
