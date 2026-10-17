/*---
description: TypeScript body functions conflict with simple parameters
negative:
  phase: early
  type: SyntaxError
---*/
function outer(f: unknown) {
  function f() {}
}
