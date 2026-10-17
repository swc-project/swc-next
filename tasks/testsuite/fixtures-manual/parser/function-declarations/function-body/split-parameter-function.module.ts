/*---
description: TypeScript body functions conflict with parameters in a separate scope
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
function outer(f = 1) {
  function f() {}
}
