/*---
description: Module lexical scopes reject duplicate function implementations
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
try {} catch {
  function f() {}
  function f() {}
}
