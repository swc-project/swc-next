/*---
description: TypeScript arrow body functions conflict with following vars
negative:
  phase: early
  type: SyntaxError
---*/
() => {
  function f() {}
  var f;
};
