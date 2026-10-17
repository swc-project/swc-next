/*---
description: TypeScript arrow body functions conflict with following vars
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
() => {
  function f() {}
  var f;
};
