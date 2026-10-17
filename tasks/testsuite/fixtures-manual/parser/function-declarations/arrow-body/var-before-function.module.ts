/*---
description: TypeScript arrow body functions conflict with preceding vars
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
() => {
  var f;
  function f() {}
};
