/*---
description: Module lexical scopes reject duplicate function implementations
flags: [module]
negative:
  phase: early
  type: SyntaxError
---*/
{
  function f() {}
  function f() {}
}
