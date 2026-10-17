/*---
description: TypeScript body functions conflict with simple parameters
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
function outer(f: unknown) {
  function f() {}
}
