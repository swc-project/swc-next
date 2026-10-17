/*---
description: TypeScript body functions conflict with parameters in a separate scope
negative:
  phase: early
  type: SyntaxError
---*/
function outer(f = 1) {
  function f() {}
}
