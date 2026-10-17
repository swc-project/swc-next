/*---
description: Strict lexical scopes reject duplicate function implementations
flags: [onlyStrict]
negative:
  phase: early
  type: SyntaxError
---*/
{
  function f() {}
  function f() {}
}
