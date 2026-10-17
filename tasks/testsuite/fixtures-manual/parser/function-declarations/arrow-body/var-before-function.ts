/*---
description: TypeScript arrow body functions conflict with preceding vars
negative:
  phase: early
  type: SyntaxError
---*/
() => {
  var f;
  function f() {}
};
